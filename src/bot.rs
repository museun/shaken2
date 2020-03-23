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

/*
struct TrackedEvents {
    tracker: Tracker,

    // the hash sets are so theres less lock contention
    seen_rooms: HashSet<u64>,
    seen_users: HashSet<u64>,

    roomstate: EventStream<Arc<messages::RoomState<'static>>>,
    privmsg: EventStream<Arc<messages::Privmsg<'static>>>,
}

impl TrackedEvents {
    fn new(tracker: Tracker, dispatcher: &Dispatcher) -> Self {
        Self {
            tracker,
            seen_rooms: HashSet::new(),
            seen_users: HashSet::new(),
            roomstate: dispatcher.subscribe::<events::RoomState>(),
            privmsg: dispatcher.subscribe::<events::Privmsg>(),
        }
    }

    async fn run_to_completion(mut self) {
        loop {
            tokio::select! {
                Some(msg) = self.roomstate.next() => {
                    if let Some(id) = msg.tags.get_parsed::<_, u64>("room-id") {
                        self.try_insert_room(id, &*msg.channel).await;
                    }
                }

                Some(msg) = self.privmsg.next() => {
                    if let Some(id) = msg.user_id() {
                        let name = msg.display_name().cloned().unwrap_or_else(|| {
                            msg.name.clone()
                        });
                        self.try_insert_user(id, name).await;
                    }
                }

                else => { break }
            }
        }
    }

    async fn try_insert_room(&mut self, id: u64, room: impl ToString) {
        if self.seen_rooms.insert(id) {
            self.tracker.rooms.set(id, room).await;
        }
    }

    async fn try_insert_user(&mut self, id: u64, username: impl ToString) {
        if self.seen_users.insert(id) {
            self.tracker.users.set(id, username).await;
        }
    }
}
*/
