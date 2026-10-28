// Copyright (c) kangpcode (Dhafa Nazula Permadi). Licensed under MIT or Apache-2.0.

//! Pustaka inti untuk kakas antarmuka baris perintah Komolang (`cli`).
//! Menyediakan perintah-perintah ekosistem, server LSP, dan runner kompilasi.

pub mod lsp;
pub mod perintah;

use std::fs;
use std::path::{Path, PathBuf};

use komo_codegen::{bangun_executable_native, jalankan_jit, KonfigurasiTarget};
use komo_diag::render_semua;
use komo_ir::ModulIr;
use komo_parser::urai;

pub const VERSI: &str = env!("CARGO_PKG_VERSION");
pub const HAK_CIPTA: &str =
    "Copyright (c) 2026 kangpcode (Dhafa Nazula Permadi). Lisensi MIT atau Apache-2.0.";

/// Mengurai kode sumber dan menurunkannya ke Komolang IR (KIR) setelah melewati verifikasi semantik.
pub fn parsing_dan_lowering(jalur: &str, optimasi: bool) -> Result<ModulIr, String> {
    let konten = fs::read_to_string(jalur)
        .map_err(|e| format!("Gagal membaca berkas '{}': {}", jalur, e))?;

    let hasil_urai = urai(&konten);
    if !hasil_urai.berhasil() {
        return Err(render_semua(&hasil_urai.diagnostik, jalur, &konten));
    }

    let hasil_sema = komo_sema::periksa(&hasil_urai.berkas);
    if !hasil_sema.berhasil() {
        return Err(render_semua(&hasil_sema.diagnostik, jalur, &konten));
    }

    let mut modul_ir = komo_ir::turunkan_ke_kir(&hasil_urai.berkas, &hasil_sema);

    if optimasi {
        let pengoptimal = komo_ir::Pengoptimal::baru(2);
        pengoptimal.optimalkan_modul(&mut modul_ir);
    }

    if let Err(galat_verif) = komo_ir::verifikasi_modul(&modul_ir) {
        let mut msg = String::from("❌ Galat Verifikasi KIR:\n");
        for g in galat_verif {
            msg.push_str(&format!("  - {}\n", g));
        }
        return Err(msg);
    }

    Ok(modul_ir)
}

/// Memeriksa sintaks dan semantik suatu berkas Komolang tanpa mengeksekusinya.
pub fn periksa_berkas(jalur: &str, dump_kir: bool) -> Result<String, String> {
    let konten = fs::read_to_string(jalur)
        .map_err(|e| format!("Gagal membaca berkas '{}': {}", jalur, e))?;

    let hasil_urai = urai(&konten);
    if !hasil_urai.berhasil() {
        return Err(render_semua(&hasil_urai.diagnostik, jalur, &konten));
    }

    let hasil_sema = komo_sema::periksa(&hasil_urai.berkas);
    if !hasil_sema.berhasil() {
        return Err(render_semua(&hasil_sema.diagnostik, jalur, &konten));
    }

    let mut ringkasan = format!(
        "✅ Sintaks dan semantik valid! Paket: '{}', Impor: {}, Deklarasi: {}",
        hasil_urai.berkas.paket.nama.nama,
        hasil_urai.berkas.impor.len(),
        hasil_urai.berkas.deklarasi.len()
    );

    if dump_kir {
        let modul_ir = komo_ir::turunkan_ke_kir(&hasil_urai.berkas, &hasil_sema);
        if let Err(galat_verif) = komo_ir::verifikasi_modul(&modul_ir) {
            let mut msg = String::from("❌ Galat Verifikasi KIR:\n");
            for g in galat_verif {
                msg.push_str(&format!("  - {}\n", g));
            }
            return Err(msg);
        }
        ringkasan.push_str("\n\n");
        ringkasan.push_str(&komo_ir::cetak_modul(&modul_ir));
    }

    Ok(ringkasan)
}

/// Menghasilkan representasi string teks dari KIR SSA untuk suatu berkas.
pub fn cetak_kir(jalur: &str, optimasi: bool) -> Result<String, String> {
    let modul_ir = parsing_dan_lowering(jalur, optimasi)?;
    Ok(komo_ir::cetak_modul(&modul_ir))
}

/// Menghasilkan kode assembly target (x86_64 atau ARM64) untuk suatu berkas.
pub fn cetak_assembly(jalur: &str, optimasi: bool, arm64: bool) -> Result<String, String> {
    let modul_ir = parsing_dan_lowering(jalur, optimasi)?;
    let target = if arm64 {
        komo_asm::TargetArsitekturAsm::AArch64
    } else {
        komo_asm::TargetArsitekturAsm::X86_64
    };
    Ok(komo_asm::kompilasi_ke_assembly(&modul_ir, target))
}

/// Membangun berkas executable native dari berkas kode sumber Komolang.
pub fn bangun_berkas(
    jalur: &str,
    output: &Path,
    optimasi: bool,
    pakai_asm: bool,
) -> Result<PathBuf, String> {
    let modul_ir = parsing_dan_lowering(jalur, optimasi)?;

    if pakai_asm {
        komo_asm::bangun_executable_asm(&modul_ir, output)
            .map_err(|e| format!("Gagal membangun biner ASM: {}", e))
    } else {
        let mut target = KonfigurasiTarget::untuk_host();
        target.optimasi = optimasi;
        bangun_executable_native(&modul_ir, &target, output)
            .map_err(|e| format!("Gagal menautkan biner native: {}", e))
    }
}

/// Menjalankan program Komolang langsung di memori menggunakan JIT executor.
pub fn jalankan_berkas(jalur: &str, optimasi: bool) -> Result<i64, String> {
    let modul_ir = parsing_dan_lowering(jalur, optimasi)?;
    jalankan_jit(&modul_ir).map_err(|e| format!("Gagal mengeksekusi program: {}", e))
}
