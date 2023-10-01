use std::{fmt::Display, future::Future, pin::Pin, time::Duration};

use backoff::{backoff::Backoff, ExponentialBackoff, ExponentialBackoffBuilder};
use futures::TryFutureExt;
use tokio::{
    net::ToSocketAddrs,
    sync::{mpsc, oneshot},
};
use tokio_stream::wrappers::ReceiverStream;
use tracing::{debug, error};

use super::connection::Connection;
use crate::{
    codec::{request::EncodedRequest, response::Response},
    Error, ReconnectInterval,
};

// Arc here is necessary to send same error to all waiting in-flights
pub(crate) type DispatcherResponse = Result<Response, Error>;
pub(crate) type DispatcherRequest = (EncodedRequest, oneshot::Sender<DispatcherResponse>);

pub(crate) struct DispatcherSender {
    tx: mpsc::Sender<DispatcherRequest>,
}

impl DispatcherSender {
    pub(crate) async fn send(&self, request: EncodedRequest) -> DispatcherResponse {
        let (tx, rx) = oneshot::channel();
        self.tx
            .send((request, tx))
            .map_err(|_| Error::ConnectionClosed)
            .and_then(|_| async {
                rx.await
                    .map_err(|_| Error::ConnectionClosed)
                    .and_then(|x| x)
            })
            .await
    }
}

type ConnectDynFuture = dyn Future<Output = Result<Connection, Error>> + Send;

/// Dispatching messages from client to connection.
///
/// Currently no-op, in future it should handle reconnects, schema reloading, pooling.
pub(crate) struct Dispatcher {
    rx: ReceiverStream<DispatcherRequest>,
    conn: Option<Connection>,
    conn_factory: Box<dyn Fn() -> Pin<Box<ConnectDynFuture>> + Send + Sync>,
    reconnect_interval: Option<ReconnectInterval>,
}

impl Dispatcher {
    pub(crate) async fn new<A>(
        addr: A,
        user: Option<&str>,
        password: Option<&str>,
        connect_timeout: Option<Duration>,
        reconnect_interval: Option<ReconnectInterval>,
        dispatcher_internal_queue_size: usize,
    ) -> Result<(impl Future<Output = ()>, DispatcherSender), Error>
    where
        A: ToSocketAddrs + Display + Clone + Send + Sync + 'static,
    {
        let user: Option<String> = user.map(Into::into);
        let password: Option<String> = password.map(Into::into);
        let conn_factory = Box::new(move || {
            let addr = addr.clone();
            let user = user.clone();
            let password = password.clone();
            let connect_timeout = connect_timeout;
            Box::pin(async move {
                Connection::new(addr, user.as_deref(), password.as_deref(), connect_timeout).await
            }) as Pin<Box<ConnectDynFuture>>
        });

        let conn = conn_factory().await?;

        let (tx, rx) = mpsc::channel(dispatcher_internal_queue_size);

        Ok((
            Self {
                rx: ReceiverStream::new(rx),
                conn: Some(conn),
                conn_factory,
                reconnect_interval,
            }
            .run(),
            DispatcherSender { tx },
        ))
    }

    async fn reconnect(&mut self) {
        let mut reconn_int_state = self
            .reconnect_interval
            .as_ref()
            .map(ReconnectIntervalState::from);
        loop {
            match (self.conn_factory)().await {
                Ok(conn) => {
                    self.conn = Some(conn);
                    return;
                }
                Err(err) => {
                    error!("Failed to reconnect to Tarantool: {:#}", err);
                    if let Some(ref mut x) = reconn_int_state {
                        tokio::time::sleep(x.next_timeout()).await;
                    }
                }
            }
        }
    }

    pub(crate) async fn run(mut self) {
        debug!("Starting dispatcher");
        loop {
            if let Some(conn) = self.conn.take() {
                if conn.run(&mut self.rx).await.is_ok() {
                    return;
                }
            } else {
                self.reconnect().await;
            }
        }
    }
}

/// Get interval before next reconnect attempt.
#[derive(Debug)]
enum ReconnectIntervalState {
    Fixed(Duration),
    ExponentialBackoff {
        state: ExponentialBackoff,
        max: Duration,
    },
}

impl ReconnectIntervalState {
    fn next_timeout(&mut self) -> Duration {
        match self {
            ReconnectIntervalState::Fixed(x) => *x,

            ReconnectIntervalState::ExponentialBackoff { ref mut state, max } => {
                dbg!(state).next_backoff().unwrap_or(*max)
            }
        }
    }
}

impl From<&ReconnectInterval> for ReconnectIntervalState {
    fn from(value: &ReconnectInterval) -> Self {
        match value {
            ReconnectInterval::Fixed(x) => Self::Fixed(*x),
            ReconnectInterval::ExponentialBackoff {
                min,
                max,
                randomization_factor,
                multiplier,
            } => {
                let state = ExponentialBackoffBuilder::new()
                    .with_initial_interval(*min)
                    .with_max_interval(*max)
                    .with_randomization_factor(*randomization_factor)
                    .with_multiplier(*multiplier)
                    .with_max_elapsed_time(None)
                    .build();
                Self::ExponentialBackoff { state, max: *max }
            }
        }
    }
}
