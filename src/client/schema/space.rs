use std::fmt;

use rmpv::Value;
use serde::Deserialize;

use super::{IndexMetadata, SystemSpacesId};
use crate::{client::ConnectionLike, utils::UniqueIdNameMap, Error};

/// Space metadata from with its indices metadata from [system views](https://www.tarantool.io/en/doc/latest/reference/reference_lua/box_space/system_views/).
#[derive(Clone, Deserialize)]
pub struct SpaceMetadata {
    id: u32,
    owner_id: u32,
    name: String,
    _engine: String, // TODO: enum
    _fields_count: u32,
    _flags: Value,       // TODO: parse flags
    _format: Vec<Value>, // TODO: parse format or remove it entirely
    // TODO: maybe implement hash directly on IndexMetadata and store in set
    // TODO: maybe vec or btreemap would be faster
    #[serde(skip)]
    indices: UniqueIdNameMap<IndexMetadata>,
}

impl fmt::Debug for SpaceMetadata {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("SpaceMetadata")
            .field("id", &self.id)
            .field("owner_id", &self.owner_id)
            .field("name", &self.name)
            .field("engine", &self._engine)
            // TODO: uncomment when fields implemented
            // .field("_flags", &self._flags)
            // .field("_format", &self._format)
            .field("indices", &self.indices)
            .finish()
    }
}

impl SpaceMetadata {
    /// Load metadata of single space by its id.
    pub async fn load_by_id(conn: impl ConnectionLike, id: u32) -> Result<Self, Error> {
        // 0 - primary id index
        Self::load(conn, 0, id).await
    }

    /// Load metadata of single space by its name.
    pub async fn load_by_name(conn: impl ConnectionLike, name: &str) -> Result<Self, Error> {
        // 2 - index on 'name' field
        Self::load(conn, 2, name).await
    }

    /// Load metadata of single space by key.
    async fn load(
        conn: impl ConnectionLike,
        index_id: u32,
        key: impl Into<Value>,
    ) -> Result<Self, Error> {
        let mut this: Self = conn
            .select(
                SystemSpacesId::VSpace as u32,
                index_id,
                None,
                None,
                None,
                vec![key.into()],
            )
            .await?
            .into_iter()
            .next()
            .ok_or(Error::SpaceNotFound)?;
        this.load_indices(conn).await?;
        Ok(this)
    }

    /// Load indices metadata into current space metadata.
    async fn load_indices(&mut self, conn: impl ConnectionLike) -> Result<(), Error> {
        self.indices = IndexMetadata::load_by_space_id(conn, self.id)
            .await
            .and_then(|x| {
                UniqueIdNameMap::try_from_iter(x).map_err(|err| {
                    Error::MetadataLoad(err.context("Failed to load indices metadata"))
                })
            })?;
        Ok(())
    }

    /// Returns the id of this space.
    pub fn id(&self) -> u32 {
        self.id
    }

    /// Returns user id of ther owner of this space.
    pub fn owner_id(&self) -> u32 {
        self.owner_id
    }

    /// Returns a name of this space.
    pub fn name(&self) -> &str {
        self.name.as_ref()
    }
}
