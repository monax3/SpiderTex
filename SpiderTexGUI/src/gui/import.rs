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
use eframe::App;
use image::DynamicImage;
use texturesforspiderman::formats::TextureFormat;
use texturesforspiderman::prelude::*;
use texturesforspiderman::util;

use super::preview::Preview;
use super::{theme, widgets};
use crate::log;

pub fn import_ui(import_files: Vec<Utf8PathBuf>, common_name: String) -> Result<()> {
    let registry = Registry::load()?;

    let import_images: Vec<DynamicImage> = import_files
        .iter()
        .map(|file| image::open(&file))
        .collect::<Result<_, _>>()?;

    let options = eframe::NativeOptions {
        initial_window_size: Some(theme::window_size()),
        min_window_size: Some(theme::window_size()),
        drag_and_drop_support: false,
        resizable: true,
        ..Default::default()
    };

    let selected_format = registry.formats.keys().next().unwrap().clone();

    let mut title = common_name.clone();

    if import_files.len() > 1 {
        title.push_str(&format!(" ({} files)", import_files.len()));
    }

    let common_name = import_files
        .first()
        .and_then(|f| f.parent())
        .ok_or_else(|| Error::message("Internal error"))?
        .join(common_name);

    let selections = ImportSelections {
        import_files,
        import_images,
        common_name,
        registry,
        selected_format,
        detected_format: None,
    };

    eframe::run_native(
        &title,
        options,
        Box::new(move |cc| {
            cc.egui_ctx.set_visuals(theme::visuals());
            cc.egui_ctx.set_style(theme::style());

            log::set_ui_context(&cc.egui_ctx);

            let preview = Preview::from_images(&cc.egui_ctx, &selections.import_images);
            let state = ImportState::Preview(Some(Box::new(selections)));

            let ui = ImportUi { state, preview };

            Box::new(ui)
        }),
    );

    Ok(())
}

struct ImportUi {
    state:   ImportState,
    preview: Preview,
}

enum ImportState {
    Preview(Option<Box<ImportSelections>>),
    Running,
    Done,
}

struct ImportSelections {
    import_files:  Vec<Utf8PathBuf>,
    import_images: Vec<DynamicImage>,
    common_name:   Utf8PathBuf,

    registry: Registry,

    detected_format: Option<FormatId>,
    selected_format: FormatId,
}

fn output_description(format: &TextureFormat) -> &'static str {
    if format.has_highres() {
        "This selection will produce two files, a .texture and a .raw file. To apply, use the \
         modding tool to replace the standard resolution texture with the .texture file, and the \
         high-resolution texture with the .raw file. Both textures need to be replaced for the new \
         texture to apply correctly."
    } else {
        "This selection will produce a .texture. To apply, use the modding tool to replace the \
         original texture with this new .texture file."
    }
}

// fn iter_array_output_files<'a>(input_file: &'a Utf8Path, is_highres: bool) ->
// impl Iterator<Item = Utf8PathBuf> + 'a {     let mut iter = (1_usize
// ..).into_iter();

//     std::iter::from_fn(move ||
//         Some(name_output_file(input_file, iter.next(), is_highres))
//     )
// }

// fn output_files<'a>(format: &TextureFormat, input_file: &'a Utf8Path) ->
// Vec<Utf8PathBuf> {     let array_size = format.array_size;
//     let has_highres = format.has_highres();

//     if array_size > 1 {
//         iter_array_output_files(input_file,
// false).take(array_size).chain(iter_array_output_files(input_file,
// true).take(array_size)).collect()     } else {
//         vec![
//             name_output_file(input_file, None, false),
//             name_output_file(input_file, None, true),
//         ]
//     }
// }

// fn name_output_file(input_file: &Utf8Path, array_index: Option<usize>,
// is_highres: bool) -> Utf8PathBuf {     let mut input_name =
// input_file.file_stem().unwrap().to_owned();     input_name.push_str("_custom"
// );

//     if is_highres {
//         input_file.with_file_name(input_name).with_extension("raw")
//     } else {
//         input_file
//             .with_file_name(input_name)
//             .with_extension("texture")
//     }
// }

fn name_output_files(input_file: &Utf8Path) -> [Utf8PathBuf; 2] {
    [
        name_output_file(input_file, false),
        name_output_file(input_file, true),
    ]
}

fn name_output_file(input_file: &Utf8Path, is_highres: bool) -> Utf8PathBuf {
    let mut input_name = input_file.file_stem().unwrap().to_owned();
    input_name.push_str("_custom");

    if is_highres {
        input_file.with_file_name(input_name).with_extension("raw")
    } else {
        input_file
            .with_file_name(input_name)
            .with_extension("texture")
    }
}

