pub use self::{
    connection::Connection,
    connection_like::ConnectionLike,
    executor::Executor,
    stream::Stream,
    transaction::{Transaction, TransactionBuilder},
};

pub mod schema;

mod connection;
mod connection_like;
mod executor;
mod stream;
mod transaction;

mod private {
    use crate::client::{Connection, Stream, Transaction};

    #[doc(hidden)]
    pub trait Sealed {}

    impl Sealed for Connection {}
    impl Sealed for Stream {}
    impl Sealed for Transaction {}
    impl<S: Sealed> Sealed for &S {}
    impl<S: Sealed> Sealed for &mut S {}
}
