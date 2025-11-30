use printpdf::*;
use std::io::BufWriter;

use crate::image_utils::image_data_to_dynamic_image;

/// Generate PDF dari multiple images
pub fn generate_pdf(
    images: Vec<(Vec<u8>, u32, u32)>,
) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
    // Membuat pdf
    let (doc, page1, layer1) = PdfDocument::new(
        "Merged Images",
        Mm(210.0),  // A4 width
        Mm(297.0),  // A4 height
        "Layer 1",
    );

    let mut current_page = doc.get_page(page1);
    let mut current_layer = current_page.get_layer(layer1);

    // Loop setiap image
    for (i, (img_data, width, height)) in images.iter().enumerate() {
        // Untuk page ke-2 dan seterusnya, tambah page baru
        if i > 0 {
            let (page_idx, layer_idx) = doc.add_page(
                Mm(210.0),
                Mm(297.0),
                format!("Page {}", i + 1),
            );
            current_page = doc.get_page(page_idx);
            current_layer = current_page.get_layer(layer_idx);
        }

        // Convert data mentah ke DynamicImage, lalu ke RGBA8
        let dyn_image = image_data_to_dynamic_image(img_data, *width, *height);
        let rgba_image = dyn_image.to_rgba8();

        // Bangun ImageXObject manual dari pixel RGBA
        let image = Image::from(ImageXObject {
            width: Px(*width as usize),
            height: Px(*height as usize),
            color_space: ColorSpace::Rgba,
            bits_per_component: ColorBits::Bit8,
            interpolate: true,
            image_data: rgba_image.into_raw(),
            image_filter: None,
            clipping_bbox: None,
            smask: None,
        });

        // Calculate scaling untuk fit di A4
        let page_width_mm = 210.0;
        let page_height_mm = 297.0;
        let img_width_mm = (*width as f32) * 0.264583;  // px to mm
        let img_height_mm = (*height as f32) * 0.264583;

        let scale_x = page_width_mm / img_width_mm;
        let scale_y = page_height_mm / img_height_mm;
        let scale = scale_x.min(scale_y) * 0.9;  // 90% untuk margin

        // Calculate position untuk center image
        let final_width = img_width_mm * scale;
        let final_height = img_height_mm * scale;
        let x = (page_width_mm - final_width) / 2.0;
        let y = (page_height_mm - final_height) / 2.0;

        // Add image ke layer dengan transform
        image.add_to_layer(
            current_layer.clone(),
            ImageTransform {
                translate_x: Some(Mm(x)),
                translate_y: Some(Mm(y)),
                scale_x: Some(scale),
                scale_y: Some(scale),
                rotate: None,
                dpi: None,
            },
        );
    }

    // Save PDF ke bytes menggunakan BufWriter
    let mut pdf_bytes = Vec::new();
    doc.save(&mut BufWriter::new(&mut pdf_bytes))?;

    Ok(pdf_bytes)
}