// Copyright (c) kangpcode (Dhafa Nazula Permadi). Licensed under MIT or Apache-2.0.

use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::process;

use komo_codegen::{bangun_executable_native, jalankan_jit, KonfigurasiTarget};
use komo_diag::render_semua;
use komo_ir::ModulIr;
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
            let dump_kir = args.iter().any(|a| a == "--dump-kir");
            periksa_berkas(&args[2], dump_kir);
        }
        "ir" => {
            if args.len() < 3 {
                eprintln!("Kesalahan: Berikan jalur berkas .komo untuk menghasilkan IR.");
                eprintln!("Contoh: komo ir berkas.komo [--optimasi]");
                process::exit(1);
            }
            let optimasi = args.iter().any(|a| a == "--optimasi" || a == "-O");
            cetak_kir(&args[2], optimasi);
        }
        "bangun" => {
            if args.len() < 3 {
                eprintln!("Kesalahan: Berikan jalur berkas .komo untuk dikompilasi.");
                eprintln!("Contoh: komo bangun berkas.komo [-o nama_biner] [--backend=asm|cranelift] [--optimasi]");
                process::exit(1);
            }
            let berkas = &args[2];
            let optimasi = args.iter().any(|a| a == "--optimasi" || a == "-O");
            let pakai_asm = args.iter().any(|a| a == "--backend=asm");
            let mut jalur_keluar = PathBuf::from(berkas.trim_end_matches(".komo"));
            for i in 0..args.len() {
                if args[i] == "-o" && i + 1 < args.len() {
                    jalur_keluar = PathBuf::from(&args[i + 1]);
                }
            }
            bangun_berkas(berkas, &jalur_keluar, optimasi, pakai_asm);
        }
        "asm" => {
            if args.len() < 3 {
                eprintln!("Kesalahan: Berikan jalur berkas .komo untuk menghasilkan kode assembly.");
                eprintln!("Contoh: komo asm berkas.komo [--arsitektur=x86_64|arm64] [--optimasi]");
                process::exit(1);
            }
            let berkas = &args[2];
            let optimasi = args.iter().any(|a| a == "--optimasi" || a == "-O");
            let arm64 = args.iter().any(|a| a == "--arsitektur=arm64" || a == "--target=arm64");
            tampilkan_assembly(berkas, optimasi, arm64);
        }
        "jalankan" => {
            if args.len() < 3 {
                eprintln!("Kesalahan: Berikan jalur berkas .komo untuk dijalankan.");
                eprintln!("Contoh: komo jalankan berkas.komo [--optimasi]");
                process::exit(1);
            }
            let berkas = &args[2];
            let optimasi = args.iter().any(|a| a == "--optimasi" || a == "-O");
            jalankan_berkas(berkas, optimasi);
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
    println!("  bangun <berkas.komo>    Mengompilasi menjadi executable (--backend=asm|cranelift, -o, --optimasi)");
    println!("  jalankan <berkas.komo>  Menjalankan program Komolang langsung via mesin JIT native (--optimasi)");
    println!("  asm <berkas.komo>       Mencetak kode assembly x86_64 / ARM64 mandiri ke stdout (--arsitektur=arm64)");
    println!("  periksa <berkas.komo>   Memeriksa sintaks leksikal dan tata bahasa (--dump-kir)");
    println!("  ir <berkas.komo>        Menghasilkan dan menampilkan dump KIR SSA (--optimasi)");
    println!("  versi                   Menampilkan informasi versi Komolang");
    println!("  bantuan                 Menampilkan pesan bantuan ini");
    println!();
    println!("Perintah Lanjutan (Rencana Tahap 7):");
    println!("  baru <nama>             Membuat proyek Komolang baru");
    println!("  uji                     Menjalankan suite tes otomatis");
    println!("  format                  Memformat berkas kode sumber Komolang");
}

fn parsing_dan_lowering(jalur: &str, optimasi: bool) -> ModulIr {
    let konten = match fs::read_to_string(jalur) {
        Ok(k) => k,
        Err(e) => {
            eprintln!("Gagal membaca berkas '{}': {}", jalur, e);
            process::exit(1);
        }
    };

    let hasil_urai = urai(&konten);
    if !hasil_urai.berhasil() {
        eprintln!("{}", render_semua(&hasil_urai.diagnostik, jalur, &konten));
        process::exit(1);
    }

    let hasil_sema = komo_sema::periksa(&hasil_urai.berkas);
    if !hasil_sema.berhasil() {
        eprintln!("{}", render_semua(&hasil_sema.diagnostik, jalur, &konten));
        process::exit(1);
    }

    let mut modul_ir = komo_ir::turunkan_ke_kir(&hasil_urai.berkas, &hasil_sema);

    if optimasi {
        let pengoptimal = komo_ir::Pengoptimal::baru(2);
        pengoptimal.optimalkan_modul(&mut modul_ir);
    }

    if let Err(galat_verif) = komo_ir::verifikasi_modul(&modul_ir) {
        eprintln!("❌ Galat Verifikasi KIR:");
        for g in galat_verif {
            eprintln!("  - {}", g);
        }
        process::exit(1);
    }

    modul_ir
}

