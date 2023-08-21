use async_trait::async_trait;
use futures::{future::BoxFuture, FutureExt, TryFutureExt};
use rmpv::Value;
use serde::de::DeserializeOwned;

use super::Executor;
use crate::{
    codec::{
        request::{
            Call, Delete, EncodedRequest, Eval, Execute, Insert, Ping, Replace, Request, Select,
            Update, Upsert,
        },
        response,
    },
    schema::{SchemaEntityKey, Space},
    tuple::Tuple,
    utils::extract_and_deserialize_iproto_data,
    IteratorType, Result, SqlResponse, TupleResponse,
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
    async fn eval<A, I>(&self, expr: I, args: A) -> Result<TupleResponse>
    where
        A: Tuple + Send,
        I: AsRef<str> + Send + Sync,
    {
        Ok(TupleResponse(
            self.send_request(Eval::new(expr.as_ref(), args)).await?,
        ))
    }

    /// Remotely call function in Tarantool.
    ///
    /// Check [docs][crate#deserializing-lua-responses-in-call-and-eval] on how to deserialize response.
    async fn call<A, I>(&self, function_name: I, args: A) -> Result<TupleResponse>
    where
        A: Tuple + Send,
        I: AsRef<str> + Send + Sync,
    {
        Ok(TupleResponse(
            self.send_request(Call::new(function_name.as_ref(), args))
                .await?,
        ))
    }

    /// Select tuples from space.
    async fn select<T, A>(
        &self,
        space_id: u32,
        index_id: u32,
        limit: Option<u32>,
        offset: Option<u32>,
        iterator: Option<IteratorType>,
        keys: A,
    ) -> Result<Vec<T>>
    where
        T: DeserializeOwned,
        A: Tuple + Send,
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
    async fn insert<T>(&self, space_id: u32, tuple: T) -> Result<Value>
    where
        T: Tuple + Send,
    {
        let resp = self.send_request(Insert::new(space_id, tuple)).await?;
        Ok(resp)
    }

    // TODO: decode response
    /// Update tuple.
    async fn update<K, O>(&self, space_id: u32, index_id: u32, keys: K, ops: O) -> Result<Value>
    where
        K: Tuple + Send,
        O: Tuple + Send,
    {
        self.send_request(Update::new(space_id, index_id, keys, ops))
            .err_into()
            .await
    }

    // TODO: decode response
    /// Update or insert tuple.
    async fn upsert<T, O>(&self, space_id: u32, tuple: T, ops: O) -> Result<()>
    where
        T: Tuple + Send,
        O: Tuple + Send,
    {
        let _ = self.send_request(Upsert::new(space_id, ops, tuple)).await?;
        Ok(())
    }

    // TODO: structured tuple
    // TODO: decode response
    /// Insert a tuple into a space. If a tuple with the same primary key already exists,
    /// replaces the existing tuple with a new one.
    async fn replace<T>(&self, space_id: u32, tuple: T) -> Result<()>
    where
        T: Tuple + Send,
    {
        let _ = self.send_request(Replace::new(space_id, tuple)).await?;
        Ok(())
    }

    // TODO: structured tuple
    // TODO: decode response
    /// Delete a tuple identified by the primary key.
    async fn delete<T>(&self, space_id: u32, index_id: u32, keys: T) -> Result<()>
    where
        T: Tuple + Send,
    {
        let _ = self
            .send_request(Delete::new(space_id, index_id, keys))
            .await?;
        Ok(())
    }

    // TODO: statement cache
    /// Perform SQL query.
    async fn execute_sql<T, I>(&self, query: I, binds: T) -> Result<SqlResponse>
    where
        T: Tuple + Send,
        I: AsRef<str> + Send + Sync,
    {
        Ok(SqlResponse(
            self.send_request(Execute::new_query(query.as_ref(), binds))
                .await?,
        ))
    }

    /// Find and load space by key.
    ///
    /// Can be called with space's index (if passed unsigned integer) or name (if passed `&str`).
    ///
    /// Returned [`Space`] object contains reference to current executor.
    async fn space<K>(&self, key: K) -> Result<Option<Space<&Self>>>
    where
        Self: Sized + Send,
        K: Into<SchemaEntityKey> + Send,
    {
        Space::load(self, key.into()).await
    }

    /// Find and load space by key, moving current executor into [`Space`].
    ///
    /// Can be called with space's index (if passed unsigned integer) or name (if passed `&str`).
    ///
    /// Returned [`Space`] object contains current executor.
    async fn into_space<K>(self, key: K) -> Result<Option<Space<Self>>>
    where
        Self: Sized + Send,
        K: Into<SchemaEntityKey> + Send,
    {
        Space::load(self, key.into()).await
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

#[cfg(test)]
mod ui {
    #![allow(unused)]

    use crate::{Connection, Transaction};

    use super::*;

    fn executor_ext_on_connection_ref() {
        async fn f(conn: &Connection) -> Space<&Connection> {
            conn.space("space").await.unwrap().unwrap()
        }
    }

    fn executor_ext_on_connection() {
        async fn f(conn: Connection) -> Space<Connection> {
            conn.into_space("space").await.unwrap().unwrap()
        }
    }

    fn executor_ext_on_connection_cloned() {
        async fn f(conn: &Connection) -> Space<Connection> {
            conn.clone().into_space("space").await.unwrap().unwrap()
        }
    }

    fn executor_ext_on_transaction_ref() {
        async fn f(tx: &Transaction) -> Space<&Transaction> {
            tx.space("space").await.unwrap().unwrap()
        }
    }

    fn executor_ext_on_transaction() {
        async fn f(tx: Transaction) {
            let space_tx: Space<Transaction> = tx.into_space("space").await.unwrap().unwrap();
            space_tx.delete((1,)).await.unwrap();
            space_tx.commit().await.unwrap();
        }
    }
}
