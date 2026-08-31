pub use testcontainers;

use std::{borrow::Cow, collections::HashMap};

use testcontainers::{
    Container, Image,
    core::{ContainerPort, IntoContainerPort, Mount, WaitFor},
    runners::SyncRunner,
};

const IMAGE_NAME: &str = "tarantool/tarantool";
const DEFAULT_IMAGE_TAG: &str = "latest";

fn image_tag() -> String {
    std::env::var("TARANTOOL_IMAGE_TAG").unwrap_or(DEFAULT_IMAGE_TAG.into())
}

#[derive(Clone, Debug)]
pub struct TarantoolImage {
    tag: String,
    env_vars: HashMap<String, String>,
    mounts: Vec<Mount>,
    cmd: Vec<String>,
}

impl Default for TarantoolImage {
    fn default() -> Self {
        Self {
            tag: image_tag(),
            env_vars: HashMap::from([("TT_MEMTX_USE_MVCC_ENGINE".to_owned(), "true".to_owned())]),
            mounts: Vec::new(),
            cmd: vec!["tarantool".to_owned()],
        }
    }
}

impl Image for TarantoolImage {
    fn name(&self) -> &str {
        IMAGE_NAME
    }

    fn tag(&self) -> &str {
        &self.tag
    }

    fn ready_conditions(&self) -> Vec<WaitFor> {
        vec![WaitFor::message_on_stderr("entering the event loop")]
    }

    fn env_vars(
        &self,
    ) -> impl IntoIterator<Item = (impl Into<Cow<'_, str>>, impl Into<Cow<'_, str>>)> {
        self.env_vars.iter().map(|(k, v)| (k.as_str(), v.as_str()))
    }

    fn mounts(&self) -> impl IntoIterator<Item = &Mount> {
        &self.mounts
    }

    fn cmd(&self) -> impl IntoIterator<Item = impl Into<Cow<'_, str>>> {
        self.cmd.iter().map(String::as_str)
    }

    fn expose_ports(&self) -> &[ContainerPort] {
        &[ContainerPort::Tcp(3301)]
    }
}

impl TarantoolImage {
    pub fn disable_mvcc(mut self) -> Self {
        drop(self.env_vars.remove("TT_MEMTX_USE_MVCC_ENGINE"));
        self
    }

    pub fn volume(mut self, host_path: String, container_path: String) -> Self {
        self.mounts
            .push(Mount::bind_mount(host_path, container_path));
        self
    }

    /// Replace the whole container command line, including the binary:
    /// pass `"tarantool"` (or another entrypoint) as the first element.
    pub fn cmd_args(mut self, args: impl IntoIterator<Item = String>) -> Self {
        self.cmd = args.into_iter().collect();
        self
    }
}

// testcontainers' blocking API drives its own tokio runtime under the hood,
// and tokio panics when a runtime is entered from a worker thread of another
// runtime (as happens in #[tokio::test]). Every blocking testcontainers call
// is therefore pushed onto a dedicated OS thread.
pub struct TarantoolTestContainer {
    container: Option<Container<TarantoolImage>>,
}

impl Default for TarantoolTestContainer {
    fn default() -> Self {
        Self::from_image(TarantoolImage::default())
    }
}

impl TarantoolTestContainer {
    pub fn from_image(image: TarantoolImage) -> Self {
        let container = std::thread::spawn(move || image.start())
            .join()
            .expect("tarantool container start thread panicked")
            .expect("failed to start tarantool test container");
        Self {
            container: Some(container),
        }
    }

    pub fn connect_port(&self) -> u16 {
        let container = self.container();
        std::thread::scope(|scope| {
            scope
                .spawn(|| container.get_host_port_ipv4(3301.tcp()))
                .join()
        })
        .expect("tarantool container port thread panicked")
        .expect("failed to get mapped port 3301 of tarantool test container")
    }

    fn container(&self) -> &Container<TarantoolImage> {
        self.container
            .as_ref()
            .expect("container is present until drop")
    }
}

impl Drop for TarantoolTestContainer {
    fn drop(&mut self) {
        if let Some(container) = self.container.take() {
            let _ = std::thread::spawn(move || drop(container)).join();
        }
    }
}
