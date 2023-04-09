// TODO: unify with eval.rs

use std::io::Write;

use rmpv::Value;

use crate::codec::{
    consts::{keys, RequestType},
    utils::{write_kv_array, write_kv_u32},
};

use super::RequestBody;

#[derive(Clone, Debug)]
pub(crate) struct Insert {
    pub space_id: u32,
    pub tuple: Vec<Value>,
}

impl Insert {
    pub(crate) fn new(space_id: u32, tuple: Vec<Value>) -> Self {
        Self { space_id, tuple }
    }
}

impl RequestBody for Insert {
    fn request_type() -> RequestType
    where
        Self: Sized,
    {
        RequestType::Insert
    }

    // NOTE: `&mut buf: mut` is required since I don't get why compiler complain
    fn encode(&self, mut buf: &mut dyn Write) -> Result<(), anyhow::Error> {
        rmp::encode::write_map_len(&mut buf, 2)?;
        write_kv_u32(buf, keys::SPACE_ID, self.space_id)?;
        write_kv_array(buf, keys::TUPLE, &self.tuple)?;
        Ok(())
    }
}
