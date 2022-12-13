use super::{stmt::Statement, ConnectionPool};

/// The trait to create a prepared statment for later queries or executions.
#[async_trait::async_trait]
pub trait Preparable {
    type DB: ConnectionPool + Sync + Send;
    async fn prepare<S>(&mut self, query: S) -> anyhow::Result<Statement<Self::DB>>
    where
        S: Into<String> + Send;

    fn driver_name(&self) -> &str;

    fn conn_str(&self) -> &str;
}
