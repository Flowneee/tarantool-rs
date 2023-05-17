use std::{cmp::max, fmt::Display, time::Duration};

use tokio::net::ToSocketAddrs;
use tracing::debug;

use crate::{
    client::Connection,
    codec::{consts::TransactionIsolationLevel, request::Id},
    errors::Error,
    transport::Dispatcher,
};

/// Interval parameters for background reconnection.
#[derive(Clone, Debug, PartialEq)]
pub enum ReconnectInterval {
    Fixed(Duration),
    ExponentialBackoff {
        min: Duration,
        max: Duration,
        randomization_factor: f64,
        multiplier: f64,
    },
}

impl Default for ReconnectInterval {
    fn default() -> Self {
        Self::exponential_backoff(Duration::from_millis(1), Duration::from_secs(1), 0.5, 5.0)
    }
}

impl ReconnectInterval {
    /// Fixed interval between reconnection attempts.
    pub fn fixed(interval: Duration) -> Self {
        Self::Fixed(interval)
    }

    /// Interval between reconnection attempts calculated as
    /// exponentially growing period.
    ///
    /// For details on this values check [`backoff::ExponentialBackoff`] docs.
    pub fn exponential_backoff(
        min_interval: Duration,
        max_interval: Duration,
        randomization_factor: f64,
        multiplier: f64,
    ) -> Self {
        Self::ExponentialBackoff {
            min: max(min_interval, Duration::from_micros(1)),
            max: max_interval,
            randomization_factor,
            multiplier,
        }
    }
}

/// Build connection to Tarantool.
#[derive(Debug)]
pub struct ConnectionBuilder {
    user: Option<String>,
    password: Option<String>,
    transaction_timeout: Option<Duration>,
    transaction_isolation_level: TransactionIsolationLevel,
    connect_timeout: Option<Duration>,
    reconnect_interval: Option<ReconnectInterval>,
}

impl Default for ConnectionBuilder {
    fn default() -> Self {
        Self {
            user: Default::default(),
            password: Default::default(),
            transaction_timeout: Default::default(),
            transaction_isolation_level: Default::default(),
            connect_timeout: Default::default(),
            reconnect_interval: Some(ReconnectInterval::default()),
        }
    }
}

impl ConnectionBuilder {
    /// Create connection to Tarantool using provided address.
    pub async fn build<A>(&self, addr: A) -> Result<Connection, Error>
    where
        A: ToSocketAddrs + Display + Clone + Send + Sync + 'static,
    {
        let (dispatcher, disaptcher_sender) = Dispatcher::new(
            addr,
            self.user.as_deref(),
            self.password.as_deref(),
            self.connect_timeout,
            self.reconnect_interval.clone(),
        )
        .await?;

        // TODO: support setting custom executor
        tokio::spawn(dispatcher.run());
        let conn = Connection::new(
            disaptcher_sender,
            self.transaction_timeout,
            self.transaction_isolation_level,
        );

        // TODO: add option to disable pre 2.10 features (ID request, streams, watchers)
        let features = Id::default();
        debug!(
            "Setting supported features: VERSION - {}, STREAMS - {}, TRANSACTIONS - {}, ERROR_EXTENSION - {}, WATCHERS = {}",
            features.protocol_version,
            features.streams,
            features.transactions,
            features.error_extension,
            features.watchers
        );
        conn.id(features).await?;

        Ok(conn)
    }

    /// Sets user login and, optionally, password, used for this connection.
    ///
    /// AUTH message sent upon connecting to server.
    pub fn auth<'a>(&mut self, user: &str, password: impl Into<Option<&'a str>>) -> &mut Self {
        self.user = Some(user.into());
        self.password = password.into().map(Into::into);
        self
    }

    /// Sets default timeout for transactions.
    ///
    /// By default disabled.
    pub fn transaction_timeout(
        &mut self,
        transaction_timeout: impl Into<Option<Duration>>,
    ) -> &mut Self {
        self.transaction_timeout = transaction_timeout.into();
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

    /// Sets timeout for connect.
    ///
    /// By default disabled.
    pub fn connect_timeout(&mut self, connect_timeout: impl Into<Option<Duration>>) -> &mut Self {
        self.connect_timeout = connect_timeout.into();
        self
    }

    /// Sets interval between reconnection attempts.
    ///
    /// If disabled, next attempt wil lbe started as soon as last one finished.
    ///
    /// By default set to `ReconnectInterval::exponential_backoff(Duration::from_millis(1), Duration::from_secs(1), 0.5, 5.0)`.
    pub fn reconnect_interval(
        &mut self,
        reconnect_interval: impl Into<Option<ReconnectInterval>>,
    ) -> &mut Self {
        self.reconnect_interval = reconnect_interval.into();
        self
    }
}
