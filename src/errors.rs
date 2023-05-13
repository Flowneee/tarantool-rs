use std::sync::Arc;

use anyhow::anyhow;
use rmp::{
    decode::{DecodeStringError, MarkerReadError, NumValueReadError, ValueReadError},
    encode::{RmpWriteErr, ValueWriteError},
};

// TODO: docs

#[derive(Clone, Debug, thiserror::Error)]
#[error("{description} (code {code})")]
pub struct ErrorResponse {
    pub code: u32,
    pub description: String,
    pub extra: Option<rmpv::Value>,
}

impl ErrorResponse {
    pub fn new(code: u32, description: String, extra: Option<rmpv::Value>) -> Self {
        Self {
            code,
            description,
            extra,
        }
    }
}

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Error while encoding response body into MessagePack: {0}")]
    RequestBodyEncode(#[source] anyhow::Error),
    #[error("Error response: {0}")]
    Response(#[from] ErrorResponse),
    #[error("Error while decoding response body: {0}")]
    ResponseBodyDecode(#[source] anyhow::Error),
    #[error("Serde deserialization error: {0}")]
    SerdeDeserialize(#[from] rmpv::ext::Error),
    #[error("Transport error: {0}")]
    Transport(#[from] Arc<TransportError>),

    #[error("Space not found")]
    SpaceNotFound,
    #[error("Failed to metadata: {0}")]
    MetadataLoad(#[source] anyhow::Error),
}

impl From<TransportError> for Error {
    fn from(value: TransportError) -> Self {
        Error::Transport(Arc::new(value))
    }
}

/// Errors related to low-level interaction with Tarantool (TCP or MessagePack).
#[derive(Debug, thiserror::Error)]
pub enum TransportError {
    #[error("Duplicated sync '{0}'")]
    DuplicatedSync(u32),
    #[error("Underlying connection error: {0}")]
    Connection(#[from] tokio::io::Error),
    #[error("MessagePack encoding error: {0}")]
    MessagePackEncode(#[source] anyhow::Error),
    #[error("MessagePack decoding error: {0}")]
    MessagePackDecode(#[source] anyhow::Error),
    #[error("Underlying connection closed")]
    ConnectionClosed,
}

impl<E> From<ValueWriteError<E>> for TransportError
where
    E: RmpWriteErr + Send + Sync,
{
    fn from(v: ValueWriteError<E>) -> Self {
        Self::MessagePackEncode(v.into())
    }
}

impl From<ValueReadError> for TransportError {
    fn from(v: ValueReadError) -> Self {
        Self::MessagePackDecode(v.into())
    }
}

impl From<rmpv::decode::Error> for TransportError {
    fn from(v: rmpv::decode::Error) -> Self {
        Self::MessagePackDecode(v.into())
    }
}

impl From<NumValueReadError> for TransportError {
    fn from(v: NumValueReadError) -> Self {
        Self::MessagePackDecode(v.into())
    }
}

impl From<DecodeStringError<'_>> for TransportError {
    fn from(v: DecodeStringError<'_>) -> Self {
        Self::MessagePackDecode(anyhow!("{}", v))
    }
}

impl From<MarkerReadError> for TransportError {
    fn from(v: MarkerReadError) -> Self {
        Self::MessagePackDecode(v.0.into())
    }
}
