use std::collections::HashMap;

use futures::{SinkExt, TryFutureExt, TryStreamExt};
use tokio::{
    io::AsyncReadExt,
    net::{TcpStream, ToSocketAddrs},
    sync::{mpsc, oneshot},
};
use tokio_util::codec::Framed;
use tracing::{debug, trace, warn};

use crate::{
    codec::{request::Request, response::Response, ClientCodec, Greeting},
    errors::{Error, TransportError},
};

type TransportResponse = Result<Response, Error>;
type TransportRequest = (Request, oneshot::Sender<TransportResponse>);

pub(crate) struct TransportSender {
    tx: mpsc::Sender<TransportRequest>,
}

impl TransportSender {
    pub(crate) async fn send(&self, request: Request) -> TransportResponse {
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

pub(crate) struct Transport {
    stream: Framed<TcpStream, ClientCodec>,
    rx: mpsc::Receiver<TransportRequest>,
    // TODO: replace HashMap with something different?
    // TODO: cleanup sometimes
    in_flights: HashMap<u32, oneshot::Sender<TransportResponse>>,
}

impl Transport {
    // TODO: builder
    // TODO: maybe hide?
    pub(crate) async fn new<A: ToSocketAddrs>(
        addr: A,
    ) -> Result<(Self, TransportSender, Vec<u8>), TransportError> {
        let mut tcp = TcpStream::connect(addr).await?;

        let mut greeting_buffer = [0u8; 128];
        tcp.read_exact(&mut greeting_buffer).await?;
        let greeting = Greeting::decode_unchecked(&greeting_buffer);
        trace!("Salt: {:?}", greeting.salt);

        // TODO: test whether increased size can help with performance
        let (tx, rx) = mpsc::channel(1);

        Ok((
            Self {
                stream: Framed::new(tcp, ClientCodec::default()),
                rx,
                in_flights: HashMap::with_capacity(5),
            },
            TransportSender { tx },
            greeting.salt,
        ))
    }

    // TODO: allow to alter logging levels
    fn pass_response(&mut self, response: Response) {
        let sync = response.sync;
        if let Some(tx) = self.in_flights.remove(&sync) {
            if tx.send(Ok(response)).is_err() {
                warn!("Failed to pass response sync {}, receiver dropped", sync);
            }
        } else {
            warn!("Unknown sync {}", sync);
        }
    }

    async fn send_request(
        &mut self,
        request: Request,
        tx: oneshot::Sender<TransportResponse>,
    ) -> Result<(), TransportError> {
        trace!(
            "Sending response with sync {}, stream_id {:?}",
            request.sync,
            request.stream_id
        );
        if self.in_flights.insert(request.sync, tx).is_some() {
            return Err(TransportError::DuplicatedSync(request.sync));
        }
        self.stream.send(request).await
    }

    /// Send error to all in flight requests and drop current channel.
    fn finish_with_error(self, err: TransportError) {
        let err = Error::from(err);
        for (_, tx) in self.in_flights.into_iter() {
            let _ = tx.send(Err(err.clone()));
        }
    }

    // TODO: unwraps
    // TODO: handle situation when some futures still alive but clients all dropped
    pub(crate) async fn run(mut self) {
        let err = loop {
            tokio::select! {
                next = self.stream.try_next() => {
                    let resp = match next {
                        Ok(Some(x)) => x,
                        Ok(None) => break TransportError::ConnectionClosed,
                        Err(e) => break e
                    };
                    trace!("Received response for sync {}", resp.sync);
                    self.pass_response(resp);
                }
                next = self.rx.recv() => {
                    if let Some((request, tx)) = next {
                        if let Err(err) = self.send_request(request, tx).await {
                            break err;
                        }
                    } else {
                        debug!("All senders dropped");
                        return
                    }
                }
            }
        };
        self.finish_with_error(err);
    }
}
