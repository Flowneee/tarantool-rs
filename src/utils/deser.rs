use rmpv::Value;
use serde::de::DeserializeOwned;
use tracing::debug;

use crate::{codec::consts::keys, errors::DecodingError};

fn value_to_map(value: Value) -> Result<Vec<(Value, Value)>, DecodingError> {
    match value {
        Value::Map(x) => Ok(x),
        rest => Err(DecodingError::type_mismatch("map", rest.to_string())),
    }
}

/// Extract IPROTO_DATA from response body and deserialize it.
pub(crate) fn extract_iproto_data(value: Value) -> Result<Value, DecodingError> {
    let map = value_to_map(value).map_err(|err| err.in_other("OK response body"))?;
    for (k, v) in map {
        if matches!(k, Value::Integer(x) if x.as_u64().map_or(false, |y| y == keys::DATA as u64)) {
            return Ok(v);
        } else {
            // NOTE: no errors or warnings in case protocol adds new data in responsese in future
            // TODO: configurable logging level?
            debug!("Unexpected key encountered in response body: {:?}", k);
        }
    }
    Err(DecodingError::missing_key("DATA").in_other("OK response body"))
}

/// Extract IPROTO_DATA from response body and deserialize it into provided type.
pub(crate) fn extract_and_deserialize_iproto_data<T: DeserializeOwned>(
    value: Value,
) -> Result<T, DecodingError> {
    extract_iproto_data(value).and_then(|x| rmpv::ext::from_value(x).map_err(Into::into))
}
