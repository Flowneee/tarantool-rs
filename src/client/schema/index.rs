use std::{borrow::Borrow, fmt, sync::Arc};

use rmpv::Value;
use serde::{de::DeserializeOwned, Deserialize};

use super::{SpaceMetadata, SystemSpacesId, PRIMARY_INDEX_ID};
use crate::{
    client::ExecutorExt, tuple::Tuple, utils::UniqueIdName, Executor, IteratorType, Result,
    Transaction,
};

/// Index metadata from [system view](https://www.tarantool.io/en/doc/latest/reference/reference_lua/box_space/system_views/).
#[derive(Clone, Deserialize)]
pub struct IndexMetadata {
    space_id: u32,
    index_id: u32,
    pub(super) name: String,
    #[serde(rename = "type")]
    type_: String, // TODO: enum
    _opts: Value,       // TODO: parse
    _parts: Vec<Value>, // TODO: parse
}

impl fmt::Debug for IndexMetadata {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("IndexMetadata")
            .field("space_id", &self.space_id)
            .field("index_id", &self.index_id)
            .field("name", &self.name)
            .field("type_", &self.type_)
            // TODO: uncomment when fields implemented
            // .field("_opts", &self._opts)
            // .field("_parts", &self._parts)
            .finish()
    }
}

impl IndexMetadata {
    // TODO: replace space_id type from u32 to something generic
    /// Load list of indices of single space.
    pub async fn load_by_space_id(conn: impl ExecutorExt, space_id: u32) -> Result<Vec<Self>> {
        conn.select(
            SystemSpacesId::VIndex as u32,
            0,
            None,
            None,
            None,
            (space_id,),
        )
        .await
    }

    /// Returns the id space to which this index belongs.
    pub fn space_id(&self) -> u32 {
        self.space_id
    }

    /// Returns the id of this index in space.
    pub fn id(&self) -> u32 {
        self.index_id
    }

    /// Returns whether this index is primary or not.
    pub fn is_primary(&self) -> bool {
        self.index_id == PRIMARY_INDEX_ID
    }

    /// Returns a name of this index.
    pub fn name(&self) -> &str {
        self.name.as_ref()
    }

    /// Returns a type of this index.
    pub fn type_(&self) -> &str {
        self.type_.as_ref()
    }
}

impl UniqueIdName for IndexMetadata {
    fn id(&self) -> &u32 {
        &self.index_id
    }

    fn name(&self) -> &str {
        &self.name
    }
}

/// Tarantool index of specific space.
///
/// This is a wrapper around [`Executor`], which allow to make index-related requests
/// on specific index.
pub struct GenericIndex<E, M, S> {
    executor: E,
    metadata: M,
    space_metadata: S,
}

/// Referenced index type, which rely on `Space` object.
pub type Index<'a, E> = GenericIndex<E, &'a IndexMetadata, &'a SpaceMetadata>;

/// Owned index type, which can exists without related `Space` object.
pub type OwnedIndex<E> = GenericIndex<E, Arc<IndexMetadata>, Arc<SpaceMetadata>>;

impl<E, M, S> Clone for GenericIndex<E, M, S>
where
    E: Clone,
    M: Clone,
    S: Clone,
{
    fn clone(&self) -> Self {
        Self {
            executor: self.executor.clone(),
            metadata: self.metadata.clone(),
            space_metadata: self.space_metadata.clone(),
        }
    }
}

impl<E, M, S> GenericIndex<E, M, S> {
    pub(super) fn new(executor: E, metadata: M, space_metadata: S) -> Self {
        Self {
            executor,
            metadata,
            space_metadata,
        }
    }

    pub fn into_executor(self) -> E {
        self.executor
    }
}

impl<E, M, S> GenericIndex<E, M, S>
where
    M: Borrow<IndexMetadata>,
    S: Borrow<SpaceMetadata>,
{
    pub fn executor(&self) -> &E {
        &self.executor
    }

    pub fn metadata(&self) -> &IndexMetadata {
        self.metadata.borrow()
    }

    pub fn space_metadata(&self) -> &SpaceMetadata {
        self.space_metadata.borrow()
    }
}

impl<E, M, S> GenericIndex<E, M, S>
where
    E: Executor,
    M: Borrow<IndexMetadata>,
    S: Borrow<SpaceMetadata>,
{
    /// Call `select` on current index.
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
                self.space_metadata.borrow().id,
                self.metadata.borrow().index_id,
                limit,
                offset,
                iterator,
                keys,
            )
            .await
    }

    /// Call `update` on current index.
    ///
    /// For details see [`ExecutorExt::update`].
    // TODO: decode response
    pub async fn update<K, O>(&self, keys: K, ops: O) -> Result<()>
    where
        K: Tuple + Send,
        O: Tuple + Send,
    {
        self.executor
            .update(
                self.space_metadata.borrow().id,
                self.metadata.borrow().index_id,
                keys,
                ops,
            )
            .await
    }

    /// Call `delete` on current index.
    ///
    /// For details see [`ExecutorExt::delete`].
    // TODO: decode response
    pub async fn delete<T>(&self, keys: T) -> Result<()>
    where
        T: Tuple + Send,
    {
        self.executor
            .delete(
                self.space_metadata.borrow().id,
                self.metadata.borrow().index_id,
                keys,
            )
            .await
    }
}

impl OwnedIndex<Transaction> {
    /// Commit inner tranasction.
    pub async fn commit(self) -> Result<()> {
        self.executor.commit().await
    }

    /// Rollback inner tranasction.
    pub async fn rollback(self) -> Result<()> {
        self.executor.rollback().await
    }
}
