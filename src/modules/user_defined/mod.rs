#![cfg_attr(debug_assertions, allow(dead_code, unused_variables, unused_mut))]

mod registry;
mod response;
use {registry::Registry, response::Response};

use {super::*, crate::*};

#[derive(Debug, Clone, Eq, Ord)]
struct UserDefinedCommand {
    name: String,
    body: String,
    room: i64,
    uses: i32,
    owner: i64,
    disabled: bool,
    created_at: time::OffsetDateTime, // maybe time::PrimitiveDateTime (we're going to assume UTC)
}

impl PartialEq for UserDefinedCommand {
    fn eq(&self, other: &Self) -> bool {
        self.name.eq(&other.name)
            && self.body.eq(&other.body)
            && self.room.eq(&other.room)
            && self.uses.eq(&other.uses)
            && self.owner.eq(&other.owner)
            && self.disabled.eq(&other.disabled)
    }
}

impl PartialOrd for UserDefinedCommand {
    fn partial_cmp(&self, other: &UserDefinedCommand) -> Option<std::cmp::Ordering> {
        let left = self.room.partial_cmp(&other.room)?;
        let right = self.name.partial_cmp(&other.name)?;
        left.partial_cmp(&right)
    }
}

pub async fn initialize<R>(init: &mut ModuleInit<'_, R>) -> anyhow::Result<()>
where
    R: Responder + Send + 'static,
{
    init.command_map.add("add", add);
    init.command_map.add("edit", edit);
    init.command_map.add("info", info);
    init.command_map.add("delete", delete);
    init.command_map.add("rename", rename);

    init.passive_list.add(user_defined);

    Registry::initialize_table(init.pool.clone()).await?;
    Registry::reserve_many(
        init.pool.clone(),
        init.command_map.command_names().map(|s| format!("!{}", s)),
    )
    .await?;

    let builtin = Registry::all_builtin(init.pool.clone()).await?;
    let mut commands = Registry::all_commands(init.pool.clone()).await?;

    let mut bad = vec![];
    for mut command in commands {
        if builtin.contains(&command.name) {
            command.disabled = true;
            bad.push(command);
        }
    }

    Registry::update_many(
        init.pool.clone(),
        bad.iter().inspect(|udc| {
            log::warn!(
                "disabling user command: '{}' owned by {}",
                udc.name,
                udc.owner
            );
        }),
    )
    .await?;

    Ok(())
}

fn parse_command(cmd: &Command) -> (Option<&String>, Option<String>) {
    let mut iter = cmd.tail.iter(); // is this right?
    let (head, tail) = (
        iter.next(),
        iter.fold(String::new(), |mut a, c| {
            if !a.is_empty() {
                a.push_str(" ");
            }
            a.push_str(c);
            a
        }),
    );

    (head, Some(tail).filter(|s| !s.is_empty()))
}

async fn assert_both<R>(
    context: &Context<Command>,
    mut responder: &mut R,
) -> anyhow::Result<Option<(String, String)>>
where
    R: Responder + Send + 'static,
{
    match parse_command(&context.args) {
        // command and body
        (Some(head), Some(tail)) => Ok(Some((head.clone(), tail))),
        // empty body
        (Some(head), None) => {
            let resp = Response::ErrorMissingTail { head: &head };
            responder.reply(&context, &resp).await?;
            Ok(None)
        }
        // empty command
        (None, _) => {
            let resp = Response::ErrorMissingHead;
            responder.reply(&context, &resp).await?;
            Ok(None)
        }
    }
}

async fn authorized(
    context: &Context<Command>,
    udc: &UserDefinedCommand,
    pool: sqlx::SqlitePool,
) -> anyhow::Result<bool> {
    let (room, user) = (context.room(), context.user());

    let state = context.state().await;
    let twitch = state.expect_get::<TwitchClient>()?;

    let owned = twitch
        .get_users(&context.config.owners)
        .await?
        .iter()
        .map(|user| user.id)
        .any(|d| d == udc.owner as u64);

    let authed = {
        let msg = &context.args.message;
        msg.is_moderator()
            || msg.badges().iter().map(|s| &s.kind).fold(false, |ok, b| {
                use twitchchat::BadgeKind::*;
                ok & match b {
                    Broadcaster | Moderator => true,
                    _ => false,
                }
            })
    };

    Ok(authed || owned)
}

async fn add<R>(context: Context<Command>, mut responder: R) -> Result
where
    R: Responder + Send + 'static,
{
    let (head, tail) = assert_both(&context, &mut responder).await?.dont_care()?;

    let state = context.state().await;
    let pool = state.expect_get::<sqlx::SqlitePool>()?.clone();

    let udc = UserDefinedCommand {
        name: head,
        body: tail,
        room: context.room().id as _,
        owner: context.user().id as _,
        uses: 0,
        disabled: false,
        created_at: time::OffsetDateTime::now(),
    };

    let resp = match Registry::add_user_command(pool, &udc).await? {
        registry::AddResult::Builtin => Response::ErrorReservedName { command: &udc.name },
        registry::AddResult::Exists => Response::ErrorAlreadyExists { command: &udc.name },
        registry::AddResult::Okay => Response::Added { command: &udc.name },
    };

    responder.say(&context, &resp).await
}

async fn edit<R>(context: Context<Command>, mut responder: R) -> Result
where
    R: Responder + Send + 'static,
{
    let (head, tail) = assert_both(&context, &mut responder).await?.dont_care()?;
    let room = context.room();

    let state = context.state().await;
    let pool = state.expect_get::<sqlx::SqlitePool>()?.clone();

    let mut udc = match Registry::lookup(pool.clone(), &head, room.id).await? {
        Some(udc) => udc,
        None => {
            return responder
                .reply(&context, &Response::ErrorCommandNotFound { command: &head })
                .await
        }
    };

    if !authorized(&context, &udc, pool.clone()).await? {
        return responder
            .reply(
                &context,
                &Response::ErrorInsufficientPrivlege { command: &head },
            )
            .await;
    }

    udc.body = tail;
    let _ = Registry::update(pool.clone(), &udc).await?; // bool

    responder
        .reply(&context, &Response::Edited { command: &udc.name })
        .await
}

async fn info<R>(context: Context<Command>, mut responder: R) -> Result
where
    R: Responder + Send + 'static,
{
    let (head, _) = parse_command(&context.args);
    let head = head.dont_care()?;
    let room = context.room();

    let state = context.state().await;
    let pool = state.expect_get::<sqlx::SqlitePool>()?.clone();

    let cmd = match Registry::lookup(pool, &head, room.id).await? {
        Some(cmd) => cmd,
        None => {
            return responder
                .reply(&context, &Response::ErrorCommandNotFound { command: &head })
                .await
        }
    };

    let owner = match state
        .expect_get::<crate::TwitchClient>()?
        .get_users_from_id(&[cmd.owner])
        .await?
        .pop()
    {
        Some(user) => user.display_name,
        None => "<unknown>".into(),
    };

    let resp = &[
        Response::CommandDescription {
            command: &cmd.name,
            body: &cmd.body,
        },
        Response::CommandCreatedAt {
            owner: &owner,
            uses: cmd.uses as _,
        },
    ];

    for resp in resp {
        responder.reply(&context, resp).await?;
    }
    Ok(())
}

async fn delete<R>(context: Context<Command>, mut responder: R) -> Result
where
    R: Responder + Send + 'static,
{
    let (head, _) = parse_command(&context.args);
    let head = head.dont_care()?;
    let (room, user) = (context.room(), context.user());

    let state = context.state().await;
    let pool = state.expect_get::<sqlx::SqlitePool>()?.clone();

    let udc = match Registry::lookup(pool.clone(), &head, room.id).await? {
        Some(udc) => udc,
        None => {
            return responder
                .reply(&context, &Response::ErrorCommandNotFound { command: &head })
                .await
        }
    };

    if !authorized(&context, &udc, pool.clone()).await? {
        return responder
            .reply(
                &context,
                &Response::ErrorInsufficientPrivlege { command: &head },
            )
            .await;
    }

    // TODO should this say that its a builtin command?
    let resp = match Registry::remove(pool, &udc).await? {
        registry::RemoveResult::Missing => Response::ErrorCommandNotFound { command: &head },
        registry::RemoveResult::Okay => Response::Deleted,
    };

    responder.reply(&context, &resp).await
}

async fn rename<R>(context: Context<Command>, mut responder: R) -> Result
where
    R: Responder + Send + 'static,
{
    let (head, tail) = assert_both(&context, &mut responder).await?.dont_care()?;
    let (room, user) = (context.room(), context.user());

    let state = context.state().await;
    let pool = state.expect_get::<sqlx::SqlitePool>()?.clone();

    let mut udc = match Registry::lookup(pool.clone(), &head, room.id).await? {
        Some(udc) => udc,
        None => {
            return responder
                .reply(&context, &Response::ErrorCommandNotFound { command: &head })
                .await
        }
    };

    if !authorized(&context, &udc, pool.clone()).await? {
        return responder
            .reply(
                &context,
                &Response::ErrorInsufficientPrivlege { command: &head },
            )
            .await;
    }

    let old = std::mem::replace(&mut udc.name, tail.clone());
    let resp = match Registry::update(pool, &udc).await? {
        true => Response::Renamed {
            from: &old,
            to: &tail,
        },
        false => Response::ErrorCommandNotFound { command: &old },
    };

    responder.reply(&context, &resp).await
}

async fn user_defined<R>(context: Context<Passive>, mut responder: R) -> Result
where
    R: Responder + Send + 'static,
{
    let head = context
        .data()
        .split(' ')
        .map(|s| s.trim())
        .next()
        .dont_care()?;
    if !head.starts_with('!') || head.len() == 1 {
        return dont_care();
    }

    let state = context.state().await;
    let pool = state.expect_get::<sqlx::SqlitePool>()?.clone();

    let mut udc = Registry::lookup(pool.clone(), &head, context.room().id)
        .await?
        .dont_care()?;

    responder
        .say(&context, &Response::Say { data: &udc.body })
        .await?;

    udc.uses += 1;
    let _ = Registry::update(pool, &udc).await?;

    Ok(())
}
