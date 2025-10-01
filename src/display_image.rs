use std::process::exit;

use image::GenericImageView;
use minifb::{Window, WindowOptions};

use crate::err;

pub fn display(data: Vec<u8>, path: String) {
    let img = image::load_from_memory(&data).unwrap_or_else(|e| {
        err!("Unsupported image type\n{}", e);
        exit(1);
    });

    let (last_width, last_height) = img.dimensions();
    let (mut last_width, mut last_height) = (last_width as usize, last_height as usize);

    let mut window = Window::new(
        &format!("{} - minau", path),
        last_width as usize,
        last_height as usize,
        WindowOptions {
            resize: true,
            ..WindowOptions::default()
        },
    )
    .unwrap();

    let image_data = img.to_rgb8().into_raw();
    let mut buffer: Vec<u32> = image_data
        .chunks_exact(3)
        .map(|px| (px[0] as u32) << 16 | (px[1] as u32) << 8 | (px[2] as u32))
        .collect();

    window
        .update_with_buffer(&buffer, last_width as usize, last_height as usize)
        .unwrap();

    while window.is_open()
        && !window.is_key_down(minifb::Key::Escape)
        && !window.is_key_down(minifb::Key::Q)
    {
        let (width, height) = window.get_size();

        if width != last_width || height != last_height {
            let resized = img.resize_exact(
                width as u32,
                height as u32,
                image::imageops::FilterType::Nearest,
            );

            let image_data = resized.to_rgba8().into_raw();
            buffer = image_data
                .chunks_exact(4)
                .map(|px| (px[0] as u32) << 16 | (px[1] as u32) << 8 | (px[2] as u32))
                .collect();

            last_width = width;
            last_height = height;
        }
        
        window.update_with_buffer(&buffer, last_width, last_height).unwrap();
    }
}
