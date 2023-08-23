use rmpv::Value;
use serde::de::DeserializeOwned;

use crate::{codec::consts::keys, errors::DecodingError};

pub fn value_to_map(value: Value) -> Result<Vec<(Value, Value)>, DecodingError> {
    match value {
        Value::Map(x) => Ok(x),
        rest => Err(DecodingError::type_mismatch("map", rest.to_string())),
    }
}

pub(crate) fn find_and_take_single_key_in_map(key: u8, map: Vec<(Value, Value)>) -> Option<Value> {
    for (k, v) in map {
        if matches!(k, Value::Integer(x) if x.as_u64().map_or(false, |y| y == key as u64)) {
            return Some(v);
        }
    }
    None
}

/// Extract IPROTO_DATA from response body and deserialize it.
pub fn extract_iproto_data(value: Value) -> Result<Value, DecodingError> {
    let map = value_to_map(value).map_err(|err| err.in_other("OK response body"))?;
    find_and_take_single_key_in_map(keys::DATA, map)
        .ok_or_else(|| DecodingError::missing_key("DATA").in_other("OK response body"))
}

/// Extract IPROTO_DATA from response body and deserialize it into provided type.
pub fn extract_and_deserialize_iproto_data<T: DeserializeOwned>(
    value: Value,
) -> Result<T, DecodingError> {
    extract_iproto_data(value).and_then(|x| rmpv::ext::from_value(x).map_err(Into::into))
}
