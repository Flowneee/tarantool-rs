use std::{
    collections::HashMap,
    fmt::Display,
    sync::atomic::{AtomicU32, Ordering},
    time::Duration,
};

use futures::{future::pending, Future, SinkExt, TryStreamExt};
use tokio::{
    io::AsyncReadExt,
    net::{
        tcp::{OwnedReadHalf, OwnedWriteHalf},
        TcpStream, ToSocketAddrs,
    },
    pin,
    sync::{mpsc, oneshot},
};
use tokio_util::{
    codec::{Framed, FramedRead, FramedWrite},
    either::Either,
};
use tracing::{debug, trace, warn};

use super::dispatcher::{DispatcherRequest, DispatcherResponse};
use crate::{
    codec::{
        request::{Auth, EncodedRequest},
        response::{Response, ResponseBody},
        ClientCodec, Greeting,
    },
    errors::{CodecDecodeError, CodecEncodeError, Error},
};

pub(crate) struct Connection {
    read_stream: Option<FramedRead<OwnedReadHalf, ClientCodec>>,
    write_stream: Option<FramedWrite<OwnedWriteHalf, ClientCodec>>,
    in_flights: HashMap<u32, oneshot::Sender<DispatcherResponse>>,
    next_sync: u32,
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

        let (read_tcp_stream, write_tcp_stream) = tcp.into_split();
        let mut read_stream = FramedRead::new(read_tcp_stream, ClientCodec::default());
        let mut write_stream = FramedWrite::new(write_tcp_stream, ClientCodec::default());

        if let Some(user) = user {
            Self::auth(
                &mut read_stream,
                &mut write_stream,
                0,
                user,
                password,
                &greeting.salt,
            )
            .await?;
        }

        let this = Self {
            read_stream: Some(read_stream),
            write_stream: Some(write_stream),
            in_flights: HashMap::with_capacity(5),
            next_sync: 0,
        };

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

    async fn auth(
        read_stream: &mut FramedRead<OwnedReadHalf, ClientCodec>,
        write_stream: &mut FramedWrite<OwnedWriteHalf, ClientCodec>,
        sync: u32,
        user: &str,
        password: Option<&str>,
        salt: &[u8],
    ) -> Result<(), Error> {
        let mut request = EncodedRequest::new(Auth::new(user, password, salt), None).unwrap();
        *request.sync_mut() = sync;

        trace!("Sending auth request");
        write_stream.send(request).await?;

        let resp = Self::get_next_stream_value2(read_stream).await?;
        match resp.body {
            ResponseBody::Ok(_x) => Ok(()),
            ResponseBody::Error(err) => Err(Error::Auth(err)),
        }
    }

    // TODO: maybe other Ordering??
    fn next_sync(&mut self) -> u32 {
        let next = self.next_sync;
        self.next_sync += 1;
        next
    }

    // pub(super) async fn send_request(
    //     &mut self,
    //     mut request: EncodedRequest,
    //     tx: oneshot::Sender<DispatcherResponse>,
    // ) -> Result<(), tokio::io::Error> {
    //     let sync = self.next_sync();
    //     *request.sync_mut() = sync;
    //     trace!(
    //         "Sending request with sync {}, stream_id {:?}",
    //         request.sync,
    //         request.stream_id
    //     );
    //     // TODO: replace with try_insert when stabilized
    //     // If sync already assigned to another request, return an error
    //     // for current request
    //     if let Some(old) = self.in_flights.insert(request.sync, tx) {
    //         let new = self
    //             .in_flights
    //             .insert(request.sync, old)
    //             .expect("Shouldn't panic, value was just inserted");
    //         if new.send(Err(Error::DuplicatedSync(request.sync))).is_err() {
    //             warn!(
    //                 "Failed to pass error to sync {}, receiver dropped",
    //                 request.sync
    //             );
    //         }
    //         return Ok(());
    //     }
    //     match self.write_stream.send(request).await {
    //         Ok(x) => Ok(x),
    //         Err(CodecEncodeError::Encode(err)) => {
    //             if self
    //                 .in_flights
    //                 .remove(&sync)
    //                 .expect("Shouldn't panic, value was just inserted")
    //                 .send(Err(err.into()))
    //                 .is_err()
    //             {
    //                 warn!("Failed to pass error to sync {}, receiver dropped", sync);
    //             }
    //             Ok(())
    //         }
    //         Err(CodecEncodeError::Io(err)) => Err(err),
    //     }
    // }

