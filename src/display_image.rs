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
    let image_data = img.to_rgb8().into_raw();
    let buffer: Vec<u32> = image_data
        .chunks_exact(3)
        .map(|px| {
            let r = px[0] as u32;
            let g = px[1] as u32;
            let b = px[2] as u32;
            (r << 16) | (g << 8) | b
        })
        .collect();

    let mut window = Window::new(
        &path,
        width as usize,
        height as usize,
        WindowOptions::default(),
    )
    .unwrap();

    while window.is_open() && !window.is_key_down(minifb::Key::Escape) {
        window
            .update_with_buffer(&buffer, width as usize, height as usize)
            .unwrap();
    }
}
