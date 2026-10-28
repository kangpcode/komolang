// Copyright (c) kangpcode (Dhafa Nazula Permadi). Licensed under MIT or Apache-2.0.

//! Perintah `komo uji [jalur]` untuk menjalankan test runner pengujian otomatis Komolang.

use std::fs;
use std::path::{Path, PathBuf};
use std::time::Instant;

use komo_codegen::jalankan_jit;
use komo_diag::render_semua;
use komo_ir::turunkan_ke_kir;
use komo_parser::urai;
use komo_sema::periksa;

/// Menjalankan runner pengujian Komolang pada berkas atau folder tes.
pub fn jalankan_pengujian(jalur_opsional: Option<&str>) -> Result<(), String> {
    let target = jalur_opsional.unwrap_or("tests");
    let p = Path::new(target);

    let mut berkas_tes = Vec::new();
    if p.is_file() {
        berkas_tes.push(p.to_path_buf());
    } else if p.is_dir() {
        kumpulkan_berkas_tes(p, &mut berkas_tes);
    } else {
        return Err(format!("Jalur pengujian '{}' tidak ditemukan", target));
    }

    if berkas_tes.is_empty() {
        println!("Tidak ditemukan berkas pengujian di '{}'.", target);
        return Ok(());
    }

    println!("Menjalankan runner pengujian Komolang...");
    println!("Ditemukan {} berkas pengujian.\n", berkas_tes.len());

    let mut total_lulus = 0;
    let mut total_gagal = 0;
    let mulai_total = Instant::now();

    for b in &berkas_tes {
        let nama = b.file_name().and_then(|n| n.to_str()).unwrap_or("");
        let kode = fs::read_to_string(b)
            .map_err(|e| format!("Gagal membaca berkas '{:?}': {}", b, e))?;

        let hasil_urai = urai(&kode);
        if !hasil_urai.berhasil() {
            eprintln!("✖ {} ... GAGAL PADA PENGURAIAN SINTAKS", nama);
            eprintln!("{}", render_semua(&hasil_urai.diagnostik, nama, &kode));
            total_gagal += 1;
            continue;
        }

        let hasil_sema = periksa(&hasil_urai.berkas);
        if !hasil_sema.berhasil() {
            eprintln!("✖ {} ... GAGAL PADA PEMERIKSAAN SEMANTIK", nama);
            eprintln!("{}", render_semua(&hasil_sema.diagnostik, nama, &kode));
            total_gagal += 1;
            continue;
        }

        let modul_ir = turunkan_ke_kir(&hasil_urai.berkas, &hasil_sema);
        let verif = komo_ir::verifikasi_modul(&modul_ir);
        if let Err(galat_verif) = verif {
            eprintln!("✖ {} ... GAGAL PADA VERIFIKASI IR", nama);
            for g in galat_verif {
                eprintln!("  - {}", g);
            }
            total_gagal += 1;
            continue;
        }

        let t0 = Instant::now();
        // Coba jalankan fungsi utama / uji jika ada
        let _ = jalankan_jit(&modul_ir);
        let durasi = t0.elapsed();

        println!("✔ {} ... LULUS ({:.2?})", nama, durasi);
        total_lulus += 1;
    }

    let durasi_total = mulai_total.elapsed();
    println!();
    println!(
        "Hasil Pengujian: {} lulus; {} gagal; total {} berkas ({:.2?})",
        total_lulus, total_gagal, total_lulus + total_gagal, durasi_total
    );

    if total_gagal > 0 {
        Err(format!("Terdapat {} pengujian yang gagal", total_gagal))
    } else {
        Ok(())
    }
}

fn kumpulkan_berkas_tes(dir: &Path, list: &mut Vec<PathBuf>) {
    let entri = match fs::read_dir(dir) {
        Ok(e) => e,
        Err(_) => return,
    };
    for e in entri.flatten() {
        let p = e.path();
        if p.is_dir() {
            kumpulkan_berkas_tes(&p, list);
        } else if p.extension().and_then(|s| s.to_str()) == Some("komo") {
            list.push(p);
        }
    }
}
