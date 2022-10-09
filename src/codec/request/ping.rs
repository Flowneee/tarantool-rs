use std::io::Write;

use super::IProtoRequestBody;
use crate::{codec::consts::IProtoType, ChannelError};

#[derive(Clone, Debug)]
pub struct IProtoPing {}

impl IProtoRequestBody for IProtoPing {
    fn request_type() -> IProtoType {
        IProtoType::Ping
    }

    fn encode(&self, _buf: &mut dyn Write) -> Result<(), ChannelError> {
        Ok(())
    }
}
