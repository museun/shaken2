use crate::{CommandMap, Config, PassiveList, Responder, State};

type Result = anyhow::Result<()>;

pub struct ModuleInit<'a, R> {
    pub config: &'a Config,
    pub secrets: &'a mut crate::secrets::Secrets,

    pub state: State,
    pub command_map: CommandMap<R>,
    pub passive_list: PassiveList<R>,

    _responder: std::marker::PhantomData<R>,
}

impl<'a, R: Responder + Send + 'static> ModuleInit<'a, R> {
    pub fn new(config: &'a Config, secrets: &'a mut crate::secrets::Secrets) -> Self {
        let (command_map, passive_list, state, _responder) = Default::default();
        Self {
            config,
            secrets,

            command_map,
            passive_list,
            state,
            _responder,
        }
    }

    pub fn initialize(mut self) -> anyhow::Result<(State, CommandMap<R>, PassiveList<R>)> {
        self.build_state()?;

        shakespeare::initialize(&mut self);
        hello::initialize(&mut self);
        uptime::initialize(&mut self);
        viewers::initialize(&mut self);
        crates::initialize(&mut self);
        version::initialize(&mut self);
        whatsong::initialize(&mut self);

        // this has to be at the end so it won't clobber the built-in commands
        // user_defined::initialize(self);

        let Self {
            state,
            command_map,
            passive_list,
            ..
        } = self;
        Ok((state, command_map, passive_list))
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
