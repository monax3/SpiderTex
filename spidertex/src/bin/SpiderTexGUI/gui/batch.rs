use std::borrow::Cow;
use std::fs::File;
use std::io::BufReader;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::{channel, Receiver, Sender};
use std::sync::Arc;
use std::time::{Duration, Instant};

use camino::{Utf8Path, Utf8PathBuf};
use eframe::egui::{
    vec2,
    Button,
    CentralPanel,
    Context,
    Event,
    Label,
    Layout,
    ProgressBar,
    RadioButton,
    Rect,
    Response,
    RichText,
    ScrollArea,
    Sense,
    SidePanel,
    TextStyle,
    Ui,
    Widget,
};
use eframe::epaint::{Color32, FontId};
use eframe::{App, Frame};
use image::{DynamicImage, ImageFormat};
use parking_lot::Mutex;
use spidertexlib::convert::{convert, TaskResult};
use spidertexlib::files::{
    is_image_ext,
    is_texture_ext,
    output_files,
    FileFormat,
    FileGroupInfo,
    FileGroup,
    FileStatus,
    FileType,
    InputGroup,
    OutputFormat,
    Scanned,
};
use spidertexlib::images::{DxImport, Image, ImageRs};
use spidertexlib::prelude::*;

use super::{theme, widgets, AppWindow};
use crate::inputs::{Action, Inputs};
use crate::log;

pub fn batch(inputs: Inputs) {
    let action = inputs.default_action();
    let progress_rx = scan_thread(inputs);

    let batch = Batch::Setup {
        action,
        imports: Vec::new(),
        exports: Vec::new(),
        progress: 0.0,
        progress_rx,
        action_changed: true,
    };

    batch.run();
}

enum Batch {
    Setup {
        action:         Action,
        imports:        Vec<FileGroup<Scanned>>,
        exports:        Vec<FileGroup<Scanned>>,
        progress:       f32,
        progress_rx:    Receiver<(f32, FileGroup<Scanned>)>,
        action_changed: bool,
    },
    Running {
        num_tasks:  usize,
        results_rx: Receiver<TaskResult>,
        results:    Vec<TaskResult>,
    },
    Complete {
        results: Vec<TaskResult>,
    },
    Empty,
}

impl Batch {
    const fn in_progress(&self) -> bool { matches!(self, Self::Running { .. }) }

    const fn in_setup(&self) -> bool { matches!(self, Self::Setup { .. }) }

    fn set_action(&mut self, new_action: Action) {
        if let Self::Setup { action, .. } = self {
            *action = new_action;
        }
    }

    fn scan_in_progress(&self) -> bool {
        if let Self::Setup { progress, .. } = self {
            *progress < 1.0
        } else {
            false
        }
    }

    const fn action(&self) -> Action {
        if let Self::Setup { action, .. } = self {
            *action
        } else {
            Action::Ignore
        }
    }

    fn setup_screen(&mut self, ui: &mut Ui) {
        if let Self::Setup {
            action,
            imports,
            exports,
            progress,
            action_changed,
            ..
        } = self
        {
            let groups = match action {
                Action::Import => imports,
                Action::Export => exports,
                Action::Ignore | Action::Error => unreachable!(),
            };

            ui.add(
                ProgressBar::new(*progress)
                    .text(theme::text::warning("Scanning files"))
                    .show_percentage()
                    .animate(true),
            );

            ui.label(format!("{} groups", groups.len()));

            let mut scroll = ScrollArea::vertical().always_show_scroll(true);

            if *action_changed {
                *action_changed = false;
                scroll = scroll.scroll_offset(vec2(0.0, 0.0))
            };

            scroll.show(ui, |ui| {
                ui.set_width(ui.available_width());

                for group in groups.iter() {
                    inputs_and_outputs(ui, group);
                }
            });
        }
    }

    fn start_work(&mut self) {
        let (tx, rx) = channel::<TaskResult>();
        let new = Self::Running {
            num_tasks:  0,
            results_rx: rx,
            results:    Vec::new(),
        };

        if let Self::Setup {
            action,
            imports,
            exports,
            ..
        } = std::mem::replace(self, new)
        {
            let groups = match action {
                Action::Import => imports,
                Action::Export => exports,
                _ => unreachable!(),
            };
            std::thread::spawn(move || {
                let start = Instant::now();

                tx.send(TaskResult::Started(groups.len()))
                    .log_failure()
                    .ignore();

                for mut group in groups {
                    todo!()
                    // if let Some(result) = convert(group) {
                    //     tx.send(result)
                    //         .log_failure_as("Reporting completed task")
                    //         .ignore();
                    // }
                }

                let duration = start.elapsed();
                event!(INFO, "Completed in {duration:?}!");
                tx.send(TaskResult::Complete(duration))
                    .log_failure()
                    .ignore();
            });
        } else {
            panic!("Trying to start work while in the wrong state")
        }
    }

