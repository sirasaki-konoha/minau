use std::process::exit;

use image::GenericImageView;
use minifb::{Window, WindowOptions};

use crate::err;

pub fn display(data: Vec<u8>, path: String) {
    let img = image::load_from_memory(&data).unwrap_or_else(|e| {
        err!("Unsupported image type\n{}", e);
        exit(1);
    });

    let (width, height) = img.dimensions();

    let mut window = Window::new(
        &path,
        width as usize,
        height as usize,
        WindowOptions {
            resize: true,
            ..WindowOptions::default()
        },
    )
    .unwrap();

    while window.is_open()
        && !window.is_key_down(minifb::Key::Escape)
        && !window.is_key_down(minifb::Key::Q)
    {
        let (width, height) = window.get_size();

        let resized = img.resize_exact(
            width as u32,
            height as u32,
            image::imageops::FilterType::Nearest,
        );

        let image_data = resized.to_rgba8().into_raw();
        let buffer: Vec<u32> = image_data
            .chunks_exact(4)
            .map(|px| (px[0] as u32) << 16 | (px[1] as u32) << 8 | (px[2] as u32))
            .collect();

        window.update_with_buffer(&buffer, width, height).unwrap();
    }
}
