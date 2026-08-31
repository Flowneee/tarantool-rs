use std::io::Write;

use crate::errors::EncodingError;

pub trait TupleElement {
    /// # Errors
    ///
    /// Returns an error if the value cannot be encoded as `MessagePack`
    /// or written into `buf`.
    fn encode_into_writer<W: Write>(&self, buf: W) -> Result<(), EncodingError>;
}

impl<T: serde::Serialize> TupleElement for T {
    fn encode_into_writer<W: Write>(&self, mut buf: W) -> Result<(), EncodingError> {
        rmp_serde::encode::write(&mut buf, self)?;
        Ok(())
    }
}

/// Trait, describing type, which can be encoded into
/// `MessagePack` tuple.
///
/// It is mostly used to pass arguments to Tarantool requests,
/// like passing arguments for `CALL`.
pub trait Tuple {
    /// # Errors
    ///
    /// Returns an error if the value cannot be encoded as `MessagePack`
    /// or written into `buf`.
    fn encode_into_writer<W: Write>(&self, buf: W) -> Result<(), EncodingError>;
}

impl<T: TupleElement> Tuple for Vec<T> {
    // A tuple longer than u32::MAX elements is not representable in IPROTO anyway.
    #[allow(clippy::cast_possible_truncation)]
    fn encode_into_writer<W: Write>(&self, mut buf: W) -> Result<(), EncodingError> {
        rmp::encode::write_array_len(&mut buf, self.len() as u32)?;
        for x in self {
            x.encode_into_writer(&mut buf)?;
        }
        Ok(())
    }
}

impl<T: TupleElement> Tuple for &[T] {
    // A tuple longer than u32::MAX elements is not representable in IPROTO anyway.
    #[allow(clippy::cast_possible_truncation)]
    fn encode_into_writer<W: Write>(&self, mut buf: W) -> Result<(), EncodingError> {
        rmp::encode::write_array_len(&mut buf, self.len() as u32)?;
        for x in *self {
            x.encode_into_writer(&mut buf)?;
        }
        Ok(())
    }
}

impl Tuple for () {
    fn encode_into_writer<W: Write>(&self, mut buf: W) -> Result<(), EncodingError> {
        rmp::encode::write_array_len(&mut buf, 0)?;
        Ok(())
    }
}

impl<T: Tuple> Tuple for &T {
    fn encode_into_writer<W: Write>(&self, buf: W) -> Result<(), EncodingError> {
        (*self).encode_into_writer(buf)
    }
}

// `= self` idea is from https://stackoverflow.com/a/56700760/5033855
macro_rules! impl_tuple_for_tuple {
    ( $param:tt ) => {
        impl<$param : $crate::TupleElement> Tuple for ($param,) {
            fn encode_into_writer<W: Write>(&self, mut buf: W) -> Result<(), EncodingError> {
                rmp::encode::write_array_len(&mut buf, 1)?;
                self.0.encode_into_writer(&mut buf)?;
                Ok(())
            }
        }
    };
    ( $param:tt, $($params:tt),* ) => {
        impl<$param : $crate::TupleElement , $($params : $crate::TupleElement,)*> Tuple for ($param, $($params,)*) {
            #[allow(non_snake_case)]
            // The element count is a compile-time constant far below u32::MAX.
            #[allow(clippy::cast_possible_truncation)]
            fn encode_into_writer<W: Write>(&self, mut buf: W) -> Result<(), EncodingError> {
                rmp::encode::write_array_len(&mut buf, count_tts!($param $($params)+) as u32)?;

                let ($param, $($params,)+) = self;

                $param.encode_into_writer(&mut buf)?;

                $(
                    $params.encode_into_writer(&mut buf)?;
                )+

                Ok(())
            }
        }

        impl_tuple_for_tuple! { $($params),* }
    };
}

// Counting macro from https://veykril.github.io/tlborm/decl-macros/building-blocks/counting.html
macro_rules! count_tts {
    ($($tts:tt)*) => {0usize $(+ replace_expr!($tts 1usize))*};
}

macro_rules! replace_expr {
    ($_t:tt $sub:expr_2021) => {
        $sub
    };
}

impl_tuple_for_tuple! {
    T32, T31, T30, T29, T28, T27, T26, T25, T24, T23,
    T22, T21, T20, T19, T18, T17, T16, T15, T14, T13,
    T12, T11, T10, T9, T8, T7, T6, T5, T4, T3, T2, T1
}
