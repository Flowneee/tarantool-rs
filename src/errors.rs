use anyhow::anyhow;
use rmp::{
    decode::{DecodeStringError, MarkerReadError, NumValueReadError, ValueReadError},
    encode::{RmpWriteErr, ValueWriteError},
};

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Connection error: {0}")]
    Connection(#[from] tokio::io::Error),
    #[error("MessagePack encoding error: {0}")]
    MessagePackEncode(#[source] anyhow::Error),
    #[error("MessagePack decoding error: {0}")]
    MessagePackDecode(#[source] anyhow::Error),
}

impl<E> From<ValueWriteError<E>> for Error
where
    E: RmpWriteErr + Send + Sync,
{
    fn from(v: ValueWriteError<E>) -> Self {
        Self::MessagePackEncode(v.into())
    }
}

impl From<ValueReadError> for Error {
    fn from(v: ValueReadError) -> Self {
        Self::MessagePackDecode(v.into())
    }
}

impl From<rmpv::decode::Error> for Error {
    fn from(v: rmpv::decode::Error) -> Self {
        Self::MessagePackDecode(v.into())
    }
}

impl From<NumValueReadError> for Error {
    fn from(v: NumValueReadError) -> Self {
        Self::MessagePackDecode(v.into())
    }
}

impl From<DecodeStringError<'_>> for Error {
    fn from(v: DecodeStringError<'_>) -> Self {
        Self::MessagePackDecode(anyhow!("{}", v))
    }
}

impl From<MarkerReadError> for Error {
    fn from(v: MarkerReadError) -> Self {
        Self::MessagePackDecode(v.0.into())
    }
}
