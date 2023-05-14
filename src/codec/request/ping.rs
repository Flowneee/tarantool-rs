use std::io::Write;

use super::RequestBody;
use crate::{codec::consts::RequestType, errors::EncodingError};

#[derive(Clone, Debug)]
pub(crate) struct Ping {}

impl RequestBody for Ping {
    fn request_type() -> RequestType {
        RequestType::Ping
    }

    fn encode(&self, _buf: &mut dyn Write) -> Result<(), EncodingError> {
        Ok(())
    }
}
