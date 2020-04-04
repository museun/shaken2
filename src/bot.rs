use std::sync::Arc;

use super::{Command, CommandMap, Config, Context, Passive, PassiveList, Responder, State};

use futures::prelude::*;
use tokio::sync::RwLock;
use twitchchat::{events, messages, Control, Dispatcher};

pub struct Bot<R: Responder + Send + 'static> {
    control: Control,
    dispatcher: Dispatcher,
    config: Config,
    command_map: CommandMap<R>,
    passive_list: PassiveList<R>,

    _spoopy: std::marker::PhantomData<R>,
}

impl<R> Bot<R>
where
    R: Responder + Send + 'static,
{
    pub fn new(
        config: Config,
        control: Control,
        dispatcher: Dispatcher,
        command_map: CommandMap<R>,
        passive_list: PassiveList<R>,
    ) -> Self {
        Self {
            config,
            control,
            dispatcher,
            command_map,
            passive_list,

            _spoopy: Default::default(),
        }
    }

    pub async fn run(mut self, responder: R, mut state: State) -> anyhow::Result<()> {
        let mut writer = self.control.writer().clone();
        let config = self.config.clone();

        let info = self
            .dispatcher
            .wait_for::<events::GlobalUserState>()
            .await?;

        let messages::GlobalUserState {
            user_id,
            display_name,
            color,
            ..
        } = &*info;

        log::info!(
            "our user: {} ({}) {}",
            display_name.as_ref().unwrap(),
            user_id,
            color
        );
        state.insert(info);

        let state = Arc::new(RwLock::new(state));

        tokio::pin! {
            let active = self.dispatch_actives(
                responder.clone(),
                state.clone(),
                config.clone(),
            );

            let passive = self.dispatch_passives(
                responder,
                state.clone(),
                config,
            );
        }

        for room in &self.config.rooms {
            log::debug!("joining: {}", room);
            writer.join(room).await?;
        }

        tokio::select! {
            _ = &mut active => { }
            _ = &mut passive => { }
        }

        Ok(())
    }

    async fn dispatch_passives(&self, responder: R, state: Arc<RwLock<State>>, config: Config) {
        let mut passive = self.dispatcher.subscribe::<events::Privmsg>();
        while let Some(passive) = passive.next().await.and_then(Passive::new) {
            let state = Context::new(passive, Arc::clone(&state), config.clone());
            for passive in self.passive_list.iter() {
                log::trace!("dispatching to: {:?}", passive);
                let fut = passive
                    .inner
                    .call(state.clone(), responder.clone())
                    .inspect_err(|err| {
                        if err.is::<crate::util::DontCareSigil>() {
                            return;
                        }
                        log::error!("cannot run passive: {}", err);
                    });
                tokio::spawn(fut);
            }
        }
    }

    async fn dispatch_actives(&self, responder: R, state: Arc<RwLock<State>>, config: Config) {
        let mut active = self.dispatcher.subscribe::<events::Privmsg>();
        while let Some(msg) = active.next().await {
            log::info!("[{}] {}: {}", msg.channel, msg.name, msg.data);
            let cmd = match Command::parse(Arc::clone(&msg)) {
                Some(cmd) => cmd,
                None => continue,
            };

            let state = Context::new(cmd.clone(), Arc::clone(&state), config.clone());
            for command in self.command_map.find(&*cmd.head) {
                log::info!("dispatching to: {:?}", command);
                let fut = command
                    .inner
                    .call(state.clone(), responder.clone())
                    .inspect_err(|err| {
                        if err.is::<crate::util::DontCareSigil>() {
                            return;
                        }
                        log::error!("cannot run command: {}", err);
                    });
                tokio::spawn(fut);
            }
        }
    }
}
