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
pub(crate) struct Replace {
    pub space_id: u32,
    pub tuple: Vec<Value>,
}

impl Replace {
    pub fn new(space_id: u32, tuple: Vec<Value>) -> Self {
        Self { space_id, tuple }
    }
}

impl Request for Replace {
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
        write_kv_array(buf, keys::TUPLE, &self.tuple)?;
        Ok(())
    }
}
