use std::{
    sync::{
        atomic::{AtomicU32, Ordering},
        Arc,
    },
    time::Duration,
};

use async_trait::async_trait;
use futures::Future;
use rmpv::Value;
use tracing::debug;

use super::{
    connection_like::ConnectionLike, ConnectionBuilder, Stream, Transaction, TransactionBuilder,
};
use crate::{
    codec::{
        consts::TransactionIsolationLevel,
        request::{Auth, Id, Request, RequestBody},
        response::ResponseBody,
    },
    errors::Error,
    transport::DispatcherSender,
};

#[derive(Clone)]
pub struct Connection {
    inner: Arc<ConnectionInner>,
}

struct ConnectionInner {
    dispatcher_sender: DispatcherSender,
    next_sync: AtomicU32,
    next_stream_id: AtomicU32,
    transaction_timeout_secs: Option<f64>,
    transaction_isolation_level: TransactionIsolationLevel,
    async_rt_handle: tokio::runtime::Handle,
}

impl Connection {
    /// Create new [`ConnectionBuilder`].
    pub fn builder() -> ConnectionBuilder {
        ConnectionBuilder::default()
    }

    pub(crate) fn new(
        dispatcher_sender: DispatcherSender,
        transaction_timeout: Option<Duration>,
        transaction_isolation_level: TransactionIsolationLevel,
    ) -> Self {
        Self {
            inner: Arc::new(ConnectionInner {
                dispatcher_sender,
                next_sync: AtomicU32::new(0),
                // TODO: check if 0 is valid value
                next_stream_id: AtomicU32::new(1),
                transaction_timeout_secs: transaction_timeout.as_ref().map(Duration::as_secs_f64),
                transaction_isolation_level,
                // NOTE: Safety: this method can be called only in async tokio context (because it
                // is called only from ConnectionBuilder).
                async_rt_handle: tokio::runtime::Handle::current(),
            }),
        }
    }

    pub(crate) async fn send_encoded_request(&self, request: Request) -> Result<Value, Error> {
        let resp = self.inner.dispatcher_sender.send(request).await?;
        match resp.body {
            ResponseBody::Ok(x) => Ok(x),
            ResponseBody::Error(x) => Err(x.into()),
        }
    }

    pub(crate) fn send_request(
        &self,
        body: impl RequestBody,
        stream_id: Option<u32>,
    ) -> impl Future<Output = Result<Value, Error>> + Send + '_ {
        let req = Request::new(body, stream_id);
        async { self.send_encoded_request(req?).await }
    }

    /// Synchronously send request to channel and drop response.
    pub(crate) fn send_request_sync_and_forget(
        &self,
        body: impl RequestBody,
        stream_id: Option<u32>,
    ) {
        let this = self.clone();
        let req = Request::new(body, stream_id);
        let _ = self.inner.async_rt_handle.spawn(async move {
            // TOOD: fix unwrap
            let res = this.clone().send_encoded_request(req.unwrap()).await;
            debug!("Response for background request: {:?}", res);
        });
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

    // TODO: return response from server
    /// Send ID request ([docs](https://www.tarantool.io/en/doc/latest/dev_guide/internals/box_protocol/#iproto-id-0x49)).
    pub(crate) async fn id(&self, features: Id) -> Result<(), Error> {
        self.send_request(features, None).await.map(drop)
    }

    pub(crate) fn stream(&self) -> Stream {
        Stream::new(self.clone())
    }

    /// Create transaction, overriding default connection's parameters.
    pub(crate) fn transaction_builder(&self) -> TransactionBuilder {
        TransactionBuilder::new(
            self.clone(),
            self.inner.transaction_timeout_secs,
            self.inner.transaction_isolation_level,
        )
    }

    /// Create transaction.
    pub(crate) async fn transaction(&self) -> Result<Transaction, Error> {
        self.transaction_builder().begin().await
    }
}

#[async_trait(?Send)]
impl ConnectionLike for Connection {
    async fn send_request(&self, body: impl RequestBody) -> Result<Value, Error> {
        self.send_request(body, None).await
    }

    fn stream(&self) -> Stream {
        self.stream()
    }

    fn transaction_builder(&self) -> TransactionBuilder {
        self.transaction_builder()
    }

    async fn transaction(&self) -> Result<Transaction, Error> {
        self.transaction().await
    }
}
