#[macro_use]
extern crate rental;

pub use testcontainers;

pub use self::test_container::TarantoolTestContainer;

use std::{collections::HashMap, fmt::Debug, marker::PhantomData};

use maplit::hashmap;
use rental::rental;
use testcontainers::{clients::Cli as DockerClient, core::WaitFor, Container, Image, ImageArgs};

const IMAGE_NAME: &str = "tarantool/tarantool";
const DEFAULT_IMAGE_TAG: &str = "latest";

fn image_tag() -> String {
    std::env::var("TARANTOOL_IMAGE_TAG").unwrap_or(DEFAULT_IMAGE_TAG.into())
}

#[derive(Clone, Debug, Default)]
pub struct TarantoolDefaultArgs {}

impl ImageArgs for TarantoolDefaultArgs {
    fn into_iterator(self) -> Box<dyn Iterator<Item = String>> {
        vec!["tarantool".into()].into_iterator()
    }
}

pub struct TarantoolImage<Args> {
    env_vars: HashMap<String, String>,
    volumes: HashMap<String, String>,
    _args: PhantomData<Args>,
}

impl<Args> Default for TarantoolImage<Args> {
    fn default() -> Self {
        Self {
            env_vars: hashmap! {
                "TT_MEMTX_USE_MVCC_ENGINE".into() => "true".into()
            },
            volumes: HashMap::new(),
            _args: PhantomData::default(),
        }
    }
}

impl<Args> Image for TarantoolImage<Args>
where
    Args: ImageArgs + Clone + Debug,
{
    type Args = Args;

    fn name(&self) -> String {
        IMAGE_NAME.into()
    }

    fn tag(&self) -> String {
        image_tag()
    }

    fn ready_conditions(&self) -> Vec<WaitFor> {
        vec![WaitFor::message_on_stderr("entering the event loop")]
    }

    fn expose_ports(&self) -> Vec<u16> {
        vec![3301]
    }

    fn env_vars(&self) -> Box<dyn Iterator<Item = (&String, &String)> + '_> {
        Box::new(self.env_vars.iter())
    }

    fn volumes(&self) -> Box<dyn Iterator<Item = (&String, &String)> + '_> {
        Box::new(self.volumes.iter())
    }
}

impl<Args> TarantoolImage<Args> {
    pub fn disable_mvcc(mut self) -> Self {
        drop(self.env_vars.remove("TT_MEMTX_USE_MVCC_ENGINE"));
        self
    }

    pub fn volume(mut self, host_path: String, container_path: String) -> Self {
        self.volumes.insert(host_path, container_path);
        self
    }
}

rental! {
    mod test_container {
        use super::*;

        #[rental]
        pub struct TarantoolTestContainer<Args: 'static>
            where Args: Clone + Debug + ImageArgs
        {
            client: Box<DockerClient>,
            container: Container<'client, TarantoolImage<Args>>
        }
    }
}

impl<Args> Default for TarantoolTestContainer<Args>
where
    Args: Clone + Debug + ImageArgs + Default,
{
    fn default() -> Self {
        Self::from_image(TarantoolImage::<Args>::default())
    }
}

impl<Args> TarantoolTestContainer<Args>
where
    Args: Clone + Debug + ImageArgs + Default,
{
    pub fn from_image(image: TarantoolImage<Args>) -> Self {
        let docker = DockerClient::default();
        Self::new(Box::new(docker), |docker| docker.run(image))
    }

    pub fn connect_port(&self) -> u16 {
        self.rent(|this| this.get_host_port_ipv4(3301))
    }
}
