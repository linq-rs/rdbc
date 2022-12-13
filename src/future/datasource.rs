//! SQL driver register center
//!
//! Global DataSource api avaliable on feature *global*

use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
};

use crate::{driver, Database};

use anyhow::*;

use super::ConnectionPool;

pub type BoxedDriver = Box<dyn driver::Driver + 'static>;

/// DataSource makes a database driver availiable by the provided name.
///
/// If register driver with same name twice, it panics.
#[derive(Clone, Default)]
#[allow(dead_code)]
pub struct DataSource {
    drivers: Arc<Mutex<HashMap<String, Arc<Mutex<BoxedDriver>>>>>,
}

impl DataSource {
    /// Register new driver with driver name.
    ///
    /// # Arguments
    /// * `name` - Driver register name, if register driver name twice, method return [`Err`].
    ///
    pub fn register<S>(&self, name: S, driver: impl driver::Driver + 'static) -> Result<()>
    where
        S: Into<String> + AsRef<str>,
    {
        let mut drivers = self.drivers.lock().unwrap();

        if drivers.contains_key(name.as_ref()) {
            return Err(anyhow!("register driver {} twice", name.as_ref()));
        }

        drivers.insert(name.into(), Arc::new(Mutex::new(Box::new(driver))));

        Ok(())
    }

    /// Create a new connection pool for the `url`
    /// using given [`ConnectionPool`](super::ConnectionPool)
    ///
    /// # Arguments
    ///
    /// * `name` - Driver name
    /// * `url` - SQL driver connection url
    ///
    pub fn open_with<S, DB>(&self, name: S, url: S) -> Result<DB>
    where
        S: Into<String> + AsRef<str>,
        DB: ConnectionPool + Sync + Send,
    {
        let drivers = self.drivers.lock().unwrap();
        let driver = drivers.get(name.as_ref());

        if let Some(driver) = driver {
            DB::new(name, driver.clone(), url)
        } else {
            return Err(anyhow!("driver {} not found", name.as_ref()));
        }
    }

    /// Create a new connection pool for the connection string
    /// using default [`ConnectionPool`](super::ConnectionPool) implementation [`Database`](super::Database)
    pub fn open<S>(&self, name: S, url: S) -> Result<Database>
    where
        S: Into<String> + AsRef<str>,
    {
        self.open_with(name, url)
    }
}

mod global {

    use super::*;

    fn global_datasource() -> &'static mut DataSource {
        static mut CONF: std::mem::MaybeUninit<DataSource> = std::mem::MaybeUninit::uninit();
        static ONCE: std::sync::Once = std::sync::Once::new();
        ONCE.call_once(|| unsafe {
            CONF.as_mut_ptr().write(DataSource::default());
        });
        unsafe { &mut *CONF.as_mut_ptr() }
    }

    pub fn register<S>(name: S, driver: impl driver::Driver + 'static) -> Result<()>
    where
        S: Into<String> + AsRef<str>,
    {
        global_datasource().register(name, driver)
    }

    pub fn open_with<S, DB>(name: S, url: S) -> Result<DB>
    where
        S: Into<String> + AsRef<str>,
        DB: ConnectionPool + Sync + Send,
    {
        global_datasource().open_with(name, url)
    }

    pub fn open<S>(name: S, url: S) -> Result<Database>
    where
        S: Into<String> + AsRef<str>,
    {
        global_datasource().open_with(name, url)
    }
}

pub use global::*;

#[cfg(test)]
mod tests {

    use crate::driver::Driver;

    use super::DataSource;

    struct NullDriver {}

    impl Driver for NullDriver {
        fn open(&mut self, _url: &str) -> anyhow::Result<Box<dyn crate::driver::Connection>> {
            unimplemented!()
        }
    }

    #[test]
    fn test_register() {
        let ds = DataSource::default();

        _ = ds.register("".to_owned(), NullDriver {});
    }
}
