use crate::{Pong, Request, Response};
use serde_json::json;

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
        "status": "error",
        "data": "my super error"
    });

    let response = Response::<()>::Error("my super error".to_owned());

    assert_eq!(serde_json::to_value(response).unwrap(), expected_response);
}
