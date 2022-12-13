//! Asynchronous wrapper type for [`crate::driver::Rows`]

use std::sync::{Arc, Mutex};

use crate::driver;
use anyhow::Result;

use super::{driver::AsyncDriver, ConnectionPool, Statement};

type Column = driver::Column;
type ArgValue = driver::ArgValue;
type ArgName = driver::ArgName;

#[allow(dead_code)]
struct Inner<DB>
where
    DB: ConnectionPool + Sync + Send,
{
    rows: Box<dyn driver::Rows>,
    stmt: Statement<DB>,
}

/// Asynchronous wrapper type for [`crate::driver::Rows`]
#[derive(Clone)]
pub struct Rows<DB>
where
    DB: ConnectionPool + Sync + Send,
{
    inner: Arc<Mutex<Inner<DB>>>,
}

impl<DB> Rows<DB>
where
    DB: ConnectionPool + Sync + Send,
{
    pub(crate) fn new(rows: Box<dyn driver::Rows>, stmt: Statement<DB>) -> Self {
        Self {
            inner: Arc::new(Mutex::new(Inner { rows, stmt })),
        }
    }

    pub async fn colunms(&mut self) -> Result<Vec<Column>> {
        let async_driver = AsyncDriver::new();

        {
            let mut inner = self.inner.lock().unwrap();

            inner.rows.colunms(async_driver.callback());
        }

        async_driver.await
    }

    pub async fn next(&mut self) -> Result<bool> {
        let async_driver = AsyncDriver::new();

        {
            let mut inner = self.inner.lock().unwrap();

            inner.rows.next(async_driver.callback());
        }

        async_driver.await
    }

    pub async fn get<N>(
        &mut self,
        name: N,
        column_type: driver::ColumnType,
    ) -> Result<Option<ArgValue>>
    where
        N: Into<ArgName>,
    {
        let async_driver = AsyncDriver::new();

        {
            let mut inner = self.inner.lock().unwrap();

            inner
                .rows
                .get(name.into(), column_type, async_driver.callback());
        }

        async_driver.await
    }
}
