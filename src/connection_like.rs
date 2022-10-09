use std::borrow::Cow;

use async_trait::async_trait;
use rmpv::Value;

use crate::{
    codec::{
        request::{IProtoCall, IProtoEval, IProtoPing, IProtoRequestBody},
        utils::data_from_response_body,
    },
    errors::Error,
    Stream, Transaction, TransactionBuilder,
};

#[async_trait]
pub trait ConnectionLike: private::Sealed {
    /// Send request using some internal connection object.
    ///
    /// It is not recommended to use this method directly, since some requests
    /// should be only sent in specific situations and might break connection.
    async fn send_request(&self, body: impl IProtoRequestBody) -> Result<Value, Error>;

    /// Get new [`Stream`].
    ///
    /// It is safe to create `Stream` from any type, implementing current trait,
    /// since all of them use underlying [`Connection`] object.
    fn stream(&self) -> Stream;

    /// Prepare [`TransactionBuilder`], which can be used to override parameters and create
    /// [`Transaction`].
    ///
    /// It is safe to create `TransactionBuilder` from any type, implementing current trait,
    /// since all of them use underlying [`Connection`] object.
    fn transaction_builder(&self) -> TransactionBuilder;

    /// Create [`Transaction`] with default connection's parameters.
    ///
    /// It is safe to create `Transaction` from any type, implementing current trait,
    /// since all of them use underlying [`Connection`] object.
    async fn transaction(&self) -> Result<Transaction, Error>;

    /// Send PING request ([docs](https://www.tarantool.io/en/doc/latest/dev_guide/internals/box_protocol/#iproto-ping-0x40)).
    async fn ping(&self) -> Result<(), Error> {
        self.send_request(IProtoPing {}).await.map(drop)
    }

    async fn eval<I>(&self, expr: I, args: Vec<Value>) -> Result<Value, Error>
    where
        I: Into<Cow<'static, str>> + Send,
    {
        let body = self.send_request(IProtoEval::new(expr, args)).await?;
        Ok(data_from_response_body(body)?)
    }

    async fn call<I>(&self, function_name: I, args: Vec<Value>) -> Result<Value, Error>
    where
        I: Into<Cow<'static, str>> + Send,
    {
        let body = self
            .send_request(IProtoCall::new(function_name, args))
            .await?;
        Ok(data_from_response_body(body)?)
    }
}

mod private {
    use crate::{Connection, Stream, Transaction};

    pub trait Sealed {}

    impl Sealed for Connection {}
    impl Sealed for Stream {}
    impl Sealed for Transaction {}
}
