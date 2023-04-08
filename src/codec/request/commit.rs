// TODO: unify with rollback/begin.rs

use std::io::Write;

use crate::{codec::consts::RequestType, TransportError};

use super::RequestBody;

#[derive(Clone, Debug, Default)]
pub struct Commit {}

impl RequestBody for Commit {
    fn request_type() -> RequestType
    where
        Self: Sized,
    {
        RequestType::Commit
    }

    // NOTE: `&mut buf: mut` is required since I don't get why compiler complain
    fn encode(&self, _buf: &mut dyn Write) -> Result<(), TransportError> {
        Ok(())
    }
}
