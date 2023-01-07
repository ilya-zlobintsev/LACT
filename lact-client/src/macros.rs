macro_rules! request_with_id {
    ($name:ident, $variant:ident, $response:ty) => {
        pub fn $name(&self, id: &str) -> anyhow::Result<ResponseBuffer<$response>> {
            self.make_request(Request::$variant { id })
        }
    };
}
