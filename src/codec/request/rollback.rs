// TODO: unify with commit/begin.rs

use std::io::Write;

use crate::{codec::consts::RequestType, errors::EncodingError};

use super::Request;

#[derive(Clone, Debug, Default)]
pub(crate) struct Rollback {}

impl Request for Rollback {
    fn request_type() -> RequestType
    where
        Self: Sized,
    {
        RequestType::Rollback
    }

    fn encode(&self, _buf: &mut dyn Write) -> Result<(), EncodingError> {
        Ok(())
    }
}
