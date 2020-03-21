use super::*;

pub fn config() -> ! {
    run_editor_for(CONFIG_FILE);
}

pub fn templates() -> ! {
    run_editor_for(USER_TEMPLATES);
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
