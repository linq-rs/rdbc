use std::sync::{Arc, Mutex};

use anyhow::Result;

use crate::{driver, BoxedDriver};

/// Database connection pool trait.
pub trait ConnectionPool: Sized {
    /// Create new connection pool
    /// # Arguments
    /// * `driver` - SQL thread safe driver instance
    /// * `url` - SQL driver connect url
    fn new<S>(driver_name: S, driver: Arc<Mutex<BoxedDriver>>, url: S) -> Result<Self>
    where
        S: Into<String> + AsRef<str>;

    /// Get new connection from pool or create new one from driver.
    fn get_conn(&self) -> Result<Box<dyn driver::Connection>>;

    /// Release one connection return to pool.
    ///
    /// # Arguments
    ///
    /// * `conn` - Unused connection instance
    fn release_conn(&self, conn: Box<dyn driver::Connection>);
}
