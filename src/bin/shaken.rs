use twitchchat::{Dispatcher, Runner, Status};

use shaken::{args, config, database, modules, resolver, secrets};
use shaken::{Bot, CommandMap, Directories, PassiveList, State};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // this uses reverse order (least specific to most specific)
    // the last one will always override previous ones
    let envs = &[
        Directories::config()?.join(".env"),
        std::path::PathBuf::from(".env"),
    ];

    config::load_env_from(envs);
    alto_logger::init(alto_logger::Style::MultiLine, Default::default())?;

    // do this before the args so its a hard error
    let secrets = secrets::Secrets::from_env()?;

    let (config, templates) = args::handle_args();

    // TODO maybe get this from the config
    database::initialize_db_conn_string(
        Directories::data()?
            .join("database.db")
            .into_os_string()
            .to_string_lossy(),
    );

    let dispatcher = Dispatcher::new();
    let (runner, mut control) = Runner::new(dispatcher.clone(), Default::default());

    let responder = shaken::LoggingResponder::new(shaken::WriterResponder::new(
        control.writer().clone(),
        resolver::new_resolver(templates)?,
    ));

    let mut commands = CommandMap::new(responder.clone());
    let mut passives = PassiveList::new(responder);
    let mut state = State::default();

    modules::ModuleInit {
        config: &config,
        command_map: &mut commands,
        passive_list: &mut passives,
        state: &mut state,
        secrets: &secrets,
    }
    .initialize()
    .await;

    // connect to twitch
    let conn = twitchchat::connect_easy_tls(
        &config.user_name, //
        &secrets.twitch_oauth_token,
    )
    .await?;

    let bot = Bot::new(
        config,     //
        control,    //
        dispatcher, //
        commands,   //
        passives,   //
    )
    .run(state);

    // TODO maybe join instead of select
    tokio::select! {
        status = runner.run(conn) => {
            match status.map_err(|err| {
                log::error!("error running: {}", err);
                err
            })? {
                Status::Canceled => log::info!("runner stopped"),
                Status::Eof => log::info!("runner ended"),
            }
        }
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
