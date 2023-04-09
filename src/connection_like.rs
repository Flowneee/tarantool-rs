use std::borrow::Cow;

use async_trait::async_trait;
use rmpv::Value;
use serde::de::DeserializeOwned;

use crate::{
    codec::{
        request::{Call, Eval, Ping, RequestBody},
        utils::deserialize_non_sql_response,
    },
    errors::Error,
    Stream, Transaction, TransactionBuilder,
};

#[async_trait]
pub trait ConnectionLike: private::Sealed {
    /// Send request, receiving raw response body.
    ///
    /// It is not recommended to use this method directly, since some requests
    /// should be only sent in specific situations and might break connection.
    async fn send_request(&self, body: impl RequestBody) -> Result<Value, Error>;

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

    async fn eval<I, T>(&self, expr: I, args: Vec<Value>) -> Result<T, Error>
    where
        I: Into<Cow<'static, str>> + Send,
        T: DeserializeOwned,
    {
        let body = self.send_request(Eval::new(expr, args)).await?;
        deserialize_non_sql_response(body)
    }

    async fn call<I, T>(&self, function_name: I, args: Vec<Value>) -> Result<T, Error>
    where
        I: Into<Cow<'static, str>> + Send,
        T: DeserializeOwned,
    {
        let body = self.send_request(Call::new(function_name, args)).await?;
        deserialize_non_sql_response(body)
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
