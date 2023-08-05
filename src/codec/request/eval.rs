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
pub(crate) struct Eval<T> {
    pub expr: Cow<'static, str>,
    pub tuple: T,
}

impl<T: Tuple> Request for Eval<T> {
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
        write_kv_tuple(buf, keys::TUPLE, &self.tuple)?;
        Ok(())
    }
}

impl<T> Eval<T> {
    // TODO: introduce some convenient way to pass arguments
    pub fn new(expr: impl Into<Cow<'static, str>>, args: T) -> Self {
        Self {
            expr: expr.into(),
            tuple: args,
        }
    }
}
