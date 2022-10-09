use std::time::Duration;

use tokio::net::ToSocketAddrs;
use tracing::debug;

use crate::{
    channel::{run_channel, Channel},
    codec::{consts::TransactionIsolationLevel, request::IProtoId},
    connection::Connection,
    errors::Error,
};

/// Build connection to Tarantool.
#[derive(Default)]
pub struct ConnectionBuilder {
    user: Option<String>,
    password: Option<String>,
    transaction_timeout: Option<Duration>,
    transaction_isolation_level: TransactionIsolationLevel,
}

impl ConnectionBuilder {
    /// Create connection to Tarantool using provided address and test it using PING.
    pub async fn build<A: ToSocketAddrs>(&self, addr: A) -> Result<Connection, Error> {
        let (chan, chan_tx, salt) = Channel::new(addr).await?;
        // TODO: support setting custom executor
        tokio::spawn(run_channel(chan));
        let conn = Connection::new(
            chan_tx,
            self.transaction_timeout,
            self.transaction_isolation_level,
        );

        // TODO: add option to disable pre 2.10 features (ID request, streams, watchers)
        let features = IProtoId::default();
        debug!(
            "Setting supported features: VERSION - {}, STREAMS - {}, TRANSACTIONS - {}, ERROR_EXTENSION - {}, WATCHERS = {}",
            features.protocol_version,
            features.streams,
            features.transactions,
            features.error_extension,
            features.watchers
        );
        conn.id(features).await?;

        if let Some(user) = self.user.clone() {
            conn.auth(user, self.password.clone(), salt).await?;
        }

        Ok(conn)
    }

    /// Sets user login ane, optinally, password, used for this connection.
    ///
    /// AUTH message sent upon connecting to server.
    pub fn auth(&mut self, user: &str, password: Option<&str>) -> &mut Self {
        self.user = Some(user.into());
        self.password = password.map(Into::into);
        self
    }

    /// Sets default timeout in transactions.
    ///
    /// By default disabled.
    pub fn transaction_timeout(&mut self, transaction_timeout: Option<Duration>) -> &mut Self {
        self.transaction_timeout = transaction_timeout;
        self
    }

    /// Sets default transaction isolation level.
    ///
    /// By default `TransactionIsolationLevel::Default` (i.e. use box.cfg default value).
    pub fn transaction_isolation_level(
        &mut self,
        transaction_isolation_level: TransactionIsolationLevel,
    ) -> &mut Self {
        self.transaction_isolation_level = transaction_isolation_level;
        self
    }
}
