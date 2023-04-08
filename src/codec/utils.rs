use std::io::Cursor;

use anyhow::anyhow;
use rmpv::Value;
use serde::Deserialize;
use tracing::debug;

use crate::TransportError;

use super::consts::keys;

pub fn value_to_map(value: Value) -> Result<Vec<(Value, Value)>, TransportError> {
    match value {
        Value::Map(x) => Ok(x),
        _ => Err(TransportError::MessagePackDecode(anyhow!(
            "OK response body for non-SQL should be map"
        ))),
    }
}

pub fn parse_non_sql_response<'de, T>(data: &[u8]) -> Result<T, ()>
where
    T: Deserialize<'de>,
{
    let mut cursor = Cursor::new(data);
    todo!()
}

/// Extract IPROTO_DATA from response body.
pub fn data_from_response_body(value: Value) -> Result<Value, TransportError> {
    let map = value_to_map(value)?;
    for (k, v) in map {
        if matches!(k, Value::Integer(x) if x.as_u64().map_or(false, |y| y == keys::DATA as u64)) {
            return Ok(v);
        } else {
            // NOTE: no errors or warnings in case protocol adds new data in responsese in future
            debug!("Unexpected key encountered in response body: {:?}", k);
        }
    }
    Err(TransportError::MessagePackDecode(anyhow!(
        "No IPROTO_DATA key in MessagePack response body"
    )))
}
