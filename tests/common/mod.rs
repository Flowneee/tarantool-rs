use async_trait::async_trait;
use tarantool_rs::Connection;
pub use tarantool_test_container::{TarantoolImage, TarantoolTestContainer};

#[async_trait]
pub trait TarantoolTestContainerExt {
    fn new_with_test_data() -> Self;
    async fn create_conn(&self) -> Result<Connection, tarantool_rs::errors::Error>;
}

#[async_trait]
impl TarantoolTestContainerExt for TarantoolTestContainer {
    fn new_with_test_data() -> Self {
        let image = TarantoolImage::default()
            .volume(
                format!("{}/tests", env!("CARGO_MANIFEST_DIR")),
                "/opt/tarantool".into(),
            )
            .cmd_args(["tarantool".into(), "/opt/tarantool/test_data.lua".into()]);
        Self::from_image(image)
    }

    async fn create_conn(&self) -> Result<Connection, tarantool_rs::errors::Error> {
        Connection::builder()
            .build(format!("127.0.0.1:{}", self.connect_port()))
            .await
    }
}