    // Return false if request shouldn't be sent and should be dropped
    // instead
    fn prepare_request(
        &mut self,
        request: &mut EncodedRequest,
        tx: oneshot::Sender<DispatcherResponse>,
    ) -> bool {
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
            return false;
        }
        true
    }

    fn handle_send_result(
        &mut self,
        sync: u32,
        result: Result<(), CodecEncodeError>,
    ) -> Result<(), tokio::io::Error> {
        match result {
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

    // async fn get_next_stream_value(&mut self) -> Result<Response, CodecDecodeError> {
    //     match self.read_stream.try_next().await {
    //         Ok(Some(x)) => Ok(x),
    //         Ok(None) => Err(CodecDecodeError::Closed),
    //         Err(e) => Err(e),
    //     }
    // }

    async fn get_next_stream_value2(
        read_stream: &mut FramedRead<OwnedReadHalf, ClientCodec>,
    ) -> Result<Response, CodecDecodeError> {
        match read_stream.try_next().await {
            Ok(Some(x)) => Ok(x),
            Ok(None) => Err(CodecDecodeError::Closed),
            Err(e) => Err(e),
        }
    }

    // pub(super) async fn handle_next_response(&mut self) -> Result<(), CodecDecodeError> {
    //     let resp = self.get_next_stream_value().await?;
    //     trace!(
    //         "Received response for sync {}, schema version {}",
    //         resp.sync,
    //         resp.schema_version
    //     );
    //     self.pass_response(resp);
    //     Ok(())
    // }

    fn handle_response(&mut self, response: Response) {
        trace!(
            "Received response for sync {}, schema version {}",
            response.sync,
            response.schema_version
        );
        self.pass_response(response);
    }

    /// Send error to all in flight requests and drop current transport.
    pub(super) fn finish_with_error(&mut self, err: CodecDecodeError) {
        for (_, tx) in self.in_flights.drain() {
            let _ = tx.send(Err(err.clone().into()));
        }
    }

    pub(crate) async fn run(mut self, rx: &mut mpsc::Receiver<DispatcherRequest>) -> bool {
        let mut write_stream = self.write_stream.take();
        let mut read_stream = self.read_stream.take().unwrap();

        let send_fut = Either::Left(pending());
        pin!(send_fut);

        let err = loop {
            tokio::select! {
                next = Self::get_next_stream_value2(&mut read_stream) => {
                    match next {
                        Ok(x) => self.handle_response(x),
                        Err(err) => break err,
                    }
                }
                next = rx.recv(), if write_stream.is_some() => {
                    if let Some((mut request, tx)) = next {
                        // Check whether tx is closed in case someone cancelled request
                        // while it was in queue
                        if !tx.is_closed() {
                            if self.prepare_request(&mut request, tx) {
                                let mut write_stream_ = write_stream.take().unwrap();
                                send_fut.set(Either::Right(async move {
                                    let sync = request.sync;
                                    let send_res = write_stream_.send(request).await;
                                    (send_res, sync, write_stream_)
                                }));
                            }
                        }
                    } else {
                        debug!("All senders dropped");
                        return true
                    }
                }
                (send_res, sync, write_stream_) = &mut send_fut => {
                    send_fut.set(Either::Left(pending()));
                    write_stream = Some(write_stream_);

                    if let Err(err) = self.handle_send_result(sync, send_res) {
                        break err.into();
                    }
                }
            }
        };
        self.finish_with_error(err);
        false
    }
}
