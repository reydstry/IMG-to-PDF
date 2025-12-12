mod pdf;
mod image_utils;

use std::fs;

#[tauri::command]
async fn convert_images_to_pdf(
    image_paths: Vec<String>,
    output_pdf_path: String,
    orientation: String,   // "portrait" | "landscape"
    margin: String,        // "none" | "small" | "large"
) -> Result<String, String> {
    let mut image_data = Vec::new(); // jangan memakai muttable, cari alternatif lain
    
    // Load semua gambar
    for path in &image_paths {
        let bytes = fs::read(path)
            .map_err(|e| format!("Failed to read {}: {}", path, e))?;
        
        let (img_bytes, width, height) = image_utils::load_image_from_bytes(&bytes)
            .map_err(|e| format!("Failed to load image: {}", e))?;
        println!("Loaded image: {} ({}x{})", path, width, height);
        image_data.push((img_bytes, width, height));
    }
    
    // Generate PDF dengan opsi orientasi & margin
    let pdf_bytes = pdf::generate_pdf(image_data, &orientation, &margin)
        .map_err(|e| format!("Failed to generate PDF: {}", e))?;
    
    // Simpan ke file
    fs::write(&output_pdf_path, pdf_bytes)
        .map_err(|e| format!("Failed to write PDF: {}", e))?;
    
    Ok(format!("PDF berhasil dibuat: {}", output_pdf_path))
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
                        .level(log::LevelFilter::Info)
                        .build(),
                )?;
            }
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}