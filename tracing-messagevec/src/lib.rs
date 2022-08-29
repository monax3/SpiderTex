pub use append_only_vec::AppendOnlyVec;
use std::sync::{Arc, Weak};
use tracing::{field::Visit, Event, Subscriber};
use tracing_subscriber::{layer::Context, registry, Layer};
pub use tracing::Level;

pub struct LogMessages<T>(Arc<AppendOnlyVec<(Level, T)>>);

impl<T> LogMessages<T> {
    pub fn iter(&self) -> impl Iterator<Item = (Level, T)> + ExactSizeIterator + DoubleEndedIterator {
        self.0.iter()
    }
}

pub struct MessageVec<T>(Weak<AppendOnlyVec<(Level, T)>>);

impl<T> MessageVec<T> {
    pub fn new() -> (Self, Arc<AppendOnlyVec<(Level, T)>>) {
        let ret = Arc::new(AppendOnlyVec::new());
        (Self(Arc::downgrade(&ret)), ret)
    }
}

impl<S, T> Layer<S> for MessageVec<T>
where
    S: Subscriber + for<'a> registry::LookupSpan<'a>,
    T: std::fmt::Write + Default + 'static,
{
    fn on_event(&self, event: &Event<'_>, _ctx: Context<'_, S>) {
        if let Some(vec) = self.0.upgrade() {
            let message: T = MessageVisitor::record(event);
            vec.push((*event.metadata().level(), message));
        }
    }
}

struct MessageVisitor<T>(T);
impl<T> MessageVisitor<T> {
    fn record(event: &Event<'_>) -> T
    where
        T: Default + std::fmt::Write,
    {
        let mut writer = MessageVisitor(T::default());
        event.record(&mut writer);
        writer.0
    }
}
impl<T> Visit for MessageVisitor<T>
where
    T: std::fmt::Write,
{
    fn record_debug(&mut self, field: &tracing::field::Field, value: &dyn std::fmt::Debug) {
        if field.name() == "message" {
            let _ = write!(self.0, "{value:?}");
        }
    }
}
