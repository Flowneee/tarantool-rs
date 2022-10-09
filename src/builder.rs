use tokio::net::ToSocketAddrs;
use tracing::debug;

use crate::{
    channel::{run_channel, Channel},
    codec::request::IProtoId,
    connection::Connection,
    errors::Error,
};

/// Build connection to Tarantool.
#[derive(Default)]
pub struct ConnectionBuilder {
    user: Option<String>,
    password: Option<String>,
}

impl ConnectionBuilder {
    /// Create connection to Tarantool using provided address and test it using PING.
    pub async fn build<A: ToSocketAddrs>(&self, addr: A) -> Result<Connection, Error> {
        let (chan, chan_tx, salt) = Channel::new(addr).await?;
        // TODO: support setting custom executor
        tokio::spawn(run_channel(chan));
        let conn = Connection::new(chan_tx);

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
}
