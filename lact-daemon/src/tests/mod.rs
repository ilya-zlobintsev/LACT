mod mock_fs;

use crate::{
    config::Config,
    server::handler::{read_pci_db, Handler},
};
use insta::{assert_debug_snapshot, assert_json_snapshot};
use lact_schema::config::GpuConfig;
use mock_fs::MockSysfs;
use std::{fs, path::PathBuf, sync::OnceLock};
use tempfile::tempdir;
use tokio::task::LocalSet;

fn init_tracing() {
    static TRACING_LOCK: OnceLock<()> = OnceLock::new();
    TRACING_LOCK.get_or_init(|| {
        tracing_subscriber::fmt().init();
    });
}

#[tokio::test]
async fn snapshot_everything() {
    init_tracing();

    let test_data_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("src/tests/data");
    let pci_db = read_pci_db();

    for vendor_dir in fs::read_dir(test_data_dir).unwrap().flatten() {
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

#[tokio::test]
async fn apply_settings() {
    init_tracing();

    let local_set = LocalSet::new();
    local_set.spawn_local(async move {
        let test_data_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("src/tests/data");
        let pci_db = read_pci_db();

        for vendor_dir in fs::read_dir(test_data_dir).unwrap().flatten() {
            for device_dir in fs::read_dir(vendor_dir.path()).unwrap().flatten() {
                if let Ok(raw_gpu_config) =
                    fs::read_to_string(device_dir.path().join("config.yaml"))
                {
                    let test_key = format!(
                        "apply_config/{}/{}",
                        vendor_dir.file_name().to_string_lossy(),
                        device_dir.file_name().to_string_lossy()
                    );
                    let gpu_config: GpuConfig = serde_yml::from_str(&raw_gpu_config).unwrap();

                    let mock_fs_dir = tempdir().unwrap();

                    let mock_fs = MockSysfs::new(device_dir.path());
                    let writes = mock_fs.writes.clone();

                    let mount = easy_fuser::spawn_mount(mock_fs, mock_fs_dir.path(), &[], 1)
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
    });

    local_set.await;
}
