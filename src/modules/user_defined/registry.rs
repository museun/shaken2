use super::UserDefinedCommand;
use std::collections::HashSet;

pub(super) struct Registry;

impl Registry {
    pub(super) async fn initialize_table(pool: sqlx::SqlitePool) -> anyhow::Result<()> {
        // TODO foreign key unique for name fields in both tables
        sqlx::query_file!("./src/modules/user_defined/sql/create_registry.sql")
            .execute(&mut &pool)
            .await?;
        Ok(())
    }

    pub(super) async fn all_builtin(pool: sqlx::SqlitePool) -> anyhow::Result<HashSet<String>> {
        struct Builtin {
            name: String,
        }

        // TODO use fetch and allocate into the hashmap instead of vec->map
        sqlx::query_file_as!(Builtin, "./src/modules/user_defined/sql/all_builtin.sql")
            .fetch_all(&mut &pool)
            .await
            .map(|vec| vec.into_iter().map(|k| k.name).collect())
            .map_err(Into::into)
    }

    pub(super) async fn reserve(pool: sqlx::SqlitePool, name: &str) -> anyhow::Result<()> {
        anyhow::ensure!(
            sqlx::query_file!("./src/modules/user_defined/sql/reserve.sql", &name)
                .execute(&mut &pool)
                .await?
                == 1,
            "command '{}' wasn't unique",
            name
        );
        Ok(())
    }

    pub(super) async fn reserve_many<I, S>(pool: sqlx::SqlitePool, names: I) -> anyhow::Result<()>
    where
        I: Iterator<Item = S>,
        S: AsRef<str>,
    {
        use sqlx::prelude::*;

        let mut conn = pool.acquire().await?;
        let mut tx = conn.begin().await?;
        for name in names {
            sqlx::query_file!("./src/modules/user_defined/sql/reserve.sql", name.as_ref())
                .execute(&mut tx)
                .await?;
        }
        tx.commit().await?;
        Ok(())
    }

    pub(super) async fn add_user_command(
        pool: sqlx::SqlitePool,
        udc: &UserDefinedCommand,
    ) -> anyhow::Result<AddResult> {
        if Self::all_builtin(pool.clone()).await?.contains(&udc.name) {
            return Ok(AddResult::Builtin);
        }

        if sqlx::query_file!(
            "./src/modules/user_defined/sql/add_user_command.sql",
            &udc.name,
            &udc.body,
            &udc.room,
            &udc.uses,
            &udc.owner,
            &udc.disabled,
            &udc.created_at.format("%F %T %z"),
        )
        .execute(&mut &pool)
        .await
        .is_err()
        {
            // this means the constraint failed
            return Ok(AddResult::Exists);
        }

        Ok(AddResult::Okay)
    }

    pub(super) async fn remove(
        pool: sqlx::SqlitePool,
        udc: &UserDefinedCommand,
    ) -> anyhow::Result<RemoveResult> {
        let UserDefinedCommand { name, room, .. } = udc;

        let res = match sqlx::query_file!(
            "./src/modules/user_defined/sql/remove.sql",
            name,
            (*room as i64)
        )
        .execute(&mut &pool)
        .await?
        {
            0 => RemoveResult::Missing,
            _ => RemoveResult::Okay,
        };

        Ok(res)
    }

    pub(super) async fn update(
        pool: sqlx::SqlitePool,
        udc: &UserDefinedCommand,
    ) -> anyhow::Result<bool> {
        let n = sqlx::query_file!(
            "./src/modules/user_defined/sql/update.sql",
            &udc.name,
            &udc.body,
            &udc.uses,
            &udc.disabled,
            &udc.name,
        )
        .execute(&mut &pool)
        .await?;
        Ok(n == 1)
    }

    pub(super) async fn update_many<'a, I>(pool: sqlx::SqlitePool, many: I) -> anyhow::Result<()>
    where
        I: Iterator<Item = &'a UserDefinedCommand> + 'a,
    {
        use sqlx::prelude::*;

        let mut conn = pool.acquire().await?;
        let mut tx = conn.begin().await?;

        for udc in many {
            sqlx::query_file!(
                "./src/modules/user_defined/sql/update.sql",
                &udc.name,
                &udc.body,
                &udc.uses,
                &udc.disabled,
                &udc.name,
            )
            .execute(&mut tx)
            .await?;
        }
        tx.commit().await?;
        Ok(())
    }

    pub(super) async fn lookup(
        pool: sqlx::SqlitePool,
        name: &str,
        room: u64,
    ) -> anyhow::Result<Option<UserDefinedCommand>> {
        Ok(sqlx::query_file_as!(
            UserDefinedCommandRow,
            "./src/modules/user_defined/sql/lookup.sql",
            name,
            (room as i64),
        )
        .fetch_optional(&mut &pool)
        .await?
        .map(Into::into))
    }

