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

#[derive(Clone, Debug)]
pub(crate) struct Upsert<O, T> {
    pub space_id: u32,
    pub ops: O,
    pub tuple: T,
}

impl<O, T> Upsert<O, T> {
    pub(crate) fn new(space_id: u32, ops: O, tuple: T) -> Self {
        Self {
            space_id,
            ops,
            tuple,
        }
    }
}

impl<O: Tuple, T: Tuple> Request for Upsert<O, T> {
    fn request_type() -> RequestType
    where
        Self: Sized,
    {
        RequestType::Replace
    }

    // TODO: test whether index_base is mandatory
    // NOTE: `&mut buf: mut` is required since I don't get why compiler complain
    fn encode(&self, mut buf: &mut dyn Write) -> Result<(), EncodingError> {
        rmp::encode::write_map_len(&mut buf, 4)?;
        write_kv_u32(buf, keys::SPACE_ID, self.space_id)?;
        write_kv_u32(buf, keys::INDEX_BASE, INDEX_BASE_VALUE)?;
        write_kv_tuple(buf, keys::OPS, &self.ops)?;
        write_kv_tuple(buf, keys::TUPLE, &self.tuple)?;
        Ok(())
    }
}
