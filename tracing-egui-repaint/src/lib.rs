use tracing::Subscriber;
use tracing_subscriber::{layer::Context, Layer};

#[cfg(feature = "tracing-oncecell")]
pub mod oncecell;

pub struct RepaintLayer(pub egui::Context);

impl From<egui::Context> for RepaintLayer {
    fn from(ctx: egui::Context) -> Self {
        Self(ctx)
    }
}

impl<S: Subscriber> Layer<S> for RepaintLayer {
    fn on_event(&self, _event: &tracing::Event<'_>, _ctx: Context<'_, S>) {
        self.0.request_repaint()
    }
}
