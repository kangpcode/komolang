// Copyright (c) kangpcode (Dhafa Nazula Permadi). Licensed under MIT or Apache-2.0.

//! Perintah `komo bench [jalur]` untuk benchmark performa eksekusi program Komolang.

use std::fs;
use std::path::Path;
use std::time::Instant;

use komo_codegen::jalankan_jit;
use komo_diag::render_semua;
use komo_ir::turunkan_ke_kir;
use komo_parser::urai;
use komo_sema::periksa;

/// Menjalankan benchmark performa pada berkas atau fungsi Komolang.
pub fn jalankan_benchmark(jalur: &str, jumlah_iterasi: usize) -> Result<(), String> {
    let p = Path::new(jalur);
    if !p.exists() {
        return Err(format!("Berkas benchmark '{}' tidak ditemukan", jalur));
    }

    let kode = fs::read_to_string(p)
        .map_err(|e| format!("Gagal membaca '{:?}': {}", p, e))?;

    let hasil_urai = urai(&kode);
    if !hasil_urai.berhasil() {
        return Err(format!(
            "Gagal kompilasi sintaks:\n{}",
            render_semua(&hasil_urai.diagnostik, jalur, &kode)
        ));
    }

    let hasil_sema = periksa(&hasil_urai.berkas);
    if !hasil_sema.berhasil() {
        return Err(format!(
            "Gagal pemeriksaan semantik:\n{}",
            render_semua(&hasil_sema.diagnostik, jalur, &kode)
        ));
    }

    let modul_ir = turunkan_ke_kir(&hasil_urai.berkas, &hasil_sema);
    println!("Memulai benchmark: {} ({} iterasi)...", jalur, jumlah_iterasi);

    // Warm-up 5 kali
    for _ in 0..5 {
        let _ = jalankan_jit(&modul_ir);
    }

    let mulai = Instant::now();
    for _ in 0..jumlah_iterasi {
        let _ = jalankan_jit(&modul_ir);
    }
    let total_durasi = mulai.elapsed();

    let total_ns = total_durasi.as_nanos();
    let ns_per_op = total_ns / (jumlah_iterasi as u128);
    let ops_per_detik = if total_durasi.as_secs_f64() > 0.0 {
        (jumlah_iterasi as f64) / total_durasi.as_secs_f64()
    } else {
        0.0
    };

    println!("--------------------------------------------------");
    println!("Hasil Benchmark Komolang:");
    println!("  Berkas:           {}", jalur);
    println!("  Total Iterasi:    {}", jumlah_iterasi);
    println!("  Total Durasi:     {:.3?}", total_durasi);
    println!("  Rata-rata Waktu:  {} ns/op", ns_per_op);
    println!("  Throughput:       {:.2} ops/detik", ops_per_detik);
    println!("--------------------------------------------------");

    Ok(())
}
