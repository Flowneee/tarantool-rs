use std::io::Write;

use crate::{
    codec::{
        consts::{keys, RequestType},
        utils::{write_kv_str, write_kv_tuple, write_kv_u32},
    },
    errors::EncodingError,
    tuple::Tuple,
};

use super::Request;

#[derive(Clone, Debug)]
enum ExecuteStatment<'a> {
    StatementId(u32),
    Query(&'a str),
}

#[derive(Clone, Debug)]
pub(crate) struct Execute<'a, T> {
    statement: ExecuteStatment<'a>,
    binds: T,
}

impl<'a, T: Tuple> Request for Execute<'a, T> {
    fn request_type() -> RequestType
    where
        Self: Sized,
    {
        RequestType::Execute
    }

    // NOTE: `&mut buf: mut` is required since I don't get why compiler complain
    fn encode(&self, mut buf: &mut dyn Write) -> Result<(), EncodingError> {
        rmp::encode::write_map_len(&mut buf, 2)?;
        match self.statement {
            ExecuteStatment::StatementId(x) => write_kv_u32(buf, keys::SQL_STMT_ID, x)?,
            ExecuteStatment::Query(x) => write_kv_str(buf, keys::SQL_TEXT, x)?,
        }
        write_kv_tuple(buf, keys::SQL_BIND, &self.binds)?;
        Ok(())
    }
}

impl<'a, T> Execute<'a, T> {
    pub(crate) fn new_statement_id(statment_id: u32, binds: T) -> Self {
        Self {
            statement: ExecuteStatment::StatementId(statment_id),
            binds,
        }
    }

    pub(crate) fn new_query(query: &'a str, binds: T) -> Self {
        Self {
            statement: ExecuteStatment::Query(query),
            binds,
        }
    }
}
