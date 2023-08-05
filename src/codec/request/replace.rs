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
pub(crate) struct Replace<T> {
    pub space_id: u32,
    pub tuple: T,
}

impl<T> Replace<T> {
    pub fn new(space_id: u32, tuple: T) -> Self {
        Self { space_id, tuple }
    }
}

impl<T: Tuple> Request for Replace<T> {
    fn request_type() -> RequestType
    where
        Self: Sized,
    {
        RequestType::Replace
    }

    // NOTE: `&mut buf: mut` is required since I don't get why compiler complain
    fn encode(&self, mut buf: &mut dyn Write) -> Result<(), EncodingError> {
        rmp::encode::write_map_len(&mut buf, 2)?;
        write_kv_u32(buf, keys::SPACE_ID, self.space_id)?;
        write_kv_tuple(buf, keys::TUPLE, &self.tuple)?;
        Ok(())
    }
}
