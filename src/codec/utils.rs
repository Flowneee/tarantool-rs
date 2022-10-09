use anyhow::anyhow;
use rmpv::Value;
use tracing::debug;

use crate::ChannelError;

use super::consts::keys;

pub fn value_to_map(value: Value) -> Result<Vec<(Value, Value)>, ChannelError> {
    match value {
        Value::Map(x) => Ok(x),
        _ => Err(ChannelError::MessagePackDecode(anyhow!(
            "OK response body for non-SQL should be map"
        ))),
    }
}

/// Extract IPROTO_DATA from response body.
pub fn data_from_response_body(value: Value) -> Result<Value, ChannelError> {
    let map = value_to_map(value)?;
    for (k, v) in map {
        if matches!(k, Value::Integer(x) if x.as_u64().map_or(false, |y| y == keys::DATA as u64)) {
            return Ok(v);
        } else {
            debug!("Unexpected key encountered in response body: {:?}", k);
        }
    }
    Err(ChannelError::MessagePackDecode(anyhow!(
        "No IPROTO_DATA in response body"
    )))
}
