use tokio::net::ToSocketAddrs;

use crate::{
    channel::{run_channel, Channel},
    connection::Connection,
    errors::Error,
};

/// Build connection to Tarantool.
#[derive(Default)]
pub struct ConnectionBuilder {
    // TODO: schema version
}

impl ConnectionBuilder {
    /// Create connection to Tarantool using provided address and test it using PING.
    pub async fn build<A: ToSocketAddrs>(&self, addr: A) -> Result<Connection, Error> {
        let (chan, chan_tx) = Channel::new(addr).await?;
        // TODO: support setting custom executor
        tokio::spawn(run_channel(chan));
        let conn = Connection::new(chan_tx);
        conn.ping().await?;
        Ok(conn)
    }
}
