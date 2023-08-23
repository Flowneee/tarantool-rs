use std::{
    fmt,
    num::NonZeroUsize,
    sync::{
        atomic::{AtomicU32, Ordering},
        Arc,
    },
    time::Duration,
};

use async_trait::async_trait;
use futures::TryFutureExt;
use lru::LruCache;
use parking_lot::Mutex;
use rmpv::Value;
use tokio::time::timeout;
use tracing::{debug, trace};

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
    // TODO: change how stream id assigned when dispatcher have more than one connection
    next_stream_id: AtomicU32,
    timeout: Option<Duration>,
    transaction_timeout_secs: Option<f64>,
    transaction_isolation_level: TransactionIsolationLevel,
    async_rt_handle: tokio::runtime::Handle,
    // TODO: tests
    // TODO: move sql statement cache to separate type
    sql_statement_cache: Option<Mutex<LruCache<String, u64>>>,
    sql_statement_cache_update_lock: Mutex<()>,
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
        sql_statement_cache_capacity: usize,
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
                sql_statement_cache: NonZeroUsize::new(sql_statement_cache_capacity)
                    .map(|x| Mutex::new(LruCache::new(x))),
                sql_statement_cache_update_lock: Mutex::new(()),
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

    /// Get prepared statement id from cache (if it is enabled).
    ///
    /// If statement not present in cache, then prepare statement and put it
    /// to cache.
    ///
    /// Only one statement can be prepared at the time. All other will immediately
    /// return None, when there is already a statement being prepared. Eventually
    /// all statements should be allowed to prepare.
    async fn get_cached_sql_statement_id_inner(&self, statement: &str) -> Option<u64> {
        // Lock cache mutex (if cache is not None) and check
        // if statement present in cache.
        let cache = self.inner.sql_statement_cache.as_ref()?;
        if let Some(stmt_id) = cache.lock().get(statement) {
            return Some(*stmt_id);
        }

        // If statement not found, try to lock update lock mutex.
        // If successful, proceed with preparing SQL statement,
        // otherwise return None.
        let update_lock = self.inner.sql_statement_cache_update_lock.try_lock();
        let stmt_id = {
            let stmt_id = match self.prepare_sql(statement).await {
                Ok(x) => {
                    let stmt_id = x.stmt_id();
                    trace!(statement, "Statement prepared with id {stmt_id}");
                    stmt_id
                }
                Err(err) => {
                    debug!("Failed to prepare statement for cache: {:#}", err);
                    return None;
                }
            };
            let _ = cache.lock().put(statement.into(), stmt_id);
            stmt_id
        };
        drop(update_lock);

        Some(stmt_id)
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

    async fn get_cached_sql_statement_id(&self, statement: &str) -> Option<u64> {
        self.get_cached_sql_statement_id_inner(statement).await
    }
}

impl fmt::Debug for Connection {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Connection")
    }
}
