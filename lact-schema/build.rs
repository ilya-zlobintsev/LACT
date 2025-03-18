use std::{env, process::Command};

fn main() {
    let commit = env::var("LACT_GIT_COMMIT")
        .ok()
        .filter(|s| !s.is_empty())
        .or_else(|| {
            run_git_command(&["tag", "--points-at", "HEAD"]).filter(|tag| tag.starts_with('v'))
        })
        .or_else(|| run_git_command(&["rev-parse", "--short", "HEAD"]));

    println!(
        "cargo:rustc-env=GIT_COMMIT={}",
        commit.unwrap_or_else(|| "unknown".to_owned())
    );
}

fn run_git_command(args: &[&str]) -> Option<String> {
    Command::new("git")
        .args(args)
        .output()
        .ok()
        .and_then(|out| String::from_utf8(out.stdout).ok())
}
