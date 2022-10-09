use std::io::Write;

use crate::{
    codec::consts::{keys, IProtoType},
    ChannelError,
};

use super::{IProtoRequestBody, PROTOCOL_VERSION};

#[derive(Clone, Debug)]
pub struct IProtoId {
    pub streams: bool,
    pub transactions: bool,
    pub error_extension: bool,
    pub watchers: bool,
    pub protocol_version: u8,
}

impl Default for IProtoId {
    fn default() -> Self {
        Self {
            streams: true,
            transactions: false,
            error_extension: true,
            watchers: false,
            protocol_version: PROTOCOL_VERSION,
        }
    }
}

impl IProtoId {
    const STREAMS: u8 = 0;
    const TRANSACTIONS: u8 = 1;
    const ERROR_EXTENSION: u8 = 2;
    const WATCHERS: u8 = 3;
}

impl IProtoRequestBody for IProtoId {
    fn request_type() -> IProtoType
    where
        Self: Sized,
    {
        IProtoType::Id
    }

    // NOTE: `&mut buf: mut` is required since I don't get why compiler complain
    fn encode(&self, mut buf: &mut dyn Write) -> Result<(), ChannelError> {
        rmp::encode::write_map_len(&mut buf, 2)?;
        rmp::encode::write_pfix(&mut buf, keys::VERSION)?;
        rmp::encode::write_u8(&mut buf, self.protocol_version)?;
        rmp::encode::write_pfix(&mut buf, keys::FEATURES)?;
        let arr_len = if self.streams { 1 } else { 0 }
            + if self.transactions { 1 } else { 0 }
            + if self.error_extension { 1 } else { 0 }
            + if self.watchers { 1 } else { 0 };
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
