// TODO:
//
// Features:
//
// * [ ] Wrappers around schema types, allowing to call methods directly on space or index
// * [ ] connections pooling
// * [ ] chunked responses (tt feature)
// * [ ] streaming responses for select
// * [ ] background schema fetching, reloading and invalidating
// * [ ] triggers on connection events (connect/disconnect/schema reloading)
//
// Other
//
// * [ ] check or remove all unsafes, unwrap, panic, expect
// * [ ] tests
// * [ ] bump version to 0.1.0
// * [ ] remove unused dependencies

pub use rmpv::Value;

pub use self::{
    client::*,
    codec::consts::{IteratorType, TransactionIsolationLevel},
    errors::{Error, TransportError},
};

pub mod utils;

mod client;
mod codec;
mod errors;
mod transport;
