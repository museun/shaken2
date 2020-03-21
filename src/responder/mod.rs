use crate::{Room, User};

use futures::prelude::*;
use template::Template;
use twitchchat::Writer;

use crate::{Context, Resolver};

mod logging;
pub use logging::LoggingResponder;

mod writer;
pub use writer::WriterResponder;

pub trait RespondableContext {
    fn room(&self) -> Room<'_>;
    fn user(&self) -> User<'_>;
}

pub type AnyhowFut<'a> = future::BoxFuture<'a, anyhow::Result<()>>;

pub trait Responder: Clone {
    fn say<'a, T, K>(&mut self, context: &'a Context<K>, template: &'a T) -> AnyhowFut<'a>
    where
        T: Template + Send + Sync,
        K: RespondableContext + Send + Sync + 'static;

    fn reply<'a, T, K>(&mut self, context: &'a Context<K>, template: &'a T) -> AnyhowFut<'a>
    where
        T: Template + Send + Sync,
        K: RespondableContext + Send + Sync + 'static;

    fn action<'a, T, K>(&mut self, context: &'a Context<K>, template: &'a T) -> AnyhowFut<'a>
    where
        T: Template + Send + Sync,
        K: RespondableContext + Send + Sync + 'static;
}

// pub(crate) async fn get_user(&self, target: u64) -> Result<String, ResponderError> {
//     self.tracker
//         .users
//         .get(target)
//         .await
//         .ok_or_else(|| ResponderError::InvalidRoom(target))
// }

// pub(crate) async fn get_room(&self, target: u64) -> Result<String, ResponderError> {
//     self.tracker
//         .rooms
//         .get(target)
//         .await
//         .ok_or_else(|| ResponderError::InvalidRoom(target))
// }
