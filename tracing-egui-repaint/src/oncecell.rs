use tracing_oncecell::{OnceCellHandle, OnceCellLayer};
use super::RepaintLayer;

pub type RepaintOption = OnceCellLayer<RepaintLayer>;

pub struct RepaintHandle(OnceCellHandle<RepaintLayer>);

impl RepaintHandle {
    pub fn set_context(&self, ctx: egui::Context) {
        self.0.maybe_set(RepaintLayer(ctx));
    }
}

pub fn repaint_once_cell() -> (RepaintOption, RepaintHandle) {
    let (layer, handle) = tracing_oncecell::once_cell();

    (layer, RepaintHandle(handle))
}
