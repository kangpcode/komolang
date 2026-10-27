// Copyright (c) kangpcode (Dhafa Nazula Permadi). Licensed under MIT or Apache-2.0.

//! Pengujian integrasi untuk Backend Assembly & Toolchain Mandiri Komolang (Tahap 4B):
//! - Linear scan register allocation & spill stack
//! - Emitter Assembly x86_64 (GNU/AT&T syntax)
//! - Emitter Assembly ARM64 / AArch64 (AAPCS64 syntax)
//! - Penulis biner objek ELF64 mandiri (Linux)
//! - Penulis biner objek Mach-O 64-bit mandiri (macOS)
//! - Penulis biner objek PE/COFF mandiri (Windows)
//! - Perakitan dan linking native executable mandiri via `komo_asm::bangun_executable_asm`
//! - Verifikasi eksekusi program Halo Dunia, Fibonacci rekursif, percabangan, dan aritmatika

use std::fs;
use std::process::Command;

use komo_asm::{
    alokasikan_register_linear_scan, bangun_executable_asm, buat_objek_elf64, buat_objek_macho64,
    buat_objek_pe_coff, kompilasi_ke_assembly, ArsitekturCoff, ArsitekturElf, ArsitekturMachO,
    FormatBiner, SimbolElf, TargetArsitekturAsm,
};
use komo_ir::turunkan_ke_kir;
use komo_parser::urai;
use komo_sema::periksa;

fn kompilasi_ke_kir(kode: &str) -> komo_ir::ModulIr {
    let hasil_urai = urai(kode);
    assert!(
        hasil_urai.berhasil(),
        "Gagal parsing kode:\n{:?}",
        hasil_urai.diagnostik
    );

    let hasil_sema = periksa(&hasil_urai.berkas);
    assert!(
        hasil_sema.berhasil(),
        "Gagal semantik:\n{:?}",
        hasil_sema.diagnostik
    );

    let modul_ir = turunkan_ke_kir(&hasil_urai.berkas, &hasil_sema);
    let verif = komo_ir::verifikasi_modul(&modul_ir);
    assert!(
        verif.is_ok(),
        "IR tidak valid:\n{:?}",
        verif.err().unwrap()
    );

    modul_ir
}

#[test]
fn tes_alokasi_register_linear_scan() {
    let kode = r#"
        paket alokasi

        fungsi hitung(a: bulat, b: bulat, c: bulat) bulat {
            var x: bulat = a + b
            var y: bulat = b * c
            var z: bulat = x - y
            kembali z
        }
    "#;

    let modul = kompilasi_ke_kir(kode);
    let f = &modul.fungsi[0];

    let intervals = komo_asm::hitung_rentang_hidup(f);
    assert!(!intervals.is_empty(), "Rentang hidup nilai harus terdeteksi");

    // Alokasikan dengan pool 4 register
    let hasil_alok = alokasikan_register_linear_scan(f, 4);

    assert!(!hasil_alok.peta_lokasi.is_empty(), "Semua nilai harus mendapatkan alokasi lokasi");
    assert_eq!(hasil_alok.ukuran_stack % 16, 0, "Ukuran stack harus selalu selaras 16-byte");
}

#[test]
fn tes_emisi_assembly_x86_64_sintaks_dan_instruksi() {
    let kode = r#"
        paket matematika

        publik fungsi kalkulasi(a: bulat, b: bulat) bulat {
            jika a > b {
                kembali a - b
            } lainnya {
                kembali a + b
            }
        }
    "#;

    let modul = kompilasi_ke_kir(kode);
    let asm = kompilasi_ke_assembly(&modul, TargetArsitekturAsm::X86_64);

    // Verifikasi struktur GNU / AT&T x86_64
    assert!(asm.contains(".file \"matematika.komo\""));
    assert!(asm.contains(".text"));
    assert!(asm.contains(".globl kalkulasi"));
    assert!(asm.contains(".type kalkulasi, @function"));
    assert!(asm.contains("pushq %rbp"));
    assert!(asm.contains("movq %rsp, %rbp"));
    assert!(asm.contains("cmpq"));
    assert!(asm.contains("subq"));
    assert!(asm.contains("addq"));
    assert!(asm.contains("retq"));
    assert!(asm.contains(".note.GNU-stack"));
}