    fn status_screen(&mut self, ui: &mut Ui) {
        // if let Self::Running { results, .. } | Self::Complete { results } = self {
        //     ScrollArea::vertical().stick_to_bottom(true).show(ui, |ui| {
        //         for result in results {
        //             match result {
        //                 TaskResult::Started(_) => (),
        //                 TaskResult::Result(group, duration) => {
        //                     widgets::file_name_labels(
        //                         ui,
        //                         &group.inputs,
        //                         theme::text::normal,
        //                         theme::text::normal,
        //                     );
        //                     ui.indent("Outputs", |ui| {
        //                         if print_outputs(ui, &group.status) {
        //                             ui.label(theme::text::good(format!(
        //                                 "Processed in {duration:?}"
        //                             )));
        //                         } else {
        //                             ui.label(theme::text::error(format!("Failed in {duration:?}")));
        //                         }
        //                     });
        //                 }
        //                 TaskResult::Complete(duration) => {
        //                     ui.label(theme::text::good(format!("Completed in {duration:?}")));
        //                 }
        //             }
        //         }
        //     });
        // }
    }

    fn finish(&mut self) {
        if let Self::Running { results, .. } = std::mem::replace(self, Self::Empty) {
            *self = Self::Complete { results }
        }
    }
}

impl AppWindow for Batch {
    fn update(&mut self) {
        match self {
            Self::Setup {
                imports,
                exports,
                progress,
                progress_rx,
                ..
            } => {
                // event!(TRACE, "in update");
                for (pg, group) in progress_rx.try_iter() {
                    *progress = pg;
                    // event!(TRACE, name="received", %pg, ?group);

                    match &group.file_type() {
                        Some(FileType::Texture) => exports.push(group),
                        Some(FileType::Image(_)) => imports.push(group),
                        None => (),
                    }

                    // event!(TRACE, ?exports, ?imports);
                }
            }

            Self::Running {
                num_tasks,
                results_rx,
                results,
            } => {
                for result in results_rx.try_iter() {
                    match &result {
                        TaskResult::Started(num) => *num_tasks = *num,
                        TaskResult::Result(..) => results.push(result),
                        TaskResult::Complete(_) => {
                            results.push(result);
                            return self.finish();
                        }
                    }
                }
            }
            Self::Complete { .. } | Self::Empty => (),
        }
    }

    fn sidebar(&mut self, ui: &mut Ui) {
        if self.in_setup() {
            let action = self.action();

            ui.group(|ui| {
                ui.set_width(ui.available_width());
                ui.label("Mode");
                ui.set_enabled(!self.in_progress());
                if ui.radio(action == Action::Import, "Import").clicked() {
                    self.set_action(Action::Import);
                }
                if ui.radio(action == Action::Export, "Export").clicked() {
                    self.set_action(Action::Export);
                }
            });
            ui.group(|ui| {
                ui.label(
                    "The list to the right shows the names of files to be processed, followed by \
                     the names of the files they will be converted to.",
                );
                ui.separator();
                ui.label(theme::text::good(
                    "Files in this color have been validated and should convert with no issues.",
                ));
                ui.separator();
                ui.label(theme::text::warning(
                    "Files in this color have potential issues and can be converted but may not \
                     work correctly in the game. Mousing over the file will list the issues.",
                ));
                ui.separator();
                ui.label(theme::text::error(
                    "Files with errors will not be processed at all.",
                ));
                if self.scan_in_progress() {
                    ui.separator();
                    ui.label(
                        "The selected files are currently being scanned for potential issues. You \
                         can start the conversion without waiting for the scan to finish.",
                    );
                }
            });

            let mut rect = ui.available_rect_before_wrap();
            *rect.bottom_mut() -= theme::BUTTON_HEIGHT * 1.1;

            let button_rect = Rect::from_center_size(rect.center_bottom(), theme::button_size());
            if ui
                .put(button_rect, Button::new(theme::text::button("Convert")))
                .clicked()
            {
                self.start_work();
            }
            if log::debug_enabled() {
                widgets::debug_notification(ui, ui.max_rect());
            }
        } else if self.in_progress() {
            widgets::log_with_heading(ui, "Working");
        } else {
            widgets::log_with_heading(ui, "Done");
        }
    }

