use crate::{Config, Directories};

use anyhow::Context as _;
use template::{FileStore, MemoryStore, PartialStore};

use std::path::{Path, PathBuf};
use std::process::exit;

type DefaultTemplateStore = PartialStore<MemoryStore, Option<FileStore>>;

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

const DEFAULT_TEMPLATES_BODY: &str = include_str!("../default_templates.toml");

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
        Some("config") => handle_edit_config(),
        Some("dirs") => handle_dirs(),
        Some("dump") => handle_dump(),
        Some("edit") => handle_edit_templates(),
        Some("init") => handle_init(),
        Some("templates") => handle_templates(),
        Some("verify") => handle_verify_and_exit(),
        Some(cmd) => handle_unknown_command(cmd),
        None => handle_verify(),
    }
}

fn handle_init() -> ! {
    fn write_default(path: &Path, name: &str, write: fn(&Path) -> anyhow::Result<()>) -> bool {
        if path.is_file() {
            eprintln!("ERROR! {} file already exists at..", name);
            println!("{}", path.display());
            println!();
            return true;
        }

        println!("writing {} file to..", name);
        println!("{}", path.display());
        println!();

        match write(path) {
            Err(err) => {
                eprintln!("ERROR! cannot write default {} file: {}", name, err);
                true
            }
            _ => false,
        }
    }

    let path = get_config_path();

    let default = write_default(
        &path.join(CONFIG_FILE),
        "configuration",
        Config::write_default,
    );

    let templates = write_default(&path.join(USER_TEMPLATES), "user templates", |path| {
        std::fs::write(path, DEFAULT_TEMPLATES_BODY).map_err(Into::into)
    });

    if maybe_exit([default, templates].iter().copied()) {
        exit(1)
    } else {
        exit(0)
    }
}

fn handle_verify_and_exit() -> ! {
    // TODO add a visible checklist
    let _ = handle_verify();
    println!("everything checked out");
    exit(0);
}

fn handle_verify() -> (Config, DefaultTemplateStore) {
    fn try_get<F, T>(
        path: Result<PathBuf, anyhow::Error>,
        kind: &str,
        mut handle: F,
    ) -> anyhow::Result<T>
    where
        F: FnMut(&Path) -> anyhow::Result<T>,
    {
        path.with_context(|| format!("cannot find {} file path", kind))
            .and_then(|path| {
                handle(&path).with_context(|| {
                    format!("cannot verify {} file at\n--> {}", kind, path.display())
                })
            })
    }

    let config = try_get(
        Directories::config().map(|path| path.join(CONFIG_FILE)),
        "configuration",
        |path| Config::load(&path),
    );

    let templates = try_get(
        Directories::config().map(|path| path.join(USER_TEMPLATES)),
        "user templates",
        |path| {
            let default = MemoryStore::new(DEFAULT_TEMPLATES_BODY, template::load_toml);
            let user = FileStore::new(path.into(), template::load_toml).ok();
            Ok(PartialStore::new(default, user))
        },
    );

    match (config, templates) {
        (Ok(config), Ok(templates)) => (config, templates),
        (left, right) => {
            if let Err(err) = left {
                crate::util::print_backtrace(err)
            }
            if let Err(err) = right {
                crate::util::print_backtrace(err)
            }
            exit(1)
        }
    }
}

fn handle_dirs() -> ! {
    let config = get_config_path();
    println!("configuration directory is at..");
    println!("{}", config.display());
    println!();

    let data = get_data_path();
    println!("data directory is at..");
    println!("{}", data.display());
    println!();

    exit(0)
}

fn handle_dump() -> ! {
    use std::io::{copy, BufReader, Cursor};
    use std::process::{Command, Output, Stdio};

    let db = get_data_path().join("database.db");

    // sqlite3 database.db .dump
    // if -cmd is used then it waits for stdin for its repl
    // and piping dev null to stdin doesn't stop this
    match Command::new("sqlite3")
        .args(&[db.to_string_lossy().as_ref(), ".dump"])
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .spawn()
        .and_then(|cmd| cmd.wait_with_output())
        .unwrap_or_exit(|err| eprintln!("ERROR: failed to run the `sqlite3` command: {}", err))
    {
        Output { status, stdout, .. } if status.success() => {
            let output = std::io::stdout();
            let mut output = output.lock();

            copy(&mut BufReader::new(Cursor::new(stdout)), &mut output).unwrap_or_exit(|err| {
                eprintln!("ERROR: cannot copy `sqlite3` output to stdout: {}", err);
            });

            exit(0)
        }
        Output { status, stderr, .. } => {
            eprintln!(
                "ERROR! `sqlite3` returned an error: {}",
                status.code().unwrap()
            );
            eprintln!("{}", String::from_utf8_lossy(&stderr));
            exit(1);
        }
    };
}

// TODO do some levenshtein distance to see if they made a typo
fn handle_unknown_command(cmd: &str) -> ! {
    eprintln!("unknown subcommand: {}", cmd);
    exit(1);
}

fn handle_templates() -> ! {
    println!("{}", DEFAULT_TEMPLATES_BODY);
    exit(0);
}

fn handle_edit_templates() -> ! {
    run_editor_for(USER_TEMPLATES);
}

fn handle_edit_config() -> ! {
    run_editor_for(CONFIG_FILE);
}

fn run_editor_for(file: &str) -> ! {
    let file = get_config_path().join(file);
    let file = file.to_string_lossy();
    let editor = std::env::var("EDITOR").unwrap_or_else(|_| "notepad".into());
    std::process::Command::new(&editor)
        .arg(file.as_ref())
        .spawn()
        .unwrap_or_exit(|err| eprintln!("error running `{}`: {}", editor, err));
    exit(0)
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
