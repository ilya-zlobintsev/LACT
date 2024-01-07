use anyhow::{anyhow, Context};
use lact_schema::{InitramfsType, SystemInfo};
use os_release::{OsRelease, OS_RELEASE};
use std::{
    fs::{self, File, Permissions},
    io::Write,
    os::unix::prelude::PermissionsExt,
    process::Command,
};
use tracing::{info, warn};

const PP_OVERDRIVE_MASK: u64 = 0x4000;
pub const PP_FEATURE_MASK_PATH: &str = "/sys/module/amdgpu/parameters/ppfeaturemask";
pub const MODULE_CONF_PATH: &str = "/etc/modprobe.d/99-amdgpu-overdrive.conf";

pub fn info() -> anyhow::Result<SystemInfo<'static>> {
    let version = env!("CARGO_PKG_VERSION");
    let profile = if cfg!(debug_assertions) {
        "debug"
    } else {
        "release"
    };
    let kernel_output = Command::new("uname")
        .arg("-r")
        .output()
        .context("Could not read kernel version")?;
    let kernel_version = String::from_utf8(kernel_output.stdout)
        .context("Invalid kernel version output")?
        .trim()
        .to_owned();

    let amdgpu_overdrive_enabled = if let Ok(mask) = read_current_mask() {
        Some((mask & PP_OVERDRIVE_MASK) > 0)
    } else {
        None
    };

    Ok(SystemInfo {
        version,
        profile,
        kernel_version,
        amdgpu_overdrive_enabled,
    })
}

pub fn enable_overdrive() -> anyhow::Result<String> {
    let current_mask = read_current_mask()?;

    let new_mask = current_mask | PP_OVERDRIVE_MASK;
    if new_mask == current_mask {
        return Err(anyhow!("Overdrive mask already enabled"));
    }

    let conf = format!("options amdgpu ppfeaturemask=0x{new_mask:X}");

    let mut file = File::create(MODULE_CONF_PATH).context("Could not open module conf file")?;
    file.set_permissions(Permissions::from_mode(0o644))
        .context("Could not conf file permissions")?;

    file.write_all(conf.as_bytes())
        .context("Could not write config")?;

    let message = match regenerate_initramfs() {
        Ok(initramfs_type) => {
            format!("Initramfs was successfully regenerated (detected type {initramfs_type:?})")
        }
        Err(err) => format!("{err:#}"),
    };

    Ok(message)
}

fn read_current_mask() -> anyhow::Result<u64> {
    let ppfeaturemask = fs::read_to_string(PP_FEATURE_MASK_PATH)?;
    let ppfeaturemask = ppfeaturemask
        .trim()
        .strip_prefix("0x")
        .context("Invalid ppfeaturemask")?;

    u64::from_str_radix(ppfeaturemask, 16).context("Invalid ppfeaturemask")
}

fn regenerate_initramfs() -> anyhow::Result<InitramfsType> {
    let os_release = OS_RELEASE.as_ref().context("Could not detect distro")?;
    match detect_initramfs_type(os_release) {
        Some(initramfs_type) => {
            info!("Detected initramfs type {initramfs_type:?}, regenerating");
            let result = match initramfs_type {
                InitramfsType::Debian => run_command("update-initramfs", &["-u"]),
                InitramfsType::Mkinitcpio => run_command("mkinitcpio", &["-P"]),
            };
            result.map(|()| initramfs_type)
        }
        None => Err(anyhow!(
            "Could not determine initramfs type, manual initramfs regeneration may be required"
        )),
    }
}

fn detect_initramfs_type(os_release: &OsRelease) -> Option<InitramfsType> {
    let id_like: Vec<_> = os_release.id_like.split_whitespace().collect();

    if os_release.id == "debian" || id_like.contains(&"debian") {
        Some(InitramfsType::Debian)
    } else if os_release.id == "arch" || id_like.contains(&"arch") {
        if Command::new("mkinitcpio").arg("--version").output().is_ok() {
            Some(InitramfsType::Mkinitcpio)
        } else {
            warn!(
                "Arch-based system with no mkinitcpio detected, refusing to regenerate initramfs"
            );
            None
        }
    } else {
        None
    }
}

fn run_command(exec: &str, args: &[&str]) -> anyhow::Result<()> {
    info!("Running {exec} with args {args:?}");
    let output = Command::new(exec)
        .args(args)
        .output()
        .context("Could not run command")?;
    if output.status.success() {
        Ok(())
    } else {
        let stdout = String::from_utf8(output.stdout).context("stdout is not valid UTF-8")?;
        let stderr = String::from_utf8(output.stderr).context("stderr is not valid UTF-8")?;
        Err(anyhow!("Command exited with error: {stdout}\n{stderr}"))
    }
}

#[cfg(test)]
mod tests {
    use crate::server::system::detect_initramfs_type;
    use lact_schema::InitramfsType;
    use os_release::OsRelease;

    #[test]
    fn detect_initramfs_debian() {
        let data = r#"
PRETTY_NAME="Debian GNU/Linux trixie/sid"
NAME="Debian GNU/Linux"
VERSION_CODENAME=trixie
ID=debian
HOME_URL="https://www.debian.org/"
SUPPORT_URL="https://www.debian.org/support"
BUG_REPORT_URL="https://bugs.debian.org/"
        "#;
        let os_release: OsRelease = data.lines().map(str::to_owned).collect();
        assert_eq!(
            Some(InitramfsType::Debian),
            detect_initramfs_type(&os_release)
        );
    }

    #[test]
    fn detect_initramfs_mint() {
        let data = r#"
NAME="Linux Mint"
VERSION="21.2 (Victoria)"
ID=linuxmint
ID_LIKE="ubuntu debian"
PRETTY_NAME="Linux Mint 21.2"
VERSION_ID="21.2"
HOME_URL="https://www.linuxmint.com/"
SUPPORT_URL="https://forums.linuxmint.com/"
BUG_REPORT_URL="http://linuxmint-troubleshooting-guide.readthedocs.io/en/latest/"
PRIVACY_POLICY_URL="https://www.linuxmint.com/"
VERSION_CODENAME=victoria
UBUNTU_CODENAME=jammy
        "#;
        let os_release: OsRelease = data.lines().map(str::to_owned).collect();
        assert_eq!(
            Some(InitramfsType::Debian),
            detect_initramfs_type(&os_release)
        );
    }
}
