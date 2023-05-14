use std::io::Write;

use rmpv::Value;
use serde::de::DeserializeOwned;
use tracing::debug;

use super::consts::keys;
use crate::errors::{DecodingError, EncodingError};

pub fn value_to_map(value: Value) -> Result<Vec<(Value, Value)>, DecodingError> {
    match value {
        Value::Map(x) => Ok(x),
        rest => {
            Err(DecodingError::type_mismatch("map", rest.to_string()).in_other("OK response body"))
        }
    }
}

/// Extract IPROTO_DATA from response body and deserialize it.
pub fn deserialize_non_sql_response<T: DeserializeOwned>(value: Value) -> Result<T, DecodingError> {
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
    Err(DecodingError::missing_key("DATA"))
}

pub fn write_kv_str(mut buf: &mut dyn Write, key: u8, value: &str) -> Result<(), EncodingError> {
    rmp::encode::write_pfix(&mut buf, key)?;
    rmp::encode::write_str(&mut buf, value)?;
    Ok(())
}

pub fn write_kv_u32(mut buf: &mut dyn Write, key: u8, value: u32) -> Result<(), EncodingError> {
    rmp::encode::write_pfix(&mut buf, key)?;
    rmp::encode::write_u32(&mut buf, value)?;
    Ok(())
}

pub fn write_kv_array(
    mut buf: &mut dyn Write,
    key: u8,
    value: &[Value],
) -> Result<(), EncodingError> {
    rmp::encode::write_pfix(&mut buf, key)?;
    // TODO: safe conversion from usize to u32
    rmp::encode::write_array_len(&mut buf, value.len() as u32)?;
    for x in value.iter() {
        rmpv::encode::write_value(&mut buf, x)?;
    }
    Ok(())
}
