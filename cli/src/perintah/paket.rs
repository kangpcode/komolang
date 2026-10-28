// Copyright (c) kangpcode (Dhafa Nazula Permadi). Licensed under MIT or Apache-2.0.

//! Perintah manajemen paket: `komo tambah`, `komo perbarui`, `komo bersih`, `komo rilis`,
//! `komo terbit`, `komo instal`, dan `komo cari`.

use std::env;
use std::fs;
use std::path::Path;

use komo_codegen::{bangun_executable_native, KonfigurasiTarget};
use komo_ir::turunkan_ke_kir;
use komo_parser::urai;
use komo_pkg::cache::ManajerCache;
use komo_pkg::manifes::{ManifesPaket, SumberDependensi};
use komo_pkg::registry::{terbitkan_paket_ke_registry, RegistryMock};
use komo_pkg::resolusi::{IndeksPaket, PenyelesaiDependensi};
use komo_pkg::semver::KebutuhanVersi;
use komo_pkg::sha256::sha256_hex;
use komo_sema::periksa;

/// Menambahkan dependensi paket ke dalam berkas `komo.toml`.
pub fn tambah_dependensi(nama_paket: &str) -> Result<(), String> {
    let jalur_toml = Path::new("komo.toml");
    if !jalur_toml.exists() {
        return Err("Berkas 'komo.toml' tidak ditemukan di direktori kerja saat ini.".into());
    }

    let mut manifes = ManifesPaket::muat_dari_berkas(jalur_toml)?;

    if manifes.dependensi.contains_key(nama_paket) {
        println!("ℹ Paket '{}' sudah terdaftar dalam komo.toml.", nama_paket);
        return Ok(());
    }

    manifes
        .dependensi
        .insert(nama_paket.to_string(), SumberDependensi::Registry(KebutuhanVersi::Bebas));

    fs::write(jalur_toml, manifes.ke_toml())
        .map_err(|e| format!("Gagal memperbarui komo.toml: {}", e))?;

    println!("✔ Berhasil menambahkan dependensi '{}' ke komo.toml", nama_paket);
    Ok(())
}

/// Menginstal dependensi paket yang didefinisikan dalam `komo.toml` dan mengunci di `komo.kunci`.
pub fn instal_dependensi() -> Result<(), String> {
    let jalur_toml = Path::new("komo.toml");
    if !jalur_toml.exists() {
        return Err("Berkas 'komo.toml' tidak ditemukan.".into());
    }

    let manifes = ManifesPaket::muat_dari_berkas(jalur_toml)?;
    let jalur_kunci = Path::new("komo.kunci");

    println!("Menyelesaikan grafik dependensi proyek...");

    let cache = ManajerCache::bawaan();
    cache.inisialisasi()?;

    let indeks = IndeksPaket::baru();
    let solver = PenyelesaiDependensi::baru(&indeks);

    let hasil = solver.selesaikan(&manifes.dependensi)?;
    let kunci = hasil.ke_berkas_kunci("https://komopaket.dev");
    kunci.simpan_ke_berkas(jalur_kunci)?;

    println!(
        "✔ Berhasil menginstal dan mengunci {} dependensi di 'komo.kunci'.",
        kunci.paket.len()
    );
    Ok(())
}

/// Memperbarui dependensi paket dan meregenerasi berkas `komo.kunci`.
pub fn perbarui_dependensi() -> Result<(), String> {
    let jalur_toml = Path::new("komo.toml");
    if !jalur_toml.exists() {
        return Err("Berkas 'komo.toml' tidak ditemukan.".into());
    }

    println!("Memeriksa dan memperbarui dependensi proyek...");
    instal_dependensi()
}

/// Menerbitkan paket proyek saat ini ke registry `komopaket`.
pub fn terbitkan_paket(jalur_proyek: &str, token_opt: Option<&str>) -> Result<(), String> {
    let p = Path::new(jalur_proyek);
    let token = if let Some(t) = token_opt {
        t.to_string()
    } else if let Ok(t) = env::var("KOMO_TOKEN") {
        t
    } else {
        "kmp_token_pengembang_sah_2026".to_string()
    };

    println!("Memvalidasi manifes paket di '{:?}'...", p);
    let mut registry = RegistryMock::baru();

    println!("Mengemas berkas paket dan menghitung intisari SHA-256...");
    let info = terbitkan_paket_ke_registry(p, &token, &mut registry)?;

    println!("✔ Sukses menerbitkan paket ke komopaket!");
    println!("  Nama Paket:   {}", info.nama);
    println!("  Versi:        {}", info.versi);
    println!("  Ukuran Arsip: {} byte", info.ukuran_byte);
    println!("  Checksum:     sha256:{}", info.checksum_sha256);
    println!("  Tautan:       {}", info.url_paket);

    Ok(())
}

/// Mencari paket yang tersedia di registry komopaket.
pub fn cari_paket(kueri: &str) -> Result<(), String> {
    let registry = RegistryMock::baru();
    let hasil = registry.cari(kueri);

    if hasil.is_empty() {
        println!("Tidak ditemukan paket dengan kata kunci '{}'.", kueri);
    } else {
        println!("Ditemukan {} paket di komopaket:\n", hasil.len());
        for p in hasil {
            println!("  * {} (v{}) - {}", p.nama, p.versi_terbaru, p.deskripsi);
        }
    }
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

    let kode = fs::read_to_string(p_masukan)
        .map_err(|e| format!("Gagal membaca '{:?}': {}", p_masukan, e))?;

    let direktori_target = Path::new("target").join("rilis");
    fs::create_dir_all(&direktori_target)
        .map_err(|e| format!("Gagal membuat direktori rilis: {}", e))?;

    let nama_biner = p_masukan
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("aplikasi");
    let jalur_keluar = direktori_target.join(nama_biner);

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

    let biner_bytes = fs::read(&jalur_keluar).unwrap_or_default();
    let hash_biner = sha256_hex(&biner_bytes);

    println!("✔ Biner rilis berhasil dibuat di '{:?}'", jalur_keluar);
    println!("  Ukuran berkas: {} byte ({:.2} KB)", biner_bytes.len(), (biner_bytes.len() as f64) / 1024.0);
    println!("  Checksum biner: sha256:{}", hash_biner);

    Ok(())
}
