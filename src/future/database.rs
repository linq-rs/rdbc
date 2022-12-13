//! Default [`super::ConnectionPool`] implementation.

use std::sync::{Arc, Mutex};

use async_trait::async_trait;
use chrono::{DateTime, Utc};

use crate::{driver, BoxedDriver};

use super::{driver::AsyncDriver, ConnectionPool, Preparable, Statement, Transaction};

use anyhow::Result;

/// Default [`super::ConnectionPool`] implementation.
#[derive(Clone)]
#[allow(dead_code)]
pub struct Database {
    driver: Arc<Mutex<BoxedDriver>>,
    url: String,
    conns: Arc<Mutex<Vec<(DateTime<Utc>, Box<dyn driver::Connection>)>>>,
    max_idle_conns: usize,
    max_lifetime: chrono::Duration,
    _driver_name: String,
}

impl ConnectionPool for Database {
    /// Implement [`super::ConnectionPool::new`]
    fn new<S>(driver_name: S, driver: Arc<Mutex<BoxedDriver>>, url: S) -> anyhow::Result<Self>
    where
        S: Into<String> + AsRef<str>,
    {
        let url: String = url.into();
        let driver_name = driver_name.into();

        Ok(Self {
            driver,
            url,
            conns: Default::default(),
            max_idle_conns: 100,
            max_lifetime: chrono::Duration::hours(1),
            _driver_name: driver_name,
        })
    }
    /// Implement [`super::ConnectionPool::get_conn`]
    fn get_conn(&self) -> anyhow::Result<Box<dyn driver::Connection>> {
        let mut conns = self.conns.lock().unwrap();

        if !conns.is_empty() {
            let (_, conn) = conns.remove(0);
            return Ok(conn);
        }

        let conn = self.driver.lock().unwrap().open(&self.url)?;

        Ok(conn)
    }

    /// Implement [`super::ConnectionPool::release_conn`]
    fn release_conn(&self, conn: Box<dyn driver::Connection>) {
        let mut conns = self.conns.lock().unwrap();

        if conns.len() == self.max_idle_conns {
            conns.remove(0);
        }

        conns.push((Utc::now(), conn));
    }
}

impl Database {
    /// Start new transaction
    pub async fn begin(&self) -> Result<Transaction<Database>> {
        let mut conn = self.get_conn()?;

        let async_driver = AsyncDriver::new();

        conn.begin(async_driver.callback());

        let tx = async_driver.await?;

        Ok(Transaction::new(
            self._driver_name.clone(),
            self.url.clone(),
            tx,
            Some(conn),
            self.clone(),
        ))
    }
}

#[async_trait]
impl Preparable for Database {
    type DB = Database;
    async fn prepare<S>(&mut self, query: S) -> Result<Statement<Self::DB>>
    where
        S: Into<String> + Send,
    {
        let mut conn = self.get_conn()?;

        let async_driver = AsyncDriver::new();

        conn.prepare(query.into(), async_driver.callback());

        let stmt = async_driver.await?;

        Ok(Statement::new(Some(self.clone()), Some(conn), stmt))
    }

    fn driver_name(&self) -> &str {
        &self._driver_name
    }

    fn conn_str(&self) -> &str {
        &self.url
    }
}