fn bangun_berkas(jalur: &str, output: &Path, optimasi: bool, pakai_asm: bool) {
    let modul_ir = parsing_dan_lowering(jalur, optimasi);

    if pakai_asm {
        println!(
            "Mengompilasi '{}' menjadi biner native '{}' (backend: ASM internal)...",
            jalur,
            output.display()
        );
        match komo_asm::bangun_executable_asm(&modul_ir, output) {
            Ok(path) => {
                println!("✅ Berhasil membangun biner native (ASM): {}", path.display());
            }
            Err(e) => {
                eprintln!("❌ Gagal membangun biner ASM: {}", e);
                process::exit(1);
            }
        }
        return;
    }

    let mut target = KonfigurasiTarget::untuk_host();
    target.optimasi = optimasi;

    println!(
        "Mengompilasi '{}' menjadi biner native '{}' (target: {})...",
        jalur,
        output.display(),
        target.sebagai_triple_str()
    );

    match bangun_executable_native(&modul_ir, &target, output) {
        Ok(path) => {
            println!("✅ Berhasil membangun biner native: {}", path.display());
        }
        Err(e) => {
            eprintln!("❌ Gagal menautkan biner native: {}", e);
            process::exit(1);
        }
    }
}

fn tampilkan_assembly(jalur: &str, optimasi: bool, arm64: bool) {
    let modul_ir = parsing_dan_lowering(jalur, optimasi);
    let target = if arm64 {
        komo_asm::TargetArsitekturAsm::AArch64
    } else {
        komo_asm::TargetArsitekturAsm::X86_64
    };
    let asm = komo_asm::kompilasi_ke_assembly(&modul_ir, target);
    print!("{}", asm);
}

fn jalankan_berkas(jalur: &str, optimasi: bool) {
    let modul_ir = parsing_dan_lowering(jalur, optimasi);
    match jalankan_jit(&modul_ir) {
        Ok(kode_keluar) => {
            if kode_keluar != 0 {
                process::exit(kode_keluar as i32);
            }
        }
        Err(e) => {
            eprintln!("❌ Gagal mengeksekusi program: {}", e);
            process::exit(1);
        }
    }
}

fn periksa_berkas(jalur: &str, dump_kir: bool) {
    let konten = match fs::read_to_string(jalur) {
        Ok(k) => k,
        Err(e) => {
            eprintln!("Gagal membaca berkas '{}': {}", jalur, e);
            process::exit(1);
        }
    };

    println!("Memeriksa berkas '{}'...", jalur);

    let hasil_urai = urai(&konten);
    if !hasil_urai.berhasil() {
        eprintln!("{}", render_semua(&hasil_urai.diagnostik, jalur, &konten));
        process::exit(1);
    }

    let hasil_sema = komo_sema::periksa(&hasil_urai.berkas);
    if !hasil_sema.berhasil() {
        eprintln!("{}", render_semua(&hasil_sema.diagnostik, jalur, &konten));
        process::exit(1);
    }

    println!(
        "✅ Sintaks dan semantik valid! Paket: '{}', Impor: {}, Deklarasi: {}",
        hasil_urai.berkas.paket.nama.nama,
        hasil_urai.berkas.impor.len(),
        hasil_urai.berkas.deklarasi.len()
    );

    if dump_kir {
        let modul_ir = komo_ir::turunkan_ke_kir(&hasil_urai.berkas, &hasil_sema);
        if let Err(galat_verif) = komo_ir::verifikasi_modul(&modul_ir) {
            eprintln!("❌ Galat Verifikasi KIR:");
            for g in galat_verif {
                eprintln!("  - {}", g);
            }
            process::exit(1);
        }
        println!("\n{}", komo_ir::cetak_modul(&modul_ir));
    }
}

fn cetak_kir(jalur: &str, optimasi: bool) {
    let modul_ir = parsing_dan_lowering(jalur, optimasi);
    println!("{}", komo_ir::cetak_modul(&modul_ir));
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
