use std::io::Write;

use crate::{
    codec::{
        consts::{keys, RequestType},
        utils::{write_kv_str, write_kv_tuple},
    },
    errors::EncodingError,
    tuple::Tuple,
};

use super::Request;

#[derive(Clone, Debug)]
pub(crate) struct Call<'a, T> {
    pub function_name: &'a str,
    pub tuple: T,
}

impl<'a, T: Tuple> Request for Call<'a, T> {
    fn request_type() -> RequestType
    where
        Self: Sized,
    {
        RequestType::Call
    }

    // NOTE: `&mut buf: mut` is required since I don't get why compiler complain
    fn encode(&self, mut buf: &mut dyn Write) -> Result<(), EncodingError> {
        rmp::encode::write_map_len(&mut buf, 2)?;
        write_kv_str(buf, keys::FUNCTION_NAME, self.function_name)?;
        write_kv_tuple(buf, keys::TUPLE, &self.tuple)?;
        Ok(())
    }
}

impl<'a, T> Call<'a, T> {
    pub(crate) fn new(function_name: &'a str, args: T) -> Self {
        Self {
            function_name,
            tuple: args,
        }
    }
}
