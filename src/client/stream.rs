use async_trait::async_trait;

use rmpv::Value;

use super::{Connection, Transaction, TransactionBuilder};
use crate::{
    codec::request::{EncodedRequest},
    Executor, Result,
};

/// Abstraction, providing sequential processing of requests.
///
/// With streams there is a guarantee that the server instance will not handle the next request in a stream until it has completed the previous one ([docs](https://www.tarantool.io/en/doc/latest/dev_guide/internals/box_protocol/#binary-protocol-streams)).
///
/// # Example
///
/// ```rust,compile
/// use tarantool_rs::{Connection, ConnectionLike, Executor};
/// # use futures::FutureExt;
/// # use rmpv::Value;
///
/// # async fn async_wrapper() {
/// let connection = Connection::builder().build("localhost:3301").await.unwrap();
///
/// // This will print 'fast' and then 'slow'
/// let eval_slow_fut = connection
///     .eval::<_, Value>("fiber = require('fiber'); fiber.sleep(0.5); return ...;", vec!["slow".into()])
///     .inspect(|res| println!("{:?}", res));
/// let eval_fast_fut = connection
///     .eval::<_, Value>("return ...;", vec!["fast".into()])
///     .inspect(|res| println!("{:?}", res));
/// let _ = tokio::join!(eval_slow_fut, eval_fast_fut);
///
/// // This will print 'slow' and then 'fast', since slow request was created first and have smaller sync
/// let stream = connection.stream();
/// let eval_slow_fut = stream
///     .eval::<_, Value>("fiber = require('fiber'); fiber.sleep(0.5); return ...;", vec!["slow".into()])
///     .inspect(|res| println!("{:?}", res));
/// let eval_fast_fut = stream
///     .eval::<_, Value>("return ...;", vec!["fast".into()])
///     .inspect(|res| println!("{:?}", res));
/// let _ = tokio::join!(eval_slow_fut, eval_fast_fut);
/// # }
/// ```

#[derive(Clone)]
pub struct Stream {
    conn: Connection,
    stream_id: u32,
}

// TODO: convert stream to transaction and back
impl Stream {
    pub(crate) fn new(conn: Connection) -> Self {
        let stream_id = conn.next_stream_id();
        Self { conn, stream_id }
    }
}

#[async_trait]
impl Executor for Stream {
    async fn send_encoded_request(&self, mut request: EncodedRequest) -> Result<Value> {
        request.stream_id = Some(self.stream_id);
        self.conn.send_encoded_request(request).await
    }

    fn stream(&self) -> Stream {
        self.conn.stream()
    }

    fn transaction_builder(&self) -> TransactionBuilder {
        self.conn.transaction_builder()
    }

    async fn transaction(&self) -> Result<Transaction> {
        self.conn.transaction().await
    }
}
