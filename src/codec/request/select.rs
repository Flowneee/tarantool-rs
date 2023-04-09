use std::{borrow::Cow, io::Write};

use rmpv::Value;

use crate::codec::{
    consts::{keys, IteratorType, RequestType},
    utils::{write_kv_array, write_kv_u32},
};

use super::RequestBody;

#[derive(Clone, Debug)]
pub(crate) struct Select {
    pub space_id: u32,
    pub index_id: u32,
    pub limit: u32,
    pub offset: u32,
    pub iterator: IteratorType,
    pub keys: Vec<Value>,
}

impl Select {
    pub fn new(
        space_id: u32,
        index_id: u32,
        limit: u32,
        offset: u32,
        iterator: IteratorType,
        keys: Vec<Value>,
    ) -> Self {
        Self {
            space_id,
            index_id,
            limit,
            offset,
            iterator,
            keys,
        }
    }
}

impl RequestBody for Select {
    fn request_type() -> RequestType
    where
        Self: Sized,
    {
        RequestType::Select
    }

    // NOTE: `&mut buf: mut` is required since I don't get why compiler complain
    fn encode(&self, mut buf: &mut dyn Write) -> Result<(), anyhow::Error> {
        rmp::encode::write_map_len(&mut buf, 6)?;
        write_kv_u32(buf, keys::SPACE_ID, self.space_id)?;
        write_kv_u32(buf, keys::INDEX_ID, self.index_id)?;
        write_kv_u32(buf, keys::LIMIT, self.limit)?;
        write_kv_u32(buf, keys::OFFSET, self.offset)?;
        write_kv_u32(buf, keys::ITERATOR, self.iterator as u32)?;
        write_kv_array(buf, keys::KEY, &self.keys)?;
        Ok(())
    }
}
