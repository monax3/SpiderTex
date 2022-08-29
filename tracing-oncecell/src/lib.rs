//! TODO: Make a no-std version using once_cell::Race

use std::sync::{Arc, Weak};

use once_cell::sync::OnceCell;
use tracing::{span, subscriber::Interest, Metadata, Subscriber};
use tracing_subscriber::{
    layer::Context,
    Layer,
};

pub struct OnceCellHandle<T>(Weak<OnceCell<T>>);
impl<T> OnceCellHandle<T> {
    pub fn maybe_set(&self, value: T) {
        if let Some(arc) = self.0.upgrade() {
            let _ = arc.set(value);
        }
    }
}

pub fn once_cell<T>() -> (OnceCellLayer<T>, OnceCellHandle<T>) {
    let arc = Arc::new(OnceCell::new());
    let weak = Arc::downgrade(&arc);
    (OnceCellLayer(arc), OnceCellHandle(weak))
}

pub struct OnceCellLayer<T>(Arc<OnceCell<T>>);

impl<T, S> Layer<S> for OnceCellLayer<T>
where
    T: Layer<S>,
    S: Subscriber,
{
    fn register_callsite(&self, metadata: &'static Metadata<'static>) -> Interest {
        if let Some(layer) = self.0.get() {
            layer.register_callsite(metadata)
        } else {
            Interest::sometimes()
        }
    }
    fn enabled(&self, metadata: &Metadata<'_>, ctx: Context<'_, S>) -> bool {
        self.0
            .get()
            .map_or(true, |layer| layer.enabled(metadata, ctx))
    }

    // FIXME: can make this work with a delayed thing but nah
    // fn on_layer(&mut self, subscriber: &mut S) {
    //     let _ = subscriber;
    // }

    fn on_new_span(&self, attrs: &span::Attributes<'_>, id: &span::Id, ctx: Context<'_, S>) {
        if let Some(layer) = self.0.get() {
            layer.on_new_span(attrs, id, ctx)
        }
    }

    fn max_level_hint(&self) -> Option<tracing::metadata::LevelFilter> {
        self.0.get().and_then(|layer| layer.max_level_hint())
    }

    fn on_record(&self, span: &span::Id, values: &span::Record<'_>, ctx: Context<'_, S>) {
        if let Some(layer) = self.0.get() {
            layer.on_record(span, values, ctx)
        }
    }

    fn on_follows_from(&self, span: &span::Id, follows: &span::Id, ctx: Context<'_, S>) {
        if let Some(layer) = self.0.get() {
            layer.on_follows_from(span, follows, ctx)
        }
    }

    fn event_enabled(&self, event: &tracing::Event<'_>, ctx: Context<'_, S>) -> bool {
        self.0
            .get()
            .map_or(true, |layer| layer.event_enabled(event, ctx))
    }

    fn on_event(&self, event: &tracing::Event<'_>, ctx: Context<'_, S>) {
        if let Some(layer) = self.0.get() {
            layer.on_event(event, ctx)
        }
    }

    fn on_enter(&self, id: &span::Id, ctx: Context<'_, S>) {
        if let Some(layer) = self.0.get() {
            layer.on_enter(id, ctx)
        }
    }

    fn on_exit(&self, id: &span::Id, ctx: Context<'_, S>) {
        if let Some(layer) = self.0.get() {
            layer.on_exit(id, ctx)
        }
    }

    fn on_close(&self, id: span::Id, ctx: Context<'_, S>) {
        if let Some(layer) = self.0.get() {
            layer.on_close(id, ctx)
        }
    }

    fn on_id_change(&self, old: &span::Id, new: &span::Id, ctx: Context<'_, S>) {
        if let Some(layer) = self.0.get() {
            layer.on_id_change(old, new, ctx)
        }
    }
}
