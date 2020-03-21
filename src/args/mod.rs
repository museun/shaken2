use crate::{Config, Directories};

use anyhow::Context as _;
use template::{FileStore, MemoryStore, PartialStore};

use std::path::{Path, PathBuf};
use std::process::exit;

mod dirs;
mod dump;
mod edit;
mod init;
mod templates;
mod unknown;
mod verify;

pub type DefaultTemplateStore = PartialStore<MemoryStore, Option<FileStore>>;

static HELP_MESSAGE: &str = "
flags:
    -h, --help      prints this message
    -v, --version   prints the version

subcommands:
    config          opens the `shaken.toml` in your editor
    dirs            prints the configuration and data directories
    dump            dump the database to stdout (schema)
    edit            opens the `user_templates.toml` in your editor
    init            initialize the config files
    templates       print out the default templates
";

const CONFIG_FILE: &str = "shaken.toml";
const USER_TEMPLATES: &str = "user_templates.toml";

const DEFAULT_TEMPLATES_BODY: &str = include_str!("../../default_templates.toml");

pub fn handle_args() -> (Config, DefaultTemplateStore) {
    let mut args = pico_args::Arguments::from_env();

    if args.contains(["-h", "--help"]) {
        println!("shaken. revision: {}", env!("SHAKEN_GIT_REVISION"));
        println!("{}", HELP_MESSAGE);
        exit(0);
    }

    if args.contains(["-v", "--version"]) {
        println!("revision: {}", env!("SHAKEN_GIT_REVISION"));
        exit(0);
    }

    let cmd = args.subcommand();
    args.finish().unwrap_or_exit(|err| {
        eprintln!("invalid arguments provided: {}", err);
    });

    let cmd = cmd
        .as_ref()
        .map(|s| s.as_deref())
        // this happens when its not valid utf-8
        .unwrap_or_exit(|err| eprintln!("cannot parse subcommand: {}", err));

    match cmd {
        Some("config") => edit::config(),
        Some("edit") => edit::templates(),

        Some("dirs") => dirs::dirs(),
        Some("dump") => dump::dump(),

        Some("init") => init::init(),
        Some("templates") => templates::print_templates(),
        Some("verify") => verify::verify_and_exit(),
        Some(cmd) => unknown::command(cmd),
        None => verify::verify(),
    }
}

fn get_config_path() -> PathBuf {
    Directories::config().unwrap_or_exit(|err| {
        eprintln!("ERROR! cannot get configuration directory: {}", err);
    })
}

fn get_data_path() -> PathBuf {
    Directories::data().unwrap_or_exit(|err| {
        eprintln!("ERROR! cannot get data directory: {}", err);
    })
}

fn maybe_exit(maybes: impl IntoIterator<Item = bool>) -> bool {
    let mut iter = maybes.into_iter();
    let head = iter.next().unwrap();
    iter.fold(head, |a, c| a | c)
}

trait ResultExt<T, E> {
    fn unwrap_or_exit<F>(self, quit: F) -> T
    where
        F: FnOnce(E);
}

impl<T, E> ResultExt<T, E> for Result<T, E> {
    fn unwrap_or_exit<F>(self, quit: F) -> T
    where
        F: FnOnce(E),
    {
        self.unwrap_or_else(|err| {
            quit(err);
            exit(1);
        })
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn maybe_exit() {
        let tests = vec![
            (vec![true], true),
            (vec![false], false),
            (vec![true, false], true),
            (vec![true, true], true),
            (vec![false, false], false),
            (vec![true, false, false], true),
            (vec![true, true, true], true),
            (vec![false, false, false], false),
        ];

        for (input, output) in tests {
            assert_eq!(super::maybe_exit(input), output);
        }
    }
}
