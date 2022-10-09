use std::io::Read;

use anyhow::anyhow;
use tracing::debug;

use crate::{codec::consts::keys, errors::ChannelError};

// TODO: add out-of-band (I.e. IPROTO_CHUNK)
// TODO: actually implement extra error data
// TODO: create bodyes for specific responses
#[derive(Clone, Debug)]
pub enum IProtoResponseBody {
    Ok(rmpv::Value), // TODO: replace
    Error {
        code: u32,
        description: String,
        extra: Option<rmpv::Value>,
    },
}

// TODO: hide fields and export them via getters
#[derive(Clone, Debug)]
pub struct IProtoResponse {
    pub sync: u32,
    pub schema_version: u32,
    pub body: IProtoResponseBody,
}

impl IProtoResponse {
    // TODO: get rid of `Error =` bound
    // TODO: split function
    pub fn decode(mut buf: impl Read) -> Result<Self, ChannelError> {
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
            ChannelError::MessagePackDecode(anyhow!("Missing response code in response"))
        })?;
        let sync = sync
            .ok_or_else(|| ChannelError::MessagePackDecode(anyhow!("Missing sync in response")))?;
        let schema_version = schema_version.ok_or_else(|| {
            ChannelError::MessagePackDecode(anyhow!("Missing schema version in response"))
        })?;
        let body = match response_code {
            0x0 => IProtoResponseBody::Ok(rmpv::decode::read_value(&mut buf)?),
            code @ 0x8000..=0x8FFF => {
                let code = code - 0x8000;
                let mut description: Option<String> = None;
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
                                ChannelError::MessagePackDecode(anyhow!(
                                    "Failed to decode error description: {}",
                                    e
                                ))
                            })?;
                            // TODO: find a way to to this safe
                            description = Some(String::from_utf8(str_buf).map_err(|e| {
                                ChannelError::MessagePackDecode(anyhow!(
                                    "Message description is not valid UTF-8 string: {}",
                                    e
                                ))
                            })?);
                        }
                        keys::ERROR => {
                            extra = Some(rmpv::decode::read_value(&mut buf)?);
                        }
                        rest => {
                            debug!("Unexpected key encountered in error description: {}", rest);
                            let _ = rmpv::decode::read_value(&mut buf)?;
                        }
                    }
                }
                let description = description.ok_or_else(|| {
                    ChannelError::MessagePackDecode(anyhow!(
                        "Missing error description in response body"
                    ))
                })?;
                IProtoResponseBody::Error {
                    code,
                    description,
                    extra,
                }
            }
            // TODO: maybe separate error for this?
            rest => {
                return Err(ChannelError::MessagePackDecode(anyhow!(
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
