use std::{cmp::min, io::Write};

use sha1::{Digest, Sha1};

use super::IProtoRequestBody;
use crate::{
    codec::consts::{keys, IProtoType},
    ChannelError,
};

#[derive(Clone, Debug)]
pub struct IProtoAuth {
    pub user_name: String,
    pub scramble: Vec<u8>,
}

impl IProtoAuth {
    pub fn new(user: String, password: Option<String>, salt: Vec<u8>) -> Self {
        Self {
            user_name: user,
            scramble: prepare_scramble(password, salt),
        }
    }
}

impl IProtoRequestBody for IProtoAuth {
    fn request_type() -> IProtoType
    where
        Self: Sized,
    {
        IProtoType::Auth
    }

    // NOTE: `&mut buf: mut` is required since I don't get why compiler complain
    fn encode(&self, mut buf: &mut dyn Write) -> Result<(), ChannelError> {
        rmp::encode::write_map_len(&mut buf, 2)?;
        rmp::encode::write_pfix(&mut buf, keys::USER_NAME)?;
        rmp::encode::write_str(&mut buf, &self.user_name)?;
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

fn prepare_scramble(password: Option<String>, salt: Vec<u8>) -> Vec<u8> {
    let password = password.as_deref().unwrap_or("");
    let mut step_1 = sha1!(password.as_bytes());
    let step_2 = sha1!(&step_1);
    let step_3 = sha1!(&salt[0..min(salt.len(), 20)], &step_2);
    // xor(step_1, step_3)
    step_1.iter_mut().zip(step_3).for_each(|(l, r)| *l ^= r);
    step_1
}
