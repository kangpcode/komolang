// Copyright (c) kangpcode (Dhafa Nazula Permadi). Licensed under MIT or Apache-2.0.

//! Sistem pemanggilan linker untuk merakit berkas objek (.o / .obj) menjadi biner executable mandiri.
//! Mendukung linker sistem standar (cc, gcc, clang di Linux/macOS, link.exe di Windows).

use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

use crate::modul::HasilKompilasiObjek;
use crate::runtime::kode_sumber_c_runtime;

/// Menautkan (link) berkas objek yang dihasilkan menjadi berkas biner executable mandiri.
pub fn tautkan_executable(
    hasil_objek: &HasilKompilasiObjek,
    jalur_keluaran: &Path,
) -> Result<PathBuf, String> {
    let direktori_sementara = std::env::temp_dir();
    let nama_dasar = format!("komo_{}", hasil_objek.nama_modul);

    let jalur_obj = direktori_sementara.join(format!("{}.o", nama_dasar));
    let jalur_runtime_c = direktori_sementara.join(format!("{}_rt.c", nama_dasar));

    // 1. Tulis berkas objek ke disk
    fs::write(&jalur_obj, &hasil_objek.bytes_objek)
        .map_err(|e| format!("Gagal menulis berkas objek sementara '{:?}': {}", jalur_obj, e))?;

    // 2. Tulis kode sumber C runtime pendukung ke disk
    fs::write(&jalur_runtime_c, kode_sumber_c_runtime())
        .map_err(|e| format!("Gagal menulis kode runtime sementara '{:?}': {}", jalur_runtime_c, e))?;

    // 3. Tentukan linker yang tersedia di sistem
    let perintah_linker = temukan_perintah_linker()?;

    // 4. Jalankan perintah kompilasi dan linking
    let status = Command::new(&perintah_linker)
        .arg(&jalur_obj)
        .arg(&jalur_runtime_c)
        .arg("-o")
        .arg(jalur_keluaran)
        .arg("-lm")
        .status()
        .map_err(|e| {
            format!(
                "Gagal menjalankan linker '{}' untuk menghasilkan executable: {}",
                perintah_linker, e
            )
        })?;

    // Bersihkan berkas sementara
    let _ = fs::remove_file(&jalur_obj);
    let _ = fs::remove_file(&jalur_runtime_c);

    if !status.success() {
        return Err(format!(
            "Linker '{}' gagal dengan kode keluar: {:?}",
            perintah_linker,
            status.code()
        ));
    }

    Ok(jalur_keluaran.to_path_buf())
}

/// Menemukan perintah compiler/linker C yang tersedia di sistem saat ini.
pub fn temukan_perintah_linker() -> Result<String, String> {
    let kandidat = if cfg!(windows) {
        vec!["clang", "gcc", "cl"]
    } else {
        vec!["cc", "gcc", "clang"]
    };

    for k in kandidat {
        if Command::new(k).arg("--version").output().is_ok() {
            return Ok(k.to_string());
        }
    }

    Err("Tidak ditemukan compiler/linker C di sistem (cc, gcc, clang, atau cl). Pastikan GCC atau Clang terpasang.".to_string())
}
