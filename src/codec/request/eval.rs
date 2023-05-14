use std::{borrow::Cow, io::Write};

use rmpv::Value;

use crate::{
    codec::{
        consts::{keys, RequestType},
        utils::{write_kv_array, write_kv_str},
    },
    errors::EncodingError,
};

use super::RequestBody;

#[derive(Clone, Debug)]
pub(crate) struct Eval {
    pub expr: Cow<'static, str>,
    pub tuple: Vec<Value>,
}

impl RequestBody for Eval {
    fn request_type() -> RequestType
    where
        Self: Sized,
    {
        RequestType::Eval
    }

    // NOTE: `&mut buf: mut` is required since I don't get why compiler complain
    fn encode(&self, mut buf: &mut dyn Write) -> Result<(), EncodingError> {
        rmp::encode::write_map_len(&mut buf, 2)?;
        write_kv_str(buf, keys::EXPR, self.expr.as_ref())?;
        write_kv_array(buf, keys::TUPLE, &self.tuple)?;
        Ok(())
    }
}

impl Eval {
    // TODO: introduce some convenient way to pass arguments
    pub fn new(expr: impl Into<Cow<'static, str>>, args: Vec<Value>) -> Self {
        Self {
            expr: expr.into(),
            tuple: args,
        }
    }
}
