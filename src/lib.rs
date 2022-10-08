// TODO:
//  - [ ] check or remove all unsafes, unwrap, panic, expect
//  - [ ] remove main.rs
//  - [ ] tests
//  - [ ] bump version to 0.1.0

pub use rmpv::Value;

pub use self::errors::Error;

pub mod codec;
pub mod connection;

mod errors;
