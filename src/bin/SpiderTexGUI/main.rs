//! TODO: Catch thread panics
//! TODO: Is it srgb? is it premultiplied alpha?
//! TODO: Make the debug mode database a lazy static
//! TODO: LUT, search for day_01_lut
//! TODO: rename expected_highres_buffer_size etc
//! TODO: window icon
//! TODO: PNG metadata: WIC gets it wrong, image_rs needs to read the entire
//! move registry to a dashmap/global
//! TODO: currently does not list unrecognized files at all
//! LUT: 32-bit dds
//! _n: 32-bit PNG
//! monos: mono PNG

#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
#![allow(non_snake_case)]

const APP_TITLE: &str = "Spider-Man Texture Converter";

use spidertexlib::files::FileGroup;
use spidertexlib::inputs;
use spidertexlib::inputs::{Inputs, Job};
use spidertexlib::prelude::*;

pub mod gui;
pub mod log;
pub mod win32;

fn run() -> Result<()> {
    win32::init();
    log::init();
    // std::thread::spawn(|| {
    //     let src = include_str!(concat!(env!("CARGO_MANIFEST_DIR"),
    // "/src/headers.rs"));

    //     for (i, line) in src.lines().enumerate() {
    //         std::thread::sleep(std::time::Duration::from_millis(100));

    //         tracing::trace!("{:<4} {line}", i+1);
    //     }
    // });

    registry::load()?;

    let inputs = inputs::gather_from_args();
    let job = inputs::make_job(inputs);
    event!(DEBUG, ?job);

    match job {
        Job::Import(group) => gui::batch(Inputs {
            textures: vec![],
            images:   vec![group.0],
        }),
        Job::Export(group) => gui::batch(Inputs {
            textures: vec![group.0],
            images:   vec![],
        }),
        Job::Batch(inputs) => {
            gui::batch(inputs);
        }
        Job::Nothing => return error_message("No valid files were specified."),
    }
    Ok(())
}

fn main() {
    match run() {
        Ok(()) => std::process::exit(0),
        Err(error) => {
            let error = format!("{error}");
            win32::message_box_error(&error, APP_TITLE);
            std::process::exit(1);
        }
    }
}
