use std::io::Read;

use anyhow::{bail, Context};
use tracing::{debug, error};

use super::consts::response_codes::{ERROR_RANGE_END, ERROR_RANGE_START, OK};
use crate::{codec::consts::keys, errors::ErrorResponse};

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
    pub(super) fn decode(mut buf: impl Read) -> Result<Self, anyhow::Error> {
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
            bail!("Missing response code in response")
        };
        let Some(sync) = sync else {
            bail!("Missing sync in response")
        };
        let Some(schema_version) = schema_version else {
            bail!("Missing schema version in response")
        };
        let body = match response_code {
            OK => ResponseBody::Ok(rmpv::decode::read_value(&mut buf)?),
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
                            let _ = buf
                                .read_exact(&mut str_buf)
                                .context("Failed to decode error description")?;
                            // TODO: find a way to to this safe
                            description = Some(
                                String::from_utf8(str_buf)
                                    .context("Error description is not valid UTF-8 string")?,
                            );
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
                    bail!( "Missing error description in response body")
                };
                ResponseBody::Error(ErrorResponse {
                    code,
                    description,
                    extra,
                })
            }
            rest => bail!("Unknown response code: {}", rest),
        };
        Ok(Self {
            sync,
            schema_version,
            body,
        })
    }
}