#[test]
fn tes_emisi_assembly_arm64_sintaks_dan_instruksi() {
    let kode = r#"
        paket matematika

        publik fungsi hitung_faktorial(n: bulat) bulat {
            jika n <= 1 {
                kembali 1
            }
            kembali n * hitung_faktorial(n - 1)
        }
    "#;

    let modul = kompilasi_ke_kir(kode);
    let asm = kompilasi_ke_assembly(&modul, TargetArsitekturAsm::AArch64);

    // Verifikasi struktur AAPCS64 ARM64
    assert!(asm.contains(".arch armv8-a"));
    assert!(asm.contains(".text"));
    assert!(asm.contains(".globl hitung_faktorial"));
    assert!(asm.contains(".type hitung_faktorial, %function"));
    assert!(asm.contains("stp x29, x30, [sp, -"));
    assert!(asm.contains("mov x29, sp"));
    assert!(asm.contains("ldp x29, x30, [sp],"));
    assert!(asm.contains("ret"));
    assert!(asm.contains("bl hitung_faktorial"));
    assert!(asm.contains("mul x0, x0,"));
}

#[test]
fn tes_biner_mandiri_elf64() {
    let dummy_code = vec![0x90, 0x90, 0xC3]; // nop, nop, ret
    let dummy_rodata = b"Halo Komo ELF\0";
    let simbol = vec![
        SimbolElf {
            nama: "utama".to_string(),
            offset_nilai: 0,
            ukuran: 3,
            section_idx: 1,
            publik: true,
        },
    ];

    // Tes ELF64 x86_64
    let elf_x86 = buat_objek_elf64(ArsitekturElf::X86_64, &dummy_code, dummy_rodata, &simbol);
    assert!(elf_x86.len() >= 64, "Header ELF harus minimal 64 bita");
    assert_eq!(&elf_x86[0..4], b"\x7fELF", "Magic bytes ELF tidak cocok");
    assert_eq!(elf_x86[4], 2, "Harus ELF 64-bit");
    assert_eq!(elf_x86[5], 1, "Harus Little-Endian");
    let e_machine_x86 = u16::from_le_bytes([elf_x86[18], elf_x86[19]]);
    assert_eq!(e_machine_x86, 0x3E, "e_machine harus EM_X86_64");

    // Tes ELF64 ARM64
    let elf_arm = buat_objek_elf64(ArsitekturElf::AArch64, &dummy_code, dummy_rodata, &simbol);
    let e_machine_arm = u16::from_le_bytes([elf_arm[18], elf_arm[19]]);
    assert_eq!(e_machine_arm, 183, "e_machine harus EM_AARCH64 (183 / 0xB7)");
}

#[test]
fn tes_biner_mandiri_macho64() {
    let dummy_code = vec![0x90, 0xC3];
    let dummy_cstring = b"Halo Mach-O\0";

    // Mach-O x86_64
    let macho_x86 = buat_objek_macho64(ArsitekturMachO::X86_64, &dummy_code, dummy_cstring);
    assert!(macho_x86.len() >= 32);
    let magic = u32::from_le_bytes([macho_x86[0], macho_x86[1], macho_x86[2], macho_x86[3]]);
    assert_eq!(magic, 0xFEEDFACF, "Magic Mach-O 64-bit tidak sesuai");

    let cputype_x86 = u32::from_le_bytes([macho_x86[4], macho_x86[5], macho_x86[6], macho_x86[7]]);
    assert_eq!(cputype_x86, 0x01000007, "CPU Type x86_64 Mach-O tidak cocok");

    // Mach-O ARM64
    let macho_arm = buat_objek_macho64(ArsitekturMachO::ARM64, &dummy_code, dummy_cstring);
    let cputype_arm = u32::from_le_bytes([macho_arm[4], macho_arm[5], macho_arm[6], macho_arm[7]]);
    assert_eq!(cputype_arm, 0x0100000c, "CPU Type ARM64 Mach-O tidak cocok");
}

#[test]
fn tes_biner_mandiri_pe_coff() {
    let dummy_code = vec![0x90, 0xC3];
    let dummy_rdata = b"Halo Windows PE\0";

    // PE COFF x86_64
    let pe_x86 = buat_objek_pe_coff(ArsitekturCoff::X86_64, &dummy_code, dummy_rdata);
    assert!(pe_x86.len() >= 20);
    let machine_x86 = u16::from_le_bytes([pe_x86[0], pe_x86[1]]);
    assert_eq!(machine_x86, 0x8664, "COFF Machine AMD64 harus 0x8664");

    // PE COFF ARM64
    let pe_arm = buat_objek_pe_coff(ArsitekturCoff::ARM64, &dummy_code, dummy_rdata);
    let machine_arm = u16::from_le_bytes([pe_arm[0], pe_arm[1]]);
    assert_eq!(machine_arm, 0xaa64, "COFF Machine ARM64 harus 0xaa64");
}

