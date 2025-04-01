// Inspired by https://github.com/ryanabx/flatpak-unsandbox/
use anyhow::{anyhow, bail, Context};
use configparser::ini::Ini;
use lact_schema::args::FlatpakCommand;
use std::{
    path::{Component, Path, PathBuf},
    process::Command,
};

pub fn cmd(cmd: FlatpakCommand) -> anyhow::Result<()> {
    let info = get_flatpak_info()?;
    match cmd {
        FlatpakCommand::GenerateDaemonCmd => generate_daemon_cmd(&info),
    }
}

fn generate_daemon_cmd(info: &Ini) -> anyhow::Result<()> {
    let app_path = info
        .get("Instance", "app-path")
        .context("Could not get app path")?;

    let runtime_path = info
        .get("Instance", "runtime-path")
        .context("Could not get runtime path")?;

    let relative_ld_path = get_relative_ld_path()?;
    let relative_ld_path_parts = relative_ld_path.components().collect::<Vec<Component>>();

    let ld_path = format!("{runtime_path}/{}", relative_ld_path.display());

    let runtime_lib_path = format!(
        "{runtime_path}/{}",
        relative_ld_path_parts[0..relative_ld_path_parts.len() - 1]
            .iter()
            .collect::<PathBuf>()
            .display()
    );
    let app_bin_path = format!("{app_path}/bin");
    let library_paths = format!("{app_path}/lib:{runtime_lib_path}");

    println!("{ld_path} --library-path {library_paths} {app_bin_path}/lact daemon");

    Ok(())
}

fn get_relative_ld_path() -> anyhow::Result<PathBuf> {
    let out = Command::new("ldconfig").arg("-p").output()?;
    for l in String::from_utf8(out.stdout)?.lines() {
        if l.trim().starts_with("ld-linux") {
            let path = l
                .split(" => ")
                .nth(1)
                .context("Invalid ld-linux line")?
                .trim();
            let relative_path = Path::new(path).components().skip(2).collect();
            return Ok(relative_path);
        }
    }
    bail!("Could not find ld-linux inside of flatpak");
}

fn get_flatpak_info() -> anyhow::Result<Ini> {
    let info_path = Path::new("/.flatpak-info");

    if !info_path.exists() {
        bail!("Not running inside of Flatpak");
    }

    let mut info = Ini::new();
    info.load(info_path)
        .map_err(|err| anyhow!("Could not read flatpak info: {err}"))?;

    Ok(info)
}
