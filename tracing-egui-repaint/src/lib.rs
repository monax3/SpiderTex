use tracing::Subscriber;
use tracing_subscriber::{layer::Context, Layer};

pub struct Repaint(pub egui::Context);

impl From<&egui::Context> for Repaint {
    fn from(ctx: &egui::Context) -> Self {
        Self(ctx.clone())
    }
}

impl<S: Subscriber> Layer<S> for Repaint {
    fn on_event(&self, _event: &tracing::Event<'_>, _ctx: Context<'_, S>) {
        self.0.request_repaint()
    }
}
