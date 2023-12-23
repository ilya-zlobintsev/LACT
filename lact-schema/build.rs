use std::process::Command;

fn main() {
    let commit_output = Command::new("git")
        .args(["rev-parse", "--short", "HEAD"])
        .output()
        .expect("Could not read current git commit");
    let version = String::from_utf8(commit_output.stdout).unwrap();

    println!("cargo:rustc-env=GIT_COMMIT={}", version);
}
