// Copyright (c) kangpcode (Dhafa Nazula Permadi). Licensed under MIT or Apache-2.0.

use std::env;
use std::fs;
use std::process;

use komo_diag::render_semua;
use komo_parser::urai;

const VERSI: &str = env!("CARGO_PKG_VERSION");
const HAK_CIPTA: &str =
    "Copyright (c) 2026 kangpcode (Dhafa Nazula Permadi). Lisensi MIT atau Apache-2.0.";

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        tampilkan_bantuan();
        return;
    }

    match args[1].as_str() {
        "versi" | "--versi" | "-v" | "--version" => {
            tampilkan_versi();
        }
        "bantuan" | "--bantuan" | "-h" | "--help" => {
            tampilkan_bantuan();
        }
        "periksa" => {
            if args.len() < 3 {
                eprintln!("Kesalahan: Berikan jalur berkas .komo untuk diperiksa.");
                eprintln!("Contoh: komo periksa berkas.komo");
                process::exit(1);
            }
            periksa_berkas(&args[2]);
        }
        perintah_tak_dikenal => {
            eprintln!("Perintah tidak dikenal: '{}'", perintah_tak_dikenal);
            eprintln!("Gunakan 'komo bantuan' untuk melihat daftar perintah yang tersedia.");
            process::exit(1);
        }
    }
}

fn tampilkan_versi() {
    println!("Komolang (Komodo Language) v{}", VERSI);
    println!("{}", HAK_CIPTA);
}

fn tampilkan_bantuan() {
    println!("Penggunaan: komo [perintah] [opsi]");
    println!();
    println!("Perintah Utama:");
    println!("  periksa <berkas.komo>   Memeriksa sintaks leksikal dan tata bahasa");
    println!("  versi                   Menampilkan informasi versi Komolang");
    println!("  bantuan                 Menampilkan pesan bantuan ini");
    println!();
    println!("Perintah Lanjutan (Rencana Tahap 7):");
    println!("  baru <nama>             Membuat proyek Komolang baru");
    println!("  bangun                  Mengompilasi proyek menjadi biner native");
    println!("  jalankan [berkas.komo]  Menjalankan program Komolang langsung");
    println!("  uji                     Menjalankan suite tes otomatis");
    println!("  format                  Memformat berkas kode sumber Komolang");
}

fn periksa_berkas(jalur: &str) {
    let konten = match fs::read_to_string(jalur) {
        Ok(k) => k,
        Err(e) => {
            eprintln!("Gagal membaca berkas '{}': {}", jalur, e);
            process::exit(1);
        }
    };

    println!("Memeriksa berkas '{}'...", jalur);

    let hasil = urai(&konten);
    if hasil.berhasil() {
        println!(
            "✅ Sintaks valid! Paket: '{}', Impor: {}, Deklarasi: {}",
            hasil.berkas.paket.nama.nama,
            hasil.berkas.impor.len(),
            hasil.berkas.deklarasi.len()
        );
    } else {
        eprintln!(
            "{}",
            render_semua(&hasil.diagnostik, jalur, &konten)
        );
        process::exit(1);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tes_tampilkan_versi() {
        assert!(!VERSI.is_empty());
        assert!(HAK_CIPTA.contains("kangpcode"));
    }
}
