#[macro_export]
macro_rules! maybe {
    ($expr:expr) => {{
        match $expr {
            Some(d) => d,
            None => {
                use crate::util::TypeName as _;
                let name = $expr.type_name();
                maybe!(@LOG &name, "")
            }
        }
    }};

    ($expr:expr, $fmt:expr) => {{
        match $expr {
            Some(d) => d,
            None => {
                use crate::util::TypeName as _;
                let name = $expr.type_name();
                maybe!(@LOG &name, $expr.type_name(), $fmt)
            }
        }
    }};

    ($expr:expr, $fmt:expr, $($args:expr),* $(,)?) => {{
        match $expr {
            Some(d) => d,
            None => {
                use crate::util::TypeName as _;
                let name = $expr.type_name();
                maybe!(@LOG &name, format_args!($fmt, $($args),*))
            }
        }
    }};

    (@LOG $ty:expr, $msg:expr) => {{
        match $msg.is_empty() {
            true => {
                log::trace!(
                    "expected a '{}' @ {}:{}:{} ({})",
                    $ty,
                    file!(),
                    line!(),
                    column!(),
                    module_path!(),
                );
            },
            false => {
                log::trace!(
                    "expected a '{}': {} @ {}:{}:{} ({})",
                    $ty,
                    $msg,
                    file!(),
                    line!(),
                    column!(),
                    module_path!(),
                );
            }
        }
        return Ok(());
    }}
}

#[doc(inline)]
pub use template::{markings, Template};

pub mod args;

mod bot;
pub use bot::Bot;

mod command;
pub use command::{Command, CommandMap};

pub mod config;
pub use config::{Config, Shakespeare};

mod context;
pub use context::Context;

pub mod database;

mod directories;
pub use directories::Directories;

pub mod format;
pub use format::Timestamp;

mod handler;
pub use handler::Handler;

pub mod http;

pub mod kv;

pub mod modules;

mod name_and_id;
pub use name_and_id::NameAndId;

mod passive;
pub use passive::{Passive, PassiveList};

pub mod resolver;
pub use resolver::Resolver;

mod responder;
pub use responder::{LoggingResponder, RespondableContext, Responder, WriterResponder};

mod room;
pub use room::Room;

pub mod serde_util;

mod state;
pub use state::{State, StateRef};

mod tracker;
pub use tracker::Tracker;

pub mod twitch;

mod user;
pub use user::User;

pub mod util;
use util::DontCare as _;

mod watcher;
pub use watcher::Watcher;
