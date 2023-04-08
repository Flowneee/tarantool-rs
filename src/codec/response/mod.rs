use std::io::Read;

use anyhow::anyhow;
use bytes::Bytes;
use tracing::{debug, error};

use super::consts::response_codes::{ERROR_RANGE_END, ERROR_RANGE_START, OK};
use crate::{
    codec::consts::keys,
    errors::{ErrorResponse, TransportError},
};

// TODO: add out-of-band (I.e. IPROTO_CHUNK)
// TODO: actually implement extra error data
// TODO: create bodies for specific responses (for optimization reasons)
#[derive(Clone, Debug)]
pub enum ResponseBody {
    // It's up to caller to decode body of the successfull response
    Ok(Bytes),
    Error(ErrorResponse),
}

// TODO: hide fields and export them via getters
#[derive(Clone, Debug)]
pub struct Response {
    pub sync: u32,
    pub schema_version: u32,
    pub body: ResponseBody,
}

impl Response {
    // TODO: get rid of `Error =` bound
    // TODO: split function
    pub fn decode(mut buf: impl Read) -> Result<Self, TransportError> {
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
                    debug!("Unexpected key encountered in response header: {}", rest);
                    let _ = rmpv::decode::read_value(&mut buf)?;
                }
            }
        }
        let response_code = response_code.ok_or_else(|| {
            TransportError::MessagePackDecode(anyhow!("Missing response code in response"))
        })?;
        let sync = sync.ok_or_else(|| {
            TransportError::MessagePackDecode(anyhow!("Missing sync in response"))
        })?;
        let schema_version = schema_version.ok_or_else(|| {
            TransportError::MessagePackDecode(anyhow!("Missing schema version in response"))
        })?;
        let body = match response_code {
            OK => {
                // TODO: Allocate some memory in advance
                let mut buffer = Vec::new();
                // TODO: improve errors
                buf.read_to_end(&mut buffer).map_err(|err| {
                    TransportError::MessagePackDecode(anyhow!("Failed to read buffer"))
                })?;
                ResponseBody::Ok(buffer.into())
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
                            let _ = buf.read_exact(&mut str_buf).map_err(|e| {
                                TransportError::MessagePackDecode(anyhow!(
                                    "Failed to decode error description: {}",
                                    e
                                ))
                            })?;
                            // TODO: find a way to to this safe
                            description = Some(String::from_utf8(str_buf).map_err(|e| {
                                TransportError::MessagePackDecode(anyhow!(
                                    "Message description is not valid UTF-8 string: {}",
                                    e
                                ))
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
                let description = description.ok_or_else(|| {
                    TransportError::MessagePackDecode(anyhow!(
                        "Missing error description in response body"
                    ))
                })?;
                ResponseBody::Error(ErrorResponse {
                    code,
                    description,
                    extra,
                })
            }
            // TODO: maybe separate error for this?
            rest => {
                return Err(TransportError::MessagePackDecode(anyhow!(
                    "Unknown response code: {}",
                    rest
                )))
            }
        };
        Ok(Self {
            sync,
            schema_version,
            body,
        })
    }
}
