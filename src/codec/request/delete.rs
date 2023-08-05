use std::io::Write;

use crate::{
    codec::{
        consts::{keys, RequestType},
        utils::{write_kv_tuple, write_kv_u32},
    },
    errors::EncodingError,
    tuple::Tuple,
};

use super::Request;

#[derive(Clone, Debug)]
pub(crate) struct Delete<T> {
    pub space_id: u32,
    pub index_id: u32,
    pub keys: T,
}

impl<T> Delete<T> {
    pub(crate) fn new(space_id: u32, index_id: u32, keys: T) -> Self {
        Self {
            space_id,
            index_id,
            keys,
        }
    }
}

impl<T: Tuple> Request for Delete<T> {
    fn request_type() -> RequestType
    where
        Self: Sized,
    {
        RequestType::Replace
    }

    // NOTE: `&mut buf: mut` is required since I don't get why compiler complain
    fn encode(&self, mut buf: &mut dyn Write) -> Result<(), EncodingError> {
        rmp::encode::write_map_len(&mut buf, 3)?;
        write_kv_u32(buf, keys::SPACE_ID, self.space_id)?;
        write_kv_u32(buf, keys::INDEX_ID, self.index_id)?;
        write_kv_tuple(buf, keys::KEY, &self.keys)?;
        Ok(())
    }
}
