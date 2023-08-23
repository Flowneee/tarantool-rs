use std::{
    fmt::{self, Debug},
    sync::Arc,
};

use anyhow::Context;
use rmpv::Value;
use serde::{de::DeserializeOwned, Deserialize};

use super::{Index, IndexMetadata, OwnedIndex, SchemaEntityKey, SystemSpacesId, PRIMARY_INDEX_ID};
use crate::{
    client::ExecutorExt, tuple::Tuple, utils::UniqueIdNameMap, Error, Executor, IteratorType,
    Result, Transaction,
};

/// Space metadata with its indices metadata from [system views](https://www.tarantool.io/en/doc/latest/reference/reference_lua/box_space/system_views/).
#[derive(Clone, Deserialize)]
pub struct SpaceMetadata {
    pub(super) id: u32,
    owner_id: u32,
    name: String,
    _engine: String, // TODO: enum
    _fields_count: u32,
    _flags: Value,       // TODO: parse flags
    _format: Vec<Value>, // TODO: parse format or remove it entirely
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
            .finish()
    }
}

impl SpaceMetadata {
    /// Load metadata of single space by key.
    ///
    /// Can be loaded by index (if passed unsigned integer) or name (if passed `&str`).
    async fn load(conn: impl ExecutorExt, key: SchemaEntityKey) -> Result<Option<Self>> {
        Ok(conn
            .select(
                SystemSpacesId::VSpace as u32,
                key.space_index_id(),
                None,
                None,
                None,
                (key.into_value(),),
            )
            .await?
            .into_iter()
            .next())
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

    // /// Returns map of idices in this space.
    // pub fn indices(&self) -> &UniqueIdNameMap<IndexMetadata> {
    //     &self.indices
    // }
}

/// Tarantool space.
///
/// This is a wrapper around [`Executor`], which allow to make space-related requests
/// on specific space. All requests over index uses primary index.
pub struct Space<E> {
    executor: E,
    metadata: Arc<SpaceMetadata>,
    primary_index_metadata: Arc<IndexMetadata>,
    indices_metadata: Arc<UniqueIdNameMap<IndexMetadata>>,
}

impl<E: Clone> Clone for Space<E> {
    fn clone(&self) -> Self {
        Self {
            executor: self.executor.clone(),
            metadata: self.metadata.clone(),
            primary_index_metadata: self.primary_index_metadata.clone(),
            indices_metadata: self.indices_metadata.clone(),
        }
    }
}

impl<E> Space<E> {
    fn get_index(&self, key: impl Into<SchemaEntityKey>) -> Option<&Arc<IndexMetadata>> {
        match key.into() {
            SchemaEntityKey::Id(x) => self.indices_metadata.get_by_id(x),
            SchemaEntityKey::Name(x) => self.indices_metadata.get_by_name(&x),
        }
    }

    pub fn executor(&self) -> &E {
        &self.executor
    }

    pub fn metadata(&self) -> &SpaceMetadata {
        &self.metadata
    }

    pub fn into_executor(self) -> E {
        self.executor
    }

    pub fn primary_index(&self) -> Index<&E> {
        Index::new(&self.executor, &self.primary_index_metadata, &self.metadata)
    }

    pub fn index(&self, key: impl Into<SchemaEntityKey>) -> Option<Index<&E>> {
        self.get_index(key)
            .map(|index| Index::new(&self.executor, index, &self.metadata))
    }
}

impl<E: Clone> Space<E> {
    pub fn owned_primary_index(&self) -> OwnedIndex<E> {
        OwnedIndex::new(
            self.executor.clone(),
            self.primary_index_metadata.clone(),
            self.metadata.clone(),
        )
    }

