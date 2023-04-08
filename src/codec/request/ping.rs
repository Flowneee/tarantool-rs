use std::io::Write;

use super::RequestBody;
use crate::{codec::consts::RequestType, TransportError};

#[derive(Clone, Debug)]
pub struct Ping {}

impl RequestBody for Ping {
    fn request_type() -> RequestType {
        RequestType::Ping
    }

    fn encode(&self, _buf: &mut dyn Write) -> Result<(), TransportError> {
        Ok(())
    }
}
