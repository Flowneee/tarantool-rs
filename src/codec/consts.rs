/// IPROTO map keys.
///
/// Describes only keys, used in this crate.
///
/// See details [here](https://github.com/tarantool/tarantool/blob/master/src/box/iproto_constants.h#L62).
pub mod keys {
    pub const REQUEST_TYPE: u8 = 0x00;
    pub const RESPONSE_CODE: u8 = 0x00;
    pub const SYNC: u8 = 0x01;
    pub const SCHEMA_VERSION: u8 = 0x05;
    pub const STREAM_ID: u8 = 0x0a;
    pub const ERROR_24: u8 = 0x31;
    pub const ERROR: u8 = 0x52;
}

/// IPROTO command codes.
///
/// Describes only types, used in this crate.
///
/// See details [here](https://github.com/tarantool/tarantool/blob/master/src/box/iproto_constants.h#L201).
#[derive(Copy, Clone, Debug)]
#[repr(u8)]
pub enum IProtoType {
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
    TypeStatMax,
    Raft = 30,
    RatPromote = 31,
    RaftDemot = 32,
    RaftConfirm = 40,
    RaftRollback = 41,
    Ping = 64,
    /// Replication JOIN command
    Join = 65,
    /// Replication SUBSCRIBE command
    Subscribe = 66,
    VoteDeprecated = 67,
    Vote = 68,
    FetchSnapshot = 69,
    Register = 70,
    JoinMeta = 71,
    JoinSnapshot = 72,
    /// Protocol features request
    Id = 73,
    Watch = 74,
    Unwatch = 75,
    Event = 76,
    /// Non-final response type
    Chunk = 128,
}
