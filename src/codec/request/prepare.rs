use std::io::Write;

use crate::{
    codec::{
        consts::{keys, RequestType},
        utils::{write_kv_str},
    },
    errors::EncodingError,
};

use super::Request;

#[derive(Clone, Debug)]
pub(crate) struct Prepare<'a> {
    sql_query: &'a str,
}

impl<'a> Request for Prepare<'a> {
    fn request_type() -> RequestType
    where
        Self: Sized,
    {
        RequestType::Prepare
    }

    // NOTE: `&mut buf: mut` is required since I don't get why compiler complain
    fn encode(&self, mut buf: &mut dyn Write) -> Result<(), EncodingError> {
        rmp::encode::write_map_len(&mut buf, 1)?;
        write_kv_str(buf, keys::SQL_TEXT, self.sql_query)?;
        Ok(())
    }
}

impl<'a> Prepare<'a> {
    pub(crate) fn new(sql_query: &'a str) -> Self {
        Self { sql_query }
    }
}
