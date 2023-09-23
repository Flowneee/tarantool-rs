use async_trait::async_trait;
use tarantool_rs::Connection;
use tarantool_test_container::testcontainers::ImageArgs;

pub type TarantoolImage = tarantool_test_container::TarantoolImage<TarantoolArgs>;
pub type TarantoolTestContainer = tarantool_test_container::TarantoolTestContainer<TarantoolArgs>;

#[derive(Clone, Debug, Default)]
pub struct TarantoolArgs {}

impl ImageArgs for TarantoolArgs {
    fn into_iterator(self) -> Box<dyn Iterator<Item = String>> {
        vec!["tarantool".into(), "/opt/tarantool/test_data.lua".into()].into_iterator()
    }
}

#[async_trait]
pub trait TarantoolTestContainerExt {
    fn new_with_test_data() -> Self;
    async fn create_conn(&self) -> Result<Connection, tarantool_rs::errors::Error>;
}

#[async_trait]
impl TarantoolTestContainerExt for TarantoolTestContainer {
    fn new_with_test_data() -> Self {
        let image = TarantoolImage::default().volume(
            format!("{}/tests", env!("CARGO_MANIFEST_DIR")),
            "/opt/tarantool".into(),
        );
        Self::from_image(image)
    }

    async fn create_conn(&self) -> Result<Connection, tarantool_rs::errors::Error> {
        Connection::builder()
            .build(format!("127.0.0.1:{}", self.connect_port()))
            .await
    }
}
