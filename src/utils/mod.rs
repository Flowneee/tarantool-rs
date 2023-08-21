//! Various helpers.

pub use self::deser::{extract_and_deserialize_iproto_data, extract_iproto_data, value_to_map};

pub(crate) use self::{
    deser::find_and_take_single_key_in_map,
    unique_id_name_map::{UniqueIdName, UniqueIdNameMap},
};

mod deser;
mod unique_id_name_map;
