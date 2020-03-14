use {super::*, crate::*};

#[derive(Debug, Template)]
#[namespace("viewers")]
enum Response<'a> {
    Offline { channel: &'a str },
    Viewers { count: u64 },
    NoViewers,
}

pub async fn initialize<R>(init: &mut ModuleInit<'_, R>)
where
    R: Responder + Send + 'static,
{
    init.command_map.add("viewers", viewers);
}

async fn viewers<R>(context: Context<Command>, mut responder: R) -> Result
where
    R: Responder + Send + 'static,
{
    let state = context.state().await;
    let client = state.expect_get::<crate::twitch::Client>()?;

    let room = context.room();
    let name = room.remove_hashes();

    let streams = client.get_streams(&[name]).await?;
    let resp = match streams
        .into_iter()
        .find(|stream| stream.user_id == room.id)
        .map(|s| s.viewer_count)
    {
        Some(0) => Response::NoViewers,
        Some(n) => Response::Viewers { count: n },
        None => Response::Offline { channel: name },
    };

    responder.say(&context, &resp).await
}
