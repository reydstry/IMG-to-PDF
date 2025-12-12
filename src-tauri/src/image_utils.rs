use image::DynamicImage;
use rayon::prelude::*;

pub fn load_image_from_bytes(
    bytes: &[u8],
) -> Result<(Vec<u8>, u32, u32), Box<dyn std::error::Error>> {
    let img = image::load_from_memory(bytes)?;
    let rgba = img.to_rgba8();
    let (width, height) = rgba.dimensions();
    Ok((rgba.into_raw(), width, height))
}

pub fn load_images_parallel(
    image_bytes: Vec<Vec<u8>>,
) -> Vec<Result<(Vec<u8>, u32, u32), String>> {
    image_bytes
        .par_iter()
        .map(|bytes| {
            load_image_from_bytes(bytes)
                .map_err(|e| format!("Failed to load image: {}", e))
        })
        .collect()
}

pub fn image_data_to_dynamic_image(data: &[u8], width: u32, height: u32) -> DynamicImage {
    DynamicImage::ImageRgba8(
        image::RgbaImage::from_raw(width, height, data.to_vec())
            .expect("Failed to create image from raw data"),
    )
}

pub fn generate_pdfs_parallel(
    image_data: Vec<(Vec<u8>, u32, u32)>,
    orientation: &str,
    margin: &str,
) -> Vec<Result<Vec<u8>, String>> {
    image_data
        .par_iter()
        .enumerate()
        .map(|(i, data)| {
            println!("Processing image {} in parallel...", i + 1);
            crate::pdf::generate_single_page_pdf(
                data.clone(),
                orientation,
                margin,
            )
            .map_err(|e| format!("Image {}: {}", i + 1, e))
        })
        .collect()
}