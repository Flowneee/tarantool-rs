use anyhow::Context;
use base64::{engine::general_purpose::STANDARD_NO_PAD, Engine};
use bytes::{Buf, BufMut, BytesMut};
use rmp::Marker;
use tokio_util::codec::{Decoder, Encoder};
use tracing::trace;

use self::{request::EncodedRequest, response::Response};
use crate::{
    errors::{CodecDecodeError, CodecEncodeError, DecodingError},
    Error,
};

pub mod consts;
pub mod request;
pub mod response;
pub mod utils;

enum LengthDecoder {
    NoMarker,
    Marker(Marker),
    Value(usize),
}

impl Default for LengthDecoder {
    fn default() -> Self {
        Self::NoMarker
    }
}

impl LengthDecoder {
    fn decode(&mut self, src: &mut BytesMut) -> Result<Option<usize>, DecodingError> {
        if src.is_empty() {
            return Ok(None);
        }
        let marker = match self {
            LengthDecoder::NoMarker => {
                // Safety: `src.get_u8` might panic if there is no enough data,
                // but in this case we checked previously that `src` is not empty.
                let marker = Marker::from_u8(src.get_u8());
                *self = Self::Marker(marker);
                trace!("decoded length marker: {:?}", marker);
                marker
            }
            LengthDecoder::Marker(x) => *x,
            LengthDecoder::Value(x) => return Ok(Some(*x)),
        };
        // Safety: `src.get_uXX` might panic if there is no enough data,
        // but in this case we check before reading, so it shouldn't panic.
        let length = match marker {
            Marker::FixPos(x) => x as usize,
            Marker::U8 => {
                if src.len() > 1 {
                    src.get_u8() as usize
                } else {
                    return Ok(None);
                }
            }
            Marker::U16 => {
                if src.len() > 2 {
                    src.get_u16() as usize
                } else {
                    return Ok(None);
                }
            }
            Marker::U32 => {
                if src.len() > 4 {
                    src.get_u32() as usize
                } else {
                    return Ok(None);
                }
            }
            Marker::U64 => {
                //
                if src.len() > 8 {
                    src.get_u64() as usize
                } else {
                    return Ok(None);
                }
            }
            rest => {
                return Err(DecodingError::type_mismatch(
                    "unsigned integer",
                    format!("{:?}", rest),
                ))
            }
        };
        trace!("decoded frame length: {}", length);
        *self = LengthDecoder::Value(length);
        Ok(Some(length))
    }

    fn reset(&mut self) {
        *self = LengthDecoder::NoMarker
    }
}

#[derive(Default)]
pub(crate) struct ClientCodec {
    length_decoder: LengthDecoder,
}

impl Decoder for ClientCodec {
    type Item = Response;

    type Error = CodecDecodeError;

    fn decode(&mut self, src: &mut BytesMut) -> Result<Option<Self::Item>, Self::Error> {
        let Some(next_frame_length) = self
            .length_decoder
            .decode(src)
            .map_err(CodecDecodeError::Decode)?
        else {
            return Ok(None);
        };
        if src.len() >= next_frame_length {
            self.length_decoder.reset();
            let frame_bytes = src.split_to(next_frame_length);
            Response::decode(frame_bytes.reader())
                .map(Some)
                .map_err(CodecDecodeError::Decode)
        } else {
            src.reserve(next_frame_length - src.len());
            Ok(None)
        }
    }
}

impl Encoder<EncodedRequest> for ClientCodec {
    type Error = CodecEncodeError;

    // To omit creating intermediate BytesMut, encode message with 0 as length,
    // and after encoding calculate size of the encoded messages and overwrite
    // length field (0) with new data.
    fn encode(&mut self, item: EncodedRequest, dst: &mut BytesMut) -> Result<(), Self::Error> {
        let begin_idx = dst.len();

        // TODO: calculate necessary integer type instead of using u64 always
        // Write message with fictional length (0)
        let mut writer = dst.writer();
        rmp::encode::write_u64(&mut writer, 0)
            .map_err(|err| CodecEncodeError::Encode(err.into()))?;
        item.encode(&mut writer).map_err(CodecEncodeError::Encode)?;

        // Calculate length and override length field with actual value
        let dst = writer.into_inner();
        let data_len = dst.len() - begin_idx - 9;
        let mut len_writer = dst[begin_idx..].writer();
        rmp::encode::write_u64(&mut len_writer, data_len as u64)
            .map_err(|err| CodecEncodeError::Encode(err.into()))?;

        Ok(())
    }
}

/// Greeting message from server.
///
/// [Docs](https://www.tarantool.io/en/doc/latest/dev_guide/internals/box_protocol/#greeting-message).
#[derive(Debug)]
pub struct Greeting {
    pub server: String,
    pub salt: Vec<u8>,
}

impl Greeting {
    /// Size of the full message from server in bytes.
    pub const SIZE: usize = 128;

    // TODO: err
    /// Decode greeting from provided buffer without checking boundaries.
    pub fn decode(buffer: [u8; Self::SIZE]) -> Result<Self, Error> {
        let line1 = &buffer[0..62];
        let line2 = &buffer[64..126];
        let salt_b64 = line2
            .iter()
            .enumerate()
            .rev()
            .find(|x| *x.1 != b' ')
            .map_or(&b""[..], |(idx, _)| &line2[0..idx]);
        let salt = STANDARD_NO_PAD
            .decode(salt_b64)
            .context("Failed to decode salt from base64")
            .map_err(Error::Other)?;
        Ok(Self {
            server: String::from_utf8_lossy(line1).into_owned(),
            salt,
        })
    }
}
