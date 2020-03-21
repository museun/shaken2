use {super::*, crate::*};

#[derive(Debug, Template)]
#[namespace("hello")]
enum Response<'a> {
    Hello { name: &'a str },
}

pub fn initialize<R>(init: &mut ModuleInit<'_, R>)
where
    R: Responder + Send + 'static,
{
    init.command_map.add(
        "hello",
        |context: Context<Command>, mut responder: R| async move {
            let resp = Response::Hello {
                name: &context.user().name,
            };
            responder.say(&context, &resp).await
        },
    );
}
