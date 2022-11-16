use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(tag = "status", content = "data", rename_all = "snake_case")]
pub enum Response<T> {
    Ok(T),
    Error(String),
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Pong;
