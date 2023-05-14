// TODO: unify with eval.rs

use std::io::Write;

use rmpv::Value;

use crate::{
    codec::{
        consts::{keys, RequestType},
        utils::{write_kv_array, write_kv_u32},
    },
    errors::EncodingError,
};

use super::RequestBody;

#[derive(Clone, Debug)]
pub(crate) struct Upsert {
    pub space_id: u32,
    pub index_base: u32,
    pub ops: Vec<Value>,
    pub tuple: Vec<Value>,
}

impl Upsert {
    pub(crate) fn new(space_id: u32, index_base: u32, ops: Vec<Value>, tuple: Vec<Value>) -> Self {
        Self {
            space_id,
            index_base,
            ops,
            tuple,
        }
    }
}

impl RequestBody for Upsert {
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
        write_kv_u32(buf, keys::INDEX_BASE, self.index_base)?;
        write_kv_array(buf, keys::OPS, &self.ops)?;
        write_kv_array(buf, keys::TUPLE, &self.tuple)?;
        Ok(())
    }
}
