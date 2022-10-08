use std::collections::HashMap;

use futures::{SinkExt, TryFutureExt, TryStreamExt};
use tokio::{
    io::AsyncReadExt,
    net::{TcpStream, ToSocketAddrs},
    sync::{mpsc, oneshot},
};
use tokio_util::codec::Framed;
use tracing::{debug, error, trace};

use crate::{
    codec::{request::IProtoRequest, response::IProtoResponse, ClientCodec, IProtoGreeting},
    errors::{ChannelError, Error},
};

type ChannelResponse = Result<IProtoResponse, Error>;
type ChannelMpscMessage = (IProtoRequest, oneshot::Sender<ChannelResponse>);

pub(crate) struct ChannelTx {
    inner: mpsc::Sender<ChannelMpscMessage>,
}

impl ChannelTx {
    pub(crate) async fn send(&self, request: IProtoRequest) -> ChannelResponse {
        let (tx, rx) = oneshot::channel();
        self.inner
            .send((request, tx))
            .map_err(|_| ChannelError::ConnectionClosed.into())
            .and_then(|_| async {
                rx.await
                    .map_err(|_| ChannelError::ConnectionClosed.into())
                    .and_then(|x| x)
            })
            .await
    }
}

pub(crate) struct Channel {
    inner: Framed<TcpStream, ClientCodec>,
    rx: mpsc::Receiver<ChannelMpscMessage>,
    // TODO: replace HashMap with something different?
    // TODO: cleanup sometimes
    in_flights: HashMap<u32, oneshot::Sender<ChannelResponse>>,
}

impl Channel {
    // TODO: builder
    // TODO: maybe hide?
    pub(crate) async fn new<A: ToSocketAddrs>(addr: A) -> Result<(Self, ChannelTx), ChannelError> {
        let mut tcp = TcpStream::connect(addr).await?;

        let mut greeting_buffer = [0u8; 128];
        tcp.read_exact(&mut greeting_buffer).await?;
        let greeting = IProtoGreeting::decode_unchecked(&greeting_buffer);
        trace!("Salt: {:?}", greeting.salt);

        let (tx, rx) = mpsc::channel(1);

        Ok((
            Self {
                inner: Framed::new(tcp, ClientCodec::default()),
                rx,
                in_flights: HashMap::with_capacity(5),
            },
            ChannelTx { inner: tx },
        ))
    }

    fn pass_response(&mut self, response: IProtoResponse) {
        let sync = response.sync;
        if let Some(tx) = self.in_flights.remove(&sync) {
            if tx.send(Ok(response)).is_err() {
                debug!("Failed to pass response sync {}, receiver dropped", sync);
            }
        } else {
            debug!("Unknown sync {}", sync);
        }
    }

    async fn send_request(
        &mut self,
        request: IProtoRequest,
        tx: oneshot::Sender<ChannelResponse>,
    ) -> Result<(), ChannelError> {
        trace!(
            "Sending response with sync {}, stream_id {:?}",
            request.sync,
            request.stream_id
        );
        if self.in_flights.insert(request.sync, tx).is_some() {
            error!(
                "Duplicated sync ({}) found in channel! This is probably bug or you have too much in flight request",
                request.sync
            );
        }
        self.inner.send(request).await
    }

    /// Send error to all in flight requests and drop current channel.
    fn finish_with_error(self, err: ChannelError) {
        let err = Error::from(err);
        for (_, tx) in self.in_flights.into_iter() {
            let _ = tx.send(Err(err.clone()));
        }
    }
}

// TODO: unwraps
// TODO: handle situation when some futures still alive but clients all dropped
pub(crate) async fn run_channel(mut chan: Channel) {
    let err = loop {
        tokio::select! {
            next = chan.inner.try_next() => {
                let resp = match next {
                    Ok(Some(x)) => x,
                    Ok(None) => break ChannelError::ConnectionClosed,
                    Err(e) => break e
                };
                trace!("Received response for sync {}", resp.sync);
                chan.pass_response(resp);
            }
            next = chan.rx.recv() => {
                if let Some((request, tx)) = next {
                    if let Err(err) = chan.send_request(request, tx).await {
                        break err;
                    }
                } else {
                    debug!("All senders dropped");
                    return
                }
            }
        }
    };
    chan.finish_with_error(err);
}
