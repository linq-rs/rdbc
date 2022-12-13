use std::fmt::Display;

use super::callback::BoxedCallback;

/// SQL argument placeholder name
#[derive(Debug, Clone, PartialEq)]
pub enum ArgName {
    String(String),
    Offset(usize),
}

impl From<usize> for ArgName {
    fn from(data: usize) -> Self {
        ArgName::Offset(data)
    }
}

impl From<String> for ArgName {
    fn from(data: String) -> Self {
        ArgName::String(data)
    }
}

impl From<&str> for ArgName {
    fn from(data: &str) -> Self {
        ArgName::String(data.to_owned())
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum ArgValue {
    I64(i64),
    F64(f64),
    String(String),
    Bytes(Vec<u8>),
    Null,
}

impl Display for ArgValue {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::I64(v) => write!(f, "{}", v),
            Self::F64(v) => write!(f, "{}", v),
            Self::String(v) => write!(f, "'{}'", v),
            Self::Bytes(v) => write!(f, "{:x?}", v),
            Self::Null => write!(f, "NULL"),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Argument {
    pub name: ArgName,
    pub value: ArgValue,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ExecResult {
    pub last_insert_id: u64,
    pub raws_affected: u64,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Column {
    pub column_index: u64,
    pub column_name: String,
    pub column_decltype: String,
    pub column_decltype_len: Option<u64>,
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum ColumnType {
    I64,
    F64,
    String,
    Bytes,
    Null,
}

/// Statement driver provider trait
pub trait Statement: Send {
    /// Returns the number of placeholder parameters.
    ///
    /// May returns [`None`], if the driver doesn't know its number of placeholder
    fn num_input(&self, callback: BoxedCallback<Option<usize>>);

    /// Executes a query that doesn't return rows, such
    /// as an INSERT or UPDATE.
    fn execute(&mut self, args: Vec<Argument>, callback: BoxedCallback<ExecResult>);

    /// executes a query that may return rows, such as a
    /// SELECT.
    fn query(&mut self, args: Vec<Argument>, callback: BoxedCallback<Box<dyn Rows>>);
}

pub trait Rows: Send {
    fn colunms(&mut self, callback: BoxedCallback<Vec<Column>>);

    fn next(&mut self, callback: BoxedCallback<bool>);

    /// Get current row value by arg name.
    fn get(
        &mut self,
        name: ArgName,
        column_type: ColumnType,
        callback: BoxedCallback<Option<ArgValue>>,
    );
}
