use std::io::Cursor;

use anyhow::{Context, Result};
use image::{DynamicImage, GenericImageView};

fn open_image(file_path: &str) -> Result<DynamicImage> {
    image::io::Reader::open(file_path)
        .context("failed to open image")?
        .with_guessed_format()
        .context("failed to guess format")?
        .decode()
        .context("failed to decode image")
}

/// returns (width, height)
pub fn get_image_dimensions(file_path: &str) -> Result<(u32, u32)> {
    let img = open_image(file_path)?;

    Ok(img.dimensions())
}

// returns webp image
pub fn scale_image(file_path: &str, max_w: Option<u32>, max_h: Option<u32>) -> Result<Vec<u8>> {
    const MAX_THUMBNAIL_SIZE: u32 = 96;

    let img = open_image(file_path)?;

    let mut bytes: Vec<u8> = Vec::new();

    let (width, height) = img.dimensions();

    let max_w = max_w.unwrap_or(width);
    let max_h = max_h.unwrap_or(height);

    let resized_img = if max_w <= MAX_THUMBNAIL_SIZE || max_h <= MAX_THUMBNAIL_SIZE {
        img.thumbnail(max_w, max_h)
    } else {
        img.resize(max_w, max_h, image::imageops::FilterType::Lanczos3)
    };

    resized_img.write_to(&mut Cursor::new(&mut bytes), image::ImageOutputFormat::WebP)?;

    Ok(bytes)
}

pub async fn scale_image_async(
    file_path: &str,
    max_w: Option<u32>,
    max_h: Option<u32>,
) -> Result<Vec<u8>> {
    let (send, recv) = tokio::sync::oneshot::channel();

    let file_path = file_path.to_string();
    rayon::spawn_fifo(move || {
        let result = scale_image(&file_path, max_w, max_h);

        let _ = send.send(result);
    });

    recv.await.expect("Panic in rayon::spawn")
}
