use crate::{FanControlMode, FanOptions, PmfwOptions, Pong, Request, Response, clean_gpu_name};
use anyhow::anyhow;
use serde_json::json;
use std::collections::BTreeMap;

#[test]
fn ping_requset() {
    let value = r#"{
        "command": "ping"
    }"#;
    let request: Request = serde_json::from_str(value).unwrap();

    assert_eq!(request, Request::Ping);
}

#[test]
fn pong_response() {
    let expected_response = json!({
        "status": "ok",
        "data": null,
    });
    let response = Response::Ok(Pong);

    assert_eq!(serde_json::to_value(response).unwrap(), expected_response);
}

#[test]
fn controllers_response() {
    let expected_response = json!({
      "status": "ok",
      "data": ["1002:67DF-1DA2:E387-0000:0f:00.0"]
    });
    let response = Response::Ok(vec!["1002:67DF-1DA2:E387-0000:0f:00.0"]);
    assert_eq!(serde_json::to_value(response).unwrap(), expected_response);
}

#[test]
fn error_response() {
    let expected_response = json!({
        "data": {
            "description": "third deeper context",
            "source": {
                "description": "second context",
                "source": {
                    "description": "first error",
                    "source": null
                }
            }
        },
        "status": "error"
    });

    let error = anyhow!("first error")
        .context("second context")
        .context(anyhow!("third deeper context"));

    let response = Response::<()>::from(error);

    assert_eq!(serde_json::to_value(response).unwrap(), expected_response);
}

#[test]
fn set_fan_clocks() {
    let value = r#"{
        "command": "set_fan_control",
        "args": {
            "id": "123",
            "enabled": true,
            "mode": "curve",
            "curve": {
                "30": 30.0,
                "50": 50.0
            }
        }
    }"#;
    let request: Request = serde_json::from_str(value).unwrap();
    let expected_request = Request::SetFanControl(FanOptions {
        id: "123",
        enabled: true,
        mode: Some(FanControlMode::Curve),
        static_speed: None,
        curve: Some(BTreeMap::from([(30, 30.0), (50, 50.0)])),
        pmfw: PmfwOptions::default(),
        spindown_delay_ms: None,
        change_threshold: None,
    });
    assert_eq!(expected_request, request);
}

#[test]
fn clean_gpu_name_removes_vendor_prefixes() {
    assert_eq!(clean_gpu_name("AMD Radeon RX 9070 XT"), "RX 9070 XT");
    assert_eq!(clean_gpu_name("NVIDIA GeForce RTX 5090"), "RTX 5090");
    assert_eq!(clean_gpu_name("NVIDIA GeForce MX450"), "MX450");
    assert_eq!(
        clean_gpu_name("NVIDIA GeForce RTX 5090 [Founders Edition]"),
        "RTX 5090"
    );
    assert_eq!(
        clean_gpu_name("NVIDIA GeForce RTX 4070 Super"),
        "RTX 4070 Super"
    );
    assert_eq!(clean_gpu_name("Intel Arc A380"), "Arc A380");
}

#[test]
fn clean_gpu_name_unwraps_pci_names() {
    assert_eq!(
        clean_gpu_name("Pitcairn XT [Radeon HD 7870 GHz Edition]"),
        "HD 7870 GHz Edition"
    );
    assert_eq!(clean_gpu_name("DG2 [Arc A380]"), "Arc A380");
    assert_eq!(clean_gpu_name("GK107M [GeForce 710A]"), "710A");
    assert_eq!(clean_gpu_name("GK107M [GeForce 820M]"), "820M");
    assert_eq!(clean_gpu_name("TU117M [GeForce MX450]"), "MX450");
    assert_eq!(
        clean_gpu_name("TigerLake-LP GT2 [Iris Xe Graphics]"),
        "Iris Xe Graphics"
    );
}

#[test]
fn clean_gpu_name_strips_consumer_brands_and_keeps_unrecognized_names() {
    assert_eq!(clean_gpu_name("AMD Radeon 780M Graphics"), "780M Graphics");
    assert_eq!(clean_gpu_name("Phoenix1"), "Phoenix1");
}

#[test]
fn clean_gpu_name_keeps_professional_product_brands() {
    assert_eq!(clean_gpu_name("GK107GL [Quadro K600]"), "Quadro K600");
    assert_eq!(clean_gpu_name("GK110GL [Tesla K20]"), "Tesla K20");
    assert_eq!(clean_gpu_name("NVIDIA Quadro RTX 6000"), "Quadro RTX 6000");
}
