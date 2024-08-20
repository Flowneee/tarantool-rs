use crate::Error;

pub(crate) use self::dispatcher::{ConnectionWatcher, DispatcherSender};

mod connection;
mod dispatcher;

pub(crate) async fn spawn_background_connection<A>(
    addr: A,
    user: Option<String>,
    password: Option<String>,
    connect_timeout: Option<Duration>,
    reconnect_interval: Option<ReconnectInterval>,
    internal_simultaneous_requests_threshold: usize,
) -> Result<DispatcherSender, Error>
where
    A: ToSocketAddrs + Display + Clone + Send + Sync + 'static,
{
    let (watcher_fut, disaptcher_sender) = ConnectionWatcher::prepare(
        addr,
        self.user.clone(),
        self.password.clone(),
        self.timeout,
        self.reconnect_interval.clone(),
        self.internal_simultaneous_requests_threshold,
    )
    .await?;

    // TODO: support setting custom executor
    tokio::spawn(dispatcher_fut);
}
