// TODO: unify with eval.rs

use std::{borrow::Cow, io::Write};

use rmpv::Value;

use crate::{
    codec::{
        consts::{keys, RequestType},
        utils::{write_kv_array, write_kv_str},
    },
    errors::EncodingError,
};

use super::Request;

#[derive(Clone, Debug)]
pub(crate) struct Call {
    pub function_name: Cow<'static, str>,
    pub tuple: Vec<Value>,
}

impl Request for Call {
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
        write_kv_array(buf, keys::TUPLE, &self.tuple)?;
        Ok(())
    }
}

impl Call {
    // TODO: introduce some convenient way to pass arguments
    pub(crate) fn new(function_name: impl Into<Cow<'static, str>>, args: Vec<Value>) -> Self {
        Self {
            function_name: function_name.into(),
            tuple: args,
        }
    }
}
