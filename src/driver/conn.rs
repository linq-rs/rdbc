use super::{callback::BoxedCallback, Statement, Transaction};

pub trait Connection: Send {
    /// Returns a prepared statement, bound to this connection.
    fn prepare(&mut self, query: String, callback: BoxedCallback<Box<dyn Statement>>);

    fn begin(&mut self, callback: BoxedCallback<Box<dyn Transaction>>);

    /// Sync returns connection status
    fn conn_status(&self) -> ConnStatus;

    /// Get connection unique id
    fn id(&self) -> &str;
}

pub enum ConnStatus {
    Connected,
    Disconnected,
}
