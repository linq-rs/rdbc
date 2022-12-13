//! RDBC predefined errors

use thiserror::Error;

#[derive(Error, Debug)]
pub enum RDBCError {
    #[error("Native error: code({0}) {1}")]
    NativeError(i32, String),

    #[error("rdbc step return unexpect rows")]
    UnexpectRows,

    #[error("Call next first or no more rows")]
    NextDataError,

    #[error("Get column data out of range {0}")]
    OutOfRange(u64),

    #[error("stmt '{0}' bind named arg({1}) failed")]
    BindArgError(String, String),

    #[error("Get column by name {0}, not found")]
    UnknownColumn(String),
}
