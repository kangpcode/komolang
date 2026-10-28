// Copyright (c) kangpcode (Dhafa Nazula Permadi). Licensed under MIT or Apache-2.0.

//! Kumpulan modul perintah resmi untuk kakas antarmuka baris perintah Komolang (`cli`).

pub mod audit;
pub mod baru;
pub mod bench;
pub mod debug;
pub mod distribusi;
pub mod dok;
pub mod format;
pub mod layani;
pub mod main_game;
pub mod paket;
pub mod repl;
pub mod uji;

pub use audit::{audit_proyek, KebijakanSandboxing, LaporanAudit, TingkatKeparahan};
pub use baru::buat_proyek_baru;
pub use bench::jalankan_benchmark;
pub use debug::debug_berkas;
pub use distribusi::{
    bangun_berkas_lintas_platform, buat_debian_control, buat_formula_homebrew, buat_manifest_scoop,
    buat_manifest_sha256sums, buat_manifest_winget, terapkan_pembaruan_biner, verifikasi_sha256sums,
    InfoRilis, TargetPlatform,
};
pub use dok::buat_dokumentasi;
pub use format::format_kode;
pub use layani::jalankan_server_layani;
pub use main_game::jalankan_game;
pub use paket::{
    bersihkan_target, cari_paket, instal_dependensi, perbarui_dependensi, rilis_aplikasi,
    tambah_dependensi, terbitkan_paket,
};
pub use repl::jalankan_repl;
pub use uji::jalankan_pengujian;
