use std::fmt::{self, Debug};

use async_trait::async_trait;
use rmpv::Value;
use serde::{de::DeserializeOwned, Deserialize};

use super::{IndexMetadata, SystemSpacesId};
use crate::{
    client::ConnectionLike,
    codec::request::{Insert, Select},
    utils::UniqueIdNameMap,
    Error, Executor, IteratorType, Result,
};

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
    pub async fn load_by_id(conn: impl ConnectionLike, id: u32) -> Result<Option<Self>> {
        // 0 - primary id index
        Self::load(conn, 0, id).await
    }

    /// Load metadata of single space by its name.
    pub async fn load_by_name(conn: impl ConnectionLike, name: &str) -> Result<Option<Self>> {
        // 2 - index on 'name' field
        Self::load(conn, 2, name).await
    }

    /// Load metadata of single space by key.
    async fn load(
        conn: impl ConnectionLike,
        index_id: u32,
        key: impl Into<Value>,
    ) -> Result<Option<Self>> {
        let Some(mut this): Option<Self> = conn
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
            .next() else {
                return Ok(None)
            };
        this.load_indices(conn).await?;
        Ok(Some(this))
    }

    /// Load indices metadata into current space metadata.
    async fn load_indices(&mut self, conn: impl ConnectionLike) -> Result<()> {
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

    /// Returns map of idices in this space.
    pub fn indices(&self) -> &UniqueIdNameMap<IndexMetadata> {
        &self.indices
    }
}

pub struct Space<E> {
    executor: E,
    metadata: SpaceMetadata,
}

impl<E: Clone> Clone for Space<E> {
    fn clone(&self) -> Self {
        Self {
            executor: self.executor.clone(),
            metadata: self.metadata.clone(),
        }
    }
}

impl<E> Space<E> {
    pub fn new(executor: E, metadata: SpaceMetadata) -> Self {
        Self { executor, metadata }
    }

    pub fn executor(&self) -> &E {
        &self.executor
    }

    pub fn metadata(&self) -> &SpaceMetadata {
        &self.metadata
    }
}

impl<E: Executor> Space<E> {
    /// Load metadata of single space by its id.
    pub async fn load_by_id(executor: E, id: u32) -> Result<Option<Self>> {
        let Some(metadata) = SpaceMetadata::load_by_id(&executor, id).await? else {
            return Ok(None);
        };
        Ok(Some(Self::new(executor, metadata)))
    }

    /// Load metadata of single space by its name.
    pub async fn load_by_name(executor: E, name: &str) -> Result<Option<Self>> {
        let Some(metadata) = SpaceMetadata::load_by_name(&executor, name).await? else {
            return Ok(None);
        };
        Ok(Some(Self::new(executor, metadata)))
    }

    // TODO: docs
    pub async fn select<T>(
        &self,
        index_id: u32,
        limit: Option<u32>,
        offset: Option<u32>,
        iterator: Option<IteratorType>,
        keys: Vec<Value>,
    ) -> Result<Vec<T>>
    where
        T: DeserializeOwned,
    {
        self.executor
            .select(self.metadata.id, index_id, limit, offset, iterator, keys)
            .await
    }

    // TODO: docs
    // TODO: decode response
    pub async fn insert(&self, tuple: Vec<Value>) -> Result<()> {
        self.executor.insert(self.metadata.id, tuple).await
    }

    // TODO: structured tuple
    // TODO: decode response
    pub async fn update(&self, index_id: u32, keys: Vec<Value>, tuple: Vec<Value>) -> Result<()> {
        self.executor
            .update(self.metadata.id, index_id, keys, tuple)
            .await
    }

    // TODO: structured tuple
    // TODO: decode response
    // TODO: maybe set index base to 1 always?
    pub async fn upsert(&self, ops: Vec<Value>, tuple: Vec<Value>) -> Result<()> {
        self.executor.upsert(self.metadata.id, ops, tuple).await
    }

    // TODO: structured tuple
    // TODO: decode response
    pub async fn replace(&self, keys: Vec<Value>) -> Result<()> {
        self.executor.replace(self.metadata.id, keys).await
    }

    // TODO: structured tuple
    // TODO: decode response
    pub async fn delete(&self, index_id: u32, keys: Vec<Value>) -> Result<()> {
        self.executor.delete(self.metadata.id, index_id, keys).await
    }
}

#[async_trait]
impl<E: Executor> Executor for Space<E> {
    async fn send_encoded_request(
        &self,
        request: crate::codec::request::EncodedRequest,
    ) -> crate::Result<Value> {
        self.executor.send_encoded_request(request).await
    }

    fn stream(&self) -> crate::Stream {
        self.executor.stream()
    }

    fn transaction_builder(&self) -> crate::TransactionBuilder {
        self.executor.transaction_builder()
    }

    async fn transaction(&self) -> crate::Result<crate::Transaction> {
        self.executor.transaction().await
    }
}

impl<E: Debug> fmt::Debug for Space<E> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("SpaceMetadata")
            .field("executor", &self.executor)
            .field("metadata", &self.metadata)
            .finish()
    }
}
