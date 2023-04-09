// TODO:
//  - [ ] check or remove all unsafes, unwrap, panic, expect
//  - [ ] remove main.rs
//  - [ ] tests
//  - [ ] bump version to 0.1.0
//  - [ ] remove unused dependencies

pub use rmpv::Value;

pub use self::{
    builder::ConnectionBuilder,
    connection::Connection,
    connection_like::ConnectionLike,
    errors::{Error, TransportError},
    stream::Stream,
    transaction::{Transaction, TransactionBuilder},
};

mod builder;
mod transport;
// TODO: export codec for those who want to write custom connectors
mod codec;
mod connection;
mod connection_like;
mod errors;
mod stream;
mod transaction;
