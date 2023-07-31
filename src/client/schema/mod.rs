//! Schema (spaces and indices) helper types.

pub use self::{
    index::{GenericIndex, Index, IndexMetadata, OwnedIndex},
    space::{Space, SpaceMetadata},
};

use std::fmt;

use rmpv::Value;
use serde::{Deserialize, Serialize};

mod index;
mod space;

/// First possible id of user space.
///
/// For details see [`SystemSpacesId`].
pub const USER_SPACE_MIN_ID: u32 = 512;

/// Id of the primary index in space.
pub const PRIMARY_INDEX_ID: u32 = 0;

// TODO: docs on variants
/// Ids of system spaces and views.
///
/// According to Tarantool [sources](https://github.com/tarantool/tarantool/blob/00a9e59927399c91158aa2bf9698c4bfa6e11322/src/box/schema_def.h#L66)
/// this values are fixed and all have an id in reserved range `[256, 511]`.
// NOTE: if this values are changed for any reason, replace them with dynamic discovery of spaces and views.
#[derive(Clone, Copy, Debug, Deserialize, Serialize, PartialEq, Eq, Hash)]
#[repr(u32)]
pub enum SystemSpacesId {
    VSpace = 281,
    VIndex = 289,
}

/// Key of space or index.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum SchemaEntityKey {
    /// Schema entity symbolic name.
    Name(String),
    /// Internal id of entity.
    Id(u32),
}

impl fmt::Display for SchemaEntityKey {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SchemaEntityKey::Name(x) => write!(f, "name '{x}'"),
            SchemaEntityKey::Id(x) => write!(f, "id '{x}'"),
        }
    }
}

impl From<&str> for SchemaEntityKey {
    fn from(value: &str) -> Self {
        Self::Name(value.to_owned())
    }
}

impl From<u32> for SchemaEntityKey {
    fn from(value: u32) -> Self {
        Self::Id(value)
    }
}

impl From<SchemaEntityKey> for Value {
    fn from(val: SchemaEntityKey) -> Self {
        match val {
            SchemaEntityKey::Name(x) => x.into(),
            SchemaEntityKey::Id(x) => x.into(),
        }
    }
}

impl SchemaEntityKey {
    pub(crate) fn space_index_id(&self) -> u32 {
        match self {
            SchemaEntityKey::Name(_) => 2,
            SchemaEntityKey::Id(_) => 0,
        }
    }
}
