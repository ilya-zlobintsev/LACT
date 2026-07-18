mod mock_fs;

use crate::{
    config::Config,
    server::handler::{Handler, read_pci_db},
};
use insta::{assert_debug_snapshot, assert_json_snapshot};
use lact_schema::config::GpuConfig;
use mock_fs::MockSysfs;
use std::{fs, path::PathBuf, sync::OnceLock};
use tempfile::tempdir;

fn init_tracing() {
    static TRACING_LOCK: OnceLock<()> = OnceLock::new();
    TRACING_LOCK.get_or_init(|| {
        tracing_subscriber::fmt()
            .with_env_filter("easy_fuser=warn,info")
            .init();
    });
}

#[tokio::test]
#[cfg_attr(miri, ignore)]
async fn snapshot_everything() {
    init_tracing();

    let test_data_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../tests/snapshots");
    let pci_db = read_pci_db();

    for vendor_dir in fs::read_dir(test_data_dir).unwrap().flatten() {
        if !vendor_dir.file_type().unwrap().is_dir() {
            continue;
        }

        for device_dir in fs::read_dir(vendor_dir.path()).unwrap().flatten() {
            let test_key = format!(
                "{}/{}",
                vendor_dir.file_name().to_string_lossy(),
                device_dir.file_name().to_string_lossy()
            );

            let handler = Handler::with_base_path(&device_dir.path(), Config::default(), &pci_db)
                .await
                .unwrap();
            let device_info = handler
                .generate_snapshot_device_info()
                .await
                .into_values()
                .next()
                .unwrap();

            assert_json_snapshot!(test_key, device_info);
        }
    }
}

#[tokio::test(flavor = "local")]
#[cfg_attr(miri, ignore)]
async fn apply_settings() {
    init_tracing();

    let test_data_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../tests/snapshots");
    let pci_db = read_pci_db();

    for vendor_dir in fs::read_dir(test_data_dir).unwrap().flatten() {
        if !vendor_dir.metadata().unwrap().is_dir() {
            continue;
        }

        for device_dir in fs::read_dir(vendor_dir.path()).unwrap().flatten() {
            for entry in fs::read_dir(device_dir.path()).unwrap().flatten() {
                let name = entry.file_name();
                let name = name.to_str().unwrap();

                #[allow(clippy::case_sensitive_file_extension_comparisons)]
                if name.starts_with("config") && name.ends_with(".yaml") {
                    let test_key = format!(
                        "apply_config/{}/{}/{name}",
                        vendor_dir.file_name().to_string_lossy(),
                        device_dir.file_name().to_string_lossy()
                    );
                    let raw_gpu_config = fs::read_to_string(entry.path()).unwrap();
                    let gpu_config: GpuConfig = serde_norway::from_str(&raw_gpu_config).unwrap();

                    let mock_fs_dir = tempdir().unwrap();

                    let mock_fs = MockSysfs::new(device_dir.path());
                    let writes = mock_fs.writes.clone();

                    let mount = easy_fuser::fuse_parallel::spawn_mount(
                        mock_fs,
                        mock_fs_dir.path(),
                        &[],
                        Some(1),
                    )
                    .expect("Could not mount mock fs");

                    let handler =
                        Handler::with_base_path(mock_fs_dir.path(), Config::default(), &pci_db)
                            .await
                            .unwrap();
                    let gpu_id = &handler.list_devices().await[0].id;

                    handler
                        .config
                        .write()
                        .await
                        .gpus_mut()
                        .unwrap()
                        .insert(gpu_id.clone(), gpu_config);

                    handler.apply_current_config().await.unwrap();

                    mount.join();
                    mock_fs_dir.close().unwrap();

                    let write_commands = writes
                        .lock()
                        .unwrap()
                        .iter()
                        .map(|(name, contents)| format!("{}: {contents}", name.to_str().unwrap()))
                        .collect::<Vec<String>>();
                    assert_debug_snapshot!(test_key, write_commands);
                }
            }
        }
    }
}

