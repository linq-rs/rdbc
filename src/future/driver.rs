use std::sync::{Arc, Mutex};

use crate::driver::callback::BoxedCallback;
use anyhow::Result;

struct AsyncDriverImpl<Output> {
    waker: Option<std::task::Waker>,
    output: Option<Result<Output>>,
}
impl<Output> Default for AsyncDriverImpl<Output> {
    fn default() -> Self {
        Self {
            waker: Default::default(),
            output: Default::default(),
        }
    }
}

#[derive(Clone)]
pub struct AsyncDriver<Output> {
    inner: Arc<Mutex<AsyncDriverImpl<Output>>>,
}

impl<Output> AsyncDriver<Output> {
    pub fn new() -> Self {
        Self {
            inner: Arc::new(Mutex::new(Default::default())),
        }
    }

    pub fn callback(&self) -> BoxedCallback<Output> {
        let inner = self.inner.clone();

        BoxedCallback::new(move |result| {
            let mut inner = inner.lock().unwrap();
            inner.output = Some(result);

            if let Some(waker) = inner.waker.take() {
                waker.wake_by_ref();
            }
        })
    }
}

impl<Output> std::future::Future for AsyncDriver<Output> {
    type Output = Result<Output>;

    fn poll(
        self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Self::Output> {
        let mut inner = self.inner.lock().unwrap();

        if let Some(result) = inner.output.take() {
            return std::task::Poll::Ready(result);
        }

        inner.waker = Some(cx.waker().clone());

        std::task::Poll::Pending
    }
}

#[cfg(test)]
mod tests {
    use crate::driver;

    use super::AsyncDriver;

    #[async_std::test]
    async fn test_async_driver() {
        let driver = AsyncDriver::<Box<dyn driver::Statement>>::new();

        let callback = driver.callback();

        callback.invoke(Err(anyhow::format_err!("not found")));

        assert!(driver.await.is_err());
    }
}
