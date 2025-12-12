mod pdf;
mod image_utils;

#[tauri::command]
async fn convert_images_to_pdf(
    image_paths: Vec<String>,
    output_pdf_path: String,
) -> Result<String, String> {
    let total_start = std::time::Instant::now();
    
    let file_bytes: Result<Vec<_>, String> = image_paths
        .iter()
        .map(|path| {
            std::fs::read(path)
                .map_err(|e| format!("Failed to read {}: {}", path, e))
        })
        .collect();
    
    let file_bytes = file_bytes?;
    
    println!("\n========== IMAGE TO PDF CONVERSION ==========");
    println!("[Start] Loading {} images in parallel...", file_bytes.len());
    
    let load_start = std::time::Instant::now();
    let loaded_images = image_utils::load_images_parallel(file_bytes);
    println!("[Load] All images loaded in {:.2}ms", load_start.elapsed().as_secs_f64() * 1000.0);
    
    let successful_images: Vec<_> = loaded_images
        .into_iter()
        .enumerate()
        .filter_map(|(i, result)| {
            match result {
                Ok(data) => {
                    let (_, w, h) = &data;
                    println!("[Image {}] Loaded: {}x{} pixels", i + 1, w, h);
                    Some(data)
                }
                Err(e) => {
                    eprintln!("[Warning] {}", e);
                    None
                }
            }
        })
        .collect();
    
    if successful_images.is_empty() {
        return Err("No images could be loaded".to_string());
    }
    
    println!("\n[Convert] Generating {} PDFs in parallel...", successful_images.len());
    let convert_start = std::time::Instant::now();
    let pdf_results = image_utils::generate_pdfs_parallel(successful_images);
    
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
    
    println!("[Convert] All PDFs generated in {:.2}ms", convert_start.elapsed().as_secs_f64() * 1000.0);
    
    let pdf_count = successful_pdfs.len();
    
    println!("\n[Merge] Merging {} PDFs...", pdf_count);
    let merge_start = std::time::Instant::now();
    let merged_pdf = pdf::merge_pdfs(successful_pdfs)
        .map_err(|e| format!("Failed to merge PDFs: {}", e))?;
    println!("[Merge] Completed in {:.2}ms", merge_start.elapsed().as_secs_f64() * 1000.0);
    
    println!("\n[Write] Saving to: {}", output_pdf_path);
    std::fs::write(&output_pdf_path, &merged_pdf)
        .map_err(|e| format!("Failed to write PDF: {}", e))?;
    
    let total_elapsed = total_start.elapsed();
    println!("[Done] Total time: {:.2}ms", total_elapsed.as_secs_f64() * 1000.0);
    println!("==========================================\n");
    
    Ok(format!(
        "✅ PDF berhasil dibuat dengan {} halaman dalam {:.2}s: {}",
        pdf_count,
        total_elapsed.as_secs_f64(),
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