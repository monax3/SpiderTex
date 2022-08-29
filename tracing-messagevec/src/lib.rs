pub use append_only_vec::AppendOnlyVec;
use std::sync::{Arc, Weak};
pub use tracing::Level;
use tracing::{field::Visit, Event, Subscriber};
use tracing_subscriber::{layer::Context, registry, Layer};

#[derive(Clone)]
pub struct LogReader<T>(Arc<AppendOnlyVec<(Level, T)>>);

impl<T> LogReader<T> {
    pub fn iter(
        &self,
    ) -> impl Iterator<Item = &(Level, T)> + ExactSizeIterator + DoubleEndedIterator {
        self.0.iter()
    }

    pub fn len(&self) -> usize {
        self.0.len()
    }
}

impl<T> std::ops::Index<usize> for LogReader<T> {
    type Output = (Level, T);

    fn index(&self, index: usize) -> &Self::Output {
        &self.0[index]
    }
}

pub fn new<T>() -> (LogReader<T>, LogWriter<T>) {
    let reader = LogReader(Arc::new(AppendOnlyVec::new()));
    let writer = LogWriter(Arc::downgrade(&reader.0));
    (reader, writer)
}

pub struct LogWriter<T>(Weak<AppendOnlyVec<(Level, T)>>);

impl<S, T> Layer<S> for LogWriter<T>
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
