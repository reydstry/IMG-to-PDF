use printpdf::*;
use std::io::BufWriter;

use crate::image_utils::image_data_to_dynamic_image;

pub fn generate_pdf(
    images: Vec<(Vec<u8>, u32, u32)>,
    orientation: &str,
    margin: &str,
) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
    if images.is_empty() {
        return Err("no images".into());
    }

    // Halaman EXACT sama dengan ukuran gambar pertama (1px -> 1mm di sini)
    let (_first_bytes, first_w, first_h) = &images[0];

    let mut page_w_mm = *first_w as f32 * 0.264583;
    let mut page_h_mm = *first_h as f32 * 0.264583;

    // orientasi hanya memutar frame, bukan scaling
    match orientation {
        "landscape" => {
            if page_h_mm > page_w_mm {
                std::mem::swap(&mut page_w_mm, &mut page_h_mm);
            }
        }
        _ => {
            if page_w_mm > page_h_mm {
                std::mem::swap(&mut page_w_mm, &mut page_h_mm);
            }
        }
    }

    let page_w = Mm(page_w_mm);
    let page_h = Mm(page_h_mm);

    let (doc, page1, layer1) =
        PdfDocument::new("Merged Images", page_w, page_h, "Layer 1");

    let mut current_page = doc.get_page(page1); // hindari pakai mutable 
    let mut current_layer = current_page.get_layer(layer1);

    for (i, (img_data, width, height)) in images.iter().enumerate() {
        if i > 0 {
            let (p, l) = doc.add_page(page_w, page_h, format!("Page {}", i + 1));
            current_page = doc.get_page(p);
            current_layer = current_page.get_layer(l);
        } // buatkan proses lebih clean

        let dyn_image = image_data_to_dynamic_image(img_data, *width, *height);
        let rgba = dyn_image.to_rgba8(); // bisa di bentuk fungsi tersendiri
        let (w, h) = rgba.dimensions();
        let raw_rgba = rgba.into_raw();

        let mut raw_rgb = Vec::with_capacity((w * h * 3) as usize); 
        for chunk in raw_rgba.chunks(4) {
            raw_rgb.push(chunk[0]);
            raw_rgb.push(chunk[1]);
            raw_rgb.push(chunk[2]);
        } // kalau bisa tidak 4 bungkus fungsi

        let image = Image::from(ImageXObject {
            width: Px(w as usize),
            height: Px(h as usize),
            color_space: ColorSpace::Rgb,
            bits_per_component: ColorBits::Bit8,
            interpolate: true,
            image_data: raw_rgb,
            image_filter: None,
            clipping_bbox: None,
        });

        // ukuran gambar
        let img_w = w as f32; // kenapa ga langsung dipanggil langsung
        let img_h = h as f32;

        // Tidak ada scaling tambahan: 1px gambar ~ 1mm di PDF
        let scale = 1.0_f32;

        // Tempatkan di (0,0)
        let x = 0.0_f32;
        let y = 0.0_f32;

        println!(
            "Image {}: {}x{}px, scale {}, x {}, y {}",
            i + 1,
            w,
            h,
            scale,
            x,
            y
        );

        image.add_to_layer(
            current_layer.clone(),
            ImageTransform {
                translate_x: Some(Mm(x)),
                translate_y: Some(Mm(y)),
                scale_x: Some(scale),
                scale_y: Some(scale),
                rotate: None,
                dpi: Some(96.0), // maksud dpi apa, dibungkus dalam fungsi
            },
        );
    }

    let mut pdf_bytes = Vec::new();
    doc.save(&mut BufWriter::new(&mut pdf_bytes))?; // hindari pakai mutable 
    Ok(pdf_bytes)
}