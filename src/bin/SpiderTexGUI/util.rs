use std::{sync::atomic::{Ordering, AtomicBool}, cell::UnsafeCell};
use color_eyre::{Result, eyre::eyre};

pub struct MaybeReady<T> {
    inner: UnsafeCell<Option<T>>,
    is_ready: AtomicBool,
}

unsafe impl<T> Sync for MaybeReady<T> where T: Sync {}

impl<T> MaybeReady<T> {
    pub const fn new() -> Self {
        Self { inner: UnsafeCell::new(None), is_ready: AtomicBool::new(false) }
    }

    pub fn get(&self) -> Option<&T> {
        let is_ready = self.is_ready.load(Ordering::SeqCst);

        if is_ready {
            unsafe { self.inner.get().as_ref() }.expect("Null pointer in MaybeReady").as_ref()
        } else {
            None
        }
    }

    pub fn ready(&self, value: T) {
        let is_ready = self.is_ready.load(Ordering::Acquire);
        assert!(!is_ready, "MaybeReady was readied more than once");

        unsafe { self.inner.get().as_mut() }.expect("Null pointer in MaybeReady").replace(value);
        self.is_ready.store(true, Ordering::Release);
    }
}

impl<T> Default for MaybeReady<T> {
    fn default() -> Self {
        Self::new()
    }
}


fn deref_str(any: &dyn std::any::Any) -> Option<&str> {
    any.downcast_ref::<&'static str>()
        .map(|s| *s)
        .or_else(|| any.downcast_ref::<String>().map(|s| &**s))
}

pub fn catch_panics(func: impl FnOnce() -> Result<()> + std::panic::UnwindSafe) -> Result<()> {
    match std::panic::catch_unwind(func) {
        Err(panic) => Err(eyre!(
            "{}",
            deref_str(panic.as_ref()).unwrap_or("Internal error")
        )),
        Ok(result) => result,
    }
}