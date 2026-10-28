// Copyright (c) kangpcode (Dhafa Nazula Permadi). Licensed under MIT or Apache-2.0.

//! Kumpulan modul perintah resmi untuk kakas antarmuka baris perintah Komolang (`cli`).

pub mod baru;
pub mod bench;
pub mod debug;
pub mod dok;
pub mod format;
pub mod paket;
pub mod repl;
pub mod uji;

pub use baru::buat_proyek_baru;
pub use bench::jalankan_benchmark;
pub use debug::debug_berkas;
pub use dok::buat_dokumentasi;
pub use format::format_kode;
pub use paket::{
    bersihkan_target, cari_paket, instal_dependensi, perbarui_dependensi, rilis_aplikasi,
    tambah_dependensi, terbitkan_paket,
};
pub use repl::jalankan_repl;
pub use uji::jalankan_pengujian;
