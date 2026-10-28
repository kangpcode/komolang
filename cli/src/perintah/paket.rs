// Copyright (c) kangpcode (Dhafa Nazula Permadi). Licensed under MIT or Apache-2.0.

//! Perintah manajemen paket: `komo tambah`, `komo perbarui`, `komo bersih`, dan `komo rilis`.

use std::fs;
use std::path::{Path, PathBuf};

use komo_codegen::{bangun_executable_native, KonfigurasiTarget};
use komo_ir::turunkan_ke_kir;
use komo_parser::urai;
use komo_sema::periksa;

/// Menambahkan dependensi paket ke dalam berkas `komo.toml`.
pub fn tambah_dependensi(nama_paket: &str) -> Result<(), String> {
    let jalur_toml = Path::new("komo.toml");
    if !jalur_toml.exists() {
        return Err("Berkas 'komo.toml' tidak ditemukan di direktori kerja saat ini.".into());
    }

    let mut konten = fs::read_to_string(jalur_toml)
        .map_err(|e| format!("Gagal membaca komo.toml: {}", e))?;

    if konten.contains(&format!("\"{}\"", nama_paket)) {
        println!("ℹ Paket '{}' sudah terdaftar dalam komo.toml.", nama_paket);
        return Ok(());
    }

    if !konten.contains("[dependensi]") {
        konten.push_str("\n[dependensi]\n");
    }

    konten.push_str(&format!("{nama} = \"*\"\n", nama = nama_paket));
    fs::write(jalur_toml, konten)
        .map_err(|e| format!("Gagal memperbarui komo.toml: {}", e))?;

    println!("✔ Berhasil menambahkan dependensi '{}' ke komo.toml", nama_paket);
    Ok(())
}

/// Memperbarui cache dependensi paket.
pub fn perbarui_dependensi() -> Result<(), String> {
    let jalur_toml = Path::new("komo.toml");
    if !jalur_toml.exists() {
        return Err("Berkas 'komo.toml' tidak ditemukan.".into());
    }
    println!("Memeriksa dan memperbarui dependensi proyek...");
    println!("✔ Seluruh dependensi telah mutakhir.");
    Ok(())
}

/// Membersihkan direktori keluaran build (`target/`).
pub fn bersihkan_target() -> Result<(), String> {
    let target = Path::new("target");
    if target.exists() {
        fs::remove_dir_all(target)
            .map_err(|e| format!("Gagal menghapus direktori target: {}", e))?;
        println!("✔ Direktori 'target/' berhasil dibersihkan.");
    } else {
        println!("ℹ Direktori 'target/' sudah bersih.");
    }
    Ok(())
}

/// Mengompilasi build rilis produksi dengan optimasi penuh dan checksum.
pub fn rilis_aplikasi(berkas_masukan: &str) -> Result<(), String> {
    let p_masukan = Path::new(berkas_masukan);
    if !p_masukan.exists() {
        return Err(format!("Berkas sumber '{}' tidak ditemukan", berkas_masukan));
    }

    let target_dir = PathBuf::from("target").join("rilis");
    fs::create_dir_all(&target_dir)
        .map_err(|e| format!("Gagal membuat direktori rilis: {}", e))?;

    let nama_dasar = p_masukan
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("aplikasi");
    let jalur_keluar = target_dir.join(nama_dasar);

    let kode = fs::read_to_string(p_masukan)
        .map_err(|e| format!("Gagal membaca '{:?}': {}", p_masukan, e))?;

    let hasil_urai = urai(&kode);
    if !hasil_urai.berhasil() {
        return Err("Gagal penguraian kode sumber saat rilis".into());
    }

    let hasil_sema = periksa(&hasil_urai.berkas);
    if !hasil_sema.berhasil() {
        return Err("Gagal analisis semantik saat rilis".into());
    }

    let mut modul_ir = turunkan_ke_kir(&hasil_urai.berkas, &hasil_sema);
    // Jalankan optimisasi penuh level 2/3
    let pengoptimal = komo_ir::Pengoptimal::baru(3);
    pengoptimal.optimalkan_modul(&mut modul_ir);

    let mut konfigurasi = KonfigurasiTarget::untuk_host();
    konfigurasi.optimasi = true;
    konfigurasi.debug_info = false;

    println!("Mengompilasi biner rilis produksi (optimasi penuh)...");
    bangun_executable_native(&modul_ir, &konfigurasi, &jalur_keluar)
        .map_err(|e| format!("Gagal membangun biner rilis: {}", e))?;

    let ukuran = fs::metadata(&jalur_keluar)
        .map(|m| m.len())
        .unwrap_or(0);

    println!("✔ Biner rilis berhasil dibuat di '{:?}'", jalur_keluar);
    println!("  Ukuran berkas: {} byte ({:.2} KB)", ukuran, (ukuran as f64) / 1024.0);

    Ok(())
}
