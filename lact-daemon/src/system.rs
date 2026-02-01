pub mod power_profiles_daemon;

use anyhow::{Context, anyhow, bail, ensure};
use lact_schema::{
    AmdgpuParamsConfigurator, BootArgConfigurator, GIT_COMMIT, InitramfsType, SystemInfo,
};
use nix::sys::{
    socket::{
        AddressFamily, MsgFlags, NetlinkAddr, SockFlag, SockProtocol, SockType, bind, recv, socket,
    },
    utsname::uname,
};
use os_release::OsRelease;
use std::{
    env,
    fs::{self, File, Permissions},
    io::{self, Write},
    iter,
    os::{fd::AsRawFd, unix::prelude::PermissionsExt},
    path::{Path, PathBuf},
    process::{self, Output},
    sync::{
        LazyLock,
        atomic::{AtomicBool, Ordering},
    },
};
use tokio::{process::Command, sync::Notify};
use tracing::{debug, error, info};

static OC_TOGGLED: AtomicBool = AtomicBool::new(false);

const PP_OVERDRIVE_MASK: u64 = 0x4000;
pub const PP_FEATURE_MASK_PATH: &str = "/sys/module/amdgpu/parameters/ppfeaturemask";
pub const BASE_MODULE_CONF_PATH: &str = "/etc/modprobe.d/99-amdgpu-overdrive.conf";
pub const DAEMON_VERSION: &str = env!("CARGO_PKG_VERSION");

pub static IS_FLATBOX: LazyLock<bool> =
    LazyLock::new(|| env::var("FLATBOX_ENV").as_deref() == Ok("1"));
static MODULE_CONF_PATH: LazyLock<PathBuf> = LazyLock::new(|| {
    if *IS_FLATBOX {
        Path::new("/run/host/root").join(BASE_MODULE_CONF_PATH.strip_prefix('/').unwrap())
    } else {
        PathBuf::from(BASE_MODULE_CONF_PATH)
    }
});

pub async fn info() -> anyhow::Result<SystemInfo> {
    let version = DAEMON_VERSION.to_owned();
    let profile = if cfg!(debug_assertions) {
        "debug"
    } else {
        "release"
    }
    .to_owned();

    let kernel_version = uname().map_or_else(
        |err| {
            error!("could not fetch kernel version: {err}");
            "<Unknown>".to_owned()
        },
        |info| info.release().to_string_lossy().into_owned(),
    );

    let amdgpu_overdrive_enabled = if let Ok(mask) = read_current_mask() {
        Some((mask & PP_OVERDRIVE_MASK) > 0)
    } else {
        None
    };

    let os_release = get_os_release().inspect_err(|err| error!("Could not detect distro: {err}"));

    Ok(SystemInfo {
        version,
        profile,
        kernel_version,
        distro: os_release.as_ref().map(|release| release.name.clone()).ok(),
        amdgpu_overdrive_enabled,
        commit: Some(GIT_COMMIT.to_owned()),
        amdgpu_params_configurator: match os_release {
            Ok(release) => detect_amdgpu_configurator(&release).await.ok(),
            Err(_) => None,
        },
    })
}

pub async fn enable_overdrive() -> anyhow::Result<String> {
    ensure!(
        !OC_TOGGLED.load(Ordering::SeqCst),
        "Overdrive support was already toggled - please reboot to apply the changes"
    );

    let current_mask = read_current_mask().context("Could not get current amdgpu feature mask")?;

    let new_mask = current_mask | PP_OVERDRIVE_MASK;
    if new_mask == current_mask {
        return Err(anyhow!("Overdrive mask already enabled"));
    }

    let os_release = get_os_release()?;
    let configurator = detect_amdgpu_configurator(&os_release).await?;
    match configurator {
        AmdgpuParamsConfigurator::Modprobe(initramfs_type) => {
            let conf = format!("options amdgpu ppfeaturemask=0x{new_mask:X}");

            let mut file =
                File::create(&*MODULE_CONF_PATH).context("Could not open module conf file")?;
            file.set_permissions(Permissions::from_mode(0o644))
                .context("Could not conf file permissions")?;

            file.write_all(conf.as_bytes())
                .context("Could not write config")?;

            if let Some(initramfs) = initramfs_type {
                regenerate_initramfs(initramfs).await?;
            }
        }
        AmdgpuParamsConfigurator::BootArg(BootArgConfigurator::RpmOstree) => {
            run_command(
                "rpm-ostree",
                &[
                    "kargs",
                    &format!("--append-if-missing=amdgpu.ppfeaturemask=0x{new_mask:X}"),
                ],
            )
            .await?;
        }
    }

    OC_TOGGLED.store(true, Ordering::SeqCst);

    // Returning a string only to maintain API compat
    Ok(String::new())
}

pub async fn disable_overdrive() -> anyhow::Result<String> {
    ensure!(
        !OC_TOGGLED.load(Ordering::SeqCst),
        "Overdrive support was already toggled - please reboot to apply the changes"
    );

    let os_release = get_os_release()?;
    let configurator = detect_amdgpu_configurator(&os_release).await?;
    match configurator {
        AmdgpuParamsConfigurator::Modprobe(initramfs_type) => {
            if Path::new(&*MODULE_CONF_PATH).exists() {
                fs::remove_file(&*MODULE_CONF_PATH)
                    .context("Could not remove module config file")?;

                if let Some(initramfs) = initramfs_type {
                    regenerate_initramfs(initramfs).await?;
                }
            } else {
                bail!(
                    "Overclocking was not enabled through LACT (file at {} does not exist)",
                    MODULE_CONF_PATH.display(),
                );
            }
        }
        AmdgpuParamsConfigurator::BootArg(BootArgConfigurator::RpmOstree) => {
            let current_mask =
                read_current_mask().context("Could not get current amdgpu feature mask")?;
            run_command(
                "rpm-ostree",
                &[
                    "kargs",
                    &format!("--delete-if-present=amdgpu.ppfeaturemask=0x{current_mask:X}"),
                ],
            )
            .await?;
        }
    }

    OC_TOGGLED.store(true, Ordering::SeqCst);

    // Returning a string only to maintain API compat
    Ok(String::new())
}

