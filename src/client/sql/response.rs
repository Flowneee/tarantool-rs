use rmpv::Value;
use serde::de::DeserializeOwned;

use crate::{
    codec::consts::keys,
    errors::DecodingError,
    utils::{extract_and_deserialize_iproto_data, find_and_take_single_key_in_map, value_to_map},
};

/// Response, returned from SQL requests.
///
/// Can be deserialized into different responses, depending on request.
#[derive(Clone, Debug, PartialEq)]
pub struct SqlResponse(pub(crate) rmpv::Value);

impl SqlResponse {
    /// Decode as response on `SELECT`.
    pub fn decode_select<T>(self) -> Result<Vec<T>, DecodingError>
    where
        T: DeserializeOwned,
    {
        self.decode_data_vec()
    }

    // TODO: separate functions for PRAGMA and VALUES

    /// Decode as data list in `IPROTO_DATA` tag.
    ///
    /// This is currently used for `SELECT`, `PRAGMA` and `VALUES` responses.
    pub fn decode_data_vec<T>(self) -> Result<Vec<T>, DecodingError>
    where
        T: DeserializeOwned,
    {
        extract_and_deserialize_iproto_data(self.0)
    }

    fn decode_sql_info_raw(self) -> Result<Vec<(Value, Value)>, DecodingError> {
        let map = value_to_map(self.0).map_err(|err| err.in_other("OK response body"))?;
        find_and_take_single_key_in_map(keys::SQL_INFO, map)
            .ok_or_else(|| DecodingError::missing_key("SQL_INFO"))
            .and_then(value_to_map)
            .map_err(|err| err.in_other("OK SQL response body"))
    }

    /// Get number of affected rows.
    pub fn row_count(self) -> Result<u64, DecodingError> {
        let sql_info = self.decode_sql_info_raw()?;
        find_and_take_single_key_in_map(keys::SQL_INFO_ROW_COUNT, sql_info)
            .ok_or_else(|| DecodingError::missing_key("SQL_INFO_ROW_COUNT"))
            .and_then(|x| rmpv::ext::from_value(x).map_err(Into::into))
    }
}
