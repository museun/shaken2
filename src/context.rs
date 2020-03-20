use crate::{Config, RespondableContext, Room, State, Tracker, User};
use std::sync::Arc;
use tokio::sync::RwLock;

#[derive(Clone)]
pub struct Context<Args> {
    user_id: u64,
    args: Args,
    tracker: Tracker,
    state: Arc<RwLock<State>>,
    config: Config, // TODO use a watch here
}

impl<Args> Context<Args> {
    pub(super) fn new(
        user_id: u64,
        args: Args,
        tracker: Tracker,
        state: Arc<RwLock<State>>,
        config: Config,
    ) -> Self {
        Self {
            user_id,
            tracker,
            args,
            state,
            config,
        }
    }

    pub async fn get_our_name(&self) -> String {
        self.tracker
            .users
            .get(self.user_id)
            .await
            .expect("our username must be tracked")
    }

    pub fn tracker(&mut self) -> &mut Tracker {
        &mut self.tracker
    }

    pub async fn with_state<F, T>(&self, func: F) -> T
    where
        F: Fn(tokio::sync::RwLockReadGuard<'_, State>) -> T,
    {
        func(self.state.read().await)
    }

    pub async fn state(&self) -> tokio::sync::RwLockReadGuard<'_, State> {
        self.state.read().await
    }

    pub async fn state_mut(&mut self) -> tokio::sync::RwLockWriteGuard<'_, State> {
        self.state.write().await
    }

    pub const fn args(&self) -> &Args {
        &self.args
    }
}

macro_rules! common_short_hand {
    ($($ty:ident),* $(,)?) => {
        $(
            impl Context<crate::$ty> {
                pub fn data(&self) -> &str {
                    &*self.args().message.data
                }

                pub fn user(&self) -> User<'_> {
                    self.args().user()
                }

                pub fn room(&self) -> Room<'_> {
                    self.args().room()
                }

                pub fn user_and_room(&self) -> (User<'_>, Room<'_>) {
                    (self.user(), self.room())
                }
            }
        )*
    };
}

common_short_hand! {
    Command, //
    Passive
}
