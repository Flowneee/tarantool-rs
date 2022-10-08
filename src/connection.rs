use std::sync::{
    atomic::{AtomicU32, Ordering},
    Arc,
};

use crate::{
    channel::ChannelTx,
    codec::{
        request::{IProtoPing, IProtoRequest, IProtoRequestBody},
        response::IProtoResponseBody,
    },
    errors::Error,
    ConnectionBuilder,
};

#[derive(Clone)]
pub struct Connection {
    inner: Arc<ConnectionInner>,
}

struct ConnectionInner {
    chan_tx: ChannelTx,
    next_sync: AtomicU32,
    next_stream_id: AtomicU32,
}

impl Connection {
    /// Create new [`ConnectionBuilder`].
    pub fn builder() -> ConnectionBuilder {
        ConnectionBuilder::default()
    }

    pub(crate) fn new(chan_tx: ChannelTx) -> Self {
        Self {
            inner: Arc::new(ConnectionInner {
                chan_tx,
                next_sync: AtomicU32::new(0),
                // TODO: check if 0 is valid value
                next_stream_id: AtomicU32::new(0),
            }),
        }
    }

    async fn send_request(
        &self,
        body: impl IProtoRequestBody,
        stream_id: Option<u32>,
    ) -> Result<rmpv::Value, Error> {
        let resp = self
            .inner
            .chan_tx
            .send(IProtoRequest::new(self.next_sync(), body, stream_id))
            .await?;
        match resp.body {
            IProtoResponseBody::Ok(x) => Ok(x),
            IProtoResponseBody::Error {
                code,
                description,
                extra,
            } => Err(Error::response(code, description, extra)),
        }
    }

    // TODO: maybe other Ordering??
    fn next_sync(&self) -> u32 {
        self.inner.next_sync.fetch_add(1, Ordering::SeqCst)
    }

    // TODO: maybe other Ordering??
    fn next_stream_id(&self) -> u32 {
        self.inner.next_stream_id.fetch_add(1, Ordering::SeqCst)
    }

    /// Send PING request ([docs](https://www.tarantool.io/en/doc/latest/dev_guide/internals/box_protocol/#iproto-ping-0x40)).
    pub async fn ping(&self) -> Result<(), Error> {
        self.send_request(IProtoPing {}, None).await.map(drop)
    }
}
