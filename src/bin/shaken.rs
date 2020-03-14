use anyhow::Context as _;
use shaken::*;

use twitchchat::{Dispatcher, Runner, Status};

const OAUTH_ENV_VAR: &str = "SHAKEN_TWITCH_OAUTH_TOKEN";
const TWITCH_CLIENT_ID_ENV_VAR: &str = "SHAKEN_TWITCH_CLIENT_ID";

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // this uses reverse order (least specific to most specific)
    // the last one will always override previous ones
    let envs = &[
        Directories::config()?.join(".env"),
        std::path::PathBuf::from(".env"),
    ];

    config::load_env_from(envs);
    alto_logger::init(
        alto_logger::Style::MultiLine, //
        Default::default(),
    )
    .expect("init logger");

    // do this before parsing the args so its a hard error
    let oauth_token = std::env::var(OAUTH_ENV_VAR).with_context(
        || format!("`{}` must be set", OAUTH_ENV_VAR), //
    )?;

    let twitch_client_id = std::env::var(TWITCH_CLIENT_ID_ENV_VAR).with_context(
        || format!("`{}` must be set", TWITCH_CLIENT_ID_ENV_VAR), //
    )?;

    // TODO detect if we've got RUST_LOG=shaken::args=warn | error
    // if so, use the logger instead of println/eprintln?
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

    let tracker = Tracker::new();
    let responder = shaken::LoggingResponder::new(shaken::WriterResponder::new(
        control.writer().clone(), //
        resolver::new_resolver(templates)?,
        tracker.clone(),
    ));

    let mut map = CommandMap::new(responder.clone());
    let mut passives = PassiveList::new(responder.clone());
    let mut state = shaken::StateRef::default();

    modules::ModuleInit {
        config: &config,
        command_map: &mut map,
        passive_list: &mut passives,
        state: std::sync::Arc::get_mut(&mut state).unwrap(),
    }
    .initialize()
    .await;

    let twitch = shaken::twitch::Client::new(&twitch_client_id);
    state.write().await.insert(twitch);

    // connect to twitch
    let conn = twitchchat::connect_easy_tls(&config.user_name, &oauth_token).await?;

    let bot = Bot::new(
        config,     //
        control,    //
        dispatcher, //
        map,        //
        passives,   //
    )
    .run(tracker, state);

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
            };
            log::info!("bot is done running")
        }
    };

    Ok(())
}
