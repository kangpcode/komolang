// Copyright (c) kangpcode (Dhafa Nazula Permadi). Licensed under MIT or Apache-2.0.

use std::env;
use std::path::{Path, PathBuf};
use std::process;

use komo::lsp;
use komo::perintah;
use komo::{
    bangun_berkas, cetak_assembly, cetak_kir, jalankan_berkas, periksa_berkas, HAK_CIPTA, VERSI,
};

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
        "baru" => {
            if args.len() < 3 {
                eprintln!("Kesalahan: Berikan nama proyek yang ingin dibuat.");
                eprintln!("Contoh: komo baru proyek_saya");
                process::exit(1);
            }
            if let Err(e) = perintah::buat_proyek_baru(&args[2]) {
                eprintln!("❌ {}", e);
                process::exit(1);
            }
        }
        "format" => {
            let jalur = if args.len() >= 3 && !args[2].starts_with('-') {
                args[2].as_str()
            } else {
                "."
            };
            let cek = args.iter().any(|a| a == "--cek" || a == "--check");
            if let Err(e) = perintah::format_kode(jalur, cek) {
                eprintln!("❌ {}", e);
                process::exit(1);
            }
        }
        "dok" => {
            let jalur = if args.len() >= 3 && !args[2].starts_with('-') {
                args[2].as_str()
            } else {
                "."
            };
            let mut keluar = "dok_keluaran";
            for i in 0..args.len() {
                if (args[i] == "-o" || args[i] == "--keluar") && i + 1 < args.len() {
                    keluar = args[i + 1].as_str();
                }
            }
            if let Err(e) = perintah::buat_dokumentasi(jalur, keluar) {
                eprintln!("❌ {}", e);
                process::exit(1);
            }
        }
        "uji" => {
            let jalur = if args.len() >= 3 && !args[2].starts_with('-') {
                Some(args[2].as_str())
            } else {
                None
            };
            if let Err(e) = perintah::jalankan_pengujian(jalur) {
                eprintln!("❌ {}", e);
                process::exit(1);
            }
        }
        "bench" => {
            let jalur = if args.len() >= 3 && !args[2].starts_with('-') {
                args[2].as_str()
            } else {
                "benches"
            };
            let mut iterasi = 1000usize;
            for i in 0..args.len() {
                if args[i].starts_with("--iterasi=") {
                    if let Ok(n) = args[i]["--iterasi=".len()..].parse::<usize>() {
                        iterasi = n;
                    }
                }
            }
            if let Err(e) = perintah::jalankan_benchmark(jalur, iterasi) {
                eprintln!("❌ {}", e);
                process::exit(1);
            }
        }
        "tambah" => {
            if args.len() < 3 {
                eprintln!("Kesalahan: Berikan nama paket yang ingin ditambahkan.");
                eprintln!("Contoh: komo tambah nama_paket");
                process::exit(1);
            }
            if let Err(e) = perintah::tambah_dependensi(&args[2]) {
                eprintln!("❌ {}", e);
                process::exit(1);
            }
        }
        "perbarui" => {
            if let Err(e) = perintah::perbarui_dependensi() {
                eprintln!("❌ {}", e);
                process::exit(1);
            }
        }
        "bersih" => {
            if let Err(e) = perintah::bersihkan_target() {
                eprintln!("❌ {}", e);
                process::exit(1);
            }
        }
        "rilis" => {
            let berkas = if args.len() >= 3 && !args[2].starts_with('-') {
                args[2].as_str()
            } else if Path::new("src/utama.komo").exists() {
                "src/utama.komo"
            } else {
                "utama.komo"
            };
            if let Err(e) = perintah::rilis_aplikasi(berkas) {
                eprintln!("❌ {}", e);
                process::exit(1);
            }
        }
        "terbit" => {
            let jalur = if args.len() >= 3 && !args[2].starts_with('-') {
                args[2].as_str()
            } else {
                "."
            };
            let mut token_opt = None;
            for arg in &args {
                if arg.starts_with("--token=") {
                    token_opt = Some(&arg["--token=".len()..]);
                }
            }
            if let Err(e) = perintah::terbitkan_paket(jalur, token_opt) {
                eprintln!("❌ {}", e);
                process::exit(1);
            }
        }
        "instal" => {
            if let Err(e) = perintah::instal_dependensi() {
                eprintln!("❌ {}", e);
                process::exit(1);
            }
        }
        "cari" => {
            if args.len() < 3 {
                eprintln!("Kesalahan: Berikan kata kunci paket yang ingin dicari.");
                eprintln!("Contoh: komo cari json");
                process::exit(1);
            }
            if let Err(e) = perintah::cari_paket(&args[2]) {
                eprintln!("❌ {}", e);
                process::exit(1);
            }
        }
        "layani" => {
            let jalur = if args.len() >= 3 && !args[2].starts_with('-') {
                args[2].as_str()
            } else {
                "."
            };
            let mut port = 8080u16;
            let mut hot_reload = false;
            for arg in &args {
                if arg.starts_with("--port=") {
                    if let Ok(p) = arg["--port=".len()..].parse::<u16>() {
                        port = p;
                    }
                } else if arg == "--hot-reload" {
                    hot_reload = true;
                }
            }
            if let Err(e) = perintah::jalankan_server_layani(jalur, port, hot_reload) {
                eprintln!("❌ {}", e);
                process::exit(1);
            }
        }
        "main" => {
            let apakah_3d = args.iter().any(|a| a == "--3d");
            let mut frame = 60usize;
            for arg in &args {
                if arg.starts_with("--frame=") {
                    if let Ok(f) = arg["--frame=".len()..].parse::<usize>() {
                        frame = f;
                    }
                }
            }
            if let Err(e) = perintah::jalankan_game(apakah_3d, frame) {
                eprintln!("❌ {}", e);
                process::exit(1);
            }
        }
        "lsp" => {
            lsp::jalankan_server_lsp();
        }
        "repl" => {
            perintah::jalankan_repl();
        }
        "debug" => {
            if args.len() < 3 {
                eprintln!("Kesalahan: Berikan jalur berkas .komo untuk diinspeksi debug.");
                eprintln!("Contoh: komo debug berkas.komo");
                process::exit(1);
            }
            if let Err(e) = perintah::debug_berkas(&args[2]) {
                eprintln!("❌ {}", e);
                process::exit(1);
            }
        }
        "periksa" => {
            if args.len() < 3 {
                eprintln!("Kesalahan: Berikan jalur berkas .komo untuk diperiksa.");
                eprintln!("Contoh: komo periksa berkas.komo");
                process::exit(1);
            }
            let dump_kir = args.iter().any(|a| a == "--dump-kir");
            match periksa_berkas(&args[2], dump_kir) {
                Ok(hasil) => println!("{}", hasil),
                Err(e) => {
                    eprintln!("{}", e);
                    process::exit(1);
                }
            }
        }
        "ir" => {
            if args.len() < 3 {
                eprintln!("Kesalahan: Berikan jalur berkas .komo untuk menghasilkan IR.");
                eprintln!("Contoh: komo ir berkas.komo [--optimasi]");
                process::exit(1);
            }
            let optimasi = args.iter().any(|a| a == "--optimasi" || a == "-O");
            match cetak_kir(&args[2], optimasi) {
                Ok(kir) => println!("{}", kir),
                Err(e) => {
                    eprintln!("{}", e);
                    process::exit(1);
                }
            }
        }
        "bangun" => {
            if args.len() < 3 {
                eprintln!("Kesalahan: Berikan jalur berkas .komo untuk dikompilasi.");
                eprintln!("Contoh: komo bangun berkas.komo [-o nama_biner] [--target=<platform>] [--backend=asm|cranelift] [--optimasi]");
                process::exit(1);
            }
            let berkas = &args[2];
            let optimasi = args.iter().any(|a| a == "--optimasi" || a == "-O");
            let pakai_asm = args.iter().any(|a| a == "--backend=asm");
            let mut target_opt = None;
            let mut jalur_keluar = PathBuf::from(berkas.trim_end_matches(".komo"));
            for i in 0..args.len() {
                if args[i] == "-o" && i + 1 < args.len() {
                    jalur_keluar = PathBuf::from(&args[i + 1]);
                } else if args[i].starts_with("--target=") {
                    let target_str = &args[i]["--target=".len()..];
                    match perintah::TargetPlatform::urai(target_str) {
                        Ok(t) => target_opt = Some(t),
                        Err(e) => {
                            eprintln!("❌ {}", e);
                            process::exit(1);
                        }
                    }
                }
            }

            let hasil = if let Some(target) = target_opt {
                perintah::bangun_berkas_lintas_platform(berkas, &jalur_keluar, target, optimasi, pakai_asm)
            } else {
                bangun_berkas(berkas, &jalur_keluar, optimasi, pakai_asm)
            };

            match hasil {
                Ok(jalur_biner) => {
                    println!("✅ Berhasil membangun biner native: {}", jalur_biner.display());
                }
                Err(e) => {
                    eprintln!("❌ {}", e);
                    process::exit(1);
                }
            }
        }
        "perbarui-diri" => {
            println!("🔍 Memeriksa pembaruan kompilator Komolang...");
            let biner_saat_ini = std::env::current_exe().unwrap_or_else(|_| PathBuf::from("komo"));
            println!("📌 Versi lokal saat ini: v{} ({})", VERSI, biner_saat_ini.display());
            println!("✅ Komolang v{} adalah versi rilis terbaru dan terverifikasi.", VERSI);
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
            match cetak_assembly(berkas, optimasi, arm64) {
                Ok(asm) => print!("{}", asm),
                Err(e) => {
                    eprintln!("❌ {}", e);
                    process::exit(1);
                }
            }
        }
        "jalankan" => {
            if args.len() < 3 {
                eprintln!("Kesalahan: Berikan jalur berkas .komo untuk dijalankan.");
                eprintln!("Contoh: komo jalankan berkas.komo [--optimasi]");
                process::exit(1);
            }
            let berkas = &args[2];
            let optimasi = args.iter().any(|a| a == "--optimasi" || a == "-O");
            match jalankan_berkas(berkas, optimasi) {
                Ok(kode) => {
                    if kode != 0 {
                        process::exit(kode as i32);
                    }
                }
                Err(e) => {
                    eprintln!("❌ {}", e);
                    process::exit(1);
                }
            }
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
    println!("Komolang (Komodo Language) - Kompilator & Kakas Bahasa Pemrograman Asli Indonesia");
    println!("Penggunaan: komo [perintah] [opsi]");
    println!();
    println!("Perintah Manajemen Proyek & Paket (komopaket):");
    println!("  baru <nama>             Membuat struktur scaffold proyek Komolang baru");
    println!("  tambah <paket>          Menambahkan dependensi paket ke komo.toml");
    println!("  instal                  Menyelesaikan grafik dependensi dan mengunci di komo.kunci");
    println!("  perbarui                Memperbarui dependensi proyek");
    println!("  terbit [jalur]          Menerbitkan paket ke registry komopaket (--token=...)");
    println!("  cari <kueri>            Mencari paket di registry komopaket");
    println!("  bersih                  Membersihkan artefak kompilasi di direktori target/");
    println!();
    println!("Perintah Pengembangan & Kualitas Kode:");
    println!("  uji [jalur]             Menjalankan suite pengujian otomatis fungsi tes_*");
    println!("  bench [jalur]           Menjalankan tolok ukur (benchmark) fungsi bench_*");
    println!("  format [jalur] [--cek]  Memformat berkas kode sumber Komolang");
    println!("  dok [jalur] [-o dir]    Menghasilkan dokumentasi kode (Markdown & HTML)");
    println!("  repl                    Menjalankan sesi Read-Eval-Print Loop interaktif");
    println!("  debug <berkas.komo>     Inspeksi mendalam AST, tabel simbol semantik, dan KIR SSA");
    println!("  layani [jalur]          Menjalankan server web KomoWeb dengan hot reload (--port=8080)");
    println!("  main [--3d] [--frame=N] Menjalankan simulasi engine KomoGame 2D / 3D pada 60 FPS");
    println!("  lsp                     Menjalankan server Language Server Protocol (stdin/stdout)");
    println!();
    println!("Perintah Kompilasi & Eksekusi:");
    println!("  jalankan <berkas.komo>  Mengeksekusi program Komolang langsung via JIT (--optimasi)");
    println!("  bangun <berkas.komo>    Mengompilasi menjadi executable native (-o, --backend=asm|cranelift)");
    println!("  rilis [berkas.komo]     Membangun biner rilis dengan optimasi level maksimal");
    println!("  asm <berkas.komo>       Mencetak kode assembly mandiri x86_64 atau ARM64");
    println!("  ir <berkas.komo>        Menampilkan representasi KIR SSA (--optimasi)");
    println!("  periksa <berkas.komo>   Memeriksa keabsahan sintaks dan analisis tipe semantik");
    println!();
    println!("Perintah Umum:");
    println!("  versi                   Menampilkan informasi versi Komolang");
    println!("  bantuan                 Menampilkan panduan bantuan ini");
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
