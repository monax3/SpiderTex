use camino::{Utf8Path, Utf8PathBuf};
use eframe::egui::{
    vec2,
    Button,
    CentralPanel,
    Context,
    Event,
    Layout,
    RadioButton,
    Rect,
    Response,
    RichText,
    Sense,
    SidePanel,
    TextStyle,
    Ui,
    Widget,
};
use eframe::emath::Align;
use eframe::{App, Frame};
use image::DynamicImage;
use texturesofspiderman::formats::{probe_textures_2, TextureFormat};
use texturesofspiderman::prelude::*;

use super::preview::Preview;
use super::{theme, widgets};
use crate::log;

pub fn export_ui(export_files: Vec<Utf8PathBuf>, common_name: String) -> Result<()> {
    let mut registry = Registry::load()?;

    let (detected_formats, smallest_file, image_buffer) =
        probe_textures_2(&mut registry, &export_files)?;
    let detected_formats: Vec<TextureFormat> = detected_formats.into_iter().cloned().collect();

    let options = eframe::NativeOptions {
        initial_window_size: Some(theme::window_size()),
        min_window_size: Some(theme::window_size()),
        drag_and_drop_support: false,
        resizable: true,
        ..Default::default()
    };

    let selected_format = detected_formats
        .first()
        .cloned()
        .or_else(|| registry.formats.values().next().cloned())
        .expect("export_ui: Failed to select a format");

    let mut title = common_name.clone();

    if export_files.len() > 1 {
        title.push_str(&format!(" ({} files)", export_files.len()));
    }

    let common_name = smallest_file
        .parent()
        .expect("No parent folder")
        .join(common_name);

    let selections = ExportSelections {
        export_files,
        image_buffer,
        common_name,
        registry,
        detected_formats,
        selected_format,
    };

    eframe::run_native(
        &title,
        options,
        Box::new(move |cc| {
            cc.egui_ctx.set_visuals(theme::visuals());
            cc.egui_ctx.set_style(theme::style());

            log::set_ui_context(&cc.egui_ctx);

            let preview = Preview::from_buffer_and_format(
                &cc.egui_ctx,
                &selections.image_buffer,
                &selections.selected_format,
            );

            let state = ExportState::Preview(Some(Box::new(selections)));

            let ui = ExportUi { state, preview };

            Box::new(ui)
        }),
    );
    Ok(())
}

struct ExportUi {
    state:   ExportState,
    preview: Preview,
}

enum ExportState {
    Preview(Option<Box<ExportSelections>>),
    Running,
    Done,
}

struct ExportSelections {
    export_files: Vec<Utf8PathBuf>,
    image_buffer: Vec<u8>,

    common_name: Utf8PathBuf,

    registry: Registry,

    detected_formats: Vec<TextureFormat>,
    selected_format:  TextureFormat,
}

impl App for ExportUi {
    fn update(&mut self, ctx: &Context, _frame: &mut Frame) {
        for event in &ctx.input().events {
            if log::is_debug_toggle(event) {
                log::toggle_debug();
            }
        }

        SidePanel::left("Status")
            .min_width(theme::SIDEBAR_WIDTH)
            .resizable(false)
            .show(ctx, |ui| ui.add(&mut self.state));

        CentralPanel::default().show(ctx, |ui| {
            ui.add(&mut self.preview);
        });
    }
}

impl Widget for &mut ExportState {
    fn ui(self, ui: &mut Ui) -> Response {
        let response = match self {
            ExportState::Preview(selections) => widgets::log_with_heading(ui, "Ready"), /* preview_state(ui, selections), */
            ExportState::Running => widgets::log_with_heading(ui, "Working"),
            ExportState::Done => todo!(),
        };

        // if let Some(new_state) = new_state {
        //     *self = new_state;
        // }

        response
    }
}
