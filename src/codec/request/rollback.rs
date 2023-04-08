// TODO: unify with commit/begin.rs

use std::io::Write;

use crate::{codec::consts::RequestType, TransportError};

use super::RequestBody;

#[derive(Clone, Debug, Default)]
pub struct Rollback {}

impl RequestBody for Rollback {
    fn request_type() -> RequestType
    where
        Self: Sized,
    {
        RequestType::Rollback
    }

    fn encode(&self, _buf: &mut dyn Write) -> Result<(), TransportError> {
        Ok(())
    }
}
