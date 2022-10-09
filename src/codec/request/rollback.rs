// TODO: unify with commit/begin.rs

use std::io::Write;

use crate::{codec::consts::IProtoType, ChannelError};

use super::IProtoRequestBody;

#[derive(Clone, Debug, Default)]
pub struct IProtoRollback {}

impl IProtoRequestBody for IProtoRollback {
    fn request_type() -> IProtoType
    where
        Self: Sized,
    {
        IProtoType::Rollback
    }

    // NOTE: `&mut buf: mut` is required since I don't get why compiler complain
    fn encode(&self, _buf: &mut dyn Write) -> Result<(), ChannelError> {
        Ok(())
    }
}
