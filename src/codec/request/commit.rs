use std::io::Write;

use crate::{codec::consts::RequestType, errors::EncodingError};

use super::Request;

#[derive(Clone, Debug, Default)]
pub(crate) struct Commit {}

impl Request for Commit {
    fn request_type() -> RequestType
    where
        Self: Sized,
    {
        RequestType::Commit
    }

    fn encode(&self, _buf: &mut dyn Write) -> Result<(), EncodingError> {
        Ok(())
    }
}
