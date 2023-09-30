use std::{collections::HashMap, fmt::Display, time::Duration};

use futures::{lock::BiLock, SinkExt, TryStreamExt};
use tokio::{
    io::{AsyncRead, AsyncReadExt},
    net::{
        tcp::{OwnedReadHalf, OwnedWriteHalf, ReadHalf, WriteHalf},
        TcpStream, ToSocketAddrs,
    },
    sync::{mpsc, oneshot},
};
use tokio_util::codec::{Framed, FramedRead, FramedWrite};
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

struct ConnectionData {
    in_flights: HashMap<u32, oneshot::Sender<DispatcherResponse>>,
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
        tx: oneshot::Sender<DispatcherResponse>,
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
            if new.send(Err(Error::DuplicatedSync(request.sync))).is_err() {
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
    fn respond_to_client(&mut self, sync: u32, result: Result<Response, Error>) {
        if let Some(tx) = self.in_flights.remove(&sync) {
            if tx.send(result).is_err() {
                warn!("Failed to pass response sync {}, receiver dropped", sync);
            }
        } else {
            warn!("Unknown sync {}", sync);
        }
    }

    /// Send error to all in-flight requests and drop them.
    #[inline]
    fn send_error_to_all_in_flights(&mut self, err: CodecDecodeError) {
        for (_, tx) in self.in_flights.drain() {
            let _ = tx.send(Err(err.clone().into()));
        }
    }
}

async fn sender_task(
    connection_data: &mut BiLock<ConnectionData>,
    write_stream: &mut FramedWrite<WriteHalf<'_>, ClientCodec>,
    rx: &mut mpsc::Receiver<DispatcherRequest>,
) -> Result<(), tokio::io::Error> {
    while let Some((mut request, tx)) = rx.recv().await {
        // Check whether tx is closed in case someone cancelled request
        // while it was in queue
        if tx.is_closed() {
            continue;
        }

        // If failed to prepare request - just go to next
        if connection_data
            .lock()
            .await
            .try_prepare_request(&mut request, tx)
            .is_err()
        {
            continue;
        }

        let sync = request.sync;
        let send_res = write_stream.send(request).await;
        let mut data_lock = connection_data.lock().await;
        if let Err(err) = Connection::handle_send_result(&mut data_lock, sync, send_res) {
            return Err(err);
        }
    }
    debug!("All senders dropped");
    return Ok(());
}

async fn receiver_task(
    connection_data: BiLock<ConnectionData>,
    mut read_stream: FramedRead<ReadHalf<'_>, ClientCodec>,
) -> Result<(), CodecDecodeError> {
    loop {
        match Connection::get_next_stream_value(&mut read_stream).await {
            Ok(x) => {
                let mut data_lock = connection_data.lock().await;
                Connection::handle_response(&mut data_lock, x)
            }
            Err(err) => return Err(err),
        }
    }
}

pub(crate) struct Connection {
    read_stream: FramedRead<OwnedReadHalf, ClientCodec>,
    write_stream: FramedWrite<OwnedWriteHalf, ClientCodec>,
    data: ConnectionData,
}

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

        let this = Self {
            read_stream,
            write_stream,
            data: conn_data,
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

        let resp = Self::get_next_stream_value(read_stream).await?;
        match resp.body {
            ResponseBody::Ok(_x) => Ok(()),
            ResponseBody::Error(err) => Err(Error::Auth(err)),
        }
    }

    #[inline]
    fn handle_send_result(
        connection_data: &mut ConnectionData,
        sync: u32,
        result: Result<(), CodecEncodeError>,
    ) -> Result<(), tokio::io::Error> {
        match result {
            Ok(x) => Ok(x),
            Err(CodecEncodeError::Encode(err)) => {
                connection_data.respond_to_client(sync, Err(err.into()));
                Ok(())
            }
            Err(CodecEncodeError::Io(err)) => Err(err),
        }
    }

    #[inline]
    async fn get_next_stream_value<T: AsyncRead + Unpin>(
        read_stream: &mut FramedRead<T, ClientCodec>,
    ) -> Result<Response, CodecDecodeError> {
        match read_stream.try_next().await {
            Ok(Some(x)) => Ok(x),
            Ok(None) => Err(CodecDecodeError::Closed),
            Err(e) => Err(e),
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

    /// Run connection.
    ///
    /// `Ok` means `rx` was closed and connection should not be restarted.
    /// `Err` means connection was dropped due to some error.
    pub(crate) async fn run(self, rx: &mut mpsc::Receiver<DispatcherRequest>) -> Result<(), ()> {
        let Self {
            read_stream,
            mut write_stream,
            data,
        } = self;

        let read = read_stream.into_inner();
        let write = write_stream.into_inner();
        let mut tcp = read.reunite(write).unwrap();
        let (read_stream, write_stream) = tcp.split();
        let mut read_stream = FramedRead::new(read_stream, ClientCodec::default());
        let mut write_stream = FramedWrite::new(write_stream, ClientCodec::default());

        let (mut sender_data_bilock, receiver_data_bilock) = BiLock::new(data);

        //let receiver_fut = tokio::spawn(receiver_task(receiver_data_bilock, read_stream));

        let res = tokio::select!(
            res = sender_task(&mut sender_data_bilock, &mut write_stream, rx) => {
                res.map_err(Into::into)
            }
            //res = receiver_fut => {
            res = receiver_task(receiver_data_bilock, read_stream) => {
                //res.unwrap().map_err(Into::into)
                res.map_err(Into::into)
            }
        );

        if let Some(err) = res.err() {
            sender_data_bilock
                .lock()
                .await
                .send_error_to_all_in_flights(err);
            Err(())
        } else {
            Ok(())
        }
    }
}