#[test]
fn tes_rakit_ke_objek_mandiri_pipeline() {
    let dummy_code = vec![0xC3];
    let dummy_data = b"Komo pipeline\0";
    let simbol = vec![];

    let biner_elf = komo_asm::rakit_ke_objek_mandiri(
        TargetArsitekturAsm::X86_64,
        FormatBiner::Elf64,
        &dummy_code,
        dummy_data,
        &simbol,
    );
    assert_eq!(&biner_elf[0..4], b"\x7fELF");

    let biner_macho = komo_asm::rakit_ke_objek_mandiri(
        TargetArsitekturAsm::X86_64,
        FormatBiner::MachO64,
        &dummy_code,
        dummy_data,
        &simbol,
    );
    assert_eq!(
        u32::from_le_bytes([biner_macho[0], biner_macho[1], biner_macho[2], biner_macho[3]]),
        0xFEEDFACF
    );

    let biner_pe = komo_asm::rakit_ke_objek_mandiri(
        TargetArsitekturAsm::X86_64,
        FormatBiner::PeCoff,
        &dummy_code,
        dummy_data,
        &simbol,
    );
    assert_eq!(u16::from_le_bytes([biner_pe[0], biner_pe[1]]), 0x8664);
}

#[test]
fn tes_eksekusi_biner_native_asm_halo_dunia() {
    let kode = r#"
        paket utama
        impor "std/cetak"

        fungsi utama() {
            cetak.baris("Halo Dunia dari Backend ASM Komolang!")
        }
    "#;

    let modul = kompilasi_ke_kir(kode);
    let out_dir = std::env::temp_dir();
    let exe_path = out_dir.join("tes_komo_asm_halo_dunia");

    let hasil_bangun = bangun_executable_asm(&modul, &exe_path);
    assert!(
        hasil_bangun.is_ok(),
        "Gagal membangun executable via ASM backend:\n{:?}",
        hasil_bangun.err()
    );

    let output = Command::new(&exe_path)
        .output()
        .expect("Harus dapat mengeksekusi biner ASM");

    let stdout_str = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout_str.contains("Halo Dunia dari Backend ASM Komolang!"),
        "Output biner ASM tidak sesuai:\n{}",
        stdout_str
    );
    assert_eq!(output.status.code(), Some(0));

    let _ = fs::remove_file(exe_path);
}

#[test]
fn tes_eksekusi_biner_native_asm_fibonacci_dan_aritmatika() {
    let kode = r#"
        paket utama

        fungsi fib(n: bulat) bulat {
            jika n <= 1 {
                kembali n
            }
            kembali fib(n - 1) + fib(n - 2)
        }

        fungsi utama() bulat {
            kembali fib(10)
        }
    "#;

    let modul = kompilasi_ke_kir(kode);
    let out_dir = std::env::temp_dir();
    let exe_path = out_dir.join("tes_komo_asm_fibonacci");

    let hasil_bangun = bangun_executable_asm(&modul, &exe_path);
    assert!(
        hasil_bangun.is_ok(),
        "Gagal membangun biner Fibonacci via ASM backend:\n{:?}",
        hasil_bangun.err()
    );

    let output = Command::new(&exe_path)
        .output()
        .expect("Harus dapat mengeksekusi biner ASM");

    // fib(10) = 55
    assert_eq!(
        output.status.code(),
        Some(55),
        "Hasil fib(10) harus menghasilkan exit code 55"
    );

    let _ = fs::remove_file(exe_path);
}

#[test]
fn tes_eksekusi_biner_native_asm_kalkulasi_aritmatika_dan_logika() {
    let kode = r#"
        paket utama

        fungsi hitung_rumus(a: bulat, b: bulat) bulat {
            var x: bulat = (a + b) * 2
            var y: bulat = x / 3
            var sisa: bulat = x % 3
            jika sisa > 0 {
                kembali y + 10
            } lainnya {
                kembali y
            }
        }

        fungsi utama() bulat {
            // a = 7, b = 5 -> x = 24 -> y = 8, sisa = 0 -> kembali 8
            kembali hitung_rumus(7, 5)
        }
    "#;

    let modul = kompilasi_ke_kir(kode);
    let out_dir = std::env::temp_dir();
    let exe_path = out_dir.join("tes_komo_asm_kalkulasi");

    let hasil_bangun = bangun_executable_asm(&modul, &exe_path);
    assert!(hasil_bangun.is_ok(), "{:?}", hasil_bangun.err());

    let output = Command::new(&exe_path)
        .output()
        .expect("Harus dapat mengeksekusi biner ASM");

    assert_eq!(
        output.status.code(),
        Some(8),
        "Hasil hitung_rumus(7, 5) harus menghasilkan exit code 8"
    );

    let _ = fs::remove_file(exe_path);
}
