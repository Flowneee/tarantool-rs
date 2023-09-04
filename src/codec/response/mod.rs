use std::io::Read;

use tracing::{debug, error};

use super::consts::response_codes::{ERROR_RANGE_END, ERROR_RANGE_START, OK};
use crate::{
    codec::consts::keys,
    errors::{DecodingError, ErrorResponse},
};

// TODO: add out-of-band (I.e. IPROTO_CHUNK)
// TODO: actually implement extra error data
// TODO: create bodies for specific responses (for optimization reasons)
#[derive(Clone, Debug)]
pub(crate) enum ResponseBody {
    Ok(rmpv::Value), // TODO: replace
    Error(ErrorResponse),
}

#[derive(Clone, Debug)]
pub(crate) struct Response {
    pub sync: u32,
    pub schema_version: u32,
    pub body: ResponseBody,
}

impl Response {
    // TODO: split function
    // Use [`anyhow::Error`] because any error would mean either entirely broken
    // implementation of protocol or underlying I/O error, which currently would be
    // implementation bug as well.
    pub(super) fn decode(mut buf: impl Read) -> Result<Self, DecodingError> {
        let map_len = rmp::decode::read_map_len(&mut buf)?;
        let mut response_code: Option<u32> = None;
        let mut sync: Option<u32> = None;
        let mut schema_version: Option<u32> = None;
        for _ in 0..map_len {
            let key: u8 = rmp::decode::read_pfix(&mut buf)?;
            match key {
                keys::RESPONSE_CODE => {
                    response_code = Some(rmp::decode::read_int(&mut buf)?);
                }
                keys::SYNC => {
                    sync = Some(rmp::decode::read_int(&mut buf)?);
                }
                keys::SCHEMA_VERSION => {
                    schema_version = Some(rmp::decode::read_int(&mut buf)?);
                }
                rest => {
                    // TODO: configurable level for this warn?
                    debug!("Unexpected key encountered in response header: {}", rest);
                    let _ = rmpv::decode::read_value(&mut buf)?;
                }
            }
        }
        let Some(response_code) = response_code else {
            return Err(DecodingError::missing_key("RESPONSE_CODE"));
        };
        let Some(sync) = sync else {
            return Err(DecodingError::missing_key("SYNC"));
        };
        let Some(schema_version) = schema_version else {
            return Err(DecodingError::missing_key("SCHEMA_VERSION"));
        };
        let body = match response_code {
            OK => {
                let v = rmpv::decode::read_value(&mut buf)?;
                debug!("{}", v);
                ResponseBody::Ok(v)
            }
            code @ ERROR_RANGE_START..=ERROR_RANGE_END => {
                let code = code - 0x8000;
                let mut description = None;
                let mut extra = None;
                let map_len = rmp::decode::read_map_len(&mut buf)?;
                for _ in 0..map_len {
                    let key: u8 = rmp::decode::read_pfix(&mut buf)?;
                    match key {
                        keys::ERROR_24 => {
                            // TODO: rewrite string decoding
                            let str_len = rmp::decode::read_str_len(&mut buf)?;
                            let mut str_buf = vec![0; str_len as usize];
                            buf.read_exact(&mut str_buf).map_err(|err| {
                                DecodingError::message_pack(err).in_key("ERROR_24")
                            })?;
                            // TODO: find a way to to this safe
                            description = Some(String::from_utf8(str_buf).map_err(|_err| {
                                DecodingError::message_pack(anyhow::anyhow!(
                                    "String is not valid UTF-8 string"
                                ))
                                .in_key("ERROR_24")
                            })?);
                        }
                        keys::ERROR => {
                            extra = Some(rmpv::decode::read_value(&mut buf)?);
                        }
                        rest => {
                            error!("Unexpected key encountered in error description: {}", rest);
                            let _ = rmpv::decode::read_value(&mut buf)?;
                        }
                    }
                }
                let Some(description) = description else {
                    return Err(DecodingError::missing_key("ERROR_24"));
                };
                ResponseBody::Error(ErrorResponse {
                    code,
                    description,
                    extra,
                })
            }
            rest => return Err(DecodingError::unknown_response_code(rest)),
        };
        Ok(Self {
            sync,
            schema_version,
            body,
        })
    }
}
