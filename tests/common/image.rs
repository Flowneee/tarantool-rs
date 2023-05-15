use std::collections::HashMap;

use maplit::hashmap;
use testcontainers::{core::WaitFor, Image, ImageArgs};

const IMAGE_NAME: &str = "tarantool/tarantool";
const IMAGE_TAG: &str = "latest";

#[derive(Clone, Debug, Default)]
pub struct TarantoolArgs {}

impl ImageArgs for TarantoolArgs {
    fn into_iterator(self) -> Box<dyn Iterator<Item = String>> {
        vec!["tarantool".into(), "/opt/tarantool/test_data.lua".into()].into_iterator()
    }
}

/// Tarantool image with data, necessary for integration tests.
pub struct Tarantool {
    env_vars: HashMap<String, String>,
    volumes: HashMap<String, String>,
}

impl Default for Tarantool {
    fn default() -> Self {
        Self {
            env_vars: hashmap! {
                "TT_MEMTX_USE_MVCC_ENGINE".into() => "true".into()
            },
            volumes: hashmap! {
                format!("{}/tests", env!("CARGO_MANIFEST_DIR")) => "/opt/tarantool".into(),
            },
        }
    }
}

impl Image for Tarantool {
    type Args = TarantoolArgs;

    fn name(&self) -> String {
        IMAGE_NAME.into()
    }

    fn tag(&self) -> String {
        IMAGE_TAG.into()
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

impl Tarantool {
    pub fn disable_mvcc(mut self) -> Self {
        drop(self.env_vars.remove("TT_MEMTX_USE_MVCC_ENGINE"));
        self
    }
}
