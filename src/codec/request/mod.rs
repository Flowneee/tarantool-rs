use anyhow::Context;
use bytes::{BufMut, Bytes, BytesMut};

use crate::Error;

pub(crate) use self::{
    auth::Auth, begin::Begin, call::Call, commit::Commit, delete::Delete, eval::Eval, id::Id,
    insert::Insert, ping::Ping, replace::Replace, rollback::Rollback, select::Select,
    update::Update, upsert::Upsert,
};

use std::io::Write;

use super::consts::{keys, RequestType};

mod auth;
mod begin;
mod call;
mod commit;
mod delete;
mod eval;
mod id;
mod insert;
mod ping;
mod replace;
mod rollback;
mod select;
mod update;
mod upsert;

pub const PROTOCOL_VERSION: u8 = 3;

const DEFAULT_ENCODE_BUFFER_SIZE: usize = 128;

// TODO: docs
pub trait RequestBody: 'static + Send {
    /// Return type of this request.
    fn request_type() -> RequestType
    where
        Self: Sized;

    /// Encode body into MessagePack and write it to provided [`Write`].
    ///
    /// Currently all implementation in this crate uses [`rmp::encode`],
    /// which throw errors only if used `Write` throw an error. And since internally
    /// crate use [`bytes::BufMut::writer`], this methods shouldn't throw error in
    /// normal case, so we don't care about actual type. If necessary, it is possible
    /// to downcast error to specific type.
    fn encode(&self, buf: &mut dyn Write) -> Result<(), anyhow::Error>;
}

pub(crate) struct Request {
    pub request_type: RequestType,
    pub sync: u32,
    pub schema_version: Option<u32>,
    pub stream_id: Option<u32>,
    pub encoded_body: Bytes,
}

impl Request {
    pub fn new<Body: RequestBody>(
        sync: u32,
        body: Body,
        stream_id: Option<u32>,
    ) -> Result<Self, Error> {
        let mut buf = BytesMut::with_capacity(DEFAULT_ENCODE_BUFFER_SIZE).writer();
        body.encode(&mut buf).map_err(Error::RequestBodyEncode)?;
        Ok(Self {
            request_type: Body::request_type(),
            sync,
            schema_version: None,
            stream_id,
            encoded_body: buf.into_inner().freeze(),
        })
    }

    pub fn encode(&self, mut buf: impl Write) -> Result<(), anyhow::Error> {
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
        buf.write_all(&self.encoded_body)
            .context("Failed to write encoded body to buffer")
    }
}
