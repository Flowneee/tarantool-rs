use bytes::{Buf, BufMut, BytesMut};
use rmp::{decode::ValueReadError, Marker};
use tokio_util::codec::{Decoder, Encoder};
use tracing::{trace};

use self::{request::Request, response::Response};
use crate::TransportError;

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
    // TODO: this function uses hidden internal functions from rmp (read_data_*)
    // need to rewrite this
    fn decode(&mut self, src: &mut BytesMut) -> Result<Option<usize>, ValueReadError> {
        if src.is_empty() {
            return Ok(None);
        }
        let mut reader = src.reader();
        let marker = match self {
            LengthDecoder::NoMarker => {
                let marker = rmp::decode::read_marker(&mut reader)?;
                *self = Self::Marker(marker);
                trace!("decoded length marker: {:?}", marker);
                marker
            }
            LengthDecoder::Marker(x) => *x,
            LengthDecoder::Value(x) => return Ok(Some(*x)),
        };
        let length = match marker {
            Marker::FixPos(x) => x as usize,
            Marker::U8 => {
                if reader.get_ref().len() > 2 {
                    rmp::decode::read_data_u8(&mut reader)? as usize
                } else {
                    return Ok(None);
                }
            }
            Marker::U16 => {
                if reader.get_ref().len() > 3 {
                    rmp::decode::read_data_u16(&mut reader)? as usize
                } else {
                    return Ok(None);
                }
            }
            Marker::U32 => {
                if reader.get_ref().len() > 5 {
                    rmp::decode::read_data_u32(&mut reader)? as usize
                } else {
                    return Ok(None);
                }
            }
            Marker::U64 => {
                if reader.get_ref().len() > 9 {
                    rmp::decode::read_data_u64(&mut reader)? as usize
                } else {
                    return Ok(None);
                }
            }
            rest => return Err(ValueReadError::TypeMismatch(rest)),
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

    type Error = TransportError;

    fn decode(&mut self, src: &mut BytesMut) -> Result<Option<Self::Item>, Self::Error> {
        let next_frame_length = if let Some(x) = self.length_decoder.decode(src)? {
            x
        } else {
            return Ok(None);
        };
        if src.len() >= next_frame_length {
            self.length_decoder.reset();
            let frame_bytes = src.split_to(next_frame_length);
            Response::decode(frame_bytes.reader())
                .map(Some)
                .map_err(TransportError::MessagePackDecode)
        } else {
            Ok(None)
        }
    }
}

impl Encoder<Request> for ClientCodec {
    type Error = TransportError;

    // To omit creating intermediate BytesMut, encode message with 0 as length,
    // and after encoding calculate size of the encoded messages and overwrite
    // length field (0) with new data.
    fn encode(&mut self, item: Request, dst: &mut BytesMut) -> Result<(), Self::Error> {
        let begin_idx = dst.len();

        // Write message with fictional length (0)
        let mut writer = dst.writer();
        rmp::encode::write_u64(&mut writer, 0)?;
        item.encode(&mut writer)
            .map_err(TransportError::MessagePackEncode)?;

        // Calculate length and override length field with actual value
        let dst = writer.into_inner();
        let data_len = dst.len() - begin_idx - 9;
        let mut len_writer = dst[begin_idx..].writer();
        rmp::encode::write_u64(&mut len_writer, data_len as u64)?;

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
    pub fn decode(buffer: [u8; Self::SIZE]) -> Self {
        let line1 = &buffer[0..62];
        let line2 = &buffer[64..126];
        let salt_b64 = line2
            .iter()
            .enumerate()
            .rev()
            .find(|x| *x.1 != b' ')
            .map_or(&b""[..], |(idx, _)| &line2[0..idx]);
        // TODO error on empty salt
        let salt = base64::decode(salt_b64).expect("Valid base64");
        Self {
            server: String::from_utf8_lossy(line1).into_owned(),
            salt,
        }
    }
}
