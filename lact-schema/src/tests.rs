use crate::{FanControlMode, FanOptions, PmfwOptions, Pong, Request, Response};
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
        "data": null
    });
    let response = Response::Ok(Pong.into());

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

    let response = Response::from(error);

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
