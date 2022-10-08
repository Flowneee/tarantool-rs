use tokio::{
    io::AsyncReadExt,
    net::{TcpStream, ToSocketAddrs},
};
use tokio_util::codec::Framed;
use tracing::trace;

use crate::{
    codec::{ClientCodec, IProtoGreeting},
    errors::Error,
};

pub struct Connection {
    inner: Framed<TcpStream, ClientCodec>,
}

impl Connection {
    // TODO: builder
    // TODO: maybe hide?
    pub async fn new<A: ToSocketAddrs>(addr: A) -> Result<Self, Error> {
        let mut tcp = TcpStream::connect(addr).await?;

        let mut greeting_buffer = [0u8; 128];
        tcp.read_exact(&mut greeting_buffer).await?;
        let greeting = IProtoGreeting::decode_unchecked(&greeting_buffer);
        trace!("Salt: {:?}", greeting.salt);

        // TODO: send PING to test connection

        Ok(Self {
            inner: Framed::new(tcp, ClientCodec::default()),
        })
    }

    // TODO: Remove
    #[doc(hidden)]
    #[deprecated = "This function should not be part of public API"]
    pub fn inner(&mut self) -> &mut Framed<TcpStream, ClientCodec> {
        &mut self.inner
    }
}
