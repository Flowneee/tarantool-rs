//! Various helpers.

pub(crate) use self::{
    deser::{extract_and_deserialize_iproto_data, extract_iproto_data},
    unique_id_name_map::{UniqueIdName, UniqueIdNameMap},
};

mod deser;
mod unique_id_name_map;
