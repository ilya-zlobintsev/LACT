macro_rules! request_with_id {
    ($name:ident, $variant:ident, $response:ty) => {
        pub async fn $name(&self, id: &str) -> anyhow::Result<$response> {
            self.make_request(Request::$variant { id }).await
        }
    };
}

macro_rules! request_plain {
    ($name:ident, $variant:ident, $response:ty) => {
        pub async fn $name(&self) -> anyhow::Result<$response> {
            self.make_request(Request::$variant).await
        }
    };
}
