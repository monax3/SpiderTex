use std::any::Any;
use std::panic::UnwindSafe;

use crate::{Error, Result};

pub fn downcast_str(any: &dyn Any) -> Option<&str> {
    any.downcast_ref::<&'static str>()
        .copied()
        .or_else(|| any.downcast_ref::<String>().map(|s| &**s))
}

pub fn catch_panics(func: impl FnOnce() -> Result<()> + UnwindSafe) -> Result<()> {
    match std::panic::catch_unwind(func) {
        Err(panic) => Err(Error::message(
            downcast_str(panic.as_ref())
                .unwrap_or("Internal error")
                .to_string(),
        )),
        Ok(result) => result,
    }
}
