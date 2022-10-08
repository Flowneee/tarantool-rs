use tokio::net::ToSocketAddrs;

use crate::{
    channel::{run_channel, Channel},
    connection::Connection,
    errors::Error,
};

/// Build connection to Tarantool.
#[derive(Default)]
pub struct ConnectionBuilder {
    user: Option<String>,
    password: Option<String>, // TODO: schema version
}

impl ConnectionBuilder {
    /// Create connection to Tarantool using provided address and test it using PING.
    pub async fn build<A: ToSocketAddrs>(&self, addr: A) -> Result<Connection, Error> {
        let (chan, chan_tx, salt) = Channel::new(addr).await?;
        // TODO: support setting custom executor
        tokio::spawn(run_channel(chan));
        let conn = Connection::new(chan_tx);
        if let Some(user) = self.user.clone() {
            conn.auth(user, self.password.clone(), salt).await?;
        } else {
            conn.ping().await?;
        }
        Ok(conn)
    }

    pub fn auth(&mut self, user: &str, password: Option<&str>) -> &mut Self {
        self.user = Some(user.into());
        self.password = password.map(Into::into);
        self
    }
}
