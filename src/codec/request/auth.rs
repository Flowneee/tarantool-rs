// Docs: https://www.tarantool.io/en/doc/latest/dev_guide/internals/iproto/authentication/

use std::{cmp::min, io::Write};

use sha1::{Digest, Sha1};

use super::RequestBody;
use crate::{
    codec::{
        consts::{keys, RequestType},
        utils::write_kv_str,
    },
    errors::EncodingError,
};

#[derive(Clone, Debug)]
pub(crate) struct Auth<'a> {
    pub user_name: &'a str,
    pub scramble: Vec<u8>,
}

impl<'a> Auth<'a> {
    pub(crate) fn new(user: &'a str, password: Option<&'a str>, salt: &'a [u8]) -> Self {
        Self {
            user_name: user,
            scramble: prepare_scramble(password, salt),
        }
    }
}

impl<'a> RequestBody for Auth<'a> {
    fn request_type() -> RequestType
    where
        Self: Sized,
    {
        RequestType::Auth
    }

    // NOTE: `&mut buf: mut` is required since I don't get why compiler complain
    fn encode(&self, mut buf: &mut dyn Write) -> Result<(), EncodingError> {
        rmp::encode::write_map_len(&mut buf, 2)?;
        write_kv_str(&mut buf, keys::USER_NAME, self.user_name)?;
        rmp::encode::write_pfix(&mut buf, keys::TUPLE)?;
        rmp::encode::write_array_len(&mut buf, 2)?;
        rmp::encode::write_str(&mut buf, "chap-sha1")?;
        rmp::encode::write_bin(&mut buf, &self.scramble)?;
        Ok(())
    }
}

macro_rules! sha1 {
    ($($data:expr),+) => {
        {
            let mut hasher = Sha1::new();
            $( hasher.update($data); )+
            hasher.finalize().to_vec()
        }
    }
}

fn prepare_scramble(password: Option<&str>, salt: &[u8]) -> Vec<u8> {
    let password = password.unwrap_or("");
    let mut step_1 = sha1!(password.as_bytes());
    let step_2 = sha1!(&step_1);
    let step_3 = sha1!(&salt[0..min(salt.len(), 20)], &step_2);
    // xor(step_1, step_3)
    step_1.iter_mut().zip(step_3).for_each(|(l, r)| *l ^= r);
    step_1
}
