pub use self::{
    call_response::CallResponse,
    connection::Connection,
    dmo_response::DmoResponse,
    executor::Executor,
    executor_ext::ExecutorExt,
    prepared_sql_statement::PreparedSqlStatement,
    sql_response::SqlResponse,
    stream::Stream,
    transaction::{Transaction, TransactionBuilder},
};

pub mod schema;

mod call_response;
mod connection;
mod dmo_response;
mod executor;
mod executor_ext;
mod prepared_sql_statement;
mod sql_response;
mod stream;
mod transaction;

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
