use anyhow::Context as _;
use rand::prelude::*;
use rusqlite::{Connection, OpenFlags};

use once_cell::sync::OnceCell;
static DB_CONN_STRING: OnceCell<String> = OnceCell::new();

/// Initialize the global connectiong string
pub fn initialize_db_conn_string(conn: impl ToString) {
    DB_CONN_STRING.get_or_init(|| conn.to_string());
}

/// Gets the global connection
pub fn get_global_connection() -> anyhow::Result<Connection> {
    if cfg!(test) {
        initialize_db_conn_string("");
    }

    DB_CONN_STRING
        .get()
        .ok_or_else(|| anyhow::anyhow!("DB_CONN_STRING is not set"))
        .and_then(|conn| get_connection(conn.as_str()))
}

/// Get a temporarly in-memory database named 'id'
pub fn get_in_memory(id: impl std::fmt::Display) -> anyhow::Result<Connection> {
    let conn = format!("file:{}?mode=memory&cache=shared", id);
    Connection::open_with_flags(
        &conn,
        OpenFlags::SQLITE_OPEN_URI | OpenFlags::SQLITE_OPEN_READ_WRITE,
    )
    .map_err(Into::into)
}

/// Generate a random name suitable for a db
pub fn rand_db_name() -> String {
    thread_rng()
        .sample_iter(&rand::distributions::Alphanumeric)
        .take(10)
        .collect::<String>()
}

/// Get an sqlite connection
///
/// In test mode it'll return a thread-local temporary in-memory database
pub fn get_connection<'a>(conn: impl Into<Option<&'a str>>) -> anyhow::Result<Connection> {
    if cfg!(test) {
        thread_local!(static TEST_DB_ID: String = rand_db_name());
        TEST_DB_ID
            .with(|id| get_in_memory(&id))
            .with_context(|| TEST_DB_ID.with(|id| format!("cannot open tmp db: {}", &id)))
    } else {
        let conn = conn
            .into()
            .ok_or_else(|| anyhow::anyhow!("connection string cannot be empty"))?;

        Connection::open_with_flags(
            conn,
            OpenFlags::SQLITE_OPEN_SHARED_CACHE
                | OpenFlags::SQLITE_OPEN_READ_WRITE
                | OpenFlags::SQLITE_OPEN_CREATE,
        )
        .with_context(|| anyhow::anyhow!("cannot open db, conn_string: {}", conn.escape_debug()))
    }
}
