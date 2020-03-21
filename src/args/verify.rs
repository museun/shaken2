use super::*;

pub fn verify_and_exit() -> ! {
    // TODO add a visible checklist
    let _ = verify();
    println!("everything checked out");
    exit(0);
}

pub fn verify() -> (Config, DefaultTemplateStore) {
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

    fn print_backtrace(error: anyhow::Error) {
        for (i, cause) in error.chain().enumerate() {
            if i > 0 {
                eprintln!();
                eprintln!("because");
                eprint!("  ");
            }
            eprintln!("{}", cause);
        }
    }

    match (config, templates) {
        (Ok(config), Ok(templates)) => (config, templates),
        (left, right) => {
            if let Err(err) = left {
                print_backtrace(err)
            }
            if let Err(err) = right {
                print_backtrace(err)
            }
            exit(1)
        }
    }
}
