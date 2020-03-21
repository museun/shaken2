use super::*;

pub fn init() -> ! {
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