impl App for ImportUi {
    fn update(&mut self, ctx: &Context, _frame: &mut eframe::Frame) {
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

impl Widget for &mut ImportState {
    fn ui(self, ui: &mut Ui) -> Response {
        let (new_state, response) = match self {
            ImportState::Preview(selections) => preview_state(ui, selections),
            ImportState::Running => (None, widgets::log_with_heading(ui, "Working")),
            ImportState::Done => todo!(),
        };

        if let Some(new_state) = new_state {
            *self = new_state;
        }

        response
    }
}

fn preview_state(
    ui: &mut Ui,
    selections_opt: &mut Option<Box<ImportSelections>>,
) -> (Option<ImportState>, Response) {
    let line_height = ui.text_style_height(&TextStyle::Body);
    let height = ui.available_height();
    let max_scroll_height = height - theme::STATUS_HEIGHT;
    let mut new_state = None;

    let selections =
        selections_opt
            .as_mut()
            .expect(concat!("Internal error at ", file!(), ":", line!()));

    ui.label("Output format:");
    let items = selections.registry.formats.len();

    eframe::egui::ScrollArea::vertical()
        .max_height(max_scroll_height)
        .show_rows(ui, line_height, items, |ui, row_range| {
            ui.set_min_width(ui.available_width());
            let iter = selections
                .registry
                .formats
                .keys()
                .skip(row_range.start)
                .take(row_range.end - row_range.start);

            for key in iter {
                let format = selections.registry.formats.get(key);

                let selected = &selections.selected_format == key;
                let detected = selections
                    .detected_format
                    .as_ref()
                    .map_or(false, |det| det == key);

                let mut text = RichText::new(key.to_string());

                if detected {
                    text = text.color(theme::TEXT_HIGHLIGHT_COLOR);
                }
                if selected {
                    text = text.strong();
                }

                let radio = RadioButton::new(selected, text);
                let enabled =
                    format.map_or(true, |f| f.array_size == selections.import_files.len());

                if ui.add_enabled(enabled, radio).clicked() {
                    if selections.detected_format.is_none() {
                        selections.detected_format = Some(key.clone());
                    }
                    selections.selected_format = key.clone();
                }
            }
        });

    ui.group(|ui| {
        let format = selections.registry.get(selections.selected_format);

        // ui.label(output_description(format));
        // ui.separator();

        let files = name_output_files(&selections.common_name);

        if format.has_highres() {
            ui.label("Using this format will create the following files:");
            ui.add_space(theme::EXTRA_SPACING);
            ui.label(theme::highlight_text(files[0].file_name().unwrap()));
            ui.add_space(theme::EXTRA_SPACING);
            ui.label(theme::highlight_text(files[1].file_name().unwrap()));
        } else {
            ui.label("Using format will create the following file:");
            ui.add_space(theme::EXTRA_SPACING);
            ui.label(theme::highlight_text(files[0].file_name().unwrap()));
        }
    });

    ui.add_space(theme::EXTRA_SPACING);

    let button_size = theme::button_size();

    let (rect, response) =
        ui.allocate_exact_size(vec2(ui.available_width(), theme::BUTTON_HEIGHT), Sense {
            click:     false,
            drag:      false,
            focusable: false,
        });

    let button_rect = Rect::from_center_size(rect.center(), button_size);

    if ui
        .put(button_rect, Button::new(theme::button_text("Convert")))
        .clicked()
    {
        let selections =
            selections_opt
                .take()
                .expect(concat!("Internal error at ", file!(), ":", line!()));
        let rx = launch_import(ui.ctx(), selections);

        new_state = Some(ImportState::Running);
    }

    if log::debug_enabled() {
        debug_notification(ui, ui.max_rect());
    }

    (new_state, response)
}

fn debug_notification(ui: &mut Ui, mut rect: Rect) {
    rect.min.y = rect.max.y - 24.0;

    let painter = ui.painter();

    let fill = ui.visuals().widgets.noninteractive.bg_fill;

    painter.rect(
        rect,
        eframe::egui::Rounding::same(10.0),
        fill,
        eframe::egui::Stroke::new(2.0, eframe::egui::Color32::RED),
    );
    painter.text(
        rect.center(),
        eframe::egui::Align2::CENTER_CENTER,
        "DEBUG MODE",
        eframe::egui::FontId::monospace(24.0),
        eframe::egui::Color32::RED,
    );
}

use std::sync::mpsc::{channel, Receiver, Sender};

fn launch_import(ctx: &Context, selections: Box<ImportSelections>) -> Receiver<Result<()>> {
    let ctx = ctx.clone();

    let (tx, rx) = channel();

    std::thread::spawn(move || {
        let result = util::catch_panics(move || {
            let format = selections.registry.get(selections.selected_format);

            // let [sd_name, hd_name] = name_output_files(&selections.import_file);

            texturesforspiderman::convert_to_texture(
                format,
                &selections.import_images,
                name_output_files(&selections.common_name),
            )
        });

        let _ignore = tx.send(result);
        ctx.request_repaint();
    });

    rx
}
