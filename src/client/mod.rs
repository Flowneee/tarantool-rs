pub use self::{
    call_response::CallResponse,
    connection::Connection,
    dmo::{DmoOperation, DmoResponse},
    executor::Executor,
    executor_ext::ExecutorExt,
    sql::{PreparedSqlStatement, SqlResponse},
    stream::Stream,
    transaction::{Transaction, TransactionBuilder},
};

// TODO: either reimport everything from schema or add dmo and sql mods
pub mod schema;

mod call_response;
mod connection;
mod dmo;
mod executor;
mod executor_ext;
mod sql;
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
