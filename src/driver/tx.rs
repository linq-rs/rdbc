use super::{callback::BoxedCallback, Statement};

/// Driver transaction trait .
///
/// The driver must ensure that uncommitted transaction objects automatically perform
/// a [`Transaction::rollback`] operation when they are dropped.
pub trait Transaction: Send {
    fn prepare(&mut self, query: String, callback: BoxedCallback<Box<dyn Statement>>);

    fn commit(&mut self, callback: BoxedCallback<()>);

    fn rollback(&mut self, callback: BoxedCallback<()>);
}
