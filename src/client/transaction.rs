use std::time::Duration;

use async_trait::async_trait;

use rmpv::Value;
use tracing::debug;

use super::{Connection, ConnectionLike, Stream};
use crate::{
    codec::{
        consts::TransactionIsolationLevel,
        request::{Begin, Commit, EncodedRequest, Rollback},
    },
    Executor, Result,
};

/// Started transaction ([docs](https://www.tarantool.io/en/doc/latest/dev_guide/internals/box_protocol/#binary-protocol-streams)).
///
/// If tranasction have a timeout and no requests made for that time, tranasction is automatically
/// rolled back.
///
/// On drop tranasaction is rolled back, if not have been commited or rolled back already.
pub struct Transaction {
    conn: Connection,
    stream_id: u32,
    finished: bool,
}

impl Transaction {
    async fn new(
        conn: Connection,
        timeout_secs: Option<f64>,
        isolation_level: TransactionIsolationLevel,
    ) -> Result<Self> {
        let stream_id = conn.next_stream_id();
        let this = Self {
            conn,
            stream_id,
            finished: false,
        };
        this.begin(isolation_level, timeout_secs).await?;
        Ok(this)
    }

    async fn begin(
        &self,
        transaction_isolation_level: TransactionIsolationLevel,
        timeout_secs: Option<f64>,
    ) -> Result<()> {
        debug!("Beginning tranasction on stream {}", self.stream_id);
        self.send_request(Begin::new(timeout_secs, transaction_isolation_level))
            .await
            .map(drop)
    }

    /// Commit tranasction.
    pub async fn commit(mut self) -> Result<()> {
        if !self.finished {
            debug!("Commiting tranasction on stream {}", self.stream_id);
            let _ = self.send_request(Commit::default()).await?;
            self.finished = true;
        }
        Ok(())
    }

    /// Rollback tranasction.
    pub async fn rollback(mut self) -> Result<()> {
        if !self.finished {
            debug!("Rolling back tranasction on stream {}", self.stream_id);
            let _ = self.send_request(Rollback::default()).await?;
            self.finished = true;
        }
        Ok(())
    }
}

impl Drop for Transaction {
    fn drop(&mut self) {
        if !self.finished {
            debug!(
                "Rolling back tranasction on stream {} (on drop)",
                self.stream_id
            );
            self.conn
                .send_request_sync_and_forget(Rollback::default(), Some(self.stream_id));
            self.finished = true;
        }
    }
}

#[async_trait]
impl Executor for Transaction {
    async fn send_encoded_request(&self, mut request: EncodedRequest) -> Result<Value> {
        request.stream_id = Some(self.stream_id);
        self.conn.send_encoded_request(request).await
    }

    // TODO: do we need to repeat this in all ConnetionLike implementations?
    fn stream(&self) -> Stream {
        self.conn.stream()
    }

    fn transaction_builder(&self) -> TransactionBuilder {
        self.conn.transaction_builder()
    }

    async fn transaction(&self) -> Result<Transaction> {
        self.conn.transaction().await
    }
}

/// Build transaction.
pub struct TransactionBuilder {
    connection: Connection,
    timeout_secs: Option<f64>,
    isolation_level: TransactionIsolationLevel,
}

impl TransactionBuilder {
    pub(crate) fn new(
        connection: Connection,
        timeout_secs: Option<f64>,
        isolation_level: TransactionIsolationLevel,
    ) -> Self {
        Self {
            connection,
            timeout_secs,
            isolation_level,
        }
    }

    pub fn timeout(&mut self, timeout: impl Into<Option<Duration>>) -> &mut Self {
        self.timeout_secs = timeout.into().as_ref().map(Duration::as_secs_f64);
        self
    }

    pub fn isolation_level(&mut self, isolation_level: TransactionIsolationLevel) -> &mut Self {
        self.isolation_level = isolation_level;
        self
    }

    pub async fn begin(&self) -> Result<Transaction> {
        Transaction::new(
            self.connection.clone(),
            self.timeout_secs,
            self.isolation_level,
        )
        .await
    }
}
