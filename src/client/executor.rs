use async_trait::async_trait;
use rmpv::Value;

use super::private::Sealed;
use crate::{codec::request::Request, Result};

// TODO: docs
#[async_trait]
pub trait Executor: Sealed + Send + Sync {
    async fn send_request(&self, request: Request) -> Result<Value>;
}

#[async_trait]
impl<E: Executor + Sealed + Sync> Executor for &E {
    async fn send_request(&self, request: Request) -> Result<Value> {
        (&*self).send_request(request).await
    }
}

#[async_trait]
impl<E: Executor + Sealed + Sync> Executor for &mut E {
    async fn send_request(&self, request: Request) -> Result<Value> {
        (&*self).send_request(request).await
    }
}

#[cfg(test)]
mod ui {
    use super::*;

    #[test]
    fn executor_trait_object_safety() {
        fn f(executor: impl Executor) {
            let _: Box<dyn Executor> = Box::new(executor);
        }
    }

    // TODO: uncomment or remove
    // #[test]
    // fn calling_conn_like_on_dyn_executor() {
    //     async fn f(conn: &dyn Executor) -> Result<Value> {
    //         conn.ping().await
    //     }
    // }
}
