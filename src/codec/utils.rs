use std::io::Write;

use rmpv::Value;

use crate::errors::EncodingError;

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

pub fn write_kv_array(
    mut buf: &mut dyn Write,
    key: u8,
    value: &[Value],
) -> Result<(), EncodingError> {
    rmp::encode::write_pfix(&mut buf, key)?;
    // TODO: safe conversion from usize to u32
    rmp::encode::write_array_len(&mut buf, value.len() as u32)?;
    for x in value.iter() {
        rmpv::encode::write_value(&mut buf, x)?;
    }
    Ok(())
}
