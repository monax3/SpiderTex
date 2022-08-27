use std::borrow::Cow;
use std::fmt::Display;

use crate::prelude::*;

pub type Result<T, E = Error> = std::result::Result<T, E>;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("IO: {0}")]
    Io(#[from] std::io::Error),

    #[error("Format JSON: {0}")]
    Json(#[from] serde_json::Error),

    #[error("Format JSON: {0}")]
    Hex(#[from] hex::FromHexError),

    #[error("Format JSON: {0}")]
    Bytemuck(#[from] bytemuck::PodCastError),

    #[cfg(windows)]
    #[error(transparent)]
    Windows(#[from] windows::core::Error),

    #[error(transparent)]
    Image(#[from] image::ImageError),

    #[error("{0}")]
    Message(Cow<'static, str>),

    #[error("Internal error")]
    Internal,
}

pub fn error_message<T>(message: impl Into<Cow<'static, str>>) -> Result<T> {
    Err(Error::message(message))
}

impl Error {
    pub fn message(message: impl Into<Cow<'static, str>>) -> Self {
        Self::Message(message.into())
    }
}

impl From<&'static str> for Error {
    fn from(message: &'static str) -> Self {
        Self::message(message)
    }
}

impl From<String> for Error {
    fn from(message: String) -> Self {
        Self::message(message)
    }
}

#[track_caller]
fn log_failure(context: Option<impl Display>, message: Option<impl Display>) {
    let location = std::panic::Location::caller();

    match (context, message) {
        (Some(context), Some(message)) => event!(
            ERROR,
            "{}:{}: {context}: {message}",
            location.file(),
            location.line()
        ),
        (None, Some(message)) => {
            event!(ERROR, "{}:{}: {message}", location.file(), location.line())
        }
        (Some(context), None) => {
            event!(ERROR, "{}:{}: {context}", location.file(), location.line())
        }
        (None, None) => event!(
            ERROR,
            "{}:{}: An operation failed",
            location.file(),
            location.line()
        ),
    };
}

pub trait LogFailure: Sized {
    type Failed: Display;

    #[must_use]
    #[track_caller]
    fn log_failure(self) -> Self {
        if let Some(failure) = self.as_failed() {
            log_failure(None::<Self::Failed>, Some(failure));
        }
        self
    }

    #[must_use]
    #[track_caller]
    fn log_failure_with<C: Display>(self, context_func: impl FnOnce() -> C) -> Self {
        if let Some(failure) = self.as_failed() {
            log_failure(Some(context_func()), Some(failure));
        }
        self
    }

    #[must_use]
    #[track_caller]
    fn log_failure_as(self, context: &str) -> Self {
        self.log_failure_with(|| context)
    }

    #[must_use]
    #[track_caller]
    fn log_failure_if(self, condition: bool) -> Self {
        if let (true, Some(failure)) = (condition, self.as_failed()) {
            log_failure(None::<Self::Failed>, Some(failure));
        }
        self
    }

    #[must_use]
    #[track_caller]
    fn as_failed(&self) -> Option<&Self::Failed>;

    fn ignore(self) {}
}

// FIXME: change where to Error after implementing thiserror
impl<T: Sized, E: Display> LogFailure for std::result::Result<T, E> {
    type Failed = E;

    #[inline]
    #[track_caller]
    fn as_failed(&self) -> Option<&Self::Failed> {
        match self {
            Ok(_) => None,
            Err(error) => Some(error),
        }
    }
}

impl<T> LogFailure for Option<T>
where
    T: Sized,
{
    type Failed = &'static str;

    #[inline]
    #[track_caller]
    fn as_failed(&self) -> Option<&Self::Failed> {
        match self {
            Some(_) => None,
            None => Some(&"Call returned no value"),
        }
    }
}

impl LogFailure for bool {
    type Failed = &'static str;

    #[inline]
    #[track_caller]
    fn as_failed(&self) -> Option<&Self::Failed> {
        (!self).then_some(&"Call returned false")
    }
}
