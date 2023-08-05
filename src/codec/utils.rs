use std::io::Write;

use crate::{errors::EncodingError, tuple::Tuple};

pub fn write_kv_str(mut buf: &mut dyn Write, key: u8, value: &str) -> Result<(), EncodingError> {
    rmp::encode::write_pfix(&mut buf, key)?;
    rmp::encode::write_str(&mut buf, value)?;
    Ok(())
}

pub fn write_kv_u32(mut buf: &mut dyn Write, key: u8, value: u32) -> Result<(), EncodingError> {
    rmp::encode::write_pfix(&mut buf, key)?;
    rmp::encode::write_u32(&mut buf, value)?;
    Ok(())
}

pub fn write_kv_tuple<T: Tuple>(
    mut buf: &mut dyn Write,
    key: u8,
    tuple: T,
) -> Result<(), EncodingError> {
    rmp::encode::write_pfix(&mut buf, key)?;
    T::encode_into_writer(&tuple, buf)?;
    Ok(())
}