    pub fn owned_index(&self, key: impl Into<SchemaEntityKey>) -> Option<OwnedIndex<E>> {
        self.get_index(key).map(|index| {
            OwnedIndex::new(self.executor.clone(), index.clone(), self.metadata.clone())
        })
    }
}

impl<E: Executor> Space<E> {
    /// Load metadata of single space by its key.
    ///
    /// Can be called with space's index (if passed unsigned integer) or name (if passed `&str`).
    pub(crate) async fn load(executor: E, key: SchemaEntityKey) -> Result<Option<Self>> {
        let Some(space_metadata) = SpaceMetadata::load(&executor, key).await? else {
            return Ok(None);
        };

        let indices = IndexMetadata::load_by_space_id(&executor, space_metadata.id)
            .await
            .and_then(|x| {
                UniqueIdNameMap::try_from_iter(x)
                    .context("Duplicate indices in space")
                    .map_err(Error::Other)
            })?;
        let Some(primary_index) = indices.get_by_id(PRIMARY_INDEX_ID).cloned() else {
            return Err(Error::SpaceMissingPrimaryIndex);
        };

        Ok(Some(Self {
            executor,
            metadata: space_metadata.into(),
            primary_index_metadata: primary_index,
            indices_metadata: indices.into(),
        }))
    }

    /// Iterator over indices in this space.
    pub fn indices(&self) -> impl Iterator<Item = Index<&E>> {
        self.indices_metadata
            .iter()
            .map(|index| Index::new(&self.executor, index, &self.metadata))
    }

    /// Call `select` with primary index on current space.
    ///
    /// For details see [`ExecutorExt::select`].
    pub async fn select<T, A>(
        &self,
        limit: Option<u32>,
        offset: Option<u32>,
        iterator: Option<IteratorType>,
        keys: A,
    ) -> Result<Vec<T>>
    where
        T: DeserializeOwned,
        A: Tuple + Send,
    {
        self.executor
            .select(
                self.metadata.id,
                PRIMARY_INDEX_ID,
                limit,
                offset,
                iterator,
                keys,
            )
            .await
    }

    /// Call `insert` on current space.
    ///
    /// For details see [`ExecutorExt::insert`].
    // TODO: decode response
    pub async fn insert<T>(&self, tuple: T) -> Result<()>
    where
        T: Tuple + Send,
    {
        self.executor.insert(self.metadata.id, tuple).await
    }

    /// Call `update` with primary index on current space.
    ///
    /// For details see [`ExecutorExt::update`].
    // TODO: decode response
    pub async fn update<K, O>(&self, keys: K, ops: O) -> Result<()>
    where
        K: Tuple + Send,
        O: Tuple + Send,
    {
        self.executor
            .update(self.metadata.id, PRIMARY_INDEX_ID, keys, ops)
            .await
    }

    /// Call `upsert` on current space.
    ///
    /// For details see [`ExecutorExt::upsert`].
    // TODO: decode response
    pub async fn upsert<T, O>(&self, tuple: T, ops: O) -> Result<()>
    where
        T: Tuple + Send,
        O: Tuple + Send,
    {
        self.executor.upsert(self.metadata.id, tuple, ops).await
    }

    /// Call `replace` on current space.
    ///
    /// For details see [`ExecutorExt::replace`].
    // TODO: decode response
    pub async fn replace<T>(&self, tuple: T) -> Result<()>
    where
        T: Tuple + Send,
    {
        self.executor.replace(self.metadata.id, tuple).await
    }

    /// Call `delete` with primary index on current space.
    ///
    /// For details see [`ExecutorExt::delete`].
    // TODO: decode response
    pub async fn delete<T>(&self, keys: T) -> Result<()>
    where
        T: Tuple + Send,
    {
        self.executor
            .delete(self.metadata.id, PRIMARY_INDEX_ID, keys)
            .await
    }
}

impl Space<Transaction> {
    /// Commit inner tranasction.
    pub async fn commit(self) -> Result<()> {
        self.executor.commit().await
    }

    /// Rollback inner tranasction.
    pub async fn rollback(self) -> Result<()> {
        self.executor.rollback().await
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
