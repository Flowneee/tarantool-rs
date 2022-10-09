// TODO: unify with rollback/begin.rs

use std::io::Write;

use crate::{codec::consts::IProtoType, ChannelError};

use super::IProtoRequestBody;

#[derive(Clone, Debug, Default)]
pub struct IProtoCommit {}

impl IProtoRequestBody for IProtoCommit {
    fn request_type() -> IProtoType
    where
        Self: Sized,
    {
        IProtoType::Commit
    }

    // NOTE: `&mut buf: mut` is required since I don't get why compiler complain
    fn encode(&self, _buf: &mut dyn Write) -> Result<(), ChannelError> {
        Ok(())
    }
}