    pub(super) async fn all_commands_for(
        pool: sqlx::SqlitePool,
        room: u64,
    ) -> anyhow::Result<Vec<UserDefinedCommand>> {
        // to fetch and allocate into the vec instead of vec->vec
        Ok(sqlx::query_file_as!(
            UserDefinedCommandRow,
            "./src/modules/user_defined/sql/all_commands_for.sql",
            (room as i64)
        )
        .fetch_all(&mut &pool)
        .await?
        .into_iter()
        .map(Into::into)
        .collect())
    }

    pub(super) async fn all_commands(
        pool: sqlx::SqlitePool,
    ) -> anyhow::Result<Vec<UserDefinedCommand>> {
        // to fetch and allocate into the vec instead of vec->vec
        Ok(sqlx::query_file_as!(
            UserDefinedCommandRow,
            "./src/modules/user_defined/sql/all_commands.sql"
        )
        .fetch_all(&mut &pool)
        .await?
        .into_iter()
        .map(Into::into)
        .collect())
    }
}

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum AddResult {
    Builtin,
    Exists,
    Okay,
}

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum RemoveResult {
    Missing,
    Okay,
}

#[derive(Debug)]
struct UserDefinedCommandRow {
    name: String,
    body: String,
    room: i32,
    uses: i32,
    owner: String,
    disabled: bool,
    created_at: Vec<u8>,
}

