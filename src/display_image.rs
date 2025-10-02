use crate::err;
use image::GenericImageView;
use minifb::{Window, WindowOptions};
use std::process::exit;

pub fn display(data: Vec<u8>, path: String) {
    let img = image::load_from_memory(&data).unwrap_or_else(|e| {
        err!("Unsupported image type\n{}", e);
        exit(1);
    });

    let (width, height) = img.dimensions();
    let (mut last_width, mut last_height) = (width as usize, height as usize);

    let mut window = Window::new(
        &format!("{} - minau", path),
        last_width,
        last_height,
        WindowOptions {
            resize: true,
            ..WindowOptions::default()
        },
    )
    .unwrap();

    let mut buffer: Vec<u32> = img
        .to_rgb8()
        .chunks_exact(3)
        .map(|px| u32::from_be_bytes([0, px[0], px[1], px[2]]))
        .collect();

    window
        .update_with_buffer(&buffer, last_width, last_height)
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

            buffer = resized
                .to_rgba8()
                .chunks_exact(4)
                .map(|px| u32::from_be_bytes([0, px[0], px[1], px[2]]))
                .collect();

            last_width = width;
            last_height = height;
        }

        window
            .update_with_buffer(&buffer, last_width, last_height)
            .unwrap();
    }
}
