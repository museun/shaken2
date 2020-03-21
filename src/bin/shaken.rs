use shaken::{
    args::{self, DefaultTemplateStore},
    config::Config,
    database, modules, resolver,
    secrets::{self, Secrets},
    Bot, Directories,
};

use std::path::PathBuf;
use twitchchat::{Dispatcher, Runner, Status};

fn handle_startup() -> anyhow::Result<(Secrets, Config, DefaultTemplateStore)> {
    // this uses reverse order (least specific to most specific)
    // the last one will always override previous ones
    simple_env_load::load_env_from(&[
        Directories::config()?.join(".env"), //
        PathBuf::from(".env"),
    ]);
    alto_logger::init(alto_logger::Style::MultiLine, Default::default())?;

    // do this before the args so its a hard error
    let secrets = Secrets::from_env()?;
    let (config, templates) = args::handle_args();

    // TODO maybe get this from the config
    database::initialize_db_conn_string(
        Directories::data()?
            .join("database.db")
            .into_os_string()
            .to_string_lossy(),
    );

    Ok((secrets, config, templates))
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let (mut secrets, config, templates) = handle_startup()?;

    // initialize all of the modules
    let (state, commands, passives) =
        modules::ModuleInit::new(&config, &mut secrets).initialize()?;

    // create required twitchchat stuff
    let dispatcher = Dispatcher::new();
    let (runner, mut control) = Runner::new(dispatcher.clone(), Default::default());

    // create a responder
    let responder = shaken::WriterResponder::new(
        control.writer().clone(), //
        resolver::new_resolver(templates)?,
    );
    // and make it log its actions
    let responder = shaken::LoggingResponder::new(responder);

    // connect to twitch
    let conn = twitchchat::connect_easy_tls(
        &config.user_name,
        &secrets.take(secrets::TWITCH_OAUTH_TOKEN)?,
    )
    .await?;

    // create the bot future
    let bot = Bot::new(
        config,     // the bot configuration // TODO: make this 'reactive' (use the 'watch' module)
        control,    // the control interface
        dispatcher, // the event dispatcher
        commands,   // the command map
        passives,   // the passive list
    )
    .run(responder, state); // run the bot with this responder and this initial state

    // TODO maybe join instead of select
    tokio::select! {
        // run the twitchchat loop to completion
        status = runner.run(conn) => {
            match status.map_err(|err| {
                log::error!("error running: {}", err);
                err
            })? {
                Status::Canceled => log::info!("runner stopped"),
                Status::Eof => log::info!("runner ended"),
            }
        }
        // run the bot loop to completion
        result = bot => {
            if let Err(err) = result {
                log::error!("error running bot: {}", err);
                return Err(err);
            }
            log::info!("bot is done running")
        }
    }

    Ok(())
}
