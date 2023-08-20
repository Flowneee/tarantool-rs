use rmpv::Value;
use serde::de::DeserializeOwned;

use crate::{errors::DecodingError, utils::extract_iproto_data, Error};

/// Tuple, returned from `call` and `eval` requests.
#[derive(Clone, Debug, PartialEq)]
pub struct TupleResponse(pub(crate) rmpv::Value);

impl TupleResponse {
    /// Decode first element of the tuple, dropping everything else.
    ///
    /// This is useful if function doesn't return an error.
    pub fn decode_first<T>(self) -> Result<T, DecodingError>
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

    /// Decode first 2 elements of the tuple, dropping everything else.
    pub fn decode_two<T1, T2>(self) -> Result<(T1, T2), DecodingError>
    where
        T1: DeserializeOwned,
        T2: DeserializeOwned,
    {
        let mut tuple_iter = self.into_data_tuple()?.into_iter();
        if tuple_iter.len() < 2 {
            return Err(DecodingError::invalid_tuple_length(2, tuple_iter.len()));
        }
        // SAFETY: this should be safe since we just checked tuple length
        let first = tuple_iter
            .next()
            .expect("tuple_iter should have length >= 2");
        let second = tuple_iter
            .next()
            .expect("tuple_iter should have length >= 2");
        Ok((
            rmpv::ext::from_value(first)?,
            rmpv::ext::from_value(second)?,
        ))
    }

    /// Decode first two elements of the tuple into result, where
    /// either first element deserialized into `T` and returned as `Ok(T)`
    /// or second element returned as `Err(Error::CallEval)`.
    ///
    /// If second element is `nil` or not present, first element will be returned,
    /// otherwise second element will be returned as error.
    pub fn decode_result<T>(self) -> Result<T, Error>
    where
        T: DeserializeOwned,
    {
        let mut tuple_iter = self.into_data_tuple()?.into_iter();
        let first = tuple_iter
            .next()
            .ok_or_else(|| DecodingError::invalid_tuple_length(1, 0))?;
        let second = tuple_iter.next();
        match second {
            Some(Value::Nil) | None => {
                Ok(rmpv::ext::from_value(first).map_err(DecodingError::from)?)
            }
            Some(err) => Err(Error::CallEval(err)),
        }
    }

    /// Decode entire response into type.
    ///
    /// Note that currently every response would be a tuple, so be careful what type
    /// you are specifying.
    pub fn decode_full<T>(self) -> Result<T, DecodingError>
    where
        T: DeserializeOwned,
    {
        Ok(rmpv::ext::from_value(extract_iproto_data(self.0)?)?)
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
    fn decode_first() {
        let resp = build_tuple_response(vec![Value::Boolean(true)]);
        assert_matches!(TupleResponse(resp).decode_first(), Ok(true));
    }

    #[test]
    fn decode_first_err_len() {
        let resp = build_tuple_response(vec![]);
        assert_matches!(TupleResponse(resp).decode_first::<()>(), Err(_));
    }

    #[test]
    fn decode_first_err_wrong_type() {
        let resp = build_tuple_response(vec![Value::Boolean(true)]);
        assert_matches!(TupleResponse(resp).decode_first::<String>(), Err(_));
    }

    #[test]
    fn decode_two() {
        let resp = build_tuple_response(vec![Value::Boolean(true), Value::Boolean(false)]);
        assert_matches!(TupleResponse(resp).decode_two(), Ok((true, false)));
    }

    #[test]
    fn decode_two_err_len() {
        let resp = build_tuple_response(vec![]);
        assert_matches!(TupleResponse(resp).decode_two::<(), ()>(), Err(_));

        let resp = build_tuple_response(vec![Value::Boolean(true)]);
        assert_matches!(TupleResponse(resp).decode_two::<(), ()>(), Err(_));
    }

    #[test]
    fn decode_result_ok() {
        let resp = build_tuple_response(vec![Value::Boolean(true)]);
        assert_matches!(TupleResponse(resp).decode_result(), Ok(true));

        let resp = build_tuple_response(vec![Value::Boolean(true), Value::Nil]);
        assert_matches!(TupleResponse(resp).decode_result(), Ok(true));
    }

    #[test]
    fn decode_result_err_present() {
        let resp = build_tuple_response(vec![Value::Boolean(true), Value::Boolean(false)]);
        assert_matches!(
            TupleResponse(resp).decode_result::<bool>(),
            Err(Error::CallEval(Value::Boolean(false)))
        );
    }

    #[test]
    fn decode_result_err_wrong_type() {
        let resp = build_tuple_response(vec![Value::Boolean(true), Value::Nil]);
        assert_matches!(TupleResponse(resp).decode_result::<String>(), Err(_));
    }

    #[test]
    fn decode_full() {
        let resp = build_tuple_response(vec![Value::Boolean(true), Value::Boolean(false)]);
        assert_matches!(TupleResponse(resp).decode_full(), Ok((true, Some(false))));
    }
}
