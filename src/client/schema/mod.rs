pub use self::{index::*, space::*};

use serde::{Deserialize, Serialize};

mod index;
mod space;

/// First possible id of user space.
///
/// For details see [`SystemSpaceId`].
pub const USER_SPACE_MIN_ID: u32 = 512;

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

pub struct SpacesMetadata {}
