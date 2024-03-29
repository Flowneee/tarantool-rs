/// IPROTO map keys.
///
/// Describes only keys, used in this crate.
///
/// See details [here](https://github.com/tarantool/tarantool/blob/master/src/box/iproto_constants.h#L62).
pub mod keys {
    pub const REQUEST_TYPE: u8 = 0x00;
    pub const RESPONSE_CODE: u8 = 0x00;
    pub const SQL_INFO_ROW_COUNT: u8 = 0x00;
    pub const SYNC: u8 = 0x01;
    pub const SCHEMA_VERSION: u8 = 0x05;
    pub const STREAM_ID: u8 = 0x0a;
    pub const SPACE_ID: u8 = 0x10;
    pub const INDEX_ID: u8 = 0x11;
    pub const LIMIT: u8 = 0x12;
    pub const OFFSET: u8 = 0x13;
    pub const ITERATOR: u8 = 0x14;
    pub const INDEX_BASE: u8 = 0x15;
    pub const KEY: u8 = 0x20;
    pub const TUPLE: u8 = 0x21;
    pub const FUNCTION_NAME: u8 = 0x22;
    pub const USER_NAME: u8 = 0x23;
    pub const EXPR: u8 = 0x27;
    pub const OPS: u8 = 0x28;
    pub const DATA: u8 = 0x30;
    pub const ERROR_24: u8 = 0x31;
    pub const SQL_TEXT: u8 = 0x40;
    pub const SQL_BIND: u8 = 0x41;
    pub const SQL_INFO: u8 = 0x42;
    pub const SQL_STMT_ID: u8 = 0x43;
    pub const ERROR: u8 = 0x52;
    pub const VERSION: u8 = 0x54;
    pub const FEATURES: u8 = 0x55;
    pub const TIMEOUT: u8 = 0x56;
    pub const TXN_ISOLATION: u8 = 0x59;
}

/// Request type constants.
///
/// Describes only types, used in this crate.
///
/// See details [here](https://github.com/tarantool/tarantool/blob/master/src/box/iproto_constants.h).
#[derive(Copy, Clone, Debug)]
#[repr(u8)]
pub enum RequestType {
    Ok = 0,
    Select = 1,
    Insert = 2,
    Replace = 3,
    Update = 4,
    Delete = 5,
    /// CALL request - wraps result into [tuple, tuple, ...] format
    Call16 = 6,
    Auth = 7,
    Eval = 8,
    Upsert = 9,
    /// CALL request - returns arbitrary MessagePack
    Call = 10,
    /// Execute an SQL statement
    Execute = 11,
    Nop = 12,
    Prepare = 13,
    Begin = 14,
    Commit = 15,
    Rollback = 16,
    Ping = 64,
    Id = 73,
    Watch = 74,
    Unwatch = 75,
    Event = 76,
    /// Non-final response type
    Chunk = 128,
}

pub mod response_codes {
    pub const OK: u32 = 0x0;
    pub const ERROR_RANGE_START: u32 = 0x8000;
    pub const ERROR_RANGE_END: u32 = 0x8FFF;
}

/// Transaction isolation level.
///
/// See docs [here](https://www.tarantool.io/en/doc/latest/concepts/atomic/txn_mode_mvcc/#txn-mode-mvcc-options).
#[derive(Copy, Clone, Debug)]
#[repr(u8)]
pub enum TransactionIsolationLevel {
    /// Use the default level from box.cfg (default),
    Default = 0,
    /// Read changes that are committed but not confirmed yet.
    ReadCommited = 1,
    /// Read confirmed changes.
    ReadConfirmed = 2,
    /// Determine isolation level automatically.
    BestEffort = 3,
}

impl Default for TransactionIsolationLevel {
    fn default() -> Self {
        Self::Default
    }
}

/// Iterator type for `select` requests.
///
/// For details check Tarantool documantation <https://www.tarantool.io/en/doc/latest/reference/reference_lua/box_index/pairs/>.
#[derive(Copy, Clone, Debug)]
#[repr(u8)]
pub enum IteratorType {
    Eq = 0,
    Req = 1,
    All = 2,
    Lt = 3,
    Le = 4,
    Ge = 5,
    Gt = 6,
    BitsAllSet = 7,
    BitsAnySet = 8,
    BitsAlLNotSet = 9,
    Overlaps = 10,
    Neighvor = 11,
}

impl Default for IteratorType {
    fn default() -> Self {
        Self::Eq
    }
}
