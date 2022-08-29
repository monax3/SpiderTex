use eframe::{
    egui::{Style, Visuals, CentralPanel},
    epaint::FontId,
    run_native, App, NativeOptions,
};
use texturesofspiderman_egui::LogWidget;
use tracing_messagevec::MessageVec;

pub fn run() {
    use tracing_subscriber::prelude::*;

    use tracing_subscriber::prelude::*;
    let (log_layer, log_handle) = MessageVec::<String>::new();
    let (repaint_layer, repaint_handle) =
        tracing_oncecell::OnceCellLayer::<tracing_egui_repaint::Repaint>::new();

    tracing_subscriber::registry()
        .with(
            tracing_subscriber::fmt::layer()
                .without_time()
                .with_target(false),
        )
        .with(log_layer.and_then(repaint_layer))
        .init();

    let window_size = eframe::egui::vec2(200.0, 600.0);

    let options = NativeOptions {
        initial_window_size: Some(window_size),
        min_window_size: Some(window_size),
        drag_and_drop_support: false,
        resizable: true,
        ..Default::default()
    };

    run_native(
        crate::APP_TITLE,
        options,
        Box::new(move |cc| {
            tracing_oncecell::maybe_set_oncecell(
                repaint_handle,
                tracing_egui_repaint::Repaint(cc.egui_ctx.clone()),
            );

            cc.egui_ctx.set_visuals(Visuals::dark());
            cc.egui_ctx.set_style(Style {
                override_font_id: Some(FontId::proportional(16.0)),
                ..Style::default()
            });

            let log = LogWidget(log_handle);

            Box::new(LogWindow { log })
        }),
    );
}

pub struct LogWindow {
    log: LogWidget,
}

impl App for LogWindow {
    fn update(&mut self, ctx: &eframe::egui::Context, frame: &mut eframe::Frame) {
        CentralPanel::new();
    }
}

// impl LogWindow {
//     pub fn run() {
//         let options = NativeOptions {
//             initial_window_size: window_size,
//             min_window_size: window_size,
//             drag_and_drop_support: false,
//             resizable: true,
//             ..Default::default()
//         };

//         run_native(
//             crate::APP_TITLE,
//             options,
//             Box::new(move |cc| {
//                 // app.ctx(&cc.egui_ctx);

//                 cc.egui_ctx.set_visuals(theme::visuals());
//                 cc.egui_ctx.set_style(theme::style());

//                 log::set_ui_context(&cc.egui_ctx);

//                 Box::new(Window(app))
//             }),
//         );
//     }
// }

// impl App for LogWindow {

// }
