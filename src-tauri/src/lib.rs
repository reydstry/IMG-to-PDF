mod pdf;
mod image_utils;

#[tauri::command]
async fn convert_images_to_pdf(
    image_paths: Vec<String>,
    output_pdf_path: String,
    orientation: String,
    margin: String,
) -> Result<String, String> {
    let file_bytes: Result<Vec<_>, String> = image_paths
        .iter()
        .map(|path| {
            std::fs::read(path)
                .map_err(|e| format!("Failed to read {}: {}", path, e))
        })
        .collect();
    
    let file_bytes = file_bytes?;
    
    println!("Loading {} images in parallel...", file_bytes.len());
    let loaded_images = image_utils::load_images_parallel(file_bytes);
    
    let successful_images: Vec<_> = loaded_images
        .into_iter()
        .enumerate()
        .filter_map(|(i, result)| {
            match result {
                Ok(data) => {
                    let (_, w, h) = &data;
                    println!("Image {} loaded: {}x{}", i + 1, w, h);
                    Some(data)
                }
                Err(e) => {
                    eprintln!("Warning: {}", e);
                    None
                }
            }
        })
        .collect();
    
    if successful_images.is_empty() {
        return Err("No images could be loaded".to_string());
    }
    
    println!("Generating {} PDFs in parallel...", successful_images.len());
    let pdf_results = image_utils::generate_pdfs_parallel(
        successful_images,
        &orientation,
        &margin,
    );
    
    let successful_pdfs: Vec<_> = pdf_results
        .into_iter()
        .filter_map(|result| {
            match result {
                Ok(pdf_bytes) => Some(pdf_bytes),
                Err(e) => {
                    eprintln!("Warning: {}", e);
                    None
                }
            }
        })
        .collect();
    
    if successful_pdfs.is_empty() {
        return Err("No PDFs could be generated".to_string());
    }
    
    let pdf_count = successful_pdfs.len();
    
    println!("Merging {} PDFs...", pdf_count);
    let merged_pdf = pdf::merge_pdfs(successful_pdfs)
        .map_err(|e| format!("Failed to merge PDFs: {}", e))?;
    
    std::fs::write(&output_pdf_path, merged_pdf)
        .map_err(|e| format!("Failed to write PDF: {}", e))?;
    
    Ok(format!(
        "✅ PDF berhasil dibuat dengan {} halaman: {}",
        pdf_count,
        output_pdf_path
    ))
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .invoke_handler(tauri::generate_handler![convert_images_to_pdf])
        .setup(|app| {
            if cfg!(debug_assertions) {
                app.handle().plugin(
                    tauri_plugin_log::Builder::default()
                        .level(log::LevelFilter::Debug)
                        .build(),
                )?;
            }
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}