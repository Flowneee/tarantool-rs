use std::{borrow::Cow, io::Write};

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
pub(crate) struct Call<T> {
    pub function_name: Cow<'static, str>,
    pub tuple: T,
}

impl<T: Tuple> Request for Call<T> {
    fn request_type() -> RequestType
    where
        Self: Sized,
    {
        RequestType::Call
    }

    // NOTE: `&mut buf: mut` is required since I don't get why compiler complain
    fn encode(&self, mut buf: &mut dyn Write) -> Result<(), EncodingError> {
        rmp::encode::write_map_len(&mut buf, 2)?;
        write_kv_str(buf, keys::FUNCTION_NAME, self.function_name.as_ref())?;
        write_kv_tuple(buf, keys::TUPLE, &self.tuple)?;
        Ok(())
    }
}

impl<T> Call<T> {
    pub(crate) fn new(function_name: impl Into<Cow<'static, str>>, args: T) -> Self {
        Self {
            function_name: function_name.into(),
            tuple: args,
        }
    }
}
