# IMG To PDF

**Authors:** <br/>

Kelompok 7 - Pemrograman Fungsional B <br/>
Abiem Akmal Fadhil, Bayu Raihan Paratama, M. Rizky Kurniadinata, Nabilla Nur Aini, Noel Ericson Rapael Sipayung

---

## Abstract
Proyek IMG To PDF adalah aplikasi desktop yang berfungsi untuk mengonversi satu atau beberapa file gambar (JPEG, PNG, JPG, WEBP) menjadi dokumen PDF dengan cepat dan efisien. Aplikasi ini dibangun menggunakan technology stack Tauri untuk sisi front-end dan Rust untuk sisi back-end. Dengan memanfaatkan sifat Rust yang aman terhadap memori serta paradigma pemrograman fungsional seperti immutability, pure functions, dan pipeline data transformations, aplikasi ini menawarkan solusi ringan, aman, dan multiplatform. Hasil akhirnya adalah aplikasi konversi yang stabil, cepat, dan memiliki konsumsi memori minimal.

---

## Introduction
Aplikasi IMG To PDF dirancang untuk menyelesaikan permasalahan umum yang sering ditemui pengguna ketika ingin mengonversi gambar menjadi dokumen PDF, yaitu:

- Proses konversi yang lambat dan membutuhkan aplikasi berat.
- Ketergantungan pada aplikasi online sehingga kurang aman dan tidak praktis.
- Minimnya aplikasi desktop yang ringan, cepat, dan aman untuk kebutuhan konversi dokumen.

### Mengapa Rust?

| Alasan              |   Penjelasan                                                                                                     |
| --------            | ---------------------------------------------------------------------------------------------------------------|
| Efisiensi memori    | Rust memastikan keamanan memori tanpa garbage collector, sehingga proses konversi lebih stabil dan minim crash.|
| Performa tinggi     | Cocok untuk memproses banyak gambar serta menghasilkan PDF dengan cepat.                                       |
| Functional friendly | Mendukung paradigma pemrograman fungsional seperti immutability, pure functions, dan iterator pipelines.       |

### Tujuan
- Membangun aplikasi konversi gambar ke PDF yang cepat, aman, dan ringan.
- Memberikan pengalaman konversi offline tanpa bergantung pada layanan internet. 
- Mengaplikasikan konsep Functional Programming dalam implementasi proses konversi menggunakan Rust.
---

## Background and Concepts

### Technology Stack

| Komponen         | Teknologi
| ---------------- | ---------
| Backend          | Rust + Axum
| Desktop Frontend | (HTML/CSS/JS) | 
| Runtime Async    | Tokio
| Image Processing | Image (Rust crate)
| PDF Generator    | Printpdf
| Logging          | Tracing

### Konsep Pemrograman Fungsional Dalam Sistem

| Konsep PF         | Implementasi Dalam Proyek |
| ----------------  | ---------                 |
| Pure Function     | load_image_from_bytes selalu menghasilkan output yang sama untuk input yang sama, transformasi RGBA → raw bytes tanpa efek samping. |
| Immutability      | Raw image disimpan sebagai Vec<u8> yang bersifat immutable; setiap halaman PDF dibuat sebagai objek baru tanpa mengubah state global.
| Pattern Matching  | match digunakan untuk error handling (Result), branch logic halaman pertama vs halaman berikutnya.
| Stateless Process | Setiap request multipart diproses secara independen tanpa menyimpan state antar request.
| Explicit Data Flow | Data gambar → konversi → scaling → penempatan → PDF final, tanpa shared mutable data.


---

## Source Code and Explanation

## Screenshot

## Conclusion