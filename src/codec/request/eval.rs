use std::{borrow::Cow, io::Write};

use rmpv::Value;

use crate::{
    codec::consts::{keys, IProtoType},
    ChannelError,
};

use super::IProtoRequestBody;

#[derive(Clone, Debug)]
pub struct IProtoEval {
    pub expr: Cow<'static, str>,
    pub tuple: Vec<Value>,
}

impl IProtoRequestBody for IProtoEval {
    fn request_type() -> IProtoType
    where
        Self: Sized,
    {
        IProtoType::Eval
    }

    // NOTE: `&mut buf: mut` is required since I don't get why compiler complain
    fn encode(&self, mut buf: &mut dyn Write) -> Result<(), ChannelError> {
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

impl IProtoEval {
    // TODO: introduce some convenient way to pass arguments
    pub fn new(expr: impl Into<Cow<'static, str>>, args: Vec<Value>) -> Self {
        Self {
            expr: expr.into(),
            tuple: args,
        }
    }
}
