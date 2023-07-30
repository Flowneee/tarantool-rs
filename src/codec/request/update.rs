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

use super::Request;

#[derive(Clone, Debug)]
pub(crate) struct Update {
    pub space_id: u32,
    pub index_id: u32,
    pub index_base: Option<u32>,
    pub keys: Vec<Value>,
    pub tuple: Vec<Value>,
}

impl Update {
    pub(crate) fn new(
        space_id: u32,
        index_id: u32,
        index_base: Option<u32>,
        keys: Vec<Value>,
        tuple: Vec<Value>,
    ) -> Self {
        Self {
            space_id,
            index_id,
            index_base,
            keys,
            tuple,
        }
    }
}

impl Request for Update {
    fn request_type() -> RequestType
    where
        Self: Sized,
    {
        RequestType::Replace
    }

    // NOTE: `&mut buf: mut` is required since I don't get why compiler complain
    fn encode(&self, mut buf: &mut dyn Write) -> Result<(), EncodingError> {
        let map_len = if self.index_base.is_some() { 5 } else { 4 };
        rmp::encode::write_map_len(&mut buf, map_len)?;
        write_kv_u32(buf, keys::SPACE_ID, self.space_id)?;
        write_kv_u32(buf, keys::INDEX_ID, self.index_id)?;
        if let Some(value) = self.index_base {
            write_kv_u32(buf, keys::INDEX_BASE, value)?;
        }
        write_kv_array(buf, keys::KEY, &self.keys)?;
        write_kv_array(buf, keys::TUPLE, &self.tuple)?;
        Ok(())
    }
}
