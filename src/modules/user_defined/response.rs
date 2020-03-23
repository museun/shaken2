use crate::*;

#[derive(Debug, Template)]
#[namespace("user_defined")]
pub enum Response<'a> {
    ErrorReservedName { command: &'a str },
    ErrorAlreadyExists { command: &'a str },
    ErrorCommandNotFound { command: &'a str },
    ErrorInsufficientPrivlege { command: &'a str },

    ErrorMissingHead,
    ErrorMissingTail { head: &'a str },

    Added { command: &'a str },
    Edited { command: &'a str },
    Renamed { from: &'a str, to: &'a str },
    Deleted,

    Say { data: &'a str },

    CommandDescription { command: &'a str, body: &'a str },
    CommandCreatedAt { owner: &'a str, uses: u64 },
}