#[tokio::test(flavor = "local")]
#[cfg_attr(miri, ignore)]
async fn detach_and_attach_nvidia_gpu_without_recreating_other_vendors() {
    init_tracing();

    let snapshots = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../tests/snapshots/amd");
    let drm_dir = tempdir().unwrap();
    let first_card = drm_dir.path().join("card0");
    let second_card = drm_dir.path().join("card1");
    let third_card = drm_dir.path().join("card2");
    copy_dir(&snapshots.join("phoenix/card1"), &first_card);
    copy_dir(&snapshots.join("rx6600/card1"), &second_card);
    copy_dir(&snapshots.join("rx580/card0"), &third_card);
    set_driver(&first_card, "nvidia");
    set_driver(&second_card, "nvidia");

    let handler = Handler::with_base_path(drm_dir.path(), Config::default(), &read_pci_db())
        .await
        .unwrap();
    let devices = handler.list_devices().await;
    assert_eq!(devices.len(), 3);

    let detached_id = devices
        .iter()
        .find(|device| device.id.ends_with("0000:64:00.0"))
        .unwrap()
        .id
        .clone();
    let detached_index = devices
        .iter()
        .position(|device| device.id == detached_id)
        .unwrap()
        .to_string();
    let paused_id = devices
        .iter()
        .find(|device| device.id.ends_with("0000:12:00.0"))
        .unwrap()
        .id
        .clone();
    let retained_id = devices
        .iter()
        .find(|device| device.id != detached_id && device.id != paused_id)
        .unwrap()
        .id
        .clone();
    let retained_address = handler.controller_address(&retained_id).await.unwrap();

    assert!(handler.detach_gpu(None).await.is_err());
    let (first_detach, second_detach, ()) = tokio::join!(
        handler.detach_gpu(Some(&detached_index)),
        handler.detach_gpu(Some(&detached_id)),
        handler.reload_gpus()
    );
    assert_eq!(first_detach.unwrap(), detached_id);
    assert_eq!(second_detach.unwrap(), detached_id);
    assert_eq!(
        handler
            .list_devices()
            .await
            .into_iter()
            .map(|device| device.id)
            .collect::<Vec<_>>(),
        vec![retained_id.clone()]
    );

    handler.reload_gpus().await;
    assert_eq!(
        handler.controller_address(&retained_id).await,
        Some(retained_address)
    );

    fs::remove_dir_all(&first_card).unwrap();
    handler.reload_gpus().await;
    assert!(
        handler
            .list_devices()
            .await
            .iter()
            .any(|device| device.id == paused_id)
    );
    let (first_attach, second_attach, ()) = tokio::join!(
        handler.attach_gpu(Some(&detached_index)),
        handler.attach_gpu(Some(&detached_id)),
        handler.reload_gpus()
    );
    assert_eq!(first_attach.unwrap(), detached_id);
    assert_eq!(second_attach.unwrap(), detached_id);
    assert_eq!(handler.list_devices().await.len(), 2);

    copy_dir(&snapshots.join("phoenix/card1"), &first_card);
    set_driver(&first_card, "nvidia");
    handler.reload_gpus().await;
    let attached_devices = handler.list_devices().await;
    assert_eq!(attached_devices.len(), 3);
    assert!(
        attached_devices
            .iter()
            .any(|device| device.id == detached_id)
    );
    assert_eq!(
        handler.controller_address(&retained_id).await,
        Some(retained_address)
    );
    assert_eq!(
        handler.attach_gpu(Some(&detached_index)).await.unwrap(),
        detached_id
    );
}

fn copy_dir(source: &std::path::Path, destination: &std::path::Path) {
    fs::create_dir(destination).unwrap();
    for entry in fs::read_dir(source).unwrap() {
        let entry = entry.unwrap();
        let target = destination.join(entry.file_name());
        if entry.file_type().unwrap().is_dir() {
            copy_dir(&entry.path(), &target);
        } else {
            fs::copy(entry.path(), target).unwrap();
        }
    }
}

fn set_driver(card_path: &std::path::Path, driver: &str) {
    let uevent_path = card_path.join("device/uevent");
    let uevent = fs::read_to_string(&uevent_path).unwrap();
    let updated = uevent.replace("DRIVER=amdgpu", &format!("DRIVER={driver}"));
    fs::write(uevent_path, updated).unwrap();
}
