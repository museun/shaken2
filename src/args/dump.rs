use super::*;

pub fn dump() -> ! {
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
