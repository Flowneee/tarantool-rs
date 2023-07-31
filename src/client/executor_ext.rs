use std::borrow::Cow;

use async_trait::async_trait;
use futures::{future::BoxFuture, FutureExt};
use rmpv::Value;
use serde::de::DeserializeOwned;

use super::Executor;
use crate::{
    codec::{
        request::{
            Call, Delete, EncodedRequest, Eval, Insert, Ping, Replace, Request, Select, Update,
            Upsert,
        },
        utils::deserialize_non_sql_response,
    },
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

    /// Send PING request ([docs](https://www.tarantool.io/en/doc/latest/dev_guide/internals/box_protocol/#iproto-ping-0x40)).
    async fn ping(&self) -> Result<()> {
        self.send_request(Ping {}).await.map(drop)
    }

    // TODO: docs
    async fn eval<I, T>(&self, expr: I, args: Vec<Value>) -> Result<T>
    where
        I: Into<Cow<'static, str>> + Send,
        T: DeserializeOwned,
    {
        let body = self.send_request(Eval::new(expr, args)).await?;
        deserialize_non_sql_response(body).map_err(Into::into)
    }

    // TODO: docs
    async fn call<I, T>(&self, function_name: I, args: Vec<Value>) -> Result<T>
    where
        I: Into<Cow<'static, str>> + Send,
        T: DeserializeOwned,
    {
        let body = self.send_request(Call::new(function_name, args)).await?;
        deserialize_non_sql_response(body).map_err(Into::into)
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
    ) -> Result<Vec<T>>
    where
        T: DeserializeOwned,
    {
        let body = self
            .send_request(Select::new(
                space_id, index_id, limit, offset, iterator, keys,
            ))
            .await?;
        deserialize_non_sql_response(body).map_err(Into::into)
    }

    // TODO: docs
    // TODO: decode response
    async fn insert(&self, space_id: u32, tuple: Vec<Value>) -> Result<()> {
        let _ = self.send_request(Insert::new(space_id, tuple)).await?;
        Ok(())
    }

    // TODO: structured tuple
    // TODO: decode response
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
    async fn upsert(&self, space_id: u32, ops: Vec<Value>, tuple: Vec<Value>) -> Result<()> {
        let _ = self.send_request(Upsert::new(space_id, ops, tuple)).await?;
        Ok(())
    }

    // TODO: structured tuple
    // TODO: decode response
    async fn replace(&self, space_id: u32, keys: Vec<Value>) -> Result<()> {
        let _ = self.send_request(Replace::new(space_id, keys)).await?;
        Ok(())
    }

    // TODO: structured tuple
    // TODO: decode response
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
