use std::io::Write;

use crate::{
    codec::{
        consts::{keys, IteratorType, RequestType},
        utils::{write_kv_tuple, write_kv_u32},
    },
    errors::EncodingError,
    tuple::Tuple,
};

use super::Request;

#[derive(Clone, Debug)]
pub(crate) struct Select<T> {
    pub space_id: u32,
    pub index_id: u32,
    pub limit: Option<u32>,
    pub offset: Option<u32>,
    pub iterator: Option<IteratorType>,
    pub keys: T,
}

impl<T> Select<T> {
    pub fn new(
        space_id: u32,
        index_id: u32,
        limit: Option<u32>,
        offset: Option<u32>,
        iterator: Option<IteratorType>,
        keys: T,
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

impl<T: Tuple> Request for Select<T> {
    fn request_type() -> RequestType
    where
        Self: Sized,
    {
        RequestType::Select
    }

    // NOTE: `&mut buf: mut` is required since I don't get why compiler complain
    fn encode(&self, mut buf: &mut dyn Write) -> Result<(), EncodingError> {
        rmp::encode::write_map_len(&mut buf, 6)?;
        write_kv_u32(buf, keys::SPACE_ID, self.space_id)?;
        write_kv_u32(buf, keys::INDEX_ID, self.index_id)?;
        // default values: https://github.com/tarantool/tarantool/blob/master/src/box/lua/net_box.c#L735
        write_kv_u32(buf, keys::LIMIT, self.limit.unwrap_or(u32::MAX))?;
        write_kv_u32(buf, keys::OFFSET, self.offset.unwrap_or(0))?;
        write_kv_u32(
            buf,
            keys::ITERATOR,
            self.iterator.unwrap_or_default() as u32,
        )?;
        write_kv_tuple(buf, keys::KEY, &self.keys)?;
        Ok(())
    }
}
