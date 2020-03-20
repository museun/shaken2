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
pub mod secrets;

mod bot;
pub use bot::Bot;

mod command;
use command::Command;
pub use command::CommandMap;

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
use passive::Passive;
pub use passive::PassiveList;

pub mod resolver;
use resolver::Resolver;

mod responder;
pub use responder::{LoggingResponder, WriterResponder};
use responder::{RespondableContext, Responder};

mod room;
use room::Room;

mod serde_util;

mod state;
pub use state::State;

mod tracker;
use tracker::Tracker;

mod twitch;

mod user;
use user::User;

mod util;
use util::DontCare as _;

mod watcher;
