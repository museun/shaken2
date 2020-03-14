use crate::{CommandMap, Config, PassiveList, Responder, State};
use tokio::sync::RwLock;

type Result = anyhow::Result<()>;

pub struct ModuleInit<'a, R> {
    pub config: &'a Config,
    pub command_map: &'a mut CommandMap<R>,
    pub passive_list: &'a mut PassiveList<R>,
    pub state: &'a mut RwLock<State>,
}

impl<'a, R: Responder + Send + 'static> ModuleInit<'a, R> {
    pub async fn initialize(&mut self) {
        shakespeare::initialize(self).await;
        hello::initialize(self).await;
        uptime::initialize(self).await;
        viewers::initialize(self).await;
        crates::initialize(self).await;
        version::initialize(self).await;

        // this has to be at the end so it won't clobber the built-in commands
        // user_defined::initialize(self).await;
    }
}

mod crates;
mod hello;
mod shakespeare;
mod uptime;
mod version;
mod viewers;

mod user_defined;
