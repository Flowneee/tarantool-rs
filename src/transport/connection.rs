use std::{collections::HashMap, fmt::Display, time::Duration};

use futures::{
    future::{Fuse, FusedFuture},
    FutureExt, SinkExt, StreamExt, TryStreamExt,
};

use tokio::{
    io::AsyncReadExt,
    net::{
        tcp::{OwnedReadHalf, OwnedWriteHalf},
        TcpStream, ToSocketAddrs,
    },
    pin,
    sync::mpsc,
    task::JoinHandle,
};
use tokio_stream::wrappers::ReceiverStream;
use tokio_util::codec::{FramedRead, FramedWrite};
use tracing::{debug, error, trace, warn};

use super::dispatcher::{DispatcherRequest, DispatcherResponse, DispatcherResponseSender};
use crate::{
    codec::{
        request::{Auth, EncodedRequest},
        response::{Response, ResponseBody},
        ClientCodec, Greeting,
    },
    errors::{CodecEncodeError, ConnectionError, Error},
};

struct ConnectionData {
    in_flights: HashMap<u32, DispatcherResponseSender>,
    next_sync: u32,
}

impl Default for ConnectionData {
    fn default() -> Self {
        Self {
            in_flights: HashMap::with_capacity(5),
            next_sync: 0,
        }
    }
}

impl ConnectionData {
    #[inline]
    fn next_sync(&mut self) -> u32 {
        let next = self.next_sync;
        self.next_sync += 1;
        next
    }

    /// Prepare request for sending to server.
    ///
    /// Set `sync` value and attempt to store this message in in-flight storage.
    ///
    /// `Err` means that message was not prepared and should not be sent.
    /// This function also take care of reporting error through `tx`.
    #[inline]
    fn try_prepare_request(
        &mut self,
        request: &mut EncodedRequest,
        tx: DispatcherResponseSender,
    ) -> Result<(), ()> {
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
            // TODO: probably could respond with NeedsResend
            if new.send(Error::DuplicatedSync(request.sync)).is_err() {
                warn!(
                    "Failed to pass error to sync {}, receiver dropped",
                    request.sync
                );
            }
            return Err(());
        }
        Ok(())
    }

    /// Send result of processing request (by sync) to client.
    #[inline]
    fn respond_to_client(&mut self, sync: u32, response: impl Into<DispatcherResponse>) {
        if let Some(tx) = self.in_flights.remove(&sync) {
            if tx.send(response).is_err() {
                warn!("Failed to pass response sync {}, receiver dropped", sync);
            }
        } else {
            warn!("Unknown sync {}", sync);
        }
    }

    /// Send error to all in-flight requests and drop them.
    #[inline]
    fn send_error_to_all_in_flights(&mut self, err: ConnectionError) {
        for (_, tx) in self.in_flights.drain() {
            let _ = tx.send(Error::from(err.clone()));
        }
    }

    /// Return requests to be resent
    #[inline]
    fn return_requests_to_be_resent(&mut self, requests: Vec<EncodedRequest>) {
        for x in requests {
            self.respond_to_client(x.sync, x)
        }
    }
}

// TODO: cancel
async fn writer_task(
    mut rx: mpsc::Receiver<EncodedRequest>,
    mut stream: FramedWrite<OwnedWriteHalf, ClientCodec>,
) -> (Result<(), (u32, CodecEncodeError)>, Vec<EncodedRequest>) {
    let mut result = Ok(());

    while let Some(x) = rx.recv().await {
        let sync = x.sync;
        if let Err(err) = stream.send(x).await {
            result = Err((sync, err));
            break;
        }
    }

    // Close internal queue and extract all remaining requests
    rx.close();
    let mut remaining_requests = Vec::new();
    while let Ok(next) = rx.try_recv() {
        remaining_requests.push(next);
    }

    // TODO: reenable or pass strema back into main task
    // if let Err(err) = stream.into_inner().shutdown().await {
    //     warn!("Failed to shutdown TCP stream cleanly: {err}");
    // }

    (result, remaining_requests)
}

type WriterTaskJoinHandle = JoinHandle<(Result<(), (u32, CodecEncodeError)>, Vec<EncodedRequest>)>;

pub(crate) struct Connection {
    read_stream: FramedRead<OwnedReadHalf, ClientCodec>,
    writer_tx: mpsc::Sender<EncodedRequest>,
    writer_task_handle: WriterTaskJoinHandle,
    data: ConnectionData,
}

impl Connection {
    async fn new_inner<A>(
        addr: A,
        user: Option<&str>,
        password: Option<&str>,
        internal_simultaneous_requests_threshold: usize,
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

        let mut conn_data = ConnectionData::default();

        if let Some(user) = user {
            Self::auth(
                &mut read_stream,
                &mut write_stream,
                conn_data.next_sync(),
                user,
                password,
                &greeting.salt,
            )
            .await?;
        }

        // TODO: review size of this queue
        // Make this queue slightly larger than queue between Client and Dispatcher
        let (writer_tx, writer_rx) =
            mpsc::channel(internal_simultaneous_requests_threshold / 100 * 105);
        let writer_task_handle = tokio::spawn(writer_task(writer_rx, write_stream));

        let this = Self {
            read_stream,
            writer_tx,
            writer_task_handle,
            data: conn_data,
        };

        Ok(this)
    }

