fn main() {
    println!(
        "cargo:rustc-env=SHAKEN_GIT_TAG={}",
        git_tag()
            .as_deref()
            .unwrap_or_else(|| env!("CARGO_PKG_VERSION"))
    );
    let rev = git_rev();
    let rev = rev.as_deref().unwrap_or("unknown");
    println!("cargo:rustc-env=SHAKEN_GIT_REVISION={}", rev);
    println!(
        "cargo:rustc-env=SHAKEN_USER_AGENT={}",
        format!("shaken_bot/{} (github.com/museun/shaken)", rev,)
    );

    // TODO initialize the reference DB here
    // TODO set the DATABASE_URL here (the code uses a path)
}

fn get_git(args: &[&str]) -> Option<String> {
    std::process::Command::new("git")
        .args(args)
        .output()
        .ok()
        .and_then(|data| {
            std::str::from_utf8(&data.stdout)
                .ok()
                .map(|s| s.trim())
                .map(ToString::to_string)
        })
        .filter(|s| !s.is_empty())
}

fn git_tag() -> Option<String> {
    get_git(&["describe", "--tags", "--abbrev=0"])
}

fn git_rev() -> Option<String> {
    get_git(&["rev-parse", "--short=12", "HEAD"])
}
