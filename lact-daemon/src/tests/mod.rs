use insta::assert_json_snapshot;

use crate::{config::Config, server::handler::Handler};
use std::{fs, path::PathBuf};

#[tokio::test]
async fn snapshot_everything() {
    let test_data_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("src/tests/data");

    for vendor_dir in fs::read_dir(test_data_dir).unwrap().flatten() {
        for device_dir in fs::read_dir(vendor_dir.path()).unwrap().flatten() {
            let test_key = format!(
                "{}/{}",
                vendor_dir.file_name().to_string_lossy(),
                device_dir.file_name().to_string_lossy()
            );

            let handler = Handler::with_base_path(&device_dir.path(), Config::default(), true)
                .await
                .unwrap();
            let mut device_info = handler
                .generate_snapshot_device_info()
                .into_values()
                .next()
                .unwrap();
            // Remove vulkan information, as it can only be fetched from the current running system and affects snapshot output
            device_info
                .get_mut("info")
                .unwrap()
                .as_object_mut()
                .unwrap()
                .remove("vulkan_info");

            assert_json_snapshot!(test_key, device_info);
        }
    }
}
