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
//! let (number,): (u64,) = conn.eval("return ...", vec![42.into()]).await?;
//! assert_eq!(number, 42);
//!
//! // Call Lua function 'rand' (assuming it exists and return 42)
//! let (number,): (u64,) = conn.call("rand", vec![]).await?;
//! assert_eq!(number, 42);
//!
//! // Get 'clients' space with 2 fields - 'id' and 'name'
//! let clients_space = conn.space("clients").await?.expect("Space exists");
//!
//! // Insert tuple into 'clients' space
//! clients_space.insert(vec![1.into(), "John Doe".into()]).await?;
//!
//! // Select tuples from clients space using primary index
//! let clients: Vec<(i64, String)> = clients_space
//!     .select(None, None, None, vec![1.into()])
//!     .await?;
//! # Ok(())
//! # }
//! ````
//!
//! ## Deserializing Lua responses in `call` and `eval`
//!
//! [`ExecutorExt::eval`] and [`ExecutorExt::call`] (on Lua functions) will always return tuple
//! of a fixed size (size is depend on `return` tatement in your code). For example function
//!
//! ```lua
//! function rand()
//!     return 42, nil
//! end
//! ````
//!
//! will always return 2 elements: integer and null. it can be deserialized into multiple different
//! types:
//!
//! * `Value` - this is most general way, which (converted to JSON) would looks like `[42, null]`;
//! * `(Value, Value)` - less general way, this will fail if function return tuple with less or more elements;
//! * `Vec<Value>` - same as before, but without length constraint (i.e. can be deserialized from any tuple);
//! * `(u64, Option<Value>)` - here type of the first argument is speified, but second is not `rmpv::Value as second`;
//! * `(u64, Option<String>)` - same as before, but type of second element is specified.
//! * `struct Response { first: u64, second: Option<Value> }` - since response is just a tuple, it can be
//!   deserialized into _any_ type, implementing `DeserializeOwned`.
//!
//! ```ignore
//! let resp: Value = conn.call("rand", vec![]).await?;
//! println!("{:?}", resp); // Array([Integer(PosInt(42)), Nil])
//!
//! let resp: (Value, Value) = conn.call("rand", vec![]).await?;
//! println!("{:?}", resp); // (Integer(PosInt(42)), Nil)
//!
//! let resp: Vec<Value> = conn.call("rand", vec![]).await?;
//! println!("{:?}", resp); // [Integer(PosInt(42)), Nil]
//!
//! let resp: (u64, Option<String>) = conn.call("rand", vec![]).await?;
//! println!("{:?}", resp); // (42, None)
//!
//! let resp: Response = conn.call("rand", vec![]).await?;
//! println!("{:?}", resp); // Response { first: 42, second: None }
//! ```
//!
//! NOTE: If Lua code return nothing (empty tuple), currently you have to use `Value` as return type.
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
};

pub mod errors;

mod builder;
mod client;
mod codec;
mod transport;
mod utils;

/// Alias for [`std::result::Result<T, crate::Error>`].
pub type Result<T> = std::result::Result<T, Error>;
