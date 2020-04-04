use crate::{CommandMap, PassiveList, Responder, State};

type Result = anyhow::Result<()>;

pub struct ModuleInit<'a, R> {
    pub secrets: &'a mut crate::secrets::Secrets,
    pub pool: sqlx::SqlitePool,
    pub config: crate::WatchedConfig,

    pub state: State,
    pub command_map: CommandMap<R>,
    pub passive_list: PassiveList<R>,

    _responder: std::marker::PhantomData<R>,
}

impl<'a, R: Responder + Send + 'static> ModuleInit<'a, R> {
    pub async fn initialize(
        secrets: &'a mut crate::secrets::Secrets,
        pool: sqlx::SqlitePool,
        config: crate::WatchedConfig,
    ) -> anyhow::Result<ModuleInit<'a, R>> {
        let (command_map, passive_list, state, _responder) = Default::default();
        let mut this = ModuleInit {
            secrets,
            pool,
            config,

            command_map,
            passive_list,
            state,
            _responder,
        };

        this.build_state()?;

        shakespeare::initialize(&mut this).await;
        hello::initialize(&mut this).await;
        uptime::initialize(&mut this).await;
        viewers::initialize(&mut this).await;
        crates::initialize(&mut this).await;
        version::initialize(&mut this).await;
        whatsong::initialize(&mut this).await;

        // this has to be at the end so it won't clobber the built-in commands
        user_defined::initialize(&mut this).await?;

        Ok(this)
    }

    fn build_state(&mut self) -> anyhow::Result<()> {
        // place the state deps here if you need them initialize before any of
        // the modules
        let twitch_client_id = self.secrets.take(crate::secrets::TWITCH_CLIENT_ID)?;
        let client = crate::TwitchClient::new(&twitch_client_id);
        self.state.insert(client);

        Ok(())
    }
}

mod crates;
mod hello;
mod shakespeare;
mod uptime;
mod version;
mod viewers;
mod whatsong;

mod user_defined;
