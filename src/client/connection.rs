use std::{
    fmt,
    sync::{
        atomic::{AtomicU32, Ordering},
        Arc,
    },
    time::Duration,
};

use async_trait::async_trait;
use futures::TryFutureExt;
use rmpv::Value;
use tokio::time::timeout;
use tracing::debug;

use crate::{
    builder::ConnectionBuilder,
    client::{Executor, Stream, Transaction, TransactionBuilder},
    codec::{
        consts::TransactionIsolationLevel,
        request::{EncodedRequest, Id, Request},
        response::ResponseBody,
    },
    transport::DispatcherSender,
    ExecutorExt, Result,
};

/// Connection to Tarantool instance.
///
/// This type doesn't represent single TCP connection, but rather an abstraction
/// for interaction with Tarantool instance.
///
/// Underling implemenation could reconnect automatically (depending on builder configuration),
/// and could do pooling in the future (not yet implemented!).
#[derive(Clone)]
pub struct Connection {
    inner: Arc<ConnectionInner>,
}

struct ConnectionInner {
    dispatcher_sender: DispatcherSender,
    // TODO: change how stream id assigned when dispathcer have more than one connection
    next_stream_id: AtomicU32,
    timeout: Option<Duration>,
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
        timeout: Option<Duration>,
        transaction_timeout: Option<Duration>,
        transaction_isolation_level: TransactionIsolationLevel,
    ) -> Self {
        Self {
            inner: Arc::new(ConnectionInner {
                dispatcher_sender,
                // TODO: check if 0 is valid value
                next_stream_id: AtomicU32::new(1),
                timeout,
                transaction_timeout_secs: transaction_timeout.as_ref().map(Duration::as_secs_f64),
                transaction_isolation_level,
                // NOTE: Safety: this method can be called only in async tokio context (because it
                // is called only from ConnectionBuilder).
                async_rt_handle: tokio::runtime::Handle::current(),
            }),
        }
    }

    /// Synchronously send request to channel and drop response.
    #[allow(clippy::let_underscore_future)]
    pub(crate) fn send_request_sync_and_forget(&self, body: impl Request, stream_id: Option<u32>) {
        let this = self.clone();
        let req = EncodedRequest::new(body, stream_id);
        let _ = self.inner.async_rt_handle.spawn(async move {
            let res = futures::future::ready(req)
                .err_into()
                .and_then(|x| this.send_encoded_request(x))
                .await;
            debug!("Response for background request: {:?}", res);
        });
    }

    // TODO: maybe other Ordering??
    pub(crate) fn next_stream_id(&self) -> u32 {
        let next = self.inner.next_stream_id.fetch_add(1, Ordering::Relaxed);
        if next != 0 {
            next
        } else {
            self.inner.next_stream_id.fetch_add(1, Ordering::Relaxed)
        }
    }

    // TODO: return response from server
    /// Send ID request ([docs](https://www.tarantool.io/en/doc/latest/dev_guide/internals/box_protocol/#iproto-id-0x49)).
    pub(crate) async fn id(&self, features: Id) -> Result<()> {
        self.send_request(features).await.map(drop)
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
    pub(crate) async fn transaction(&self) -> Result<Transaction> {
        self.transaction_builder().begin().await
    }
}

#[async_trait]
impl Executor for Connection {
    async fn send_encoded_request(&self, request: EncodedRequest) -> Result<Value> {
        let fut = self.inner.dispatcher_sender.send(request);
        let resp = match self.inner.timeout {
            Some(x) => timeout(x, fut).await??,
            None => fut.await?,
        };
        match resp.body {
            ResponseBody::Ok(x) => Ok(x),
            ResponseBody::Error(x) => Err(x.into()),
        }
    }

    fn stream(&self) -> Stream {
        self.stream()
    }

    fn transaction_builder(&self) -> TransactionBuilder {
        self.transaction_builder()
    }

    async fn transaction(&self) -> Result<Transaction> {
        self.transaction().await
    }
}

impl fmt::Debug for Connection {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Connection")
    }
}
