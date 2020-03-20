use crate::{CommandMap, Config, PassiveList, Responder, State};

type Result = anyhow::Result<()>;

pub struct ModuleInit<'a, R> {
    pub config: &'a Config,
    pub command_map: &'a mut CommandMap<R>,
    pub passive_list: &'a mut PassiveList<R>,
    pub state: &'a mut State,
    pub secrets: &'a crate::secrets::Secrets,
}

impl<'a, R: Responder + Send + 'static> ModuleInit<'a, R> {
    pub async fn initialize(&mut self) {
        shakespeare::initialize(self).await;
        hello::initialize(self).await;
        uptime::initialize(self).await;
        viewers::initialize(self).await;
        crates::initialize(self).await;
        version::initialize(self).await;
        whatsong::initialize(self).await;

        // this has to be at the end so it won't clobber the built-in commands
        // user_defined::initialize(self).await;

        use crate::twitch::Client;
        let client = Client::new(&self.secrets.twitch_client_id);
        self.state.insert(client);
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
