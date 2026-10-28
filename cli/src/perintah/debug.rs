// Copyright (c) kangpcode (Dhafa Nazula Permadi). Licensed under MIT or Apache-2.0.

//! Perintah `komo debug <berkas.komo>` untuk inspeksi diagnostik compiler dan informasi debug DWARF.

use std::fs;
use std::path::Path;

use komo_diag::render_semua;
use komo_ir::turunkan_ke_kir;
use komo_parser::urai;
use komo_sema::periksa;

/// Melakukan inspeksi debug mendalam terhadap berkas sumber Komolang.
pub fn debug_berkas(jalur: &str) -> Result<(), String> {
    let p = Path::new(jalur);
    if !p.exists() {
        return Err(format!("Berkas sumber '{}' tidak ditemukan", jalur));
    }

    let kode = fs::read_to_string(p)
        .map_err(|e| format!("Gagal membaca '{:?}': {}", p, e))?;

    println!("=== 1. DIAGNOSTIK PENGURAI SINTAKS (AST) ===");
    let hasil_urai = urai(&kode);
    if !hasil_urai.berhasil() {
        eprintln!("{}", render_semua(&hasil_urai.diagnostik, jalur, &kode));
        return Err("Penguraian gagal".into());
    }
    println!("Paket: {}", hasil_urai.berkas.paket.nama.nama);
    println!("Jumlah Deklarasi: {}", hasil_urai.berkas.deklarasi.len());
    println!("Jumlah Impor:     {}", hasil_urai.berkas.impor.len());

    println!("\n=== 2. DIAGNOSTIK ANALISIS SEMANTIK & TABEL SIMBOL ===");
    let hasil_sema = periksa(&hasil_urai.berkas);
    if !hasil_sema.berhasil() {
        eprintln!("{}", render_semua(&hasil_sema.diagnostik, jalur, &kode));
        return Err("Analisis semantik gagal".into());
    }
    println!("Simbol terdaftar berhasil diverifikasi.");

    println!("\n=== 3. DUMP KIR SSA (KOMO INTERMEDIATE REPRESENTATION) ===");
    let modul_ir = turunkan_ke_kir(&hasil_urai.berkas, &hasil_sema);
    println!("{}", komo_ir::cetak_modul(&modul_ir));

    println!("\n=== 4. VERIFIKASI BENTUK SSA IR ===");
    match komo_ir::verifikasi_modul(&modul_ir) {
        Ok(_) => println!("✔ Modul KIR SSA lolos verifikasi 100% tanpa pelanggaran."),
        Err(galat) => {
            eprintln!("✖ Ditemukan pelanggaran verifikasi SSA:");
            for g in galat {
                eprintln!("  - {}", g);
            }
        }
    }

    Ok(())
}
