// Copyright (c) kangpcode (Dhafa Nazula Permadi). Licensed under MIT or Apache-2.0.

//! Perintah `komo format [jalur] [--cek]` untuk pemformatan kode baku Komolang.

use std::fs;
use std::path::Path;

use komo_ast::cetak_berkas;
use komo_diag::render_semua;
use komo_parser::urai;

/// Memformat berkas atau seluruh direktori Komolang.
pub fn format_kode(jalur: &str, hanya_cek: bool) -> Result<(), String> {
    let p = Path::new(jalur);
    if !p.exists() {
        return Err(format!("Jalur berkas atau direktori '{}' tidak ditemukan", jalur));
    }

    if p.is_file() {
        let terubah = format_satu_berkas(p, hanya_cek)?;
        if hanya_cek && terubah {
            return Err(format!("Berkas '{:?}' belum terformat sesuai standar baku Komolang", p));
        }
        Ok(())
    } else {
        format_direktori(p, hanya_cek)
    }
}

fn format_direktori(dir: &Path, hanya_cek: bool) -> Result<(), String> {
    let mut berkas_terformat = 0;
    let mut berkas_perlu_format = 0;

    let mut tumpukan = vec![dir.to_path_buf()];
    while let Some(current) = tumpukan.pop() {
        let entri = match fs::read_dir(&current) {
            Ok(e) => e,
            Err(_) => continue,
        };

        for e in entri.flatten() {
            let path = e.path();
            if path.is_dir() {
                let nama = path.file_name().and_then(|n| n.to_str()).unwrap_or("");
                if nama != "target" && nama != ".git" {
                    tumpukan.push(path);
                }
            } else if path.extension().and_then(|s| s.to_str()) == Some("komo") {
                match format_satu_berkas(&path, hanya_cek) {
                    Ok(terubah) => {
                        if terubah {
                            berkas_perlu_format += 1;
                        } else {
                            berkas_terformat += 1;
                        }
                    }
                    Err(e) => eprintln!("Peringatan: {}", e),
                }
            }
        }
    }

    if hanya_cek {
        if berkas_perlu_format > 0 {
            return Err(format!(
                "{} berkas belum terformat sesuai standar baku Komolang",
                berkas_perlu_format
            ));
        } else {
            println!("✔ Seluruh berkas ({} berkas) sudah terformat dengan benar.", berkas_terformat);
        }
    } else {
        println!(
            "✔ Pemformatan selesai: {} berkas diformat ulang, {} berkas sudah rapi.",
            berkas_perlu_format, berkas_terformat
        );
    }

    Ok(())
}

fn format_satu_berkas(path: &Path, hanya_cek: bool) -> Result<bool, String> {
    let kode_asli = fs::read_to_string(path)
        .map_err(|e| format!("Gagal membaca '{:?}': {}", path, e))?;

    let hasil = urai(&kode_asli);
    if !hasil.berhasil() {
        let rendered = render_semua(&hasil.diagnostik, path.to_str().unwrap_or(""), &kode_asli);
        return Err(format!("Gagal mengurai '{:?}':\n{}", path, rendered));
    }

    let kode_terformat = cetak_berkas(&hasil.berkas);
    if kode_asli == kode_terformat {
        return Ok(false);
    }

    if hanya_cek {
        println!("Perlu format: {:?}", path);
        return Ok(true);
    }

    fs::write(path, kode_terformat)
        .map_err(|e| format!("Gagal menyimpan berkas terformat '{:?}': {}", path, e))?;
    println!("Diformat: {:?}", path);

    Ok(true)
}