    fn main(&mut self, ui: &mut Ui) {
        match self {
            Self::Setup { .. } => self.setup_screen(ui),
            _ => self.status_screen(ui),
        }
    }

    fn can_close(&mut self) -> bool { !self.in_progress() }
}

fn scan_thread(inputs: Inputs) -> Receiver<(f32, FileGroup<Scanned>)> {
    let (tx, rx) = std::sync::mpsc::channel();
    let total = inputs.len();

    std::thread::spawn(move || {
        event!(INFO, "Starting scan");
        let mut current = 0;

        for group in inputs.into_iter().map(FileGroup).map(FileGroup::scan) {
            current += 1;
            let progress = current as f32 / total as f32;
            tx.send((progress, group))
                .log_failure_as("Failed to send scan update")
                .ignore();
            log::request_repaint();
        }
        event!(INFO, "Scan complete");
    });

    rx
}

fn inputs_and_outputs<G>(ui: &mut Ui, group: &FileGroup<G>)
where G: FileGroupInfo {
    let output_format = group.output_format().unwrap_or(&OutputFormat::Unknown);
    match group.input().as_ref() {
        FileStatus::Unknown => return,
        FileStatus::Ok(warnings, files) => {
            if warnings.is_empty() {
                widgets::file_name_labels(
                    ui,
                    files,
                    theme::text::highlight,
                    theme::text::highlight,
                );
            } else {
                widgets::file_name_labels(ui, files, theme::text::warning, |mut tooltip| {
                    tooltip.push_str("\n\n");

                    for warning in warnings.iter() {
                        tooltip.push_str(warning);
                        tooltip.push('\n');
                    }
                    theme::text::warning(tooltip)
                });
            }
            if let OutputFormat::Exact { format, .. } = output_format {
                ui.label(theme::text::normal(format.to_string()));
            }
        }
        FileStatus::Error(error) => {
            ui.label(theme::text::error(error));
            return;
        }
    }

    ui.indent("Outputs", |ui| match group.output().as_ref() {
        FileStatus::Unknown => (),
        FileStatus::Ok(warnings, files) => {
            if warnings.is_empty() {
                widgets::file_name_labels(ui, files, theme::text::good, theme::text::highlight);
            } else {
                widgets::file_name_labels(ui, files, theme::text::warning, |mut tooltip| {
                    tooltip.push_str("\n\n");

                    for warning in warnings.iter() {
                        tooltip.push_str(warning);
                        tooltip.push('\n');
                    }
                    theme::text::warning(tooltip)
                });
            }
        }
        FileStatus::Error(error) => {
            let mut tooltip = String::new();
            if let OutputFormat::Candidates(formats) = output_format {
                for format in formats.iter() {
                    tooltip.push_str(&format.to_string());
                    tooltip.push('\n');
                }
                tooltip.push('\n');
            }
            tooltip.push_str("This file must be manually converted.");
            ui.label(theme::text::error(error.to_string()))
                .on_hover_text(theme::text::highlight(tooltip));
        }
    });
}

fn print_outputs(ui: &mut Ui, status: &FileStatus) -> bool {
    // let is_successful = match status {
    //     FileStatus::Error(error) => {
    //         ui.label(theme::text::error(error));
    //         false
    //     }
    //     FileStatus::Unknown | FileStatus::Ok(None) | FileStatus::Warnings(None,
    // _) => false,     FileStatus::Ok(Some(outputs)) => {
    //         widgets::file_name_labels(ui, outputs, theme::text::good,
    // theme::text::normal);         true
    //     }
    //     FileStatus::Warnings(Some(outputs), warnings) => {
    //         widgets::file_name_labels(ui, outputs, theme::text::warning, |mut
    // text| {             text.push_str("\n\n");
    //             for warning in warnings {
    //                 text.push_str(warning);
    //                 text.push('\n');
    //             }
    //             theme::text::warning(text)
    //         });
    //         true
    //     }
    // };
    let is_successful = true;

    #[allow(clippy::let_and_return)]
    is_successful
}
