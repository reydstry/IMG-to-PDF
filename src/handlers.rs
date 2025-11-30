use axum::{
    body::Body,
    extract::Multipart,
    http::{header, StatusCode},
    response::Response,
};
use tracing::info;

use crate::image_utils::load_image_from_bytes;
use crate::pdf::generate_pdf;

// health check
pub async fn health_check() -> &'static str {
    "OK"
}

pub async fn convert_images_to_pdf(
    mut multipart: Multipart,
) -> Result<Response, (StatusCode, String)> {
    info!("Received convert request");
    
    let mut image_data: Vec<(Vec<u8>, u32, u32)> = Vec::new();
    let mut file_count = 0;

    // Extract images dari multipart
    while let Some(field) = multipart.next_field().await.unwrap() {
        let name = field.name().unwrap_or("").to_string();
        
        if name == "images" {
            file_count += 1;
            let data = field.bytes().await.unwrap();
            
            info!("Processing file #{}, size: {} bytes", file_count, data.len());
            
            // Load image menggunakan utility function
            match load_image_from_bytes(&data) {
                Ok((img_bytes, width, height)) => {
                    info!("Image loaded: {}x{}", width, height);
                    image_data.push((img_bytes, width, height));
                }
                Err(e) => {
                    return Err((
                        StatusCode::BAD_REQUEST,
                        format!("Failed to load image #{}: {}", file_count, e),
                    ));
                }
            }
        }
    }

    // Validasi: minimal 1 image
    if image_data.is_empty() {
        return Err((
            StatusCode::BAD_REQUEST,
            "No images provided. Send files with field name 'images'".to_string(),
        ));
    }

    info!("Generating PDF from {} images...", image_data.len());

    // Generate PDF menggunakan pdf module
    match generate_pdf(image_data) {
        Ok(pdf_bytes) => {
            info!("PDF generated successfully! Size: {} bytes", pdf_bytes.len());
            
            // Return PDF sebagai HTTP response
            Ok(Response::builder()
                .status(StatusCode::OK)
                .header(header::CONTENT_TYPE, "application/pdf")
                .header(
                    header::CONTENT_DISPOSITION,
                    "attachment; filename=\"merged.pdf\"",
                )
                .body(Body::from(pdf_bytes))
                .unwrap())
        }
        Err(e) => Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Failed to generate PDF: {}", e),
        )),
    }
}