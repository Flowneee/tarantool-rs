use rmpv::Value;
use serde::de::DeserializeOwned;

use crate::{errors::DecodingError, utils::extract_iproto_data};

// TODO: unify with call_response.rs

/// Tuple, returned from all data-manipulation operations (insert, update, upsert, replace, delete).
#[derive(Clone, Debug, PartialEq)]
pub struct DmoResponse(pub(crate) rmpv::Value);

impl DmoResponse {
    /// Decode row into type.
    ///
    /// Raises error if no rows returned.
    pub fn decode<T>(self) -> Result<T, DecodingError>
    where
        T: DeserializeOwned,
    {
        let first = self
            .into_data_tuple()?
            .into_iter()
            .next()
            .ok_or_else(|| DecodingError::invalid_tuple_length(1, 0))?;
        Ok(rmpv::ext::from_value(first)?)
    }

    /// Decode row into type or return `None` if no rows returned.
    pub fn decode_opt<T>(self) -> Result<Option<T>, DecodingError>
    where
        T: DeserializeOwned,
    {
        self.into_data_tuple()?
            .into_iter()
            .next()
            .map(rmpv::ext::from_value::<T>)
            .transpose()
            .map_err(Into::into)
    }

    fn into_data_tuple(self) -> Result<Vec<Value>, DecodingError> {
        match extract_iproto_data(self.0)? {
            Value::Array(x) => Ok(x),
            rest => Err(DecodingError::type_mismatch("array", rest.to_string())),
        }
    }
}

#[cfg(test)]
mod tests {
    use assert_matches::assert_matches;

    use crate::codec::consts::keys::DATA;

    use super::*;

    fn build_tuple_response(data: Vec<Value>) -> Value {
        Value::Map(vec![(DATA.into(), Value::Array(data))])
    }

    #[test]
    fn decode() {
        let resp = build_tuple_response(vec![Value::Boolean(true)]);
        assert_matches!(DmoResponse(resp).decode(), Ok(true));
    }

    #[test]
    fn decode_err_len() {
        let resp = build_tuple_response(vec![]);
        assert_matches!(DmoResponse(resp).decode::<()>(), Err(_));
    }

    #[test]
    fn decode_opt() {
        let resp = build_tuple_response(vec![Value::Boolean(true)]);
        assert_matches!(DmoResponse(resp).decode_opt(), Ok(Some(true)));
    }

    #[test]
    fn decode_opt_none() {
        let resp = build_tuple_response(vec![]);
        assert_matches!(DmoResponse(resp).decode_opt::<()>(), Ok(None));
    }
}
