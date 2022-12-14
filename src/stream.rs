use async_trait::async_trait;
use rmpv::Value;

use crate::{
    codec::request::IProtoRequestBody, errors::Error, Connection, ConnectionLike, Transaction,
    TransactionBuilder,
};

/// Abstraction, providing sequential processing of requests.
///
/// With streams there is a guarantee that the server instance will not handle the next request in a stream until it has completed the previous one ([docs](https://www.tarantool.io/en/doc/latest/dev_guide/internals/box_protocol/#binary-protocol-streams)).
///
/// # Example
///
/// ```rust,compile
/// use tarantool_rs::{Connection, ConnectionLike};
/// # use futures::FutureExt;
///
/// # async fn async_wrapper() {
/// let connection = Connection::builder().build("localhost:3301").await.unwrap();
///
/// // This will print 'fast' and then 'slow'
/// let eval_slow_fut = connection
///     .eval("fiber = require('fiber'); fiber.sleep(0.5); return ...;", vec!["slow".into()])
///     .inspect(|res| println!("{:?}", res));
/// let eval_fast_fut = connection
///     .eval("return ...;", vec!["fast".into()])
///     .inspect(|res| println!("{:?}", res));
/// let _ = tokio::join!(eval_slow_fut, eval_fast_fut);
///
/// // This will print 'slow' and then 'fast', since slow request was created first and have smaller sync
/// let stream = connection.stream();
/// let eval_slow_fut = stream
///     .eval("fiber = require('fiber'); fiber.sleep(0.5); return ...;", vec!["slow".into()])
///     .inspect(|res| println!("{:?}", res));
/// let eval_fast_fut = stream
///     .eval("return ...;", vec!["fast".into()])
///     .inspect(|res| println!("{:?}", res));
/// let _ = tokio::join!(eval_slow_fut, eval_fast_fut);
/// # }
/// ```

#[derive(Clone)]
pub struct Stream {
    connection: Connection,
    stream_id: u32,
}

impl Stream {
    pub(crate) fn new(conn: Connection) -> Self {
        let stream_id = conn.next_stream_id();
        Self {
            connection: conn,
            stream_id,
        }
    }
}

#[async_trait]
impl ConnectionLike for Stream {
    async fn send_request(&self, body: impl IProtoRequestBody) -> Result<Value, Error> {
        self.connection
            .send_request(body, Some(self.stream_id))
            .await
    }

    fn stream(&self) -> Stream {
        self.connection.stream()
    }

    fn transaction_builder(&self) -> TransactionBuilder {
        self.connection.transaction_builder()
    }

    async fn transaction(&self) -> Result<Transaction, Error> {
        self.connection.transaction().await
    }
}
