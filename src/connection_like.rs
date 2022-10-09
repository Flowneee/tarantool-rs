use std::borrow::Cow;

use async_trait::async_trait;
use rmpv::Value;

use crate::{
    codec::{
        request::{IProtoCall, IProtoEval, IProtoPing, IProtoRequestBody},
        utils::data_from_response_body,
    },
    errors::Error,
    Stream,
};

#[async_trait]
pub trait ConnectionLike: private::Sealed {
    /// Send request using some internal connection object.
    async fn send_request(&self, body: impl IProtoRequestBody) -> Result<Value, Error>;

    /// Get new [`Stream`].
    fn stream(&self) -> Stream;

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
    use crate::{Connection, Stream};

    pub trait Sealed {}

    impl Sealed for Connection {}
    impl Sealed for Stream {}
}
