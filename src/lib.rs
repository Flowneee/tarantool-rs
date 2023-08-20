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
// * [ ] check or remove all unsafes, panic
// * [ ] tests
// * [ ] bump version to 0.1.0
// * [ ] remove unused dependencies

//! `tarantool-rs` - Asyncronous Tokio-based client for Tarantool.
//!
//! This crate provide async connector and necessary abstractions for interaction with Tarantool instance.
//!
//! ## Example
//!
//! ```no_run
//! # use rmpv::Value;
//! # use serde::Deserialize;
//! use tarantool_rs::{Connection, ExecutorExt, IteratorType};
//!
//! # #[tokio::main]
//! # async fn main() -> Result<(), anyhow::Error> {
//! # pretty_env_logger::init();
//! // Create connection to Tarantool instace
//! let conn = Connection::builder().build("127.0.0.1:3301").await?;
//!
//! // Execute Lua code with one argument, returning this argument
//! let number: u64 = conn.eval("return ...", (42, )).await?.decode_result()?;
//! assert_eq!(number, 42);
//!
//! // Call Lua function 'rand' (assuming it exists and return 42)
//! let number: u64 = conn.call("rand", ()).await?.decode_first()?;
//! assert_eq!(number, 42);
//!
//! // Get 'clients' space with 2 fields - 'id' and 'name'
//! let clients_space = conn.space("clients").await?.expect("Space exists");
//!
//! // Insert tuple into 'clients' space
//! clients_space.insert((1, "John Doe")).await?;
//!
//! // Select tuples from clients space using primary index
//! let clients: Vec<(i64, String)> = clients_space
//!     .select(None, None, None, (1, ))
//!     .await?;
//! # Ok(())
//! # }
//! ````
//!
//! ## Features
//!
//! * [x] authorization
//! * [x] evaluating Lua expressions
//! * [x] remote function calling
//! * [x] CRUD operations
//! * [x] transaction control (begin/commit/rollback)
//! * [x] reconnection in background
//! * [ ] SQL requests
//! * [ ] chunked responses
//! * [ ] watchers and events
//! * [ ] connection pooling
//! * [ ] automatic schema fetching and reloading
//! * [ ] graceful shutdown protocol support
//! * [ ] pre Tarantool 2.10 versions support
//! * [ ] customizable connection features (streams/watchers/mvcc)
//! * [ ] custom Tarantool MP types (UUID, ...)
//! * [ ] ...

pub use rmpv::Value;

#[doc(inline)]
pub use self::{
    builder::{ConnectionBuilder, ReconnectInterval},
    client::*,
    codec::consts::{IteratorType, TransactionIsolationLevel},
    errors::Error,
    tuple::Tuple,
};

pub mod errors;

mod builder;
mod client;
mod codec;
mod transport;
mod tuple;
mod utils;

/// Alias for [`std::result::Result<T, crate::Error>`].
pub type Result<T> = std::result::Result<T, Error>;
