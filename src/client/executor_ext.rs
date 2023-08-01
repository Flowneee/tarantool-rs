use std::borrow::Cow;

use async_trait::async_trait;
use futures::{future::BoxFuture, FutureExt};
use rmpv::Value;
use serde::de::DeserializeOwned;

use super::Executor;
use crate::{
    codec::request::{
        Call, Delete, EncodedRequest, Eval, Insert, Ping, Replace, Request, Select, Update, Upsert,
    },
    utils::extract_and_deserialize_iproto_data,
    IteratorType, Result,
};

/// Helper trait around [`Executor`] trait, which allows to send specific requests
/// with any type, implementing `Execitor` trait.
#[async_trait]
pub trait ExecutorExt: Executor {
    /// Send request, receiving raw response body.
    ///
    /// It is not recommended to use this method directly, since some requests
    /// should be only sent in specific situations and might break connection.
    fn send_request<R>(&self, body: R) -> BoxFuture<Result<Value>>
    where
        R: Request;

    /// Ping tarantool instance.
    async fn ping(&self) -> Result<()> {
        self.send_request(Ping {}).await.map(drop)
    }

    // TODO: add examples

    /// Evaluate Lua expression.
    ///
    /// Check [docs][crate#deserializing-lua-responses-in-call-and-eval] on how to deserialize response.
    async fn eval<I, T>(&self, expr: I, args: Vec<Value>) -> Result<T>
    where
        I: Into<Cow<'static, str>> + Send,
        T: DeserializeOwned,
    {
        let body = self.send_request(Eval::new(expr, args)).await?;
        extract_and_deserialize_iproto_data(body).map_err(Into::into)
    }

    /// Remotely call function in Tarantool.
    ///
    /// Check [docs][crate#deserializing-lua-responses-in-call-and-eval] on how to deserialize response.
    async fn call<I, T>(&self, function_name: I, args: Vec<Value>) -> Result<T>
    where
        I: Into<Cow<'static, str>> + Send,
        T: DeserializeOwned,
    {
        let body = self.send_request(Call::new(function_name, args)).await?;
        extract_and_deserialize_iproto_data(body).map_err(Into::into)
    }

    /// Select tuples from space.
    async fn select<T>(
        &self,
        space_id: u32,
        index_id: u32,
        limit: Option<u32>,
        offset: Option<u32>,
        iterator: Option<IteratorType>,
        keys: Vec<Value>,
    ) -> Result<Vec<T>>
    where
        T: DeserializeOwned,
    {
        let body = self
            .send_request(Select::new(
                space_id, index_id, limit, offset, iterator, keys,
            ))
            .await?;
        extract_and_deserialize_iproto_data(body).map_err(Into::into)
    }

    // TODO: decode response
    /// Insert tuple.
    async fn insert(&self, space_id: u32, tuple: Vec<Value>) -> Result<()> {
        let _ = self.send_request(Insert::new(space_id, tuple)).await?;
        Ok(())
    }

    // TODO: structured tuple
    // TODO: decode response
    /// Update tuple.
    async fn update(
        &self,
        space_id: u32,
        index_id: u32,
        keys: Vec<Value>,
        tuple: Vec<Value>,
    ) -> Result<()> {
        let _ = self
            .send_request(Update::new(space_id, index_id, keys, tuple))
            .await?;
        Ok(())
    }

    // TODO: structured tuple
    // TODO: decode response
    // TODO: maybe set index base to 1 always?
    /// Update or insert tuple.
    async fn upsert(&self, space_id: u32, ops: Vec<Value>, tuple: Vec<Value>) -> Result<()> {
        let _ = self.send_request(Upsert::new(space_id, ops, tuple)).await?;
        Ok(())
    }

    // TODO: structured tuple
    // TODO: decode response
    /// Insert a tuple into a space. If a tuple with the same primary key already exists,
    /// replaces the existing tuple with a new one.
    async fn replace(&self, space_id: u32, keys: Vec<Value>) -> Result<()> {
        let _ = self.send_request(Replace::new(space_id, keys)).await?;
        Ok(())
    }

    // TODO: structured tuple
    // TODO: decode response
    /// Delete a tuple identified by the primary key.
    async fn delete(&self, space_id: u32, index_id: u32, keys: Vec<Value>) -> Result<()> {
        let _ = self
            .send_request(Delete::new(space_id, index_id, keys))
            .await?;
        Ok(())
    }
}

#[async_trait]
impl<E: Executor + ?Sized> ExecutorExt for E {
    fn send_request<R>(&self, body: R) -> BoxFuture<Result<Value>>
    where
        R: Request,
    {
        let req = EncodedRequest::new(body, None);
        async move { (*self).send_encoded_request(req?).await }.boxed()
    }
}