    pub(super) async fn new<A>(
        addr: A,
        user: Option<&str>,
        password: Option<&str>,
        timeout: Option<Duration>,
        internal_simultaneous_requests_threshold: usize,
    ) -> Result<Self, Error>
    where
        A: ToSocketAddrs + Display,
    {
        match timeout {
            Some(dur) => tokio::time::timeout(
                dur,
                Self::new_inner(
                    addr,
                    user,
                    password,
                    internal_simultaneous_requests_threshold,
                ),
            )
            .await
            .map_err(|_| Error::ConnectTimeout)
            .and_then(|x| x),
            None => {
                Self::new_inner(
                    addr,
                    user,
                    password,
                    internal_simultaneous_requests_threshold,
                )
                .await
            }
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

        let resp = Self::get_next_stream_value(read_stream).await?;
        match resp.body {
            ResponseBody::Ok(_x) => Ok(()),
            ResponseBody::Error(err) => Err(Error::Auth(err)),
        }
    }

    #[inline]
    async fn get_next_stream_value(
        read_stream: &mut FramedRead<OwnedReadHalf, ClientCodec>,
    ) -> Result<Response, ConnectionError> {
        match read_stream.try_next().await {
            Ok(Some(x)) => Ok(x),
            Ok(None) => Err(ConnectionError::ConnectionClosed),
            Err(e) => Err(e.into()),
        }
    }

    #[inline]
    fn handle_response(connection_data: &mut ConnectionData, response: Response) {
        trace!(
            "Received response for sync {}, schema version {}",
            response.sync,
            response.schema_version
        );
        connection_data.respond_to_client(response.sync, Ok(response));
    }

    /// Run connection until it breaks of `rx` is closed.
    ///
    /// `Ok` means `rx` was closed and connection should not be restarted.
    /// `Err` means connection was dropped due to some error.
    pub(crate) async fn run(
        self,
        client_rx: &mut ReceiverStream<DispatcherRequest>,
    ) -> Result<(), ()> {
        let Self {
            mut read_stream,
            writer_tx,
            writer_task_handle,
            mut data,
        } = self;

        let mut not_sent_requests = Vec::new();

        let send_to_writer_future = Fuse::terminated();
        pin!(send_to_writer_future);

        let result = loop {
            tokio::select! {
                // Read value from TCP stream
                next = Connection::get_next_stream_value(&mut read_stream) => {
                    match next {
                        Ok(x) => Connection::handle_response(&mut data, x),
                        Err(err) => break Err(err),
                    }
                }

                // Read value from internal queue if nothing being sent to writer
                next = client_rx.next(), if send_to_writer_future.is_terminated() => {
                    if let Some((mut request, tx)) = next {
                        // If failed to prepare request or client already
                        // dropped oneshot - just go to next
                        if tx.is_closed() || data
                            .try_prepare_request(&mut request, tx)
                            .is_err()
                        {
                            continue;
                        }

                        send_to_writer_future.set(writer_tx.send(request).fuse());
                    } else {
                        // TODO: actually don't quit until all in-flights processed
                        debug!("All senders dropped");
                        break Ok(());
                    }
                }

                // Await sending request to writer.
                // NOTE: For some reason checking Fuse for termination makes code _slightly_ faster
                send_res = &mut send_to_writer_future, if !send_to_writer_future.is_terminated() => {
                    // Error means writer rx is closed and connection should be terminated.
                    if let Err(err) = send_res {
                        not_sent_requests.push(err.0);
                        break Err(ConnectionError::ConnectionClosed)
                    }
                    // TODO: somehow return EncodedRequest from Err variant, so it can be retried

                    // Do nothing, since on success there is nothing to do,
                    // and on error we can only response to client with ConnectionClosed,
                    // which will happen anyway in next branch on next (or so) iteration.
                }
            }
        };

        // Wait for writer task to finish
        match writer_task_handle.await {
            Err(err) => {
                error!("Failed to await writer task's handle: {err}");
            }
            Ok((result, not_sent_requests_from_writer)) => {
                not_sent_requests.extend(not_sent_requests_from_writer);

                if let Err((sync, err)) = result {
                    data.respond_to_client(sync, Err(err.into()))
                }
            }
        }

        // Respond to all in flights with error
        data.send_error_to_all_in_flights(
            result
                .clone()
                .err()
                .unwrap_or(ConnectionError::ConnectionClosed),
        );

        // Schedule all not sent requests to resend
        data.return_requests_to_be_resent(not_sent_requests);

        result.map_err(drop)
    }
}
