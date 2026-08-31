use std::io::Write;

use crate::{
    codec::consts::{RequestType, keys},
    errors::EncodingError,
};

use super::{PROTOCOL_VERSION, Request};

#[derive(Clone, Debug)]
// Mirrors the independent IPROTO_ID feature flags.
#[allow(clippy::struct_excessive_bools)]
pub(crate) struct Id {
    pub streams: bool,
    pub transactions: bool,
    pub error_extension: bool,
    pub watchers: bool,
    pub protocol_version: u8,
}

impl Default for Id {
    fn default() -> Self {
        Self {
            streams: true,
            transactions: true,
            error_extension: true,
            watchers: false,
            protocol_version: PROTOCOL_VERSION,
        }
    }
}

impl Id {
    const STREAMS: u8 = 0;
    const TRANSACTIONS: u8 = 1;
    const ERROR_EXTENSION: u8 = 2;
    const WATCHERS: u8 = 3;
}

impl Request for Id {
    fn request_type() -> RequestType
    where
        Self: Sized,
    {
        RequestType::Id
    }

    // NOTE: `&mut buf: mut` is required since I don't get why compiler complain
    fn encode(&self, mut buf: &mut dyn Write) -> Result<(), EncodingError> {
        rmp::encode::write_map_len(&mut buf, 2)?;
        rmp::encode::write_pfix(&mut buf, keys::VERSION)?;
        rmp::encode::write_u8(&mut buf, self.protocol_version)?;
        rmp::encode::write_pfix(&mut buf, keys::FEATURES)?;
        let arr_len = u32::from(self.streams)
            + u32::from(self.transactions)
            + u32::from(self.error_extension)
            + u32::from(self.watchers);
        rmp::encode::write_array_len(&mut buf, arr_len)?;
        if self.streams {
            rmp::encode::write_u8(&mut buf, Self::STREAMS)?;
        }
        if self.transactions {
            rmp::encode::write_u8(&mut buf, Self::TRANSACTIONS)?;
        }
        if self.error_extension {
            rmp::encode::write_u8(&mut buf, Self::ERROR_EXTENSION)?;
        }
        if self.watchers {
            rmp::encode::write_u8(&mut buf, Self::WATCHERS)?;
        }
        Ok(())
    }
}
