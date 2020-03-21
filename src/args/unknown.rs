use super::*;

// TODO do some levenshtein distance to see if they made a typo
pub fn command(cmd: &str) -> ! {
    eprintln!("unknown subcommand: {}", cmd);
    exit(1);
}
