use std::borrow::Cow;

use async_trait::async_trait;
use rmpv::Value;
use serde::de::DeserializeOwned;

use crate::{
    codec::{
        request::{Call, Delete, Eval, Insert, Ping, Replace, RequestBody, Select, Update, Upsert},
        utils::deserialize_non_sql_response,
    },
    errors::Error,
    IteratorType, Stream, Transaction, TransactionBuilder,
};

#[async_trait]
pub trait ConnectionLike: private::Sealed + Sync {
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

    // TODO: docs
    async fn eval<I, T>(&self, expr: I, args: Vec<Value>) -> Result<T, Error>
    where
        I: Into<Cow<'static, str>> + Send,
        T: DeserializeOwned,
    {
        let body = self.send_request(Eval::new(expr, args)).await?;
        deserialize_non_sql_response(body)
    }

    // TODO: docs
    async fn call<I, T>(&self, function_name: I, args: Vec<Value>) -> Result<T, Error>
    where
        I: Into<Cow<'static, str>> + Send,
        T: DeserializeOwned,
    {
        let body = self.send_request(Call::new(function_name, args)).await?;
        deserialize_non_sql_response(body)
    }

    // TODO: docs
    async fn select<T>(
        &self,
        space_id: u32,
        index_id: u32,
        limit: Option<u32>,
        offset: Option<u32>,
        iterator: Option<IteratorType>,
        keys: Vec<Value>,
    ) -> Result<Vec<T>, Error>
    where
        T: DeserializeOwned,
    {
        let body = self
            .send_request(Select::new(
                space_id, index_id, limit, offset, iterator, keys,
            ))
            .await?;
        deserialize_non_sql_response(body)
    }

    // TODO: docs
    // TODO: decode response
    async fn insert(&self, space_id: u32, tuple: Vec<Value>) -> Result<(), Error> {
        let _ = self.send_request(Insert::new(space_id, tuple)).await?;
        Ok(())
    }

    // TODO: structured tuple
    // TODO: decode response
    async fn update(
        &self,
        space_id: u32,
        index_id: u32,
        index_base: Option<u32>,
        keys: Vec<Value>,
        tuple: Vec<Value>,
    ) -> Result<(), Error> {
        let _ = self
            .send_request(Update::new(space_id, index_id, index_base, keys, tuple))
            .await?;
        Ok(())
    }

    // TODO: structured tuple
    // TODO: decode response
    // TODO: maybe set index base to 1 always?
    async fn upsert(
        &self,
        space_id: u32,
        index_base: u32,
        ops: Vec<Value>,
        tuple: Vec<Value>,
    ) -> Result<(), Error> {
        let _ = self
            .send_request(Upsert::new(space_id, index_base, ops, tuple))
            .await?;
        Ok(())
    }

    // TODO: structured tuple
    // TODO: decode response
    async fn replace(&self, space_id: u32, keys: Vec<Value>) -> Result<(), Error> {
        let _ = self.send_request(Replace::new(space_id, keys)).await?;
        Ok(())
    }

    // TODO: structured tuple
    // TODO: decode response
    async fn delete(&self, space_id: u32, index_id: u32, keys: Vec<Value>) -> Result<(), Error> {
        let _ = self
            .send_request(Delete::new(space_id, index_id, keys))
            .await?;
        Ok(())
    }
}

#[async_trait]
impl<C: ConnectionLike + private::Sealed + Sync> ConnectionLike for &C {
    async fn send_request(&self, body: impl RequestBody) -> Result<Value, Error> {
        (*self).send_request(body).await
    }

    fn stream(&self) -> Stream {
        (*self).stream()
    }

    fn transaction_builder(&self) -> TransactionBuilder {
        (*self).transaction_builder()
    }

    async fn transaction(&self) -> Result<Transaction, Error> {
        (*self).transaction().await
    }
}

mod private {
    use crate::{Connection, Stream, Transaction};

    #[doc(hidden)]
    pub trait Sealed {}

    impl Sealed for Connection {}
    impl Sealed for Stream {}
    impl Sealed for Transaction {}
    impl<S: Sealed> Sealed for &S {}
    impl<S: Sealed> Sealed for &mut S {}
}
