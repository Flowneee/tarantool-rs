use std::{fmt::Display, sync::Arc};

use futures::TryFutureExt;
use tokio::{
    net::ToSocketAddrs,
    sync::{mpsc, oneshot},
};
use tracing::debug;

use super::connection::Connection;
use crate::{
    codec::{request::Request, response::Response},
    TransportError,
};

// Arc here is necessary to send same error to all waiting in-flights
pub(crate) type DispatcherResponse = Result<Response, Arc<TransportError>>;
pub(crate) type DispatcherRequest = (Request, oneshot::Sender<DispatcherResponse>);

pub(crate) struct DispatcherSender {
    tx: mpsc::Sender<DispatcherRequest>,
}

impl DispatcherSender {
    pub(crate) async fn send(&self, request: Request) -> DispatcherResponse {
        let (tx, rx) = oneshot::channel();
        self.tx
            .send((request, tx))
            .map_err(|_| TransportError::ConnectionClosed.into())
            .and_then(|_| async {
                rx.await
                    .map_err(|_| TransportError::ConnectionClosed.into())
                    .and_then(|x| x)
            })
            .await
    }
}

/// Dispatching messages from client to connection.
///
/// Currently no-op, in future it should handle reconnects, schema reloading, pooling.
pub(crate) struct Dispatcher {
    rx: mpsc::Receiver<DispatcherRequest>,
    conn: Connection,
}

impl Dispatcher {
    pub(crate) async fn new<A>(
        addr: A,
        user: Option<&str>,
        password: Option<&str>,
    ) -> Result<(Self, DispatcherSender), TransportError>
    where
        A: ToSocketAddrs + Display,
    {
        let conn = Connection::new(addr, user, password).await?;

        // TODO: test whether increased size can help with performance
        let (tx, rx) = mpsc::channel(1);

        Ok((Self { rx, conn }, DispatcherSender { tx }))
    }

    pub(crate) async fn run(mut self) {
        debug!("Starting dispatcher");
        let err = loop {
            tokio::select! {
                next = self.conn.handle_next_response() => {
                    if let Err(e) = next {
                        break e;
                    }
                }
                next = self.rx.recv() => {
                    if let Some((request, tx)) = next {
                        if let Err(err) = self.conn.send_request(request, tx).await {
                            break err;
                        }
                    } else {
                        debug!("All senders dropped");
                        return
                    }
                }
            }
        };
        self.conn.finish_with_error(err);
    }
}
