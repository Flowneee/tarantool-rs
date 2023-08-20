use std::io::Write;

use crate::{
    codec::{
        consts::{keys, RequestType},
        utils::{write_kv_tuple, write_kv_u32},
    },
    errors::EncodingError,
    tuple::Tuple,
};

use super::{Request, INDEX_BASE_VALUE};

// TODO: replace keys with structured operations description
#[derive(Clone, Debug)]
pub(crate) struct Update<K, T> {
    pub space_id: u32,
    pub index_id: u32,
    pub keys: K,
    pub tuple: T,
}

impl<K, T> Update<K, T> {
    pub(crate) fn new(space_id: u32, index_id: u32, keys: K, ops: T) -> Self {
        Self {
            space_id,
            index_id,
            keys,
            tuple: ops,
        }
    }
}

impl<K: Tuple, T: Tuple> Request for Update<K, T> {
    fn request_type() -> RequestType
    where
        Self: Sized,
    {
        RequestType::Update
    }

    // NOTE: `&mut buf: mut` is required since I don't get why compiler complain
    fn encode(&self, mut buf: &mut dyn Write) -> Result<(), EncodingError> {
        rmp::encode::write_map_len(&mut buf, 5)?;
        write_kv_u32(buf, keys::SPACE_ID, self.space_id)?;
        write_kv_u32(buf, keys::INDEX_ID, self.index_id)?;
        write_kv_u32(buf, keys::INDEX_BASE, INDEX_BASE_VALUE)?;
        write_kv_tuple(buf, keys::KEY, &self.keys)?;
        write_kv_tuple(buf, keys::TUPLE, &self.tuple)?;
        Ok(())
    }
}
