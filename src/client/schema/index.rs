use std::fmt;

use rmpv::Value;
use serde::Deserialize;

use super::SystemSpacesId;
use crate::{client::ConnectionLike, utils::UniqueIdName, Error};

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
    pub async fn load_by_space_id(
        conn: impl ConnectionLike,
        space_id: u32,
    ) -> Result<Vec<Self>, Error> {
        conn.select(
            SystemSpacesId::VIndex as u32,
            0,
            None,
            None,
            None,
            vec![space_id.into()],
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
