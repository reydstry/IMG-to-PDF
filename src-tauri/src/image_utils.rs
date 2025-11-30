use image::DynamicImage;

/// Load image dari bytes (uploaded file)
pub fn load_image_from_bytes(
    bytes: &[u8],
) -> Result<(Vec<u8>, u32, u32), Box<dyn std::error::Error>> {
    let img = image::load_from_memory(bytes)?;
    let rgba = img.to_rgba8();
    let (width, height) = rgba.dimensions();
    Ok((rgba.into_raw(), width, height))
}

/// Convert raw image data ke DynamicImage (untuk printpdf)
pub fn image_data_to_dynamic_image(data: &[u8], width: u32, height: u32) -> DynamicImage {
    DynamicImage::ImageRgba8(
        image::RgbaImage::from_raw(width, height, data.to_vec())
            .expect("Failed to create image from raw data"),
    )
}