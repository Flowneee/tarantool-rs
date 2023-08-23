use std::result::Result as StdResult;

use rmpv::Value;

use crate::{
    codec::{consts::keys, request::Execute},
    errors::DecodingError,
    utils::{find_and_take_single_key_in_map, value_to_map},
    Executor, ExecutorExt, Result, SqlResponse, Tuple,
};

#[derive(Debug)]
pub struct PreparedSqlStatement<E> {
    stmt_id: u64,
    executor: E,
}

impl<E> PreparedSqlStatement<E> {
    fn new(stmt_id: u64, executor: E) -> Self {
        Self { stmt_id, executor }
    }

    pub fn from_prepare_response(response: Value, executor: E) -> StdResult<Self, DecodingError> {
        let map = value_to_map(response).map_err(|err| err.in_other("OK prepare response body"))?;
        let value = find_and_take_single_key_in_map(keys::SQL_STMT_ID, map).ok_or_else(|| {
            DecodingError::missing_key("SQL_STMT_ID").in_other("OK prepare response body")
        })?;
        let stmt_id: u64 = rmpv::ext::deserialize_from(value)
            .map_err(|err| DecodingError::from(err).in_key("SQL_STMT_ID"))?;
        Ok(Self::new(stmt_id, executor))
    }
}

impl<E: Clone> Clone for PreparedSqlStatement<E> {
    fn clone(&self) -> Self {
        Self {
            stmt_id: self.stmt_id,
            executor: self.executor.clone(),
        }
    }
}

impl<E: Executor> PreparedSqlStatement<E> {
    /// Execute prepared SQL query with parameters.
    pub async fn execute<T>(&self, binds: T) -> Result<SqlResponse>
    where
        T: Tuple + Send,
    {
        Ok(SqlResponse(
            self.executor
                .send_request(Execute::new_statement_id(self.stmt_id, binds))
                .await?,
        ))
    }
}

impl<E: Clone> PreparedSqlStatement<&E> {
    pub fn with_cloned_executor(&self) -> PreparedSqlStatement<E> {
        PreparedSqlStatement {
            stmt_id: self.stmt_id,
            executor: self.executor.clone(),
        }
    }
}
