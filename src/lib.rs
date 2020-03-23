#![cfg_attr(debug_assertions, allow(dead_code))]

#[macro_use]
mod macros;

#[doc(inline)]
pub use template::{markings, Template};

pub mod args;

mod bot;
pub use bot::Bot;

pub mod config;
pub use config::*;

mod context;
use context::Context;

mod store;
pub use store::resolver;
use store::{Resolver, State};

mod format;
pub use format::Timestamp;

mod handler;
pub use handler::{Command, CommandMap, DynHandler, Handler, Passive, PassiveList};

mod http;
pub use http::twitch::Client as TwitchClient;

pub mod modules;

mod name_and_id;
use name_and_id::NameAndId;

mod responder;

pub use responder::{LoggingResponder, NullResponder, WriterResponder};
use responder::{RespondableContext, Responder};

mod room;
use room::Room;

mod user;
use user::User;

pub mod util;
use util::{dont_care, DontCare as _};

mod watcher;
