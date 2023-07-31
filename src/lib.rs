// TODO:
//
// Features:
//
// * [ ] connections pooling
// * [ ] chunked responses (tt feature)
// * [ ] streaming responses for select
// * [ ] background schema fetching, reloading and invalidating
// * [ ] triggers on connection events (connect/disconnect/schema reloading)
// * [ ] SQL
// * [ ] graceful shutdown protocol
//
// Other
//
// * [ ] check or remove all unsafes, unwrap, panic, expect
// * [ ] tests
// * [ ] bump version to 0.1.0
// * [ ] remove unused dependencies

pub use rmpv::Value;

#[doc(inline)]
pub use self::{
    builder::{ConnectionBuilder, ReconnectInterval},
    client::*,
    codec::consts::{IteratorType, TransactionIsolationLevel},
    errors::Error,
};

pub mod errors;
pub mod utils;

mod builder;
mod client;
mod codec;
mod transport;

/// Alias for [`std::result::Result<T, crate::Error>`].
pub type Result<T> = std::result::Result<T, Error>;
