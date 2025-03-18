use std::{env, process::Command};

fn main() {
    println!("cargo::rerun-if-changed=../.git/");
    println!("cargo::rerun-if-env-changed=LACT_GIT_REV");

    let rev = env::var("LACT_GIT_REV")
        .ok()
        .filter(|s| !s.is_empty())
        .or_else(|| {
            run_git_command(&["tag", "--points-at", "HEAD"]).filter(|tag| tag.starts_with('v'))
        })
        .or_else(|| run_git_command(&["rev-parse", "--short", "HEAD"]));

    println!(
        "cargo::rustc-env=GIT_COMMIT={}",
        rev.unwrap_or_else(|| "unknown".to_owned())
    );
}

fn run_git_command(args: &[&str]) -> Option<String> {
    Command::new("git")
        .args(args)
        .output()
        .ok()
        .and_then(|out| String::from_utf8(out.stdout).ok())
}
