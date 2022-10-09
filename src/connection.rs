use std::{
    borrow::Cow,
    sync::{
        atomic::{AtomicU32, Ordering},
        Arc,
    },
};

use async_trait::async_trait;
use rmpv::Value;

use crate::{
    channel::ChannelTx,
    codec::{
        request::{
            IProtoAuth, IProtoCall, IProtoEval, IProtoId, IProtoPing, IProtoRequest,
            IProtoRequestBody,
        },
        response::IProtoResponseBody,
        utils::data_from_response_body,
    },
    connection_like::ConnectionLike,
    errors::Error,
    ConnectionBuilder, Stream,
};

#[derive(Clone)]
pub struct Connection {
    inner: Arc<ConnectionInner>,
}

struct ConnectionInner {
    chan_tx: ChannelTx,
    next_sync: AtomicU32,
    next_stream_id: AtomicU32,
    // TODO: add features disabling
    //streams_supported: bool,
    //transactions_supported: bool,
    //watchers_supported: bool,
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
                next_stream_id: AtomicU32::new(1),
            }),
        }
    }

    pub(crate) async fn send_request(
        &self,
        body: impl IProtoRequestBody,
        stream_id: Option<u32>,
    ) -> Result<Value, Error> {
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
    pub(crate) fn next_sync(&self) -> u32 {
        self.inner.next_sync.fetch_add(1, Ordering::SeqCst)
    }

    // TODO: maybe other Ordering??
    pub(crate) fn next_stream_id(&self) -> u32 {
        let next = self.inner.next_stream_id.fetch_add(1, Ordering::SeqCst);
        if next != 0 {
            next
        } else {
            self.inner.next_stream_id.fetch_add(1, Ordering::SeqCst)
        }
    }

    /// Send AUTH request ([docs](https://www.tarantool.io/en/doc/latest/dev_guide/internals/box_protocol/#iproto-auth-0x07)).
    pub(crate) async fn auth(
        &self,
        user: String,
        password: Option<String>,
        salt: Vec<u8>,
    ) -> Result<(), Error> {
        self.send_request(IProtoAuth::new(user, password, salt), None)
            .await
            .map(drop)
    }

    // TODO: return response from server
    /// Send ID request ([docs](https://www.tarantool.io/en/doc/latest/dev_guide/internals/box_protocol/#iproto-id-0x49)).
    pub(crate) async fn id(&self, features: IProtoId) -> Result<(), Error> {
        self.send_request(features, None).await.map(drop)
    }

    pub fn stream(&self) -> Stream {
        Stream::new(self.clone(), self.next_stream_id())
    }
}

#[async_trait]
impl ConnectionLike for Connection {
    async fn send_request(&self, body: impl IProtoRequestBody) -> Result<Value, Error> {
        self.send_request(body, None).await
    }

    fn stream(&self) -> Stream {
        self.stream()
    }
}
