use std::sync::{Arc, Mutex};

use crate::driver;
use anyhow::Result;

use super::{driver::AsyncDriver, ConnectionPool, Preparable, Statement};

struct Inner<DB>
where
    DB: ConnectionPool + Sync + Send,
{
    tx: Box<dyn driver::Transaction>,
    pub(crate) conn: Option<Box<dyn driver::Connection>>,
    db: DB,
}

impl<DB> Drop for Inner<DB>
where
    DB: ConnectionPool + Sync + Send,
{
    fn drop(&mut self) {
        if let Some(conn) = self.conn.take() {
            self.db.release_conn(conn);
        }
    }
}

/// Asynchronous wrapper type for [`crate::driver::Transaction`]
#[derive(Clone)]
pub struct Transaction<DB>
where
    DB: ConnectionPool + Sync + Send,
{
    inner: Arc<Mutex<Inner<DB>>>,
    driver_name: String,
    conn_url: String,
}

impl<DB> Transaction<DB>
where
    DB: ConnectionPool + Sync + Send,
{
    pub(crate) fn new(
        driver_name: String,
        conn_url: String,
        tx: Box<dyn driver::Transaction>,
        conn: Option<Box<dyn driver::Connection>>,
        db: DB,
    ) -> Self {
        Self {
            inner: Arc::new(Mutex::new(Inner { tx, conn, db })),
            driver_name,
            conn_url,
        }
    }

    pub async fn commit(&mut self) -> Result<()> {
        let async_driver = AsyncDriver::new();
        self.inner
            .lock()
            .unwrap()
            .tx
            .commit(async_driver.callback());

        async_driver.await
    }

    pub async fn rollback(&mut self) -> Result<()> {
        let async_driver = AsyncDriver::new();
        self.inner
            .lock()
            .unwrap()
            .tx
            .rollback(async_driver.callback());

        async_driver.await
    }
}

#[async_trait::async_trait]
impl<DB> Preparable for Transaction<DB>
where
    DB: ConnectionPool + Sync + Send + Clone,
{
    type DB = DB;
    async fn prepare<S>(&mut self, query: S) -> anyhow::Result<Statement<Self::DB>>
    where
        S: Into<String> + Send,
    {
        let async_driver = AsyncDriver::new();

        self.inner
            .lock()
            .unwrap()
            .tx
            .prepare(query.into(), async_driver.callback());

        let stmt = async_driver.await?;

        Ok(Statement::new(None, None, stmt))
    }

    fn driver_name(&self) -> &str {
        &self.driver_name
    }

    fn conn_str(&self) -> &str {
        &self.conn_url
    }
}
