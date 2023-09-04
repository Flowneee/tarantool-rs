use std::io::Write;

use rmpv::ValueRef;

use crate::{errors::EncodingError, Tuple, TupleElement};

/// Key of index in operation.
///
/// Can be string, unsigned or signed number ([docs](https://www.tarantool.io/en/doc/latest/reference/reference_lua/box_space/update/#box-space-update)).
#[derive(Debug)]
pub struct DmoOperationFieldKey<'a>(ValueRef<'a>);

impl<'a> From<&'a str> for DmoOperationFieldKey<'a> {
    fn from(value: &'a str) -> Self {
        Self(value.into())
    }
}

impl<'a> From<u32> for DmoOperationFieldKey<'a> {
    fn from(value: u32) -> Self {
        Self(value.into())
    }
}

impl<'a> From<i32> for DmoOperationFieldKey<'a> {
    fn from(value: i32) -> Self {
        Self(value.into())
    }
}

// TODO: docs and doctests
/// Operation in `upsert` or `update` request.
#[derive(Debug)]
pub struct DmoOperation<'a> {
    // TODO: id support
    // TODO: negative id support
    operation: &'static str,
    field_name: ValueRef<'a>,
    args: Args<'a>,
}

impl<'a> DmoOperation<'a> {
    fn new(
        operation: &'static str,
        field_name: impl Into<DmoOperationFieldKey<'a>>,
        args: Args<'a>,
    ) -> Self {
        Self {
            operation,
            field_name: field_name.into().0,
            args,
        }
    }

    pub fn add(
        field_name: impl Into<DmoOperationFieldKey<'a>>,
        value: impl Into<ValueRef<'a>>,
    ) -> Self {
        Self::new(ops::ADD, field_name, Args::One(value.into()))
    }

    pub fn sub(
        field_name: impl Into<DmoOperationFieldKey<'a>>,
        value: impl Into<ValueRef<'a>>,
    ) -> Self {
        Self::new(ops::SUB, field_name, Args::One(value.into()))
    }

    pub fn and(
        field_name: impl Into<DmoOperationFieldKey<'a>>,
        value: impl Into<ValueRef<'a>>,
    ) -> Self {
        Self::new(ops::AND, field_name, Args::One(value.into()))
    }

    pub fn or(
        field_name: impl Into<DmoOperationFieldKey<'a>>,
        value: impl Into<ValueRef<'a>>,
    ) -> Self {
        Self::new(ops::OR, field_name, Args::One(value.into()))
    }

    pub fn xor(
        field_name: impl Into<DmoOperationFieldKey<'a>>,
        value: impl Into<ValueRef<'a>>,
    ) -> Self {
        Self::new(ops::XOR, field_name, Args::One(value.into()))
    }

    pub fn string_splice(
        field_name: impl Into<DmoOperationFieldKey<'a>>,
        from: usize,
        len: usize,
        value: &'a str,
    ) -> Self {
        Self::new(
            ops::STRING_SPLICE,
            field_name,
            Args::Three(from.into(), len.into(), value.into()),
        )
    }

    pub fn insert(
        field_name: impl Into<DmoOperationFieldKey<'a>>,
        value: impl Into<ValueRef<'a>>,
    ) -> Self {
        Self::new(ops::INSERT, field_name, Args::One(value.into()))
    }

    pub fn assign(
        field_name: impl Into<DmoOperationFieldKey<'a>>,
        value: impl Into<ValueRef<'a>>,
    ) -> Self {
        Self::new(ops::ASSIGN, field_name, Args::One(value.into()))
    }

    pub fn delete(field_name: impl Into<DmoOperationFieldKey<'a>>) -> Self {
        Self::new(ops::DEL, field_name, Args::None)
    }
}

impl<'a> TupleElement for DmoOperation<'a> {
    fn encode_into_writer<W: Write>(&self, mut buf: W) -> Result<(), EncodingError> {
        let arr_len = 2 + match self.args {
            Args::None => 0,
            Args::One(_) => 1,
            Args::Three(_, _, _) => 3,
        };
        rmp::encode::write_array_len(&mut buf, arr_len)?;
        rmp::encode::write_str(&mut buf, self.operation)?;
        rmpv::encode::write_value_ref(&mut buf, &self.field_name)?;
        match &self.args {
            Args::None => {}
            Args::One(x) => {
                rmpv::encode::write_value_ref(&mut buf, x)?;
            }
            Args::Three(x, y, z) => {
                rmpv::encode::write_value_ref(&mut buf, x)?;
                rmpv::encode::write_value_ref(&mut buf, y)?;
                rmpv::encode::write_value_ref(&mut buf, z)?;
            }
        }
        Ok(())
    }
}

/// Implementation for allow single operation to be used as argument for `update` and `upsert`.
impl<'a> Tuple for DmoOperation<'a> {
    fn encode_into_writer<W: Write>(&self, mut buf: W) -> Result<(), EncodingError> {
        rmp::encode::write_array_len(&mut buf, 1)?;
        TupleElement::encode_into_writer(self, &mut buf)?;
        Ok(())
    }
}

#[derive(Debug)]
enum Args<'a> {
    None,
    One(rmpv::ValueRef<'a>),
    Three(rmpv::ValueRef<'a>, rmpv::ValueRef<'a>, rmpv::ValueRef<'a>),
}

mod ops {
    pub(super) const ADD: &str = "+";
    pub(super) const SUB: &str = "-";
    pub(super) const AND: &str = "&";
    pub(super) const OR: &str = "|";
    pub(super) const XOR: &str = "^";
    pub(super) const STRING_SPLICE: &str = ":";
    pub(super) const INSERT: &str = "|";
    pub(super) const DEL: &str = "#";
    pub(super) const ASSIGN: &str = "=";
}
