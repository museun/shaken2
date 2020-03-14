use {super::*, crate::*};

#[derive(Debug, Template)]
#[namespace("version")]
enum Response<'a> {
    Version {
        repo: &'a str,
        revision: &'a str,
        tag: &'a str,
    },
}

pub async fn initialize<R>(init: &mut ModuleInit<'_, R>)
where
    R: Responder + Send + 'static,
{
    init.command_map.add("version", version);
}

async fn version<R>(context: Context<Command>, mut responder: R) -> Result
where
    R: Responder + Send + 'static,
{
    let resp = Response::Version {
        repo: &env!("CARGO_PKG_REPOSITORY"),
        tag: &env!("SHAKEN_GIT_TAG"),
        revision: &env!("SHAKEN_GIT_REVISION"),
    };
    responder.reply(&context, &resp).await
}
