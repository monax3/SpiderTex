//! This is just a Lazy with static initialisation. The logic is very simple and
//! failing to meet any invariants should only be able to cause panics, not UB.
#![allow(unsafe_code)]
use std::cell::UnsafeCell;
use std::sync::atomic::{AtomicBool, Ordering};

pub struct MaybeReady<T> {
    inner: UnsafeCell<Option<T>>,
    is_ready: AtomicBool,
}

unsafe impl<T> Sync for MaybeReady<T> where T: Sync {}

impl<T> MaybeReady<T> {
    #[must_use]
    pub const fn new() -> Self {
        Self {
            inner: UnsafeCell::new(None),
            is_ready: AtomicBool::new(false),
        }
    }

    #[must_use]
    pub const fn const_ready(inner: T) -> Self {
        Self {
            inner: UnsafeCell::new(Some(inner)),
            is_ready: AtomicBool::new(true),
        }
    }

    pub fn get(&self) -> &T {
        let is_ready = self.is_ready.load(Ordering::SeqCst);

        if is_ready {
            unsafe { self.inner.get().as_ref() }
                .and_then(Option::as_ref)
                .expect("Null pointer inside MaybeReady")
        } else {
            panic!("MaybeReady not ready")
        }
    }

    pub fn try_get(&self) -> Option<&T> {
        let is_ready = self.is_ready.load(Ordering::SeqCst);

        if is_ready {
            unsafe { self.inner.get().as_ref() }
                .expect("Null pointer in MaybeReady")
                .as_ref()
        } else {
            None
        }
    }

    pub fn is_ready(&self) -> bool {
        self.is_ready.load(Ordering::SeqCst)
    }

    pub fn ready(&self, value: T) {
        let is_ready = self.is_ready.load(Ordering::Acquire);
        assert!(!is_ready, "MaybeReady was readied more than once");

        unsafe { self.inner.get().as_mut() }
            .expect("Null pointer in MaybeReady")
            .replace(value);
        self.is_ready.store(true, Ordering::Release);
    }
}

impl<T> Default for MaybeReady<T> {
    fn default() -> Self {
        Self::new()
    }
}