impl From<UserDefinedCommandRow> for UserDefinedCommand {
    fn from(udc: UserDefinedCommandRow) -> Self {
        UserDefinedCommand {
            disabled: udc.disabled,
            created_at: time::OffsetDateTime::parse(
                std::str::from_utf8(&udc.created_at).unwrap(),
                "%F %T %z",
            )
            .unwrap(),
            name: udc.name,
            body: udc.body,
            room: udc.room as _, // this cast should be different
            uses: udc.uses,
            owner: udc.owner.parse().unwrap(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    async fn get_db() -> sqlx::SqlitePool {
        use rand::prelude::*;
        // env = sqlite://sqlx_reference_database.db
        //
        //?mode=memory&cache=shared
        let db_name = thread_rng()
            .sample_iter(&rand::distributions::Alphanumeric)
            .take(7)
            .collect::<String>();
        let db_name = format!("file:{}?mode=memory&cache=shared", db_name);
        let pool = sqlx::SqlitePool::new(&db_name).await.unwrap();
        Registry::initialize_table(pool.clone()).await.unwrap();
        pool
    }

    fn make_udc(name: &str) -> UserDefinedCommand {
        UserDefinedCommand {
            name: name.into(),
            body: "test body".into(),
            room: 1234,
            uses: 0,
            owner: 1234,
            disabled: false,
            created_at: time::OffsetDateTime::now_local(),
        }
    }

    #[tokio::test]
    async fn reserve() {
        let pool = get_db().await;
        Registry::reserve(pool.clone(), "foobar").await.unwrap();
        assert!(Registry::all_builtin(pool.clone())
            .await
            .unwrap()
            .contains("foobar"));

        Registry::reserve(pool.clone(), "bazbar").await.unwrap();
        assert!(Registry::all_builtin(pool.clone())
            .await
            .unwrap()
            .contains("bazbar"));

        Registry::reserve_many(pool.clone(), ["asdf", "fdsa", "testing"].iter())
            .await
            .unwrap();

        assert_eq!(Registry::all_builtin(pool.clone()).await.unwrap().len(), 5);

        Registry::reserve(pool.clone(), "foobar").await.unwrap_err();
        assert_eq!(Registry::all_builtin(pool.clone()).await.unwrap().len(), 5);
    }

    #[tokio::test]
    async fn add_user_command() {
        let pool = get_db().await;
        Registry::reserve(pool.clone(), "taken").await.unwrap();

        assert!(matches!(
            Registry::add_user_command(pool.clone(), &make_udc("testing"))
                .await
                .unwrap(),
            AddResult::Okay
        ));

        assert!(matches!(
            Registry::add_user_command(pool.clone(), &make_udc("testing"))
                .await
                .unwrap(),
            AddResult::Exists
        ));

        assert!(matches!(
            Registry::add_user_command(pool.clone(), &{
                let mut udc = make_udc("testing");
                udc.room = 4321;
                udc
            })
            .await
            .unwrap(),
            AddResult::Okay
        ));

        assert!(matches!(
            Registry::add_user_command(pool.clone(), &{
                let mut udc = make_udc("testing");
                udc.room = 4321;
                udc
            })
            .await
            .unwrap(),
            AddResult::Exists
        ));

        assert!(matches!(
            Registry::add_user_command(pool.clone(), &make_udc("taken"))
                .await
                .unwrap(),
            AddResult::Builtin
        ));
    }

    #[tokio::test]
    async fn remove() {
        let pool = get_db().await;
        Registry::reserve(pool.clone(), "taken").await.unwrap();

        let mut udc = make_udc("taken");
        udc.room = 4321;

        assert!(matches!(
            Registry::remove(pool.clone(), &udc).await.unwrap(),
            RemoveResult::Missing
        ));

        let mut udc = make_udc("foobar");
        Registry::add_user_command(pool.clone(), &udc)
            .await
            .unwrap();

        udc.room = 4321;

        // wrong room
        assert!(matches!(
            Registry::remove(pool.clone(), &udc).await.unwrap(),
            RemoveResult::Missing
        ));

        udc.room = 1234;
        assert!(matches!(
            Registry::remove(pool.clone(), &udc).await.unwrap(),
            RemoveResult::Okay
        ));

        // ensure its been removed
        assert!(matches!(
            Registry::remove(pool.clone(), &udc).await.unwrap(),
            RemoveResult::Missing
        ));
    }

    #[tokio::test]
    async fn update() {
        let pool = get_db().await;
        Registry::reserve(pool.clone(), "taken").await.unwrap();

        let udc = make_udc("testing");
        assert!(matches!(
            Registry::add_user_command(pool.clone(), &udc)
                .await
                .unwrap(),
            AddResult::Okay
        ));

        assert_eq!(
            Registry::update(
                pool.clone(),
                &UserDefinedCommand {
                    body: "asdf".into(),
                    ..udc
                },
            )
            .await
            .unwrap(),
            true
        );

        assert_eq!(
            Registry::lookup(pool.clone(), "testing", 1234)
                .await
                .unwrap()
                .unwrap()
                .body,
            "asdf"
        );

        assert_eq!(
            Registry::update(pool.clone(), &make_udc("not there"))
                .await
                .unwrap(),
            false
        );
    }

    #[tokio::test]
    async fn lookup() {
        let pool = get_db().await;
        Registry::reserve(pool.clone(), "taken").await.unwrap();

        assert!(Registry::lookup(pool.clone(), "taken", 1234)
            .await
            .unwrap()
            .is_none());

        let udc = make_udc("testing");

        assert!(matches!(
            Registry::add_user_command(pool.clone(), &udc)
                .await
                .unwrap(),
            AddResult::Okay
        ));

        assert!(Registry::lookup(pool.clone(), "testing", 4321)
            .await
            .unwrap()
            .is_none());

        assert_eq!(
            Registry::lookup(pool.clone(), "testing", 1234)
                .await
                .unwrap()
                .unwrap(),
            udc
        );

        let mut udc = udc;
        udc.body = "asdf".into();
        udc.room = 4321;

        assert!(matches!(
            Registry::add_user_command(pool.clone(), &udc)
                .await
                .unwrap(),
            AddResult::Okay
        ));

        assert_eq!(
            Registry::lookup(pool.clone(), "testing", 4321)
                .await
                .unwrap()
                .unwrap(),
            udc
        );
    }

    #[tokio::test]
    async fn all_commands() {
        let pool = get_db().await;
        Registry::reserve(pool.clone(), "taken").await.unwrap();

        let mut commands = Registry::all_commands(pool.clone()).await.unwrap();

        let mut names = ["foo", "bar", "baz"]
            .iter()
            .flat_map(|d| {
                let udc = make_udc(d);
                let mut udc2 = udc.clone();
                udc2.room = 4321;
                std::iter::once(udc).chain(std::iter::once(udc2))
            })
            .collect::<Vec<_>>();

        for name in &names {
            assert!(matches!(
                Registry::add_user_command(pool.clone(), name)
                    .await
                    .unwrap(),
                AddResult::Okay
            ));
        }

        let mut commands = Registry::all_commands(pool.clone()).await.unwrap();

        names.sort();
        commands.sort();

        assert_eq!(names, commands);
    }

    #[tokio::test]
    async fn all_commands_for() {
        let pool = get_db().await;
        Registry::reserve(pool.clone(), "taken").await.unwrap();

        let mut names = ["foo", "bar", "baz"]
            .iter()
            .flat_map(|d| {
                let udc = make_udc(d);
                let mut udc2 = udc.clone();
                udc2.room = 4321;
                std::iter::once(udc).chain(std::iter::once(udc2))
            })
            .collect::<Vec<_>>();

        for name in &names {
            assert!(matches!(
                Registry::add_user_command(pool.clone(), name)
                    .await
                    .unwrap(),
                AddResult::Okay
            ))
        }

        let mut commands = Registry::all_commands_for(pool.clone(), 4321)
            .await
            .unwrap();

        let mut left = names
            .into_iter()
            .filter(|k| k.room == 4321)
            .collect::<Vec<_>>();
        left.sort();

        commands.sort();
        assert_eq!(left, commands);

        assert!(Registry::all_commands_for(pool.clone(), 1)
            .await
            .unwrap()
            .is_empty());
    }
}
