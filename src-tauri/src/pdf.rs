use printpdf::{PdfDocument, Image, ImageXObject, ImageTransform, Mm, Px, ColorSpace, ColorBits};
use lopdf::{Document, Object, ObjectId, Dictionary};
use std::collections::HashMap;
use std::time::Instant;
use crate::image_utils::image_data_to_dynamic_image;

const DPI: f32 = 150.0;
const INCH_TO_MM: f32 = 25.4;
const WHITE_BG_R: f32 = 255.0;
const WHITE_BG_G: f32 = 255.0;
const WHITE_BG_B: f32 = 255.0;

fn convert_rgba_to_rgb_with_white_bg(rgba_data: &[u8]) -> Vec<u8> {
    let mut rgb_data = Vec::with_capacity((rgba_data.len() / 4) * 3);
    
    for pixel in rgba_data.chunks_exact(4) {
        let r = pixel[0] as f32;
        let g = pixel[1] as f32;
        let b = pixel[2] as f32;
        let alpha = pixel[3] as f32 / 255.0;
        
        let blended_r = (r * alpha + WHITE_BG_R * (1.0 - alpha)) as u8;
        let blended_g = (g * alpha + WHITE_BG_G * (1.0 - alpha)) as u8;
        let blended_b = (b * alpha + WHITE_BG_B * (1.0 - alpha)) as u8;
        
        rgb_data.push(blended_r);
        rgb_data.push(blended_g);
        rgb_data.push(blended_b);
    }
    
    rgb_data
}

fn pixels_to_mm(pixels: u32, dpi: f32) -> f32 {
    (pixels as f32 / dpi) * INCH_TO_MM
}

pub fn generate_single_page_pdf(
    image_data: (Vec<u8>, u32, u32),
    image_index: usize,
) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
    let start_time = Instant::now();
    
    let (img_bytes, width, height) = image_data;

    let page_width_mm = pixels_to_mm(width, DPI);
    let page_height_mm = pixels_to_mm(height, DPI);
    
    let page_w = Mm(page_width_mm);
    let page_h = Mm(page_height_mm);

    let (doc, page1, layer1) = PdfDocument::new("Image", page_w, page_h, "Layer 1");
    
    let current_page = doc.get_page(page1);
    let current_layer = current_page.get_layer(layer1);

    let dyn_image = image_data_to_dynamic_image(&img_bytes, width, height);
    let rgba = dyn_image.to_rgba8();
    let (w, h) = rgba.dimensions();
    
    let raw_rgb = convert_rgba_to_rgb_with_white_bg(&rgba.into_raw());

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

    let transform = ImageTransform {
        translate_x: Some(Mm(0.0)),
        translate_y: Some(Mm(0.0)),
        scale_x: None,
        scale_y: None,
        rotate: None,
        dpi: Some(DPI),
    };
    
    image.add_to_layer(current_layer, transform);

    let result = doc.save_to_bytes()?;
    
    let elapsed = start_time.elapsed();
    println!(
        "[PDF] Image {} ({}x{}) converted in {:.2}ms",
        image_index + 1,
        width,
        height,
        elapsed.as_secs_f64() * 1000.0
    );
    
    Ok(result)
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
        .map(|bytes| Document::load_mem(bytes))
        .collect();
    
    let documents = documents?;

    let merged_doc = merge_documents(documents)?;

    let output = save_document_to_bytes(merged_doc)?;
    Ok(output)
}

fn merge_documents(documents: Vec<Document>) -> Result<Document, Box<dyn std::error::Error>> {
    let mut merged = Document::with_version("1.5");
    let mut max_id: u32 = 1;
    let mut all_page_refs: Vec<ObjectId> = Vec::new();

    for doc in documents {
        let id_mapping = copy_all_objects(&doc, &mut merged, max_id);
        
        let new_max = id_mapping.values().map(|id| id.0).max().unwrap_or(max_id);
        max_id = new_max + 1;

        let pages = doc.get_pages();
        for (_, &old_page_id) in pages.iter() {
            if let Some(&new_page_id) = id_mapping.get(&old_page_id) {
                all_page_refs.push(new_page_id);
            }
        }
    }

    let pages_id: ObjectId = (max_id, 0);
    max_id += 1;

    let kids_array: Vec<Object> = all_page_refs
        .iter()
        .map(|&id| Object::Reference(id))
        .collect();
    
    let mut pages_dict = Dictionary::new();
    pages_dict.set("Type", Object::Name(b"Pages".to_vec()));
    pages_dict.set("Kids", Object::Array(kids_array));
    pages_dict.set("Count", Object::Integer(all_page_refs.len() as i64));
    
    merged.objects.insert(pages_id, Object::Dictionary(pages_dict));

    for &page_id in &all_page_refs {
        if let Ok(Object::Dictionary(ref mut page_dict)) = merged.get_object_mut(page_id) {
            page_dict.set("Parent", Object::Reference(pages_id));
        }
    }

    let catalog_id: ObjectId = (max_id, 0);
    
    let mut catalog = Dictionary::new();
    catalog.set("Type", Object::Name(b"Catalog".to_vec()));
    catalog.set("Pages", Object::Reference(pages_id));
    
    merged.objects.insert(catalog_id, Object::Dictionary(catalog));
    merged.trailer.set("Root", Object::Reference(catalog_id));

    Ok(merged)
}

fn copy_all_objects(
    source: &Document,
    target: &mut Document,
    start_id: u32,
) -> HashMap<ObjectId, ObjectId> {
    let mut id_mapping: HashMap<ObjectId, ObjectId> = HashMap::new();
    let mut current_id = start_id;

    for (&old_id, _) in &source.objects {
        let new_id: ObjectId = (current_id, 0);
        id_mapping.insert(old_id, new_id);
        current_id += 1;
    }

    for (&old_id, obj) in &source.objects {
        let new_id = id_mapping[&old_id];
        let remapped_obj = remap_object_references(obj, &id_mapping);
        target.objects.insert(new_id, remapped_obj);
    }

    id_mapping
}

fn remap_object_references(obj: &Object, mapping: &HashMap<ObjectId, ObjectId>) -> Object {
    match obj {
        Object::Reference(old_id) => {
            let new_id = mapping.get(old_id).copied().unwrap_or(*old_id);
            Object::Reference(new_id)
        }
        Object::Array(arr) => {
            let new_arr: Vec<Object> = arr
                .iter()
                .map(|item| remap_object_references(item, mapping))
                .collect();
            Object::Array(new_arr)
        }
        Object::Dictionary(dict) => {
            let mut new_dict = Dictionary::new();
            for (key, value) in dict.iter() {
                new_dict.set(key.clone(), remap_object_references(value, mapping));
            }
            Object::Dictionary(new_dict)
        }
        Object::Stream(stream) => {
            let mut new_dict = Dictionary::new();
            for (key, value) in stream.dict.iter() {
                new_dict.set(key.clone(), remap_object_references(value, mapping));
            }
            Object::Stream(lopdf::Stream::new(new_dict, stream.content.clone()))
        }
        other => other.clone(),
    }
}

fn save_document_to_bytes(
    mut doc: Document,
) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
    let mut bytes = Vec::new();
    doc.save_to(&mut bytes)?;
    Ok(bytes)
}