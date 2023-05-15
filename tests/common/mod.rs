use tarantool_rs::Connection;
use testcontainers::{clients::Cli as DockerClient, Container};

pub use self::{
    image::{Tarantool, TarantoolArgs},
    test_container::TarantoolTestContainer,
};

mod image;

rental::rental! {
    mod test_container {
        use super::*;

        #[rental]
        pub struct TarantoolTestContainer {
            client: Box<DockerClient>,
            container: Container<'client, Tarantool>
        }
    }
}

impl Default for TarantoolTestContainer {
    fn default() -> Self {
        Self::from_image(Tarantool::default())
    }
}

impl TarantoolTestContainer {
    pub fn from_image(image: Tarantool) -> Self {
        // Initialize logging here is ugly, but simple.
        init_logging();

        let docker = DockerClient::default();
        Self::new(Box::new(docker), |docker| docker.run(image))
    }

    pub fn connect_port(&self) -> u16 {
        self.rent(|this| this.get_host_port_ipv4(3301))
    }

    pub async fn create_conn(&self) -> Result<Connection, tarantool_rs::errors::Error> {
        Connection::builder()
            .build(format!("127.0.0.1:{}", self.connect_port()))
            .await
    }
}

pub fn init_logging() {
    let _ = pretty_env_logger::try_init();
}
