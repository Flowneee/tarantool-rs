// TODO: unify with eval.rs

use std::{borrow::Cow, io::Write};

use rmpv::Value;

use crate::{
    codec::consts::{keys, IProtoType},
    ChannelError,
};

use super::IProtoRequestBody;

#[derive(Clone, Debug)]
pub struct IProtoCall {
    pub function_name: Cow<'static, str>,
    pub tuple: Vec<Value>,
}

impl IProtoRequestBody for IProtoCall {
    fn request_type() -> IProtoType
    where
        Self: Sized,
    {
        IProtoType::Call
    }

    // NOTE: `&mut buf: mut` is required since I don't get why compiler complain
    fn encode(&self, mut buf: &mut dyn Write) -> Result<(), ChannelError> {
        rmp::encode::write_map_len(&mut buf, 2)?;
        rmp::encode::write_pfix(&mut buf, keys::FUNCTION_NAME)?;
        rmp::encode::write_str(&mut buf, &self.function_name)?;
        rmp::encode::write_pfix(&mut buf, keys::TUPLE)?;
        // TODO: safe conversion from usize to u32
        rmp::encode::write_array_len(&mut buf, self.tuple.len() as u32)?;
        for x in self.tuple.iter() {
            rmpv::encode::write_value(&mut buf, x)?;
        }
        Ok(())
    }
}

impl IProtoCall {
    // TODO: introduce some convenient way to pass arguments
    pub fn new(function_name: impl Into<Cow<'static, str>>, args: Vec<Value>) -> Self {
        Self {
            function_name: function_name.into(),
            tuple: args,
        }
    }
}
