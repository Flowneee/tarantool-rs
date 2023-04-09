use std::io::Cursor;

use anyhow::anyhow;
use rmpv::Value;
use serde::de::DeserializeOwned;
use tracing::debug;

use super::consts::keys;
use crate::Error;

pub fn value_to_map(value: Value) -> Result<Vec<(Value, Value)>, Error> {
    match value {
        Value::Map(x) => Ok(x),
        _ => Err(Error::ResponseBodyDecode(anyhow!(
            "OK response body for non-SQL should be map"
        ))),
    }
}

/// Extract IPROTO_DATA from response body and deserialize it.
pub fn deserialize_non_sql_response<T: DeserializeOwned>(value: Value) -> Result<T, Error> {
    let map = value_to_map(value)?;
    for (k, v) in map {
        if matches!(k, Value::Integer(x) if x.as_u64().map_or(false, |y| y == keys::DATA as u64)) {
            return Ok(rmpv::ext::from_value(v)?);
        } else {
            // NOTE: no errors or warnings in case protocol adds new data in responsese in future
            // TODO: configurable logging level?
            debug!("Unexpected key encountered in response body: {:?}", k);
        }
    }
    Err(Error::ResponseBodyDecode(anyhow!(
        "No IPROTO_DATA key in MessagePack response body"
    )))
}
