use anyhow::{anyhow, Context};
use lact_schema::SystemInfo;
use std::{
    fs::{self, File, Permissions},
    io::Write,
    os::unix::prelude::PermissionsExt,
    process::Command,
};

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

pub fn enable_overdrive() -> anyhow::Result<()> {
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

    Ok(())
}

fn read_current_mask() -> anyhow::Result<u64> {
    let ppfeaturemask = fs::read_to_string(PP_FEATURE_MASK_PATH)?;
    let ppfeaturemask = ppfeaturemask
        .trim()
        .strip_prefix("0x")
        .context("Invalid ppfeaturemask")?;

    u64::from_str_radix(ppfeaturemask, 16).context("Invalid ppfeaturemask")
}
