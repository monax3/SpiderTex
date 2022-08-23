use eframe::egui::{
    ColorImage,
    Context,
    Image,
    Rect,
    Response,
    Rounding,
    Sense,
    TextureHandle,
    Ui,
    Widget,
};
use eframe::epaint::{vec2, Vec2};
use image::DynamicImage;
use spidertexlib::formats::{guess_dimensions, ColorPlanes, TextureFormat};
use spidertexlib::prelude::*;
use spidertexlib::util::into_n_slices;

use super::theme;

fn to_colorimage(image: &DynamicImage) -> ColorImage {
    let scalar = std::cmp::max(image.width(), image.height()) as f32;

    let scale = theme::PREVIEW_SIZE / scalar;

    let (width, height) = (
        (image.width() as f32 * scale) as u32,
        (image.height() as f32 * scale) as u32,
    );

    let scaled = image
        .resize(width, height, image::imageops::CatmullRom)
        .to_rgba8();

    ColorImage::from_rgba_unmultiplied([width as usize, height as usize], scaled.as_raw())
}

fn to_texturehandle(ctx: &Context, image: ColorImage, name: impl Into<String>) -> TextureHandle {
    ctx.load_texture(name, image, Default::default())
}

pub struct Preview {
    images:      Vec<TextureHandle>,
    current:     usize,
    placeholder: TextureHandle,
}

fn placeholder(ctx: &Context) -> TextureHandle {
    ctx.load_texture("placeholder", ColorImage::example(), Default::default())
}

fn compressed_to_texturehandles(
    ctx: &Context,
    format: &TextureFormat,
    mut data: &[u8],
) -> Option<Vec<TextureHandle>> {
    if !matches!(format.planes(), ColorPlanes::Rgba) {
        // FIXME
        return None;
    }

    // FIXME: handle multiple formats
    let (dimensions, strip_header) = guess_dimensions(data.len(), &[format.clone()])?;
    if strip_header {
        data = &data[TEXTURE_HEADER_SIZE ..];
    }

    spidertexlib::dxtex::decompress_texture(
        format.dxgi_format,
        dimensions.width,
        dimensions.height,
        format.array_size,
        dimensions.mipmaps,
        data,
    )
    .log_failure()
    .ok()
    .and_then(|buf| {
        into_n_slices(&buf, format.array_size)
            .log_failure()
            .map(|bufs| rgba_to_texturehandles(ctx, [dimensions.width, dimensions.height], bufs))
    })
}

fn rgba_to_texturehandles<'a>(
    ctx: &Context,
    dimensions: [usize; 2],
    data: impl Iterator<Item = &'a [u8]> + 'a,
) -> Vec<TextureHandle> {
    data.enumerate()
        .map(|(i, img)| {
            let ci = ColorImage::from_rgba_unmultiplied(dimensions, img);
            ctx.load_texture(format!("Image {}", i + 1), ci, Default::default())
        })
        .collect()
}

impl Preview {
    pub fn replace_images(&mut self, ctx: &Context, images: &[DynamicImage]) { todo!() }

    pub fn from_images(ctx: &Context, images: &[DynamicImage]) -> Self {
        let images: Vec<TextureHandle> = images
            .iter()
            .enumerate()
            .map(|(i, img)| to_texturehandle(ctx, to_colorimage(img), format!("Image {}", i + 1)))
            .collect();

        Self {
            images,
            current: 0,
            placeholder: placeholder(ctx),
        }
    }

    pub fn from_buffer_and_format(ctx: &Context, data: &[u8], format: &TextureFormat) -> Self {
        let placeholder = placeholder(ctx);
        let images = compressed_to_texturehandles(ctx, format, data).unwrap_or_default();

        Self {
            images,
            current: 0,
            placeholder,
        }
    }

    fn num_images(&self) -> usize { self.images.len() }

    fn current_image(&self) -> &TextureHandle {
        self.images.get(self.current).unwrap_or(&self.placeholder)
    }

    fn next(&mut self) -> usize {
        self.current += 1;

        if self.current >= self.num_images() {
            self.current = 0;
        }

        self.current
    }

    fn prev(&mut self) -> usize {
        if self.current == 0 {
            self.current = self.num_images();
        }
        self.current -= 1;

        self.current
    }
}

impl Widget for &mut Preview {
    fn ui(self, ui: &mut Ui) -> Response {
        let current_image = self.current_image();
        let (rect, response) = ui.allocate_exact_size(ui.available_size(), Sense::hover());

        let image_size = current_image.size_vec2();

        let window_size = rect.size();
        let window_ar = rect.aspect_ratio();
        let image_ar = image_size.x / image_size.y;

        let scalar = if window_ar > image_ar {
            window_size.y / image_size.y
        } else {
            window_size.x / image_size.x
        };

        let scaled_size = image_size * scalar;
        let frame_rect = Rect::from_center_size(rect.center(), scaled_size);
        let image_rect = frame_rect.shrink(theme::PREVIEW_FRAME_SIZE);

        ui.painter()
            .rect_filled(frame_rect, Rounding::none(), theme::PREVIEW_FRAME_COLOR);

        Image::new(current_image, image_size).paint_at(ui, image_rect);

        if self.num_images() > 1 {
            let mut nav_rect = rect.center_bottom();
            let button_size = theme::nav_button_size();
            let distance = vec2(button_size.x * 0.75, 0.0);

            nav_rect.y -= 1.5 * button_size.y;

            let prev_rect = Rect::from_center_size(nav_rect - distance, button_size);
            let next_rect = Rect::from_center_size(nav_rect + distance, button_size);

            if ui
                .put(
                    prev_rect,
                    eframe::egui::Button::new(theme::button_text("<")),
                )
                .clicked()
            {
                self.prev();
                ui.ctx().request_repaint();
            }
            if ui
                .put(
                    next_rect,
                    eframe::egui::Button::new(theme::button_text(">")),
                )
                .clicked()
            {
                self.next();
                ui.ctx().request_repaint();
            }
        }

        response
    }
}
