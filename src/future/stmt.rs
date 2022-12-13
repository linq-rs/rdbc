use std::sync::{Arc, Mutex};

use crate::driver;

use super::{driver::AsyncDriver, ConnectionPool, Rows};

use anyhow::Result;

/// Statement query/execute argument type.
pub type Argument = driver::Argument;

/// Statement execute result type.
pub type ExecResult = driver::ExecResult;

/// Statement query/execute argument name.
pub type ArgName = driver::ArgName;

/// Statement query/execute argument value.
pub type ArgValue = driver::ArgValue;

/// num_input return type.
pub type Column = driver::Column;

/// Column type enum.
pub type ColumnType = driver::ColumnType;

#[allow(dead_code)]
struct Inner<DB>
where
    DB: ConnectionPool + Sync + Send,
{
    db: Option<DB>,
    conn: Option<Box<dyn driver::Connection>>,
    stmt: Option<Box<dyn driver::Statement>>,
}
/// Implement [`Drop`] trait to return conn to [`super::ConnectionPool`]
impl<DB> Drop for Inner<DB>
where
    DB: ConnectionPool + Sync + Send,
{
    fn drop(&mut self) {
        if let Some(conn) = self.conn.take() {
            if let Some(db) = self.db.take() {
                drop(self.stmt.take().unwrap());
                db.release_conn(conn);
            }
        }
    }
}

/// Asynchronous wrapper type for [`crate::driver::Statement`]
#[allow(dead_code)]
#[derive(Clone)]
pub struct Statement<DB>
where
    DB: ConnectionPool + Sync + Send,
{
    inner: Arc<Mutex<Inner<DB>>>,
}

impl<DB> Statement<DB>
where
    DB: ConnectionPool + Sync + Send + Clone,
{
    pub fn new(
        db: Option<DB>,
        conn: Option<Box<dyn driver::Connection>>,
        stmt: Box<dyn driver::Statement>,
    ) -> Self {
        Self {
            inner: Arc::new(Mutex::new(Inner {
                db,
                conn,
                stmt: Some(stmt),
            })),
        }
    }

    pub async fn num_input(&self) -> Result<Option<usize>> {
        let async_driver = AsyncDriver::new();

        self.inner
            .lock()
            .unwrap()
            .stmt
            .as_ref()
            .unwrap()
            .num_input(async_driver.callback());

        async_driver.await
    }

    pub async fn execute(&mut self, args: Vec<Argument>) -> Result<ExecResult> {
        let async_driver = AsyncDriver::new();

        self.inner
            .lock()
            .unwrap()
            .stmt
            .as_mut()
            .unwrap()
            .execute(args, async_driver.callback());

        async_driver.await
    }

    pub async fn query(&mut self, args: Vec<Argument>) -> Result<Rows<DB>> {
        let async_driver = AsyncDriver::new();

        self.inner
            .lock()
            .unwrap()
            .stmt
            .as_mut()
            .unwrap()
            .query(args, async_driver.callback());

        Ok(Rows::new(async_driver.await?, self.clone()))
    }
}
