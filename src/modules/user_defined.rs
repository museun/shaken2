#![cfg_attr(debug_assertions, allow(dead_code, unused_variables, unused_mut))]

use std::{
    collections::{HashMap, HashSet},
    sync::Arc,
};
use {super::*, crate::*};

#[derive(Debug, Template)]
#[namespace("user_defined")]
enum Response<'a> {
    ErrorReservedName {
        command: &'a str,
    },
    ErrorAlreadyExists {
        command: &'a str,
    },
    ErrorFailedToAdd {
        command: &'a str,
    },
    ErrorEditFailed {
        command: &'a str,
    },
    ErrorInvalidCommand {
        command: &'a str,
    },

    Added {
        command: &'a str,
    },
    Edited {
        command: &'a str,
    },

    Renamed {
        from: &'a str,
        to: &'a str,
    },
    Deleted,

    CommandDescription {
        command: &'a str,
        description: &'a str,
    },
    CommandCreatedAt {
        user: &'a str,
        uses: u64,
    },
}

#[derive(Debug)]
struct BuiltinCommands {
    commands: HashSet<Arc<str>>,
}

struct UserDefineCommand {
    command: String,
    body: String,
    description: String,
    creator: u64,
    creatred_at: u64,
    uses: u64,
    disabled: bool,
}

struct UserDefineCommands {
    user_defined: HashMap<String, UserDefineCommand>,
}

impl UserDefineCommands {
    fn new() -> Self {
        let mut user_defined = HashMap::default();
        Self { user_defined }
    }
}

pub async fn initialize<R>(init: &mut ModuleInit<'_, R>)
where
    R: Responder + Send + 'static,
{
    init.command_map.add("add", add);
    init.command_map.add("edit", edit);
    init.command_map.add("info", info);
    init.command_map.add("delete", delete);
    init.command_map.add("help", help);

    init.passive_list.add(user_defined);

    // ensure
    let conn = database::get_global_connection().expect("must be able to open db conn");
    conn.execute(
        r#"
        CREATE TABLE IF NOT EXISTS user_commands (
            id          INTEGER PRIMARY KEY AUTOINCREMENT,
            command     TEXT NOT NULL,
            body        TEXT NOT NULL,
            description TEXT NOT NULL,
            creator     INTEGER NOT NULL,
            created_at  INTEGER NOT NULL,
            users       INTEGER NOT NULL,
            disabled    INTEGER,
            unique(commands)
        )"#,
        rusqlite::NO_PARAMS,
    )
    .expect("create user_commands table");

    let builtin = BuiltinCommands {
        commands: init.command_map.command_names().collect(),
    };

    // get all of the existing commands
    // get all of the registered commands
    // remove any existing adjoint with registered

    init.state.write().await.insert(builtin);
}

async fn add<R>(context: Context<Command>, mut responder: R) -> Result
where
    R: Responder + Send + 'static,
{
    let command = "";
    let body = "";
    let description = "";
    let creator = "";
    let creatred_at = "";
    let uses = "";
    let disabled = "";

    let conn = database::get_global_connection()?;
    conn.execute_named(
        r#"
        INSERT OR IGNORE INTO user_commands (
            command,
            body,
            description,
            creator,
            creatred_at,
            uses,
            disabled
        ) VALUES (
            :command,
            :body,
            :description,
            :creator,
            :creatred_at,
            :uses,
            :disabled
        )"#,
        rusqlite::named_params! {
            ":command": &command,
            ":body": &body,
            ":description": &description,
            ":creator": &creator,
            ":creatred_at": &creatred_at,
            ":uses": &uses,
            ":disabled": &disabled,
        },
    )?;

    todo!()
}

async fn edit<R>(context: Context<Command>, mut responder: R) -> Result
where
    R: Responder + Send + 'static,
{
    let description = "";
    let command = "";

    let conn = database::get_global_connection()?;
    conn.execute_named(
        r#"
        UPDATE user_commands SET
            description = :description
        WHERE
            command = :command
        "#,
        rusqlite::named_params! {
            ":description": &description,
            ":command": &command,
        },
    )?;

    todo!()
}

async fn info<R>(context: Context<Command>, mut responder: R) -> Result
where
    R: Responder + Send + 'static,
{
    todo!()
}

async fn delete<R>(context: Context<Command>, mut responder: R) -> Result
where
    R: Responder + Send + 'static,
{
    let command = "";

    let conn = database::get_global_connection()?;
    conn.execute_named(
        r#"
        DELETE FROM user_commands
        WHERE command = :command
        "#,
        rusqlite::named_params! {
            ":command": &command,
        },
    )?;

    todo!()
}

async fn rename<R>(context: Context<Command>, mut responder: R) -> Result
where
    R: Responder + Send + 'static,
{
    todo!()
}

async fn help<R>(context: Context<Command>, mut responder: R) -> Result
where
    R: Responder + Send + 'static,
{
    todo!()
}

async fn user_defined<R>(context: Context<Passive>, mut responder: R) -> Result
where
    R: Responder + Send + 'static,
{
    todo!()
}

async fn fetch_command_names() -> anyhow::Result<Vec<String>> {
    tokio::task::spawn_blocking(move || {
        let conn = database::get_global_connection()?;
        let mut commands = conn
            .prepare("SELECT command FROM user_commands")?
            .query_map(rusqlite::NO_PARAMS, |row| row.get(0))?
            .flatten()
            .collect::<Vec<_>>();
        commands.sort_unstable();
        Ok(commands)
    })
    .await
    .unwrap()
}

async fn try_get_command(name: &str) -> anyhow::Result<UserDefineCommand> {
    let name = name.to_string();
    tokio::task::spawn_blocking(move || {
        let conn = database::get_global_connection()?;

        let ok = conn
            .prepare(
                r#"
                SELECT
                    command,
                    body,
                    description,
                    creator,
                    created_at,
                    users,
                    disabled
                FROM user_commands
                WHERE command = :command
                "#,
            )?
            .query_map_named(rusqlite::named_params! {":command": &name}, |row| {
                Ok(UserDefineCommand {
                    command: row.get(0)?,
                    body: row.get(1)?,
                    description: row.get(2)?,
                    creator: row.get::<_, i64>(3)? as _,
                    creatred_at: row.get::<_, i64>(4)? as _,
                    uses: row.get::<_, i64>(5)? as _,
                    disabled: row.get(6)?,
                })
            })?
            .next()
            .ok_or_else(|| anyhow::anyhow!("no command found for: '{}'", name))?;

        ok.map_err(Into::into)
    })
    .await
    .unwrap()
}

async fn disable_bad_command(cmd: &str) -> Result {
    let cmd = cmd.to_string();
    tokio::task::spawn_blocking(move || {
        let conn = database::get_global_connection()?;
        conn.execute_named(
            r#"
            UPDATE user_commands
            SET disabled = :disabled
            WHERE command = :command
            "#,
            rusqlite::named_params! {
                ":disabled": &true,
                ":command": &cmd,
            },
        )?;
        log::info!("disable bad command: {}", cmd);
        Ok::<_, anyhow::Error>(())
    })
    .await
    .unwrap()
    .map_err(Into::into)
}
