#[macro_use]
mod macros;

#[doc(inline)]
pub use template::{markings, Template};

pub mod args;
pub mod secrets;

mod bot;
pub use bot::Bot;

mod command;
use command::{Command, CommandMap};

pub mod config;
use config::Config;

mod context;
use context::Context;

pub mod database;

mod directories;
pub use directories::Directories;

mod format;
pub use format::Timestamp;

mod handler;
pub use handler::Handler;

mod http;
mod kv;

pub mod modules;

mod name_and_id;
use name_and_id::NameAndId;

mod passive;
use passive::{Passive, PassiveList};

pub mod resolver;
use resolver::Resolver;

mod responder;
pub use responder::{LoggingResponder, WriterResponder};
use responder::{RespondableContext, Responder};

mod room;
use room::Room;

mod serde_util;

mod state;
use state::State;

mod tracker;
use tracker::Tracker;

mod twitch;
pub use twitch::Client as TwitchClient;

mod user;
use user::User;

mod util;
use util::{dont_care, DontCare as _};

mod watcher;
