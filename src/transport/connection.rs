use std::{
    collections::HashMap,
    fmt::Display,
    sync::atomic::{AtomicU32, Ordering},
    time::Duration,
};

use futures::{SinkExt, TryStreamExt};
use tokio::{
    io::AsyncReadExt,
    net::{TcpStream, ToSocketAddrs},
    sync::oneshot,
};
use tokio_util::codec::Framed;
use tracing::{debug, trace, warn};

use super::dispatcher::DispatcherResponse;
use crate::{
    codec::{
        request::{Auth, EncodedRequest},
        response::{Response, ResponseBody},
        ClientCodec, Greeting,
    },
    errors::{CodecDecodeError, CodecEncodeError, Error},
};

pub(crate) struct Connection {
    stream: Framed<TcpStream, ClientCodec>,
    in_flights: HashMap<u32, oneshot::Sender<DispatcherResponse>>,
    next_sync: AtomicU32,
}

// TODO: cancel
impl Connection {
    async fn new_inner<A>(
        addr: A,
        user: Option<&str>,
        password: Option<&str>,
    ) -> Result<Self, Error>
    where
        A: ToSocketAddrs + Display,
    {
        debug!("Starting connection to Tarantool {}", addr);
        let mut tcp = TcpStream::connect(&addr).await?;
        trace!("Connection established to {}", addr);

        let mut greeting_buffer = [0u8; Greeting::SIZE];
        tcp.read_exact(&mut greeting_buffer).await?;
        let greeting = Greeting::decode(greeting_buffer)?;
        debug!("Server: {}", greeting.server);
        trace!("Salt: {:?}", greeting.salt);

        let mut this = Self {
            stream: Framed::new(tcp, ClientCodec::default()),
            in_flights: HashMap::with_capacity(5),
            next_sync: AtomicU32::new(0),
        };

        if let Some(user) = user {
            this.auth(user, password, &greeting.salt).await?;
        }

        Ok(this)
    }

    pub(super) async fn new<A>(
        addr: A,
        user: Option<&str>,
        password: Option<&str>,
        timeout: Option<Duration>,
    ) -> Result<Self, Error>
    where
        A: ToSocketAddrs + Display,
    {
        match timeout {
            Some(dur) => tokio::time::timeout(dur, Self::new_inner(addr, user, password))
                .await
                .map_err(|_| Error::ConnectTimeout)
                .and_then(|x| x),
            None => Self::new_inner(addr, user, password).await,
        }
    }

    async fn auth(&mut self, user: &str, password: Option<&str>, salt: &[u8]) -> Result<(), Error> {
        let mut request = EncodedRequest::new(Auth::new(user, password, salt), None).unwrap();
        *request.sync_mut() = self.next_sync();

        trace!("Sending auth request");
        self.stream.send(request).await?;

        let resp = self.get_next_stream_value().await?;
        match resp.body {
            ResponseBody::Ok(_x) => Ok(()),
            ResponseBody::Error(err) => Err(Error::Auth(err)),
        }
    }

    // TODO: maybe other Ordering??
    fn next_sync(&self) -> u32 {
        self.next_sync.fetch_add(1, Ordering::SeqCst)
    }

    pub(super) async fn send_request(
        &mut self,
        mut request: EncodedRequest,
        tx: oneshot::Sender<DispatcherResponse>,
    ) -> Result<(), tokio::io::Error> {
        let sync = self.next_sync();
        *request.sync_mut() = sync;
        trace!(
            "Sending request with sync {}, stream_id {:?}",
            request.sync,
            request.stream_id
        );
        // TODO: replace with try_insert when stabilized
        // If sync already assigned to another request, return an error
        // for current request
        if let Some(old) = self.in_flights.insert(request.sync, tx) {
            let new = self
                .in_flights
                .insert(request.sync, old)
                .expect("Shouldn't panic, value was just inserted");
            if new.send(Err(Error::DuplicatedSync(request.sync))).is_err() {
                warn!(
                    "Failed to pass error to sync {}, receiver dropped",
                    request.sync
                );
            }
            return Ok(());
        }
        match self.stream.send(request).await {
            Ok(x) => Ok(x),
            Err(CodecEncodeError::Encode(err)) => {
                if self
                    .in_flights
                    .remove(&sync)
                    .expect("Shouldn't panic, value was just inserted")
                    .send(Err(err.into()))
                    .is_err()
                {
                    warn!("Failed to pass error to sync {}, receiver dropped", sync);
                }
                Ok(())
            }
            Err(CodecEncodeError::Io(err)) => Err(err),
        }
    }

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

    async fn get_next_stream_value(&mut self) -> Result<Response, CodecDecodeError> {
        match self.stream.try_next().await {
            Ok(Some(x)) => Ok(x),
            Ok(None) => Err(CodecDecodeError::Closed),
            Err(e) => Err(e),
        }
    }

    pub(super) async fn handle_next_response(&mut self) -> Result<(), CodecDecodeError> {
        let resp = self.get_next_stream_value().await?;
        trace!(
            "Received response for sync {}, schema version {}",
            resp.sync,
            resp.schema_version
        );
        self.pass_response(resp);
        Ok(())
    }

    /// Send error to all in flight requests and drop current transport.
    pub(super) fn finish_with_error(&mut self, err: CodecDecodeError) {
        for (_, tx) in self.in_flights.drain() {
            let _ = tx.send(Err(err.clone().into()));
        }
    }
}
