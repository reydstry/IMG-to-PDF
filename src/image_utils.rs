use image::DynamicImage;
// HAPUS: use std::io::Cursor;

/// Load image dari bytes (uploaded file)
pub fn load_image_from_bytes(
    bytes: &[u8],
) -> Result<(Vec<u8>, u32, u32), Box<dyn std::error::Error>> {
    // Load image dari bytes
    let img = image::load_from_memory(bytes)?;
    
    // Convert ke RGBA8 format
    let rgba = img.to_rgba8();
    let (width, height) = rgba.dimensions();
    
    // Return: (raw_bytes, width, height)
    Ok((rgba.into_raw(), width, height))
}

/// Convert raw image data ke DynamicImage (untuk printpdf)
pub fn image_data_to_dynamic_image(data: &[u8], width: u32, height: u32) -> DynamicImage {
    DynamicImage::ImageRgba8(
        image::RgbaImage::from_raw(width, height, data.to_vec())
            .expect("Failed to create image from raw data"),
    )
}