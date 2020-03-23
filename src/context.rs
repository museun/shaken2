use crate::{Config, RespondableContext, Room, State, User};
use std::sync::Arc;
use tokio::sync::RwLock;

pub type ShakenInfo = twitchchat::messages::GlobalUserState<'static>;

#[derive(Clone)]
pub struct Context<Args> {
    pub args: Args,
    pub config: Config, // TODO use a watch here
    state: Arc<RwLock<State>>,
}

impl<Args> Context<Args> {
    pub(super) fn new(args: Args, state: Arc<RwLock<State>>, config: Config) -> Self {
        Self {
            args,
            config,
            state,
        }
    }

    pub async fn get_our_user(&self) -> crate::User<'_> {
        let state = self.state().await;
        let info = state
            .expect_get::<ShakenInfo>()
            .expect("this must always be valid");

        crate::User {
            name: info
                .display_name
                .as_ref()
                .expect("display name must be set")
                .clone(),
            id: info.user_id.parse().expect("twitch to have valid types"),
        }
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
}

macro_rules! common_short_hand {
    ($($ty:ident),* $(,)?) => {
        $(
            impl Context<crate::$ty> {
                pub fn data(&self) -> &str {
                    &*self.args.message.data
                }

                pub fn user(&self) -> User<'_> {
                    self.args.user()
                }

                pub fn room(&self) -> Room<'_> {
                    self.args.room()
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
