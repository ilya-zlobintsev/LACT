use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
#[serde(tag = "status", content = "data", rename_all = "snake_case")]
pub enum Response<T> {
    Ok(T),
    Error(serde_error::Error),
}

impl<T> From<anyhow::Error> for Response<T> {
    fn from(value: anyhow::Error) -> Self {
        Response::Error(serde_error::Error::new(&*value))
    }
}
