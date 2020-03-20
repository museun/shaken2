use {super::*, crate::*};

#[derive(Debug, Template)]
#[namespace("uptime")]
enum Response<'a> {
    Uptime { room: &'a str, uptime: String },
    Offline { room: &'a str },
}

pub async fn initialize<R>(init: &mut ModuleInit<'_, R>)
where
    R: Responder + Send + 'static,
{
    init.command_map.add("uptime", uptime);
}

async fn uptime<R>(context: Context<Command>, mut responder: R) -> Result
where
    R: Responder + Send + 'static,
{
    let cache = context.state().await;
    let client = cache.expect_get::<crate::twitch::Client>()?;

    let room = context.room();
    let name = room.remove_hashes();

    let resp = match client
        .get_streams(&[name])
        .await?
        .into_iter()
        .find(|stream| stream.user_id == room.id)
    {
        Some(stream) => {
            let dur = time::OffsetDateTime::now() - stream.started_at;
            Response::Uptime {
                room: name,
                uptime: dur.as_readable_time(),
            }
        }
        None => Response::Offline { room: name },
    };

    responder.say(&context, &resp).await
}
