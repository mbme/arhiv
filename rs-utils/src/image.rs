use std::io::{BufRead, Cursor, Seek};

use anyhow::{Context, Result};
use image::{DynamicImage, GenericImageView, ImageReader};
use qrcode::{QrCode, render::svg};

fn open_image(img: impl BufRead + Seek) -> Result<DynamicImage> {
    ImageReader::new(img)
        .with_guessed_format()
        .context("failed to guess format")?
        .decode()
        .context("failed to decode image")
}

// returns webp image
fn scale_image(img: &DynamicImage, max_w: Option<u32>, max_h: Option<u32>) -> Result<Vec<u8>> {
    const MAX_THUMBNAIL_SIZE: u32 = 96;

    let mut bytes: Vec<u8> = Vec::new();

    let (width, height) = img.dimensions();

    let max_w = max_w.unwrap_or(width);
    let max_h = max_h.unwrap_or(height);

    let resized_img = if max_w <= MAX_THUMBNAIL_SIZE || max_h <= MAX_THUMBNAIL_SIZE {
        img.thumbnail(max_w, max_h)
    } else {
        img.resize(max_w, max_h, image::imageops::FilterType::Lanczos3)
    };

    resized_img.write_to(&mut Cursor::new(&mut bytes), image::ImageFormat::WebP)?;

    Ok(bytes)
}

pub fn scale_image_file(
    img_reader: impl BufRead + Seek,
    max_w: Option<u32>,
    max_h: Option<u32>,
) -> Result<Vec<u8>> {
    let img = open_image(img_reader)?;

    scale_image(&img, max_w, max_h)
}

pub fn generate_qrcode_svg(data: &[u8]) -> Result<Vec<u8>> {
    let qrcode = QrCode::with_error_correction_level(data, qrcode::EcLevel::Q)
        .context("Failed to generate qrcode")?;

    let result = qrcode
        .render()
        .dark_color(svg::Color("#000000"))
        .light_color(svg::Color("#ffffff"))
        .build();

    Ok(result.into_bytes())
}
