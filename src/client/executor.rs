use async_trait::async_trait;
use rmpv::Value;

use super::private::Sealed;
use crate::{codec::request::EncodedRequest, Result, Stream, Transaction, TransactionBuilder};

// TODO: docs
#[async_trait]
pub trait Executor: Sealed + Send + Sync {
    /// Send encoded request.
    async fn send_encoded_request(&self, request: EncodedRequest) -> Result<Value>;

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
    async fn transaction(&self) -> Result<Transaction>;
}

#[async_trait]
impl<E: Executor + Sealed + Sync> Executor for &E {
    async fn send_encoded_request(&self, request: EncodedRequest) -> Result<Value> {
        (**self).send_encoded_request(request).await
    }

    fn stream(&self) -> Stream {
        (**self).stream()
    }

    fn transaction_builder(&self) -> TransactionBuilder {
        (**self).transaction_builder()
    }

    async fn transaction(&self) -> Result<Transaction> {
        (**self).transaction().await
    }
}

#[async_trait]
impl<E: Executor + Sealed + Sync> Executor for &mut E {
    async fn send_encoded_request(&self, request: EncodedRequest) -> Result<Value> {
        (**self).send_encoded_request(request).await
    }

    fn stream(&self) -> Stream {
        (**self).stream()
    }

    fn transaction_builder(&self) -> TransactionBuilder {
        (**self).transaction_builder()
    }

    async fn transaction(&self) -> Result<Transaction> {
        (**self).transaction().await
    }
}

#[cfg(test)]
mod ui {
    use super::*;
    use crate::ConnectionLike;

    #[test]
    fn executor_trait_object_safety() {
        fn _f(executor: impl Executor) {
            let _: Box<dyn Executor> = Box::new(executor);
        }
    }

    #[test]
    fn calling_conn_like_on_dyn_executor() {
        async fn _f(conn: &dyn Executor) -> Result<()> {
            conn.ping().await
        }
    }

    #[test]
    fn calling_conn_like_on_boxed_dyn_executor() {
        async fn _f(conn: &Box<dyn Executor>) -> Result<()> {
            conn.ping().await
        }
    }
}
