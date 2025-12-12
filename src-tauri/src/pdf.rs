use printpdf::{PdfDocument, Image, ImageXObject, ImageTransform, Mm, Px, ColorSpace, ColorBits};
use crate::image_utils::image_data_to_dynamic_image;

fn convert_rgba_to_rgb(rgba_data: &[u8]) -> Vec<u8> {
    rgba_data
        .chunks_exact(4)
        .flat_map(|pixel| pixel.iter().take(3).copied())
        .collect()
}

pub fn generate_single_page_pdf(
    image_data: (Vec<u8>, u32, u32),
    orientation: &str,
    _margin: &str,
) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
    let (img_bytes, width, height) = image_data;

    let aspect_ratio = width as f32 / height as f32;
    
    let (page_w, page_h) = match orientation {
        "landscape" => {
            if aspect_ratio >= 1.0 {
                (Mm(297.0), Mm(297.0 / aspect_ratio))
            } else {
                (Mm(297.0 * aspect_ratio), Mm(297.0))
            }
        }
        _ => {
            if aspect_ratio >= 1.0 {
                (Mm(297.0 / aspect_ratio), Mm(297.0))
            } else {
                (Mm(297.0), Mm(297.0 * aspect_ratio))
            }
        }
    };

    let (doc, page1, layer1) = PdfDocument::new("Image", page_w, page_h, "Layer 1");
    
    let current_page = doc.get_page(page1);
    let current_layer = current_page.get_layer(layer1);

    let dyn_image = image_data_to_dynamic_image(&img_bytes, width, height);
    let rgba = dyn_image.to_rgba8();
    let (w, h) = rgba.dimensions();
    
    let raw_rgb = convert_rgba_to_rgb(&rgba.into_raw());

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

    let mm_to_pt = 72.0 / 25.4;
    let scale = (page_w.0 * mm_to_pt) / w as f32;
    
    let transform = ImageTransform {
        translate_x: Some(Mm(0.0)),
        translate_y: Some(Mm(0.0)),
        scale_x: Some(scale),
        scale_y: Some(scale),
        rotate: None,
        dpi: None,
    };
    
    image.add_to_layer(current_layer, transform);

    Ok(doc.save_to_bytes()?)
}

pub fn merge_pdfs(
    pdf_bytes_list: Vec<Vec<u8>>,
) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
    if pdf_bytes_list.is_empty() {
        return Err("No PDFs to merge".into());
    }

    if pdf_bytes_list.len() == 1 {
        return Ok(pdf_bytes_list.into_iter().next().unwrap());
    }

    let documents: Result<Vec<_>, _> = pdf_bytes_list
        .iter()
        .map(|bytes| lopdf::Document::load_mem(bytes))
        .collect();
    
    let documents = documents?;

    let mut merged_doc = lopdf::Document::with_version("1.5");
    let mut next_id = 1u32;
    
    let catalog_id = (next_id, 0);
    next_id += 1;
    
    let pages_id = (next_id, 0);
    next_id += 1;
    
    let mut all_page_ids = Vec::new();

    for current_doc in &documents {
        let page_map = current_doc.get_pages();
        
        for (_, &page_object_id) in page_map.iter() {
            if let Ok(page_obj) = current_doc.get_object(page_object_id) {
                let new_page_id = (next_id, 0);
                next_id += 1;
                
                merged_doc.objects.insert(new_page_id, page_obj.clone());
                all_page_ids.push(new_page_id);
            }
        }
    }
    
    let kids_array: Vec<lopdf::Object> = all_page_ids
        .iter()
        .map(|&id| lopdf::Object::Reference(id))
        .collect();
    
    let mut pages_dict = lopdf::Dictionary::new();
    pages_dict.set("Type", lopdf::Object::Name(b"Pages".to_vec()));
    pages_dict.set("Kids", lopdf::Object::Array(kids_array));
    pages_dict.set("Count", lopdf::Object::Integer(all_page_ids.len() as i64));
    
    merged_doc.objects.insert(pages_id, lopdf::Object::Dictionary(pages_dict));
    
    let mut catalog = lopdf::Dictionary::new();
    catalog.set("Type", lopdf::Object::Name(b"Catalog".to_vec()));
    catalog.set("Pages", lopdf::Object::Reference(pages_id));
    
    merged_doc.objects.insert(catalog_id, lopdf::Object::Dictionary(catalog));
    
    merged_doc.trailer.set("Root", lopdf::Object::Reference(catalog_id));

    let output = save_document_to_bytes(merged_doc)?;
    Ok(output)
}

fn save_document_to_bytes(
    mut doc: lopdf::Document,
) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
    let mut bytes = Vec::new();
    doc.save_to(&mut bytes)?;
    Ok(bytes)
}