async fn regenerate_initramfs(initramfs_type: InitramfsType) -> anyhow::Result<()> {
    info!("detected initramfs type {initramfs_type:?}, regenerating");
    let result = match initramfs_type {
        InitramfsType::Debian => run_command("update-initramfs", &["-u"]).await,
        InitramfsType::Mkinitcpio => run_command("mkinitcpio", &["-P"]).await,
        InitramfsType::Dracut => run_command("dracut", &["--regenerate-all", "--force"]).await,
    };
    result.context("Initramfs generation command failed")?;
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

pub(crate) async fn detect_amdgpu_configurator(
    os_release: &OsRelease,
) -> anyhow::Result<AmdgpuParamsConfigurator> {
    let ids = iter::once(os_release.id.as_str()).chain(os_release.id_like.split_ascii_whitespace());

    for id in ids {
        match id {
            "debian" => {
                return Ok(AmdgpuParamsConfigurator::Modprobe(Some(
                    InitramfsType::Debian,
                )));
            }
            "arch" | "cachyos" if run_command("mkinitcpio", &["--version"]).await.is_ok() => {
                return Ok(AmdgpuParamsConfigurator::Modprobe(Some(
                    InitramfsType::Mkinitcpio,
                )));
            }
            "fedora" if Path::new("/run/ostree-booted").exists() => {
                return Ok(AmdgpuParamsConfigurator::BootArg(
                    BootArgConfigurator::RpmOstree,
                ));
            }
            "fedora" if run_command("dracut", &["--version"]).await.is_ok() => {
                return Ok(AmdgpuParamsConfigurator::Modprobe(Some(
                    InitramfsType::Dracut,
                )));
            }
            "nixos" => {
                return Err(anyhow!(
                    "Overdrive should be toggled through system config on NixOS"
                ));
            }
            _ => (),
        }
    }

    Ok(AmdgpuParamsConfigurator::Modprobe(None))
}

pub fn get_os_release() -> io::Result<OsRelease> {
    let release = if *IS_FLATBOX {
        OsRelease::new_from("/run/host/root/etc/os-release")
    } else {
        OsRelease::new()
    };
    debug!("read os-release info: {release:?}");
    release
}

pub async fn run_command(exec: &str, args: &[&str]) -> anyhow::Result<Output> {
    debug!("running {exec} with args {args:?}");

    let mut command;
    if *IS_FLATBOX {
        command = Command::new("flatpak-spawn");
        command.arg("--host").arg(exec).args(args);
    } else {
        command = Command::new(exec);
        command.args(args);
    }

    let output = command.output().await.context("Could not run command")?;
    if output.status.success() {
        Ok(output)
    } else {
        let stdout = String::from_utf8(output.stdout).context("stdout is not valid UTF-8")?;
        let stderr = String::from_utf8(output.stderr).context("stderr is not valid UTF-8")?;
        Err(anyhow!("Command exited with error: {stdout}\n{stderr}"))
    }
}

pub(crate) fn listen_netlink_kernel_event(notify: &Notify) -> anyhow::Result<()> {
    let socket = socket(
        AddressFamily::Netlink,
        SockType::Raw,
        SockFlag::empty(),
        SockProtocol::NetlinkKObjectUEvent,
    )
    .context("Could not setup netlink socket")?;

    let sa = NetlinkAddr::new(process::id(), 1);
    bind(socket.as_raw_fd(), &sa).context("Could not bind netlink socket")?;

    let mut buf = Vec::new();
    loop {
        // Read only the size using the peek and truncate flags first
        let msg_size = recv(
            socket.as_raw_fd(),
            &mut [],
            MsgFlags::MSG_PEEK | MsgFlags::MSG_TRUNC,
        )
        .context("Could not read netlink message")?;
        buf.clear();
        buf.resize(msg_size, 0);

        // Read the actual message into the buffer
        recv(socket.as_raw_fd(), &mut buf, MsgFlags::empty())
            .context("Could not read netlink message")?;

        for raw_line in buf.split(|c| *c == b'\0') {
            match std::str::from_utf8(raw_line) {
                Ok(line) => {
                    if line.is_empty() {
                        continue;
                    }

                    if let Some(subsystem) = line.strip_prefix("SUBSYSTEM=")
                        && subsystem == "drm"
                    {
                        notify.notify_one();
                    }
                }
                Err(_) => {
                    error!(
                        "Got invalid unicode in uevent line {}",
                        String::from_utf8_lossy(raw_line)
                    );
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::detect_amdgpu_configurator;
    use lact_schema::{AmdgpuParamsConfigurator, InitramfsType};
    use os_release::OsRelease;

    #[tokio::test]
    async fn detect_initramfs_debian() {
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
            AmdgpuParamsConfigurator::Modprobe(Some(InitramfsType::Debian)),
            detect_amdgpu_configurator(&os_release).await.unwrap()
        );
    }

    #[tokio::test]
    async fn detect_initramfs_mint() {
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
            AmdgpuParamsConfigurator::Modprobe(Some(InitramfsType::Debian)),
            detect_amdgpu_configurator(&os_release).await.unwrap()
        );
    }
}
