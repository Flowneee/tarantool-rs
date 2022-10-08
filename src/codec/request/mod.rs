use std::io::Write;

use crate::errors::ChannelError;

use super::consts::{keys, IProtoType};

pub trait IProtoRequestBody: 'static + Send {
    /// Return type of this request.
    fn request_type() -> IProtoType
    where
        Self: Sized;

    /// Encode body into MessagePack and write it to provided [`Write`].
    fn encode(&self, buf: &mut dyn Write) -> Result<(), ChannelError>;
}

// TODO: hide fields and export them via getters
pub struct IProtoRequest {
    pub request_type: IProtoType,
    pub sync: u32,
    pub schema_version: Option<u32>,
    pub stream_id: Option<u32>,
    pub body: Box<dyn IProtoRequestBody>,
}

impl IProtoRequest {
    pub fn new<Body: IProtoRequestBody>(sync: u32, body: Body, stream_id: Option<u32>) -> Self {
        Self {
            request_type: Body::request_type(),
            sync,
            schema_version: None,
            stream_id,
            body: Box::new(body),
        }
    }

    pub fn encode(&self, mut buf: impl Write) -> Result<(), ChannelError> {
        let map_len = 2
            + if self.schema_version.is_some() { 1 } else { 0 }
            + if self.stream_id.is_some() { 1 } else { 0 };
        rmp::encode::write_map_len(&mut buf, map_len)?;
        rmp::encode::write_pfix(&mut buf, keys::REQUEST_TYPE)?;
        rmp::encode::write_u8(&mut buf, self.request_type as u8)?;
        rmp::encode::write_pfix(&mut buf, keys::SYNC)?;
        rmp::encode::write_u32(&mut buf, self.sync)?;
        if let Some(x) = self.schema_version {
            rmp::encode::write_pfix(&mut buf, keys::SCHEMA_VERSION)?;
            rmp::encode::write_u32(&mut buf, x)?;
        }
        if let Some(x) = self.stream_id {
            rmp::encode::write_pfix(&mut buf, keys::STREAM_ID)?;
            rmp::encode::write_u32(&mut buf, x)?;
        }
        self.body.encode(&mut buf)?;
        Ok(())
    }
}

#[derive(Clone, Debug)]
pub struct IProtoPing {}

impl IProtoRequestBody for IProtoPing {
    fn request_type() -> IProtoType {
        IProtoType::Ping
    }

    // NOTE: `&mut buf: mut` is required since I don't get why compiler complain
    fn encode(&self, mut buf: &mut dyn Write) -> Result<(), ChannelError> {
        rmp::encode::write_map_len(&mut buf, 0)?;
        Ok(())
    }
}
