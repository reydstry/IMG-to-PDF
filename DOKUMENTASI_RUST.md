# 📚 PANDUAN LENGKAP KODE RUST
## Image to PDF Converter - Tauri Application

> **Dokumentasi ini dibuat untuk programmer pemula yang ingin belajar Rust**
> 
> Dibuat pada: Desember 2025

---

## 📑 Daftar Isi

1. [Pendahuluan](#pendahuluan)
2. [Struktur Proyek](#struktur-proyek)
3. [File 1: main.rs](#file-1-mainrs)
4. [File 2: lib.rs](#file-2-librs)
5. [File 3: image_utils.rs](#file-3-image_utilsrs)
6. [File 4: pdf.rs](#file-4-pdfrs)
7. [Ringkasan Keyword Rust](#ringkasan-keyword-rust)
8. [Ringkasan Operator Casting](#ringkasan-operator-casting)
9. [Konsep Penting: Ownership & Borrowing](#konsep-penting-ownership--borrowing)
10. [Konsep Penting: Mutable vs Immutable](#konsep-penting-mutable-vs-immutable)

---

## Pendahuluan

Proyek ini adalah aplikasi desktop yang mengkonversi gambar-gambar menjadi satu file PDF. Dibangun menggunakan:

- **Rust** - Bahasa pemrograman sistem yang aman dan cepat
- **Tauri** - Framework untuk membuat aplikasi desktop dengan web frontend
- **Rayon** - Library untuk parallel processing
- **printpdf** - Library untuk membuat PDF
- **lopdf** - Library untuk manipulasi PDF
- **image** - Library untuk memproses gambar

### Alur Kerja Aplikasi

```
┌─────────────────┐
│  User memilih   │
│    gambar       │
└────────┬────────┘
         │
         ▼
┌─────────────────┐
│  Baca file      │
│  gambar         │
└────────┬────────┘
         │
         ▼
┌─────────────────┐
│  Load gambar    │◄──── Parallel Processing
│  ke memory      │      (rayon)
└────────┬────────┘
         │
         ▼
┌─────────────────┐
│  Konversi ke    │◄──── Parallel Processing
│  PDF per gambar │      (rayon)
└────────┬────────┘
         │
         ▼
┌─────────────────┐
│  Merge semua    │
│  PDF jadi satu  │
└────────┬────────┘
         │
         ▼
┌─────────────────┐
│  Simpan file    │
│  PDF            │
└─────────────────┘
```

---

## Struktur Proyek

```
src-tauri/
└── src/
    ├── main.rs        # Entry point aplikasi
    ├── lib.rs         # Library utama & Tauri command
    ├── image_utils.rs # Utility untuk memproses gambar
    └── pdf.rs         # Logic untuk membuat & merge PDF
```

---

## File 1: main.rs

File ini adalah **entry point** (titik masuk) aplikasi Rust.

### Kode Lengkap

```rust
// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

fn main() {
    app_lib::run();
}
```

### Daftar Fungsi

| Fungsi | Kegunaan |
|--------|----------|
| `main()` | Entry point - titik awal program dijalankan |

### Penjelasan Detail

#### Baris 1-2: Attribute Macro

```rust
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
```

| Bagian | Penjelasan |
|--------|------------|
| `#![ ]` | **Inner attribute** - berlaku untuk seluruh file/crate ini |
| `cfg_attr()` | **Conditional attribute** - terapkan attribute jika kondisi terpenuhi |
| `not(debug_assertions)` | Kondisi: BUKAN mode debug (artinya: mode release) |
| `windows_subsystem = "windows"` | Attribute yang diterapkan: sembunyikan console window |

**Analogi Sederhana:**
- Bayangkan ini seperti "aturan khusus" untuk Windows
- Saat kamu build untuk release (bukan development), jendela console hitam akan disembunyikan
- User hanya melihat GUI, tidak ada window terminal yang mengganggu

#### Baris 4-6: Fungsi Main

```rust
fn main() {
    app_lib::run();
}
```

| Keyword | Penjelasan |
|---------|------------|
| `fn` | Mendefinisikan **function** (fungsi) |
| `main` | Nama khusus - Rust selalu memulai dari fungsi ini |
| `app_lib` | Nama crate/library yang didefinisikan di `lib.rs` |
| `::` | **Path separator** - untuk mengakses item dalam module/crate |
| `run()` | Fungsi yang dipanggil dari `app_lib` |

**Mengapa struktur ini?**
- `main.rs` hanya sebagai "starter" yang memanggil `lib.rs`
- `lib.rs` berisi logic utama
- Memisahkan entry point dari logic memudahkan testing dan reusability

---

## File 2: lib.rs

File ini adalah **library utama** yang mengatur alur konversi gambar ke PDF.

### Kode Lengkap

```rust
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
```

### Daftar Fungsi

| Fungsi | Kegunaan |
|--------|----------|
| `convert_images_to_pdf()` | Mengkonversi gambar-gambar menjadi satu file PDF |
| `run()` | Menjalankan aplikasi Tauri |

### Penjelasan Detail Setiap Bagian

#### Baris 1-2: Module Declaration

```rust
mod pdf;
mod image_utils;
```

| Keyword | Penjelasan |
|---------|------------|
| `mod` | **Module declaration** - memberitahu Rust untuk mencari file dengan nama tersebut |
| `pdf` | Rust akan mencari `pdf.rs` atau `pdf/mod.rs` |
| `image_utils` | Rust akan mencari `image_utils.rs` |

**Analogi:**
- Seperti `import` di Python atau `require` di JavaScript
- Tapi di Rust, ini memberitahu compiler "ada file lain yang merupakan bagian dari proyek ini"

#### Baris 4: Tauri Command Attribute

```rust
#[tauri::command]
```

| Bagian | Penjelasan |
|--------|------------|
| `#[ ]` | **Outer attribute** - berlaku untuk item di bawahnya |
| `tauri::command` | Macro dari Tauri yang membuat fungsi bisa dipanggil dari JavaScript/frontend |

**Mengapa diperlukan?**
- Tauri adalah framework untuk membuat aplikasi desktop dengan web frontend
- Attribute ini "mengekspos" fungsi Rust agar bisa dipanggil dari HTML/JavaScript

#### Baris 5-8: Function Signature

```rust
async fn convert_images_to_pdf(
    image_paths: Vec<String>,
    output_pdf_path: String,
) -> Result<String, String> {
```

| Keyword/Bagian | Penjelasan |
|----------------|------------|
| `async` | Fungsi ini **asynchronous** - tidak memblokir thread utama |
| `fn` | Mendefinisikan fungsi |
| `image_paths: Vec<String>` | Parameter: vector (array dinamis) berisi path gambar |
| `output_pdf_path: String` | Parameter: path output file PDF |
| `-> Result<String, String>` | Return type: bisa sukses (String) atau error (String) |

**Mengapa `async`?**
- Operasi file (baca/tulis) bisa lambat
- Dengan `async`, UI tidak freeze saat memproses gambar besar
- Tauri command perlu async untuk komunikasi dengan frontend

**Mengapa `Result<String, String>`?**
- `Result` adalah enum dengan 2 varian: `Ok(T)` dan `Err(E)`
- `Ok(String)` = sukses, return pesan sukses
- `Err(String)` = gagal, return pesan error

#### Baris 9: Instant untuk Timing

```rust
let total_start = std::time::Instant::now();
```

| Bagian | Penjelasan |
|--------|------------|
| `let` | Mendefinisikan variable baru (immutable by default) |
| `total_start` | Nama variable |
| `std::time::Instant` | Type dari standard library untuk mengukur waktu |
| `::now()` | Method untuk mendapatkan waktu sekarang |

**Method yang digunakan:**
- `.now()` - membuat Instant dengan waktu saat ini
- `.elapsed()` - menghitung berapa lama sejak Instant dibuat

#### Baris 11-18: Membaca File

```rust
let file_bytes: Result<Vec<_>, String> = image_paths
    .iter()
    .map(|path| {
        std::fs::read(path)
            .map_err(|e| format!("Failed to read {}: {}", path, e))
    })
    .collect();
```

| Bagian | Penjelasan |
|--------|------------|
| `Result<Vec<_>, String>` | Type annotation: Result berisi Vec atau error String |
| `Vec<_>` | `_` artinya "Rust, tolong infer type-nya otomatis" |
| `.iter()` | Membuat iterator dari vector |
| `.map(\|path\| ...)` | Transform setiap elemen dengan closure |
| `\|path\|` | **Closure parameter** - `path` adalah setiap elemen |
| `std::fs::read(path)` | Baca file dari path, return `Result<Vec<u8>, Error>` |
| `.map_err()` | Transform error menjadi format yang kita mau |
| `format!()` | Macro untuk membuat String dengan formatting |
| `.collect()` | Kumpulkan hasil iterator menjadi collection |

**Tabel Method Iterator:**

| Method | Dari Tipe | Kegunaan |
|--------|-----------|----------|
| `.iter()` | `Vec<T>` | Membuat iterator yang meminjam elemen |
| `.map()` | `Iterator` | Transform setiap elemen |
| `.map_err()` | `Result` | Transform error type |
| `.collect()` | `Iterator` | Kumpulkan ke collection |

#### Baris 20: Error Propagation

```rust
let file_bytes = file_bytes?;
```

| Bagian | Penjelasan |
|--------|------------|
| `?` | **Error propagation operator** - jika error, langsung return error |

**Cara kerja `?`:**
- Jika `file_bytes` adalah `Ok(value)`, maka `value` diambil
- Jika `file_bytes` adalah `Err(e)`, maka fungsi langsung return `Err(e)`
- Shorthand untuk:
  ```rust
  let file_bytes = match file_bytes {
      Ok(v) => v,
      Err(e) => return Err(e),
  };
  ```

#### Baris 28-45: Filter dan Map

```rust
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
```

| Bagian | Penjelasan |
|--------|------------|
| `.into_iter()` | Membuat iterator yang **mengambil ownership** (bukan meminjam) |
| `.enumerate()` | Menambahkan index ke setiap elemen: `(0, elem), (1, elem), ...` |
| `.filter_map()` | Filter + map sekaligus: `Some(x)` diambil, `None` dibuang |
| `(i, result)` | **Destructuring** tuple: `i` = index, `result` = elemen |
| `match` | **Pattern matching** - seperti switch tapi lebih powerful |
| `Ok(data)` | Pattern untuk Result sukses |
| `Err(e)` | Pattern untuk Result error |
| `let (_, w, h) = &data` | Destructuring tuple: `_` = abaikan, ambil `w` dan `h` |
| `Some(data)` | Return value untuk `filter_map` - berarti "simpan ini" |
| `None` | Return value untuk `filter_map` - berarti "buang ini" |
| `println!()` | Macro untuk print ke stdout |
| `eprintln!()` | Macro untuk print ke stderr (untuk error/warning) |

**Perbedaan `.iter()` vs `.into_iter()`:**

| Method | Ownership | Hasil |
|--------|-----------|-------|
| `.iter()` | Meminjam (`&T`) | Original vector masih ada |
| `.into_iter()` | Mengambil ownership (`T`) | Original vector dikonsumsi |

#### Baris 46-48: Empty Check

```rust
if successful_images.is_empty() {
    return Err("No images could be loaded".to_string());
}
```

**Method yang digunakan:**

| Method | Dari Tipe | Kegunaan |
|--------|-----------|----------|
| `.is_empty()` | `Vec<T>` | Check apakah vector kosong |
| `.to_string()` | `&str` | Konversi string literal ke `String` |

#### Baris 73-75: Writing File

```rust
std::fs::write(&output_pdf_path, &merged_pdf)
    .map_err(|e| format!("Failed to write PDF: {}", e))?;
```

**Mengapa `&output_pdf_path` pakai `&`?**
- `std::fs::write` meminjam path, tidak perlu ownership
- `&` membuat **reference** (pinjaman) ke data
- Setelah fungsi selesai, `output_pdf_path` masih bisa digunakan

#### Baris 93-109: Tauri Setup

```rust
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
```

| Bagian | Penjelasan |
|--------|------------|
| `#[cfg_attr(mobile, ...)]` | Apply attribute hanya jika compile untuk mobile |
| `pub` | **Public** - fungsi bisa diakses dari luar module |
| `tauri::Builder::default()` | Membuat Tauri builder dengan config default |
| `.plugin()` | Menambahkan plugin ke Tauri |
| `.invoke_handler()` | Mendaftarkan command yang bisa dipanggil dari frontend |
| `tauri::generate_handler![]` | Macro untuk generate handler dari fungsi-fungsi |
| `.setup(\|app\| ...)` | Closure untuk setup saat app start |
| `cfg!(debug_assertions)` | Macro untuk check apakah mode debug |
| `.run()` | Menjalankan aplikasi |
| `.expect()` | Jika error, panic dengan pesan ini |

---

## File 3: image_utils.rs

File ini berisi **utility functions** untuk memproses gambar.

### Kode Lengkap

```rust
use image::DynamicImage;
use rayon::prelude::*;

pub fn load_image_from_bytes(
    bytes: &[u8],
) -> Result<(Vec<u8>, u32, u32), Box<dyn std::error::Error>> {
    let img = image::load_from_memory(bytes)?;
    let rgba = img.to_rgba8();
    let (width, height) = rgba.dimensions();
    Ok((rgba.into_raw(), width, height))
}

pub fn load_images_parallel(
    image_bytes: Vec<Vec<u8>>,
) -> Vec<Result<(Vec<u8>, u32, u32), String>> {
    image_bytes
        .par_iter()
        .map(|bytes| {
            load_image_from_bytes(bytes)
                .map_err(|e| format!("Failed to load image: {}", e))
        })
        .collect()
}

pub fn image_data_to_dynamic_image(data: &[u8], width: u32, height: u32) -> DynamicImage {
    DynamicImage::ImageRgba8(
        image::RgbaImage::from_raw(width, height, data.to_vec())
            .expect("Failed to create image from raw data"),
    )
}

pub fn generate_pdfs_parallel(
    image_data: Vec<(Vec<u8>, u32, u32)>,
) -> Vec<Result<Vec<u8>, String>> {
    println!("[Parallel] Starting PDF generation with {} threads...", rayon::current_num_threads());
    
    image_data
        .par_iter()
        .enumerate()
        .map(|(i, data)| {
            println!("[Thread] Processing image {}...", i + 1);
            crate::pdf::generate_single_page_pdf(data.clone(), i)
                .map_err(|e| format!("Image {}: {}", i + 1, e))
        })
        .collect()
}
```

### Daftar Fungsi

| Fungsi | Kegunaan |
|--------|----------|
| `load_image_from_bytes()` | Load satu gambar dari bytes |
| `load_images_parallel()` | Load banyak gambar secara paralel |
| `image_data_to_dynamic_image()` | Konversi raw bytes ke DynamicImage |
| `generate_pdfs_parallel()` | Generate PDF dari gambar secara paralel |

### Penjelasan Detail

#### Baris 1-2: Use Statements

```rust
use image::DynamicImage;
use rayon::prelude::*;
```

| Bagian | Penjelasan |
|--------|------------|
| `use` | Import item ke scope saat ini |
| `image::DynamicImage` | Type dari crate `image` |
| `rayon::prelude::*` | Import semua dari rayon prelude (parallel iterator traits) |
| `*` | **Glob import** - import semua public items |

#### Baris 4-11: load_image_from_bytes

```rust
pub fn load_image_from_bytes(
    bytes: &[u8],
) -> Result<(Vec<u8>, u32, u32), Box<dyn std::error::Error>> {
    let img = image::load_from_memory(bytes)?;
    let rgba = img.to_rgba8();
    let (width, height) = rgba.dimensions();
    Ok((rgba.into_raw(), width, height))
}
```

| Bagian | Penjelasan |
|--------|------------|
| `pub` | Public - bisa diakses dari module lain |
| `bytes: &[u8]` | Parameter: **slice** of bytes (pinjaman ke array/vector) |
| `Box<dyn std::error::Error>` | Return error: boxed dynamic error (bisa error type apapun) |
| `(Vec<u8>, u32, u32)` | Return sukses: tuple berisi (pixel data, width, height) |
| `image::load_from_memory()` | Decode gambar dari bytes di memory |
| `.to_rgba8()` | Konversi ke format RGBA 8-bit per channel |
| `.dimensions()` | Dapatkan (width, height) |
| `.into_raw()` | Ambil raw bytes dari image, consuming ownership |

**Mengapa `&[u8]` bukan `Vec<u8>`?**
- `&[u8]` adalah **slice** - reference ke data yang sudah ada
- Lebih fleksibel: bisa terima `&Vec<u8>`, `&[u8; N]`, dll
- Tidak perlu ownership, hanya perlu baca data

**Mengapa `Box<dyn std::error::Error>`?**
- `dyn` = **dynamic dispatch** - type ditentukan saat runtime
- `Box` = smart pointer ke heap
- Bisa return berbagai jenis error tanpa tahu type pastinya

#### Baris 13-23: load_images_parallel

```rust
pub fn load_images_parallel(
    image_bytes: Vec<Vec<u8>>,
) -> Vec<Result<(Vec<u8>, u32, u32), String>> {
    image_bytes
        .par_iter()
        .map(|bytes| {
            load_image_from_bytes(bytes)
                .map_err(|e| format!("Failed to load image: {}", e))
        })
        .collect()
}
```

| Bagian | Penjelasan |
|--------|------------|
| `Vec<Vec<u8>>` | Vector of vectors - setiap inner vector adalah satu gambar |
| `.par_iter()` | **Parallel iterator** dari rayon - proses di multiple threads |
| `.map()` | Transform setiap elemen (dijalankan paralel!) |
| `.collect()` | Kumpulkan hasil ke vector |

**Mengapa `.par_iter()` bukan `.iter()`?**
- `.par_iter()` = rayon's parallel iterator
- Otomatis mendistribusikan pekerjaan ke semua CPU cores
- Untuk load 10 gambar: bisa diproses 10 thread sekaligus
- Jauh lebih cepat untuk I/O-bound operations

#### Baris 25-30: image_data_to_dynamic_image

```rust
pub fn image_data_to_dynamic_image(data: &[u8], width: u32, height: u32) -> DynamicImage {
    DynamicImage::ImageRgba8(
        image::RgbaImage::from_raw(width, height, data.to_vec())
            .expect("Failed to create image from raw data"),
    )
}
```

| Bagian | Penjelasan |
|--------|------------|
| `data: &[u8]` | Slice of raw pixel data |
| `width: u32, height: u32` | Dimensi gambar |
| `DynamicImage::ImageRgba8()` | Konstruktor enum variant |
| `RgbaImage::from_raw()` | Buat image dari raw bytes |
| `.expect()` | Unwrap Option, panic jika None dengan pesan custom |

**Mengapa `data.to_vec()`?**
- `from_raw()` membutuhkan ownership data (`Vec<u8>`)
- `data` hanya reference (`&[u8]`)
- `.to_vec()` membuat copy baru yang di-owned

#### Baris 32-46: generate_pdfs_parallel

```rust
pub fn generate_pdfs_parallel(
    image_data: Vec<(Vec<u8>, u32, u32)>,
) -> Vec<Result<Vec<u8>, String>> {
    println!("[Parallel] Starting PDF generation with {} threads...", rayon::current_num_threads());
    
    image_data
        .par_iter()
        .enumerate()
        .map(|(i, data)| {
            println!("[Thread] Processing image {}...", i + 1);
            crate::pdf::generate_single_page_pdf(data.clone(), i)
                .map_err(|e| format!("Image {}: {}", i + 1, e))
        })
        .collect()
}
```

| Bagian | Penjelasan |
|--------|------------|
| `Vec<(Vec<u8>, u32, u32)>` | Vector of tuples: (pixel data, width, height) |
| `rayon::current_num_threads()` | Dapatkan jumlah thread yang digunakan rayon |
| `.par_iter()` | Parallel iterator |
| `.enumerate()` | Tambahkan index |
| `data.clone()` | Buat copy dari data (diperlukan karena par_iter meminjam) |
| `crate::pdf::` | Akses module `pdf` dari crate ini |

**Mengapa perlu `.clone()`?**
- `.par_iter()` meminjam data (`&T`)
- `generate_single_page_pdf` membutuhkan ownership (`T`)
- `.clone()` membuat copy baru yang bisa di-owned

---

## File 4: pdf.rs

File ini berisi **logic utama** untuk membuat dan menggabungkan PDF.

### Kode Lengkap

```rust
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
```

### Daftar Fungsi

| Fungsi | Kegunaan |
|--------|----------|
| `convert_rgba_to_rgb_with_white_bg()` | Konversi RGBA ke RGB dengan background putih |
| `pixels_to_mm()` | Konversi pixel ke millimeter |
| `generate_single_page_pdf()` | Buat satu halaman PDF dari gambar |
| `merge_pdfs()` | Gabung beberapa PDF menjadi satu |
| `merge_documents()` | Logic internal untuk merge dokumen |
| `copy_all_objects()` | Salin semua objek PDF ke dokumen baru |
| `remap_object_references()` | Update referensi objek setelah disalin |
| `save_document_to_bytes()` | Simpan dokumen ke bytes |

### Penjelasan Detail

#### Konstanta

```rust
const DPI: f32 = 150.0;
const INCH_TO_MM: f32 = 25.4;
const WHITE_BG_R: f32 = 255.0;
const WHITE_BG_G: f32 = 255.0;
const WHITE_BG_B: f32 = 255.0;
```

| Keyword | Penjelasan |
|---------|------------|
| `const` | **Compile-time constant** - nilai tetap, diketahui saat compile |
| `f32` | 32-bit floating point number |

**Mengapa `const` bukan `let`?**
- `const` = nilai tetap sepanjang waktu, diinline oleh compiler
- `let` = variable runtime, bisa saja berbeda tiap eksekusi
- Konstanta lebih efisien untuk nilai yang tidak pernah berubah

#### convert_rgba_to_rgb_with_white_bg (Alpha Blending)

```rust
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
```

**Operator Casting & Penjelasan:**

| Casting | Penjelasan |
|---------|------------|
| `pixel[0] as f32` | Konversi `u8` (0-255) ke `f32` untuk perhitungan desimal |
| `... as u8` | Konversi hasil perhitungan `f32` kembali ke `u8` |

**Mengapa perlu casting?**
- `u8` adalah integer 0-255, tidak bisa menyimpan desimal
- Alpha blending memerlukan perhitungan dengan angka desimal
- `as f32` memungkinkan: `128 / 255.0 = 0.502` (bukan `0`)
- `as u8` membulatkan hasil ke integer terdekat

**Formula Alpha Blending:**
```
hasil = (warna_depan × alpha) + (warna_belakang × (1 - alpha))
```

**Contoh Perhitungan:**

Misalkan pixel dengan nilai:
- R = 100, G = 50, B = 200, Alpha = 128 (50% transparan)
- Background putih: R = 255, G = 255, B = 255

```
alpha = 128 / 255 = 0.502

blended_r = (100 × 0.502) + (255 × (1 - 0.502))
          = 50.2 + 127.0
          = 177.2 ≈ 177

blended_g = (50 × 0.502) + (255 × 0.498)
          = 25.1 + 127.0
          = 152.1 ≈ 152

blended_b = (200 × 0.502) + (255 × 0.498)
          = 100.4 + 127.0
          = 227.4 ≈ 227
```

**Method yang digunakan:**

| Method | Kegunaan |
|--------|----------|
| `Vec::with_capacity()` | Pre-alokasi memory untuk efisiensi |
| `.chunks_exact(4)` | Pecah slice menjadi potongan 4 elemen |
| `.push()` | Tambahkan elemen ke vector |

**Mengapa `mut` diperlukan pada `rgb_data`?**
- Vector `rgb_data` akan **diisi secara bertahap** dalam loop
- Setiap iterasi menambah 3 byte baru (R, G, B)
- Tanpa `mut`, Rust tidak mengizinkan modifikasi vector

#### pixels_to_mm

```rust
fn pixels_to_mm(pixels: u32, dpi: f32) -> f32 {
    (pixels as f32 / dpi) * INCH_TO_MM
}
```

**Operator Casting & Penjelasan:**

| Casting | Penjelasan |
|---------|------------|
| `pixels as f32` | Konversi `u32` ke `f32` untuk division |

**Mengapa perlu casting?**
- `pixels` adalah `u32` (unsigned integer)
- Division dengan `dpi` (f32) memerlukan kedua operand bertipe sama
- Tanpa cast: `1500 / 150.0` akan error karena type mismatch

#### generate_single_page_pdf

```rust
pub fn generate_single_page_pdf(
    image_data: (Vec<u8>, u32, u32),
    image_index: usize,
) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
    let start_time = Instant::now();
    
    let (img_bytes, width, height) = image_data;  // Destructuring
    // ... implementation
}
```

| Bagian | Penjelasan |
|--------|------------|
| `(Vec<u8>, u32, u32)` | Tuple type dengan 3 elemen |
| `usize` | Platform-dependent unsigned integer (untuk indexing) |
| `let (img_bytes, width, height) = image_data` | **Destructuring** - pecah tuple ke variables |
| `Mm()` | Konstruktor untuk type `Mm` (millimeters) |

**Mengapa `usize` untuk index?**
- `usize` adalah type yang sesuai untuk indexing di Rust
- Di 64-bit system: `usize` = `u64`
- Di 32-bit system: `usize` = `u32`
- Menjamin kompatibilitas platform

#### Struct Initialization (ImageXObject)

```rust
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
```

| Casting | Penjelasan |
|---------|------------|
| `w as usize` | Konversi `u32` ke `usize` karena `Px` membutuhkan `usize` |

| Bagian | Penjelasan |
|--------|------------|
| `Px()` | Konstruktor untuk type Pixels |
| `ColorSpace::Rgb` | Enum variant |
| `ColorBits::Bit8` | Enum variant |
| `true` | Boolean literal |
| `None` | Option variant untuk "tidak ada nilai" |

#### merge_documents

```rust
fn merge_documents(documents: Vec<Document>) -> Result<Document, Box<dyn std::error::Error>> {
    let mut merged = Document::with_version("1.5");
    let mut max_id: u32 = 1;
    let mut all_page_refs: Vec<ObjectId> = Vec::new();

    for doc in documents {
        let id_mapping = copy_all_objects(&doc, &mut merged, max_id);
        
        let new_max = id_mapping.values().map(|id| id.0).max().unwrap_or(max_id);
        max_id = new_max + 1;
        // ...
    }
    // ...
}
```

**Mengapa perlu `mut`?**

1. **`mut merged`**: Dokumen yang dibangun secara bertahap
   - Setiap iterasi menambah objek dari dokumen sumber
   - `.objects.insert()` memodifikasi dokumen
   
2. **`mut max_id`**: Counter yang terus bertambah
   - Nilai awal: 1
   - Setiap dokumen baru, ID harus bertambah untuk menghindari konflik
   - `max_id = new_max + 1` adalah **mutasi nilai**
   
3. **`mut all_page_refs`**: List yang bertumbuh
   - Dimulai kosong: `Vec::new()`
   - Setiap halaman ditambahkan: `.push(new_page_id)`
   - Tanpa `mut`, tidak bisa menambah elemen

| Bagian | Penjelasan |
|--------|------------|
| `for doc in documents` | For loop yang mengambil ownership setiap element |
| `&doc` | Reference ke doc (tidak ambil ownership) |
| `&mut merged` | Mutable reference ke merged |
| `id.0` | Akses elemen pertama dari tuple |
| `.values()` | Iterator over HashMap values |
| `.max()` | Dapatkan nilai maksimum |
| `.unwrap_or()` | Jika None, gunakan default value |

#### Pattern Matching di remap_object_references

```rust
fn remap_object_references(obj: &Object, mapping: &HashMap<ObjectId, ObjectId>) -> Object {
    match obj {
        Object::Reference(old_id) => {
            let new_id = mapping.get(old_id).copied().unwrap_or(*old_id);
            Object::Reference(new_id)
        }
        Object::Array(arr) => { /* ... */ }
        Object::Dictionary(dict) => { /* ... */ }
        Object::Stream(stream) => { /* ... */ }
        other => other.clone(),
    }
}
```

| Bagian | Penjelasan |
|--------|------------|
| `match obj` | Pattern match terhadap `obj` |
| `Object::Reference(old_id)` | Match jika obj adalah variant Reference, extract inner value ke `old_id` |
| `mapping.get(old_id)` | Dapatkan value dari HashMap, return `Option<&V>` |
| `.copied()` | Convert `Option<&T>` ke `Option<T>` (copy inner value) |
| `.unwrap_or(*old_id)` | Jika None, dereference dan gunakan `old_id` |
| `*old_id` | **Dereference** - ambil nilai dari reference |
| `other => other.clone()` | Catch-all pattern, clone objek apa adanya |

**Method yang digunakan:**

| Method | Kegunaan |
|--------|----------|
| `.get()` | Dapatkan value dari HashMap by key |
| `.copied()` | Copy nilai dalam Option |
| `.unwrap_or()` | Provide default jika None |
| `.clone()` | Buat copy dari objek |
| `.iter()` | Iterate over collection |

---

## Ringkasan Keyword Rust

| Keyword | Kegunaan | Contoh |
|---------|----------|--------|
| `let` | Deklarasi variable (immutable) | `let x = 5;` |
| `let mut` | Deklarasi variable mutable | `let mut x = 5;` |
| `const` | Konstanta compile-time | `const PI: f32 = 3.14;` |
| `fn` | Definisi fungsi | `fn add(a: i32) -> i32` |
| `pub` | Public visibility | `pub fn run()` |
| `async` | Fungsi asynchronous | `async fn fetch()` |
| `use` | Import items ke scope | `use std::fs;` |
| `mod` | Deklarasi module | `mod utils;` |
| `for` | Loop | `for x in vec` |
| `if` | Conditional | `if x > 0 { }` |
| `match` | Pattern matching | `match result { }` |
| `return` | Return dari fungsi | `return Ok(x);` |
| `&` | Reference (borrow) | `&data` |
| `&mut` | Mutable reference | `&mut data` |
| `*` | Dereference | `*ptr` |
| `as` | Type casting | `x as f32` |
| `?` | Error propagation | `file.read()?` |

---

## Ringkasan Operator Casting

| Casting | Dari | Ke | Alasan |
|---------|------|-----|--------|
| `pixel[0] as f32` | `u8` | `f32` | Perhitungan desimal dalam alpha blending |
| `... as u8` | `f32` | `u8` | Konversi hasil kembali ke byte |
| `pixels as f32` | `u32` | `f32` | Division dengan float |
| `w as usize` | `u32` | `usize` | Kompatibilitas dengan API yang butuh usize |
| `len() as i64` | `usize` | `i64` | PDF spec membutuhkan signed integer |

**Mengapa perlu casting di Rust?**

Rust adalah bahasa yang **strongly typed** - tidak ada konversi otomatis antar tipe data. Ini mencegah bug yang sulit dilacak.

```rust
// ❌ Error di Rust (tidak ada implicit casting)
let a: u8 = 100;
let b: f32 = a / 3.0;  // Error! u8 dan f32 tidak bisa langsung dihitung

// ✅ Benar di Rust (explicit casting)
let a: u8 = 100;
let b: f32 = a as f32 / 3.0;  // OK! 33.333...
```

---

## Konsep Penting: Ownership & Borrowing

### Ownership Rules

1. **Setiap nilai memiliki satu owner**
2. **Hanya boleh ada satu owner pada satu waktu**
3. **Ketika owner keluar scope, nilai di-drop (dihapus)**

### Borrowing

| Jenis | Syntax | Penjelasan |
|-------|--------|------------|
| **Immutable borrow** | `&T` | Boleh banyak sekaligus, tidak bisa modify |
| **Mutable borrow** | `&mut T` | Hanya boleh satu, bisa modify |

```rust
let data = vec![1, 2, 3];

// Immutable borrow - boleh banyak
let ref1 = &data;
let ref2 = &data;
println!("{:?} {:?}", ref1, ref2);  // OK!

// Mutable borrow - hanya satu
let mut data2 = vec![1, 2, 3];
let ref_mut = &mut data2;
ref_mut.push(4);  // OK, bisa modify
// let ref2 = &mut data2;  // ❌ Error! sudah ada mutable borrow
```

---

## Konsep Penting: Mutable vs Immutable

### Kapan Perlu `mut`?

| Situasi | Perlu `mut`? | Alasan |
|---------|-------------|--------|
| **Membaca data** | ❌ Tidak | Immutable reference cukup |
| **Menambah elemen ke collection** | ✅ Ya | `.push()`, `.insert()` memodifikasi |
| **Increment/decrement counter** | ✅ Ya | `x += 1` adalah mutasi |
| **Membangun struct bertahap** | ✅ Ya | Field diisi satu per satu |
| **Parameter yang dimodifikasi** | ✅ Ya | Fungsi mengubah data yang dipinjam |
| **Buffer untuk write operations** | ✅ Ya | Data ditulis ke buffer |

### Contoh Konkret

```rust
// ❌ Tanpa mut - Error!
let vec = Vec::new();
vec.push(1);  // Error: cannot borrow as mutable

// ✅ Dengan mut - OK!
let mut vec = Vec::new();
vec.push(1);  // OK!
```

---

## Tips untuk Pemula

1. **Baca error message dengan teliti** - Rust punya error message yang sangat informatif
2. **Gunakan `cargo check`** - Lebih cepat dari `cargo build` untuk memeriksa syntax
3. **Gunakan `rustfmt`** - Format kode secara otomatis
4. **Baca The Rust Book** - https://doc.rust-lang.org/book/
5. **Practice dengan Rustlings** - https://github.com/rust-lang/rustlings

---

## Referensi

- [The Rust Programming Language Book](https://doc.rust-lang.org/book/)
- [Rust by Example](https://doc.rust-lang.org/rust-by-example/)
- [Tauri Documentation](https://tauri.app/v1/guides/)
- [Rayon Documentation](https://docs.rs/rayon/)
- [printpdf Documentation](https://docs.rs/printpdf/)
- [lopdf Documentation](https://docs.rs/lopdf/)

---

> **Catatan:** Dokumentasi ini dibuat sebagai panduan belajar. Untuk pemahaman lebih mendalam, disarankan untuk membaca dokumentasi resmi Rust.

---

*Dibuat dengan ❤️ untuk programmer Rust pemula*
