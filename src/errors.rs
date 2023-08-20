//! Errors module.

use std::{borrow::Cow, fmt, sync::Arc};

use rmp::{
    decode::{MarkerReadError, NumValueReadError, ValueReadError},
    encode::{RmpWriteErr, ValueWriteError},
};
use rmpv::Value;
use tokio::time::error::Elapsed;

/// Error returned by Tarantool in response to a request.
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

/// Represents all possible errors of this crate.
#[non_exhaustive]
#[derive(Debug, thiserror::Error)]
pub enum Error {
    /// Error, returned in response from Tarantool instance.
    #[error("Error response: {0}")]
    Response(#[from] ErrorResponse),
    /// Error, returned in response on `call` or `eval`.
    #[error("Call or eval error: {0}")]
    CallEval(Value),

    /// Timeout.
    #[error("Timeout")]
    Timeout,
    /// Timeout while establishing connection.
    #[error("Connect timeout")]
    ConnectTimeout,

    /// Authorization error.
    #[error("Authorization error: {} (code {})" ,.0.description, .0.code)]
    Auth(#[source] ErrorResponse),

    /// Errors, related to encoding requests.
    #[error(transparent)]
    Encode(#[from] EncodingError),
    /// Errors, related to decoding responses.
    #[error(transparent)]
    Decode(#[from] DecodingError),

    /// Duplicated sync detected.
    #[error("Duplicated sync '{0}'")]
    DuplicatedSync(u32),

    /// Space is missing primary index (with 0 id).
    #[error("Space is missing primary index")]
    SpaceMissingPrimaryIndex,

    /// Underlying TCP connection closed.
    #[error("TCP connection error")]
    ConnectionError(#[from] Arc<tokio::io::Error>),
    /// Underlying TCP connection was closed.
    #[error("TCP connection closed")]
    ConnectionClosed,

    /// Other errors that don't deserve a separate variant.
    #[error(transparent)]
    Other(anyhow::Error),
}

impl From<tokio::io::Error> for Error {
    fn from(v: tokio::io::Error) -> Self {
        Self::ConnectionError(Arc::new(v))
    }
}

impl From<CodecDecodeError> for Error {
    fn from(value: CodecDecodeError) -> Self {
        match value {
            CodecDecodeError::Io(x) => x.into(),
            CodecDecodeError::Closed => Self::ConnectionClosed,
            CodecDecodeError::Decode(x) => x.into(),
        }
    }
}

impl From<CodecEncodeError> for Error {
    fn from(value: CodecEncodeError) -> Self {
        match value {
            CodecEncodeError::Io(x) => x.into(),
            CodecEncodeError::Encode(x) => x.into(),
        }
    }
}

impl From<Elapsed> for Error {
    fn from(_: Elapsed) -> Self {
        Self::Timeout
    }
}

/// Errors, related to encoding requests.
#[non_exhaustive]
#[derive(Debug, thiserror::Error)]
pub enum EncodingError {
    /// Error while encoding data into MessagePack format.
    #[error("Failed to encode data into MessagePack")]
    MessagePack(#[source] anyhow::Error),
}

impl<E> From<ValueWriteError<E>> for EncodingError
where
    E: RmpWriteErr + Send + Sync,
{
    fn from(v: ValueWriteError<E>) -> Self {
        Self::MessagePack(v.into())
    }
}

impl From<std::io::Error> for EncodingError {
    fn from(value: std::io::Error) -> Self {
        Self::MessagePack(value.into())
    }
}

impl From<rmp_serde::encode::Error> for EncodingError {
    fn from(value: rmp_serde::encode::Error) -> Self {
        Self::MessagePack(value.into())
    }
}

/// Errors, related to decoding responses.
#[derive(Clone, Debug, thiserror::Error)]
#[error("{kind}{}", DecodingErrorLocation::display_in_error(.location))]
pub struct DecodingError {
    #[source]
    kind: Arc<DecodingErrorDetails>,
    location: Option<DecodingErrorLocation>,
}

impl DecodingError {
    pub(crate) fn new(kind: DecodingErrorDetails) -> Self {
        Self {
            kind: Arc::new(kind),
            location: None,
        }
    }

    pub(crate) fn missing_key(key: &'static str) -> Self {
        DecodingErrorDetails::MissingKey(key).into()
    }

    pub(crate) fn type_mismatch(
        expected: &'static str,
        actual: impl Into<Cow<'static, str>>,
    ) -> Self {
        DecodingErrorDetails::TypeMismatch {
            expected,
            actual: actual.into(),
        }
        .into()
    }

    pub(crate) fn message_pack(err: impl Into<anyhow::Error>) -> Self {
        DecodingErrorDetails::MessagePack(err.into()).into()
    }

    pub(crate) fn unknown_response_code(code: u32) -> Self {
        DecodingErrorDetails::UnknownResponseCode(code).into()
    }

    pub(crate) fn invalid_tuple_length(expected: usize, actual: usize) -> Self {
        DecodingErrorDetails::InvalidTupleLength { expected, actual }.into()
    }

    pub(crate) fn with_location(mut self, location: DecodingErrorLocation) -> Self {
        self.location = Some(location);
        self
    }

    pub(crate) fn in_key(self, key: &'static str) -> Self {
        self.with_location(DecodingErrorLocation::Key(key))
    }

    pub(crate) fn in_other(self, other: &'static str) -> Self {
        self.with_location(DecodingErrorLocation::Other(other))
    }

    pub fn kind(&self) -> &DecodingErrorDetails {
        &self.kind
    }

    pub fn location(&self) -> Option<&DecodingErrorLocation> {
        self.location.as_ref()
    }
}

impl From<DecodingErrorDetails> for DecodingError {
    fn from(value: DecodingErrorDetails) -> Self {
        Self::new(value)
    }
}

/// Details of [`DecodingError`].
#[non_exhaustive]
#[derive(Debug, thiserror::Error)]
pub enum DecodingErrorDetails {
    /// Unknown response code.
    #[error("unknown response code: {0}")]
    UnknownResponseCode(u32),
    /// Certain key missing in response.
    #[error("Missing key in response: {0}")]
    MissingKey(&'static str),
    /// Value have different type than expected for that key or field.
    #[error("Type mismatch, expected '{expected}', actual '{actual}'")]
    TypeMismatch {
        expected: &'static str,
        actual: Cow<'static, str>,
    },
    /// Tuple have invalid length
    #[error("Invalid tuple length {actual}, expected {expected}")]
    InvalidTupleLength { expected: usize, actual: usize },

    /// Error while deserializing [`rmpv::Value`] into concrete type.
    #[error("Failed to deserialize rmpv::Value")]
    Serde(#[source] rmpv::ext::Error),
    /// Error while decoding data from MessagePack format.
    #[error("Failed to decode data from MessagePack")]
    MessagePack(#[source] anyhow::Error),
}

impl From<ValueReadError> for DecodingError {
    fn from(v: ValueReadError) -> Self {
        DecodingErrorDetails::MessagePack(v.into()).into()
    }
}

impl From<rmpv::decode::Error> for DecodingError {
    fn from(v: rmpv::decode::Error) -> Self {
        DecodingErrorDetails::MessagePack(v.into()).into()
    }
}

impl From<rmpv::ext::Error> for DecodingError {
    fn from(v: rmpv::ext::Error) -> Self {
        DecodingErrorDetails::Serde(v).into()
    }
}

impl From<NumValueReadError> for DecodingError {
    fn from(v: NumValueReadError) -> Self {
        DecodingErrorDetails::MessagePack(v.into()).into()
    }
}

impl From<MarkerReadError> for DecodingError {
    fn from(v: MarkerReadError) -> Self {
        DecodingErrorDetails::MessagePack(v.0.into()).into()
    }
}

#[derive(Clone, Debug)]
pub enum DecodingErrorLocation {
    Key(&'static str),
    FrameLengthField,
    Other(&'static str),
}

impl fmt::Display for DecodingErrorLocation {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            DecodingErrorLocation::Key(x) => write!(f, "key '{x}'"),
            DecodingErrorLocation::FrameLengthField => write!(f, "frame length field"),
            DecodingErrorLocation::Other(x) => write!(f, "{x}"),
        }
    }
}

impl DecodingErrorLocation {
    fn display_in_error(value: &Option<Self>) -> String {
        if let Some(x) = value {
            format!(" (in {x})")
        } else {
            String::new()
        }
    }
}

/// Helper type to return errors from decoder.
#[derive(Clone)]
pub(crate) enum CodecDecodeError {
    Io(Arc<tokio::io::Error>),
    Closed,
    Decode(DecodingError),
}

impl From<tokio::io::Error> for CodecDecodeError {
    fn from(v: tokio::io::Error) -> Self {
        Self::Io(Arc::new(v))
    }
}

/// Helper type to return errors from encoder.
#[derive(Debug)]
pub(crate) enum CodecEncodeError {
    Io(tokio::io::Error),
    Encode(EncodingError),
}

impl From<tokio::io::Error> for CodecEncodeError {
    fn from(v: tokio::io::Error) -> Self {
        Self::Io(v)
    }
}
