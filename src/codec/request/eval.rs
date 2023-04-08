use std::{borrow::Cow, io::Write};

use rmpv::Value;

use crate::{
    codec::consts::{keys, RequestType},
    TransportError,
};

use super::RequestBody;

#[derive(Clone, Debug)]
pub struct Eval {
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
    fn encode(&self, mut buf: &mut dyn Write) -> Result<(), TransportError> {
        rmp::encode::write_map_len(&mut buf, 2)?;
        rmp::encode::write_pfix(&mut buf, keys::EXPR)?;
        rmp::encode::write_str(&mut buf, &self.expr)?;
        rmp::encode::write_pfix(&mut buf, keys::TUPLE)?;
        // TODO: safe conversion from usize to u32
        rmp::encode::write_array_len(&mut buf, self.tuple.len() as u32)?;
        for x in self.tuple.iter() {
            rmpv::encode::write_value(&mut buf, x)?;
        }
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
