use std::borrow::Cow;

use async_trait::async_trait;
use bytes::Bytes;
use rmpv::Value;

use crate::{
    codec::{
        request::{Call, Eval, Ping, RequestBody},
        utils::data_from_response_body,
    },
    errors::Error,
    Stream, Transaction, TransactionBuilder,
};

#[async_trait]
pub trait ConnectionLike: private::Sealed {
    /// Send request.
    ///
    /// On successfull response return raw bytes which should be manually parsed
    /// accordingly to Tarantool binary protocol.
    ///
    /// It is not recommended to use this method directly, since some requests
    /// should be only sent in specific situations and might break connection.
    async fn send_request(&self, body: impl RequestBody) -> Result<Bytes, Error>;

    /// Get new [`Stream`].
    ///
    /// It is safe to create `Stream` from any type, implementing current trait.
    fn stream(&self) -> Stream;

    /// Prepare [`TransactionBuilder`], which can be used to override parameters and create
    /// [`Transaction`].
    ///
    /// It is safe to create `TransactionBuilder` from any type.
    fn transaction_builder(&self) -> TransactionBuilder;

    /// Create [`Transaction`] with parameters from builder.
    ///
    /// It is safe to create `Transaction` from any type, implementing current trait.
    async fn transaction(&self) -> Result<Transaction, Error>;

    /// Send PING request ([docs](https://www.tarantool.io/en/doc/latest/dev_guide/internals/box_protocol/#iproto-ping-0x40)).
    async fn ping(&self) -> Result<(), Error> {
        self.send_request(Ping {}).await.map(drop)
    }

    async fn eval<I>(&self, expr: I, args: Vec<Value>) -> Result<Value, Error>
    where
        I: Into<Cow<'static, str>> + Send,
    {
        let body = self.send_request(Eval::new(expr, args)).await?;
        let parsed_body = rmpv::decode::read_value_ref(body).unwrap();
        Ok(data_from_response_body(body)?)
    }

    async fn call<I>(&self, function_name: I, args: Vec<Value>) -> Result<Value, Error>
    where
        I: Into<Cow<'static, str>> + Send,
    {
        let body = self.send_request(Call::new(function_name, args)).await?;
        Ok(data_from_response_body(body)?)
    }
}

mod private {
    use crate::{Connection, Stream, Transaction};

    #[doc(hidden)]
    pub trait Sealed {}

    impl Sealed for Connection {}
    impl Sealed for Stream {}
    impl Sealed for Transaction {}
}
