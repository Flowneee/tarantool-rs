use std::sync::Arc;

use anyhow::anyhow;
use rmp::{
    decode::{DecodeStringError, MarkerReadError, NumValueReadError, ValueReadError},
    encode::{RmpWriteErr, ValueWriteError},
};

#[derive(Clone, Debug, thiserror::Error)]
pub enum Error {
    #[error("{description} (code {code})")]
    Response {
        code: u32,
        description: String,
        extra: Option<rmpv::Value>,
    },
    #[error("{0}")]
    Channel(#[source] Arc<ChannelError>),
}

impl From<ChannelError> for Error {
    fn from(value: ChannelError) -> Self {
        Error::channel(value)
    }
}

impl Error {
    pub(crate) fn channel(value: ChannelError) -> Self {
        Self::Channel(Arc::new(value))
    }

    pub(crate) fn response(code: u32, description: String, extra: Option<rmpv::Value>) -> Self {
        Self::Response {
            code,
            description,
            extra,
        }
    }
}

/// Errors related to low-level interaction with Tarantool (TCP or MessagePack).
#[derive(Debug, thiserror::Error)]
pub enum ChannelError {
    #[error("Connection error: {0}")]
    Connection(#[from] tokio::io::Error),
    #[error("MessagePack encoding error: {0}")]
    MessagePackEncode(#[source] anyhow::Error),
    #[error("MessagePack decoding error: {0}")]
    MessagePackDecode(#[source] anyhow::Error),
    #[error("Connection closed")]
    ConnectionClosed,
}

impl<E> From<ValueWriteError<E>> for ChannelError
where
    E: RmpWriteErr + Send + Sync,
{
    fn from(v: ValueWriteError<E>) -> Self {
        Self::MessagePackEncode(v.into())
    }
}

impl From<ValueReadError> for ChannelError {
    fn from(v: ValueReadError) -> Self {
        Self::MessagePackDecode(v.into())
    }
}

impl From<rmpv::decode::Error> for ChannelError {
    fn from(v: rmpv::decode::Error) -> Self {
        Self::MessagePackDecode(v.into())
    }
}

impl From<NumValueReadError> for ChannelError {
    fn from(v: NumValueReadError) -> Self {
        Self::MessagePackDecode(v.into())
    }
}

impl From<DecodeStringError<'_>> for ChannelError {
    fn from(v: DecodeStringError<'_>) -> Self {
        Self::MessagePackDecode(anyhow!("{}", v))
    }
}

impl From<MarkerReadError> for ChannelError {
    fn from(v: MarkerReadError) -> Self {
        Self::MessagePackDecode(v.0.into())
    }
}
