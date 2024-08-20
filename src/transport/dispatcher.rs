use std::{fmt::Display, future::Future, pin::Pin, time::Duration};

use backoff::{backoff::Backoff, ExponentialBackoff, ExponentialBackoffBuilder};
use futures::{FutureExt, TryFutureExt};
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
pub(crate) type DispatcherRequest = (EncodedRequest, DispatcherResponseSender);

pub(crate) enum DispatcherResponse {
    Finished(Result<Response, Error>),
    NeedsResend(EncodedRequest),
}

impl From<Result<Response, Error>> for DispatcherResponse {
    #[inline]
    fn from(value: Result<Response, Error>) -> Self {
        Self::Finished(value)
    }
}

impl From<Error> for DispatcherResponse {
    #[inline]
    fn from(value: Error) -> Self {
        Self::Finished(Err(value))
    }
}

impl From<EncodedRequest> for DispatcherResponse {
    #[inline]
    fn from(value: EncodedRequest) -> Self {
        Self::NeedsResend(value)
    }
}

#[repr(transparent)]
pub(crate) struct DispatcherResponseSender(oneshot::Sender<DispatcherResponse>);

impl DispatcherResponseSender {
    #[inline]
    pub(crate) fn send(
        self,
        value: impl Into<DispatcherResponse>,
    ) -> Result<(), DispatcherResponse> {
        self.0.send(value.into())
    }

    #[inline]
    pub(crate) fn is_closed(&self) -> bool {
        self.0.is_closed()
    }
}

pub(crate) struct DispatcherSender {
    tx: mpsc::Sender<DispatcherRequest>,
}

impl DispatcherSender {
    pub(crate) async fn send(&self, request: EncodedRequest) -> Result<Response, Error> {
        let mut request = Some(request);
        loop {
            let (tx, rx) = oneshot::channel();
            let tx = DispatcherResponseSender(tx);

            // SAFETY: initial value is put in Option immediately.
            // On next iterations value is put in Option right before `continue` expression.
            if let Err(send_err) = self.tx.send((request.take().unwrap(), tx)).await {
                request = Some(send_err.0 .0);
                continue;
            }

            match rx.await {
                Ok(DispatcherResponse::Finished(x)) => return x,
                Ok(DispatcherResponse::NeedsResend(x)) => {
                    request = Some(x);
                    continue;
                }
                Err(_) => return Err(Error::ConnectionClosed),
            }
        }
    }
}

type ConnectDynFuture = dyn Future<Output = Result<Connection, Error>> + Send;

/// Dispatching messages from client to connection.
///
/// Currently no-op, in future it should handle reconnects, schema reloading, pooling.
pub(crate) struct ConnectionWatcher {
    rx: ReceiverStream<DispatcherRequest>,
    conn: Option<Connection>,
    conn_factory: Box<dyn Fn() -> Pin<Box<ConnectDynFuture>> + Send + Sync>,
    reconnect_interval: Option<ReconnectInterval>,
}

impl ConnectionWatcher {
    pub(crate) async fn prepare<A>(
        addr: A,
        user: Option<String>,
        password: Option<String>,
        connect_timeout: Option<Duration>,
        reconnect_interval: Option<ReconnectInterval>,
        internal_simultaneous_requests_threshold: usize,
    ) -> Result<(impl Future<Output = ()>, DispatcherSender), Error>
    where
        A: ToSocketAddrs + Display + Clone + Send + Sync + 'static,
    {
        let conn_factory = Box::new(move || {
            Box::pin(
                tokio::time::timeout(
                    connect_timeout.unwrap_or(Duration::MAX),
                    Connection::new(
                        addr.clone(),
                        user.clone(),
                        password.clone(),
                        internal_simultaneous_requests_threshold,
                    ),
                )
                .map_err(|_| Error::ConnectTimeout)
                .and_then(|x| async { x }),
            ) as Pin<Box<ConnectDynFuture>>
        });

        let conn = conn_factory().await?;

        let (tx, rx) = mpsc::channel(internal_simultaneous_requests_threshold);

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
        debug!("Starting connection watcher");
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
