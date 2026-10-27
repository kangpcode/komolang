// Copyright (c) kangpcode (Dhafa Nazula Permadi). Licensed under MIT or Apache-2.0.

//! Assembler mandiri dan penghubung toolchain assembly Komolang.
//! Menerjemahkan kode assembly x86_64 / ARM64 menjadi berkas objek biner dan executable.

use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::atomic::{AtomicUsize, Ordering};

static PENGHITUNG_BERKAS_ASM: AtomicUsize = AtomicUsize::new(1);

use crate::format::elf::{buat_objek_elf64, ArsitekturElf, SimbolElf};
use crate::format::macho::{buat_objek_macho64, ArsitekturMachO};
use crate::format::pe_coff::{buat_objek_pe_coff, ArsitekturCoff};

/// Format berkas biner objek target.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FormatBiner {
    Elf64,
    MachO64,
    PeCoff,
}

/// Pilihan arsitektur untuk assembler.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TargetArsitekturAsm {
    X86_64,
    AArch64,
}

/// Merakit kode sumber assembly langsung ke berkas biner objek mandiri tanpa dependensi luar.
pub fn rakit_ke_objek_mandiri(
    arsitektur: TargetArsitekturAsm,
    format: FormatBiner,
    kode_text_stub: &[u8],
    data_rodata: &[u8],
    simbol: &[SimbolElf],
) -> Vec<u8> {
    match format {
        FormatBiner::Elf64 => {
            let ars_elf = match arsitektur {
                TargetArsitekturAsm::X86_64 => ArsitekturElf::X86_64,
                TargetArsitekturAsm::AArch64 => ArsitekturElf::AArch64,
            };
            buat_objek_elf64(ars_elf, kode_text_stub, data_rodata, simbol)
        }
        FormatBiner::MachO64 => {
            let ars_macho = match arsitektur {
                TargetArsitekturAsm::X86_64 => ArsitekturMachO::X86_64,
                TargetArsitekturAsm::AArch64 => ArsitekturMachO::ARM64,
            };
            buat_objek_macho64(ars_macho, kode_text_stub, data_rodata)
        }
        FormatBiner::PeCoff => {
            let ars_coff = match arsitektur {
                TargetArsitekturAsm::X86_64 => ArsitekturCoff::X86_64,
                TargetArsitekturAsm::AArch64 => ArsitekturCoff::ARM64,
            };
            buat_objek_pe_coff(ars_coff, kode_text_stub, data_rodata)
        }
    }
}

/// Merakit kode assembly teks (GNU syntax) menjadi berkas executable native mandiri.
pub fn rakit_dan_tautkan_asm(
    kode_asm: &str,
    nama_modul: &str,
    jalur_keluaran: &Path,
) -> Result<PathBuf, String> {
    let dir_temp = std::env::temp_dir();
    let id_unik = PENGHITUNG_BERKAS_ASM.fetch_add(1, Ordering::SeqCst);
    let pid = std::process::id();
    let jalur_s = dir_temp.join(format!("komo_asm_{}_{}_{}.s", nama_modul, pid, id_unik));
    let jalur_rt_c = dir_temp.join(format!("komo_asm_{}_{}_{}_rt.c", nama_modul, pid, id_unik));

    // 1. Tulis berkas teks assembly .s
    fs::write(&jalur_s, kode_asm)
        .map_err(|e| format!("Gagal menulis berkas assembly '{:?}': {}", jalur_s, e))?;

    // 2. Tulis C runtime helper
    let c_runtime = r#"/* Copyright (c) kangpcode (Dhafa Nazula Permadi) */
#include <stdio.h>
#include <stdint.h>

void komo_std_cetak_baris(const char* s) {
    if (s) puts(s); else puts("");
}

void komo_std_cetak_teks(const char* s) {
    if (s) { fputs(s, stdout); fflush(stdout); }
}

void komo_std_cetak_bulat(int64_t n) {
    printf("%lld\n", (long long)n);
}

void komo_std_cetak_desimal(double d) {
    printf("%g\n", d);
}

const char* komo_rt_format_teks(const char* s) {
    return s;
}
"#;
    fs::write(&jalur_rt_c, c_runtime)
        .map_err(|e| format!("Gagal menulis kode runtime sementara: {}", e))?;

    // 3. Cari assembler/compiler C di sistem (cc, gcc, atau clang)
    let compiler = temukan_assembler_sistem()?;

    // 4. Kompilasi dan link biner
    let status = Command::new(&compiler)
        .arg(&jalur_s)
        .arg(&jalur_rt_c)
        .arg("-o")
        .arg(jalur_keluaran)
        .arg("-no-pie")
        .status()
        .map_err(|e| format!("Gagal menjalankan toolchain '{}': {}", compiler, e))?;

    let _ = fs::remove_file(&jalur_s);
    let _ = fs::remove_file(&jalur_rt_c);

    if !status.success() {
        return Err(format!(
            "Toolchain '{}' gagal dengan status keluar: {:?}",
            compiler,
            status.code()
        ));
    }

    Ok(jalur_keluaran.to_path_buf())
}

fn temukan_assembler_sistem() -> Result<String, String> {
    let kandidat = if cfg!(windows) {
        vec!["clang", "gcc"]
    } else {
        vec!["cc", "gcc", "clang"]
    };

    for k in kandidat {
        if Command::new(k).arg("--version").output().is_ok() {
            return Ok(k.to_string());
        }
    }

    Err("Tidak ditemukan compiler/assembler sistem (cc, gcc, atau clang).".to_string())
}
