//! # This mode provides a generic interface around SQL database.

pub mod callback;
pub mod conn;
pub mod error;
pub mod stmt;
pub mod tx;

pub use conn::*;
pub use error::*;
pub use stmt::*;
pub use tx::*;

pub trait Driver: Send {
    fn open(&mut self, url: &str) -> anyhow::Result<Box<dyn Connection>>;
}
