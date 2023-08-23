pub use self::{
    connection::Connection,
    executor::Executor,
    executor_ext::ExecutorExt,
    prepared_sql_statement::PreparedSqlStatement,
    sql_response::SqlResponse,
    stream::Stream,
    transaction::{Transaction, TransactionBuilder},
    tuple_response::TupleResponse,
};

pub mod schema;

mod connection;
mod executor;
mod executor_ext;
mod prepared_sql_statement;
mod sql_response;
mod stream;
mod transaction;
mod tuple_response;

mod private {
    use crate::client::{Connection, Stream, Transaction};

    #[doc(hidden)]
    pub trait Sealed {}

    impl Sealed for Connection {}
    impl Sealed for Stream {}
    impl Sealed for Transaction {}
    impl<S: Sealed + ?Sized> Sealed for &S {}
    impl<S: Sealed + ?Sized> Sealed for &mut S {}
}
