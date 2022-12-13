//! Callback A [`BoxedCallback<Output>`] is a handle for waking up driver
//! caller by notifying asynchronous task is completed.
//!
//! This handle encapsulates a [`FnOnce(Result<Output>)`] instance,
//! which defines the caller-specific wakeup behavior.

use anyhow::Result;
use std::ptr::NonNull;

/// A virtual function pointer table (vtable) that specifies the
/// behavior of a [`FnOnce(Result<Output>)`](FnOnce).
#[repr(C)]
pub struct CallbackVTable<Output> {
    invoke: unsafe fn(NonNull<CallbackVTable<Output>>, result: Result<Output>),

    drop: unsafe fn(NonNull<CallbackVTable<Output>>),
}

impl<Output> CallbackVTable<Output> {
    fn new<F>() -> Self
    where
        F: FnOnce(Result<Output>),
    {
        CallbackVTable {
            drop: drop::<F, Output>,
            invoke: invoke::<F, Output>,
        }
    }
}

unsafe fn drop<F, Output>(_: NonNull<CallbackVTable<Output>>)
where
    F: FnOnce(Result<Output>),
{
}

unsafe fn invoke<F, Output>(vtable: NonNull<CallbackVTable<Output>>, result: Result<Output>)
where
    F: FnOnce(Result<Output>),
{
    let mut raw = vtable.cast::<Callback<F, Output>>();

    let f = raw.as_mut().f.take();

    f.unwrap()(result);
}

#[repr(C)]
struct Callback<F, Output>
where
    F: FnOnce(Result<Output>),
{
    vtable: CallbackVTable<Output>,
    f: Option<F>,
}

/// Type erased Callback wrapper struct
pub struct BoxedCallback<Output> {
    vtable: NonNull<CallbackVTable<Output>>,
}

impl<Output> BoxedCallback<Output> {
    pub fn new<F>(f: F) -> Self
    where
        F: FnOnce(Result<Output>),
    {
        let boxed = Box::new(Callback::<F, Output> {
            vtable: CallbackVTable::<Output>::new::<F>(),
            f: Some(f),
        });

        let ptr =
            unsafe { NonNull::new_unchecked(Box::into_raw(boxed) as *mut CallbackVTable<Output>) };

        Self { vtable: ptr }
    }

    /// Call the callback function and pass the return value to the caller
    pub fn invoke(&self, result: Result<Output>) {
        unsafe {
            let invoke = self.vtable.as_ref().invoke;

            invoke(self.vtable, result)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::BoxedCallback;

    #[test]
    fn test_boxed_callback() {
        let boxed: BoxedCallback<usize> = BoxedCallback::new(|v| {
            assert_eq!(v.unwrap(), 1);
        });

        boxed.invoke(Ok(1));
    }
}
