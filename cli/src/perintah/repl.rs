// Copyright (c) kangpcode (Dhafa Nazula Permadi). Licensed under MIT or Apache-2.0.

//! Perintah `komo repl` untuk sesi interaktif Read-Eval-Print Loop.

use std::io::{self, BufRead, Write};

use komo_codegen::jalankan_jit;
use komo_diag::render_semua;
use komo_ir::turunkan_ke_kir;
use komo_parser::urai;
use komo_sema::periksa;

/// Menjalankan REPL interaktif Komolang.
pub fn jalankan_repl() {
    println!("Komolang REPL (Read-Eval-Print Loop) v{}", env!("CARGO_PKG_VERSION"));
    println!("Ketik ':bantuan' untuk petunjuk perintah, atau ':keluar' untuk mengakhiri sesi.");
    println!();

    let stdin = io::stdin();
    let mut baris_input = String::new();

    loop {
        print!("komo> ");
        let _ = io::stdout().flush();

        baris_input.clear();
        let bytes_read = match stdin.lock().read_line(&mut baris_input) {
            Ok(b) => b,
            Err(_) => break,
        };

        if bytes_read == 0 {
            // EOF
            println!();
            break;
        }

        let trimmed = baris_input.trim();
        if trimmed.is_empty() {
            continue;
        }

        match trimmed {
            ":keluar" | ":exit" | ":q" => {
                println!("Sampai jumpa!");
                break;
            }
            ":bantuan" | ":help" => {
                println!("Perintah REPL:");
                println!("  :keluar       Keluar dari sesi REPL");
                println!("  :bantuan      Menampilkan panduan ini");
                println!("  :bersih       Membersihkan layar konsol");
                println!("  <ekspresi>    Mengevaluasi ekspresi Komolang secara langsung");
                println!("  <fungsi>      Mendefinisikan fungsi Komolang");
                continue;
            }
            ":bersih" | ":clear" => {
                print!("\x1B[2J\x1B[1;1H");
                let _ = io::stdout().flush();
                continue;
            }
            _ => {}
        }

        evaluasi_masukan_repl(trimmed);
    }
}

fn evaluasi_masukan_repl(masukan: &str) {
    // Bungkus masukan menjadi program komolang utuh
    let kode = if masukan.starts_with("fungsi ")
        || masukan.starts_with("publik fungsi ")
        || masukan.starts_with("tipe ")
    {
        format!(
            "paket repl\n\n{}\n\npublik fungsi utama() bulat {{\n    kembali 0\n}}\n",
            masukan
        )
    } else {
        format!(
            "paket repl\n\nimpor \"std/cetak\"\n\npublik fungsi utama() bulat {{\n    var hasil = {}\n    cetak.tulis_bulat(hasil)\n    kembali hasil\n}}\n",
            masukan
        )
    };

    let hasil_urai = urai(&kode);
    if !hasil_urai.berhasil() {
        eprintln!("{}", render_semua(&hasil_urai.diagnostik, "repl", &kode));
        return;
    }

    let hasil_sema = periksa(&hasil_urai.berkas);
    if !hasil_sema.berhasil() {
        eprintln!("{}", render_semua(&hasil_sema.diagnostik, "repl", &kode));
        return;
    }

    let modul_ir = turunkan_ke_kir(&hasil_urai.berkas, &hasil_sema);
    match jalankan_jit(&modul_ir) {
        Ok(kode_keluar) => {
            if !masukan.starts_with("fungsi ") && !masukan.starts_with("tipe ") {
                println!("=> {}", kode_keluar);
            } else {
                println!("✔ Terdefinisi");
            }
        }
        Err(e) => {
            eprintln!("Galat eksekusi: {}", e);
        }
    }
}
