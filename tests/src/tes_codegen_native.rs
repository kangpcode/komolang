// Copyright (c) kangpcode (Dhafa Nazula Permadi). Licensed under MIT or Apache-2.0.

//! Pengujian integrasi untuk Backend Native Komolang (Tahap 4A):
//! - Emisi kode mesin untuk arsitektur x86_64 dan ARM64 (AArch64)
//! - Linking menjadi binary executable mandiri di Linux, Windows, macOS
//! - Program Halo Dunia, Fibonacci rekursif, kalkulasi aritmatika & logika
//! - Eksekusi JIT in-memory dan eksekusi biner native via subprocess

use std::fs;
use std::process::Command;

use komo_codegen::{
    bangun_executable_native, jalankan_jit, kompilasi_ke_objek, EksekutorJit, KonfigurasiTarget,
    TargetArsitektur, TargetSistemOperasi,
};
use komo_ir::turunkan_ke_kir;
use komo_parser::urai;
use komo_sema::periksa;

fn kompilasi_kode_ke_kir(kode: &str) -> komo_ir::ModulIr {
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
fn tes_codegen_emisi_objek_x86_64_dan_arm64_lintas_platform() {
    let kode = r#"
        paket matematika

        publik fungsi hitung_kuadrat(x: bulat) bulat {
            kembali x * x
        }
    "#;

    let modul_ir = kompilasi_kode_ke_kir(kode);

    // Target arsitektur dan sistem operasi yang diuji
    let konfigurasi_list = vec![
        // x86_64
        KonfigurasiTarget {
            arsitektur: TargetArsitektur::X86_64,
            sistem_operasi: TargetSistemOperasi::Linux,
            debug_info: true,
            optimasi: false,
        },
        KonfigurasiTarget {
            arsitektur: TargetArsitektur::X86_64,
            sistem_operasi: TargetSistemOperasi::Windows,
            debug_info: true,
            optimasi: true,
        },
        KonfigurasiTarget {
            arsitektur: TargetArsitektur::X86_64,
            sistem_operasi: TargetSistemOperasi::MacOS,
            debug_info: false,
            optimasi: false,
        },
        // ARM64 (AArch64)
        KonfigurasiTarget {
            arsitektur: TargetArsitektur::AArch64,
            sistem_operasi: TargetSistemOperasi::Linux,
            debug_info: true,
            optimasi: false,
        },
        KonfigurasiTarget {
            arsitektur: TargetArsitektur::AArch64,
            sistem_operasi: TargetSistemOperasi::MacOS,
            debug_info: true,
            optimasi: true,
        },
        KonfigurasiTarget {
            arsitektur: TargetArsitektur::AArch64,
            sistem_operasi: TargetSistemOperasi::Windows,
            debug_info: false,
            optimasi: false,
        },
    ];

    for cfg in konfigurasi_list {
        let hasil = kompilasi_ke_objek(&modul_ir, &cfg);
        assert!(
            hasil.is_ok(),
            "Gagal kompilasi objek untuk target {}: {:?}",
            cfg.sebagai_triple_str(),
            hasil.err().unwrap()
        );

        let objek = hasil.unwrap();
        assert!(
            !objek.bytes_objek.is_empty(),
            "Berkas objek biner untuk {} tidak boleh kosong",
            cfg.sebagai_triple_str()
        );
        assert_eq!(objek.nama_modul, "matematika");
    }
}

#[test]
fn tes_codegen_fibonacci_rekursif_jit() {
    let kode = r#"
        paket fib_test

        publik fungsi fibonacci(n: bulat) bulat {
            jika n <= 1 {
                kembali n
            } lainnya {
                kembali fibonacci(n - 1) + fibonacci(n - 2)
            }
        }

        publik fungsi utama() bulat {
            kembali fibonacci(10)
        }
    "#;

    let modul_ir = kompilasi_kode_ke_kir(kode);
    let eksekutor = EksekutorJit::baru(&modul_ir).expect("Inisialisasi JIT berhasil");

    // fib(10) = 55
    let hasil_utama = eksekutor.jalankan_utama().expect("Eksekusi JIT berhasil");
    assert_eq!(hasil_utama, 55);

    // Pengujian argumen langsung
    let fib_5 = eksekutor
        .jalankan_fungsi_i64_ke_i64("fibonacci", 5)
        .expect("Panggilan fungsi berhasil");
    assert_eq!(fib_5, 5);

    let fib_7 = eksekutor
        .jalankan_fungsi_i64_ke_i64("fibonacci", 7)
        .expect("Panggilan fungsi berhasil");
    assert_eq!(fib_7, 13);
}

#[test]
fn tes_codegen_aritmatika_dan_percabangan_native() {
    let kode = r#"
        paket hitung

        publik fungsi uji_kondisi(a: bulat, b: bulat) bulat {
            var x = a * 2 + b
            jika x > 20 {
                x = x - 5
            } lainnya {
                x = x + 10
            }
            kembali x
        }

        publik fungsi utama() bulat {
            kembali uji_kondisi(10, 5) // x = 25 -> x - 5 = 20
        }
    "#;

    let modul_ir = kompilasi_kode_ke_kir(kode);
    let eksekutor = EksekutorJit::baru(&modul_ir).expect("Inisialisasi JIT berhasil");
    let hasil = eksekutor.jalankan_utama().expect("Eksekusi berhasil");
    assert_eq!(hasil, 20);
}

#[test]
fn tes_codegen_pengulangan_selagi_native() {
    let kode = r#"
        paket loop_test

        publik fungsi hitung_total(batas: bulat) bulat {
            var i = 0
            var sum = 0
            selagi i <= batas {
                sum = sum + i
                i = i + 1
            }
            kembali sum
        }

        publik fungsi utama() bulat {
            kembali hitung_total(10) // sum(0..=10) = 55
        }
    "#;

    let modul_ir = kompilasi_kode_ke_kir(kode);
    let eksekutor = EksekutorJit::baru(&modul_ir).expect("JIT berhasil");
    let hasil = eksekutor.jalankan_utama().expect("Eksekusi berhasil");
    assert_eq!(hasil, 55);
}

#[test]
fn tes_codegen_halo_dunia_jit_dan_panggilan_cetak() {
    let kode = r#"
        paket utama

        impor "std/cetak"

        publik fungsi utama() {
            cetak.baris("Halo, Dunia dari Komolang Native!")
        }
    "#;

    let modul_ir = kompilasi_kode_ke_kir(kode);
    let hasil = jalankan_jit(&modul_ir);
    assert!(hasil.is_ok(), "JIT jalankan Halo Dunia harus sukses: {:?}", hasil.err());
    assert_eq!(hasil.unwrap(), 0);
}

#[test]
fn tes_codegen_linking_binary_executable_dan_eksekusi_native() {
    let kode = r#"
        paket utama

        impor "std/cetak"

        publik fungsi fibonacci(n: bulat) bulat {
            jika n <= 1 {
                kembali n
            } lainnya {
                kembali fibonacci(n - 1) + fibonacci(n - 2)
            }
        }

        publik fungsi utama() bulat {
            cetak.baris("Memulai kalkulasi native...")
            biar f = fibonacci(10)
            cetak.baris("Kalkulasi selesai!")
            kembali f // exit code 55
        }
    "#;

    let modul_ir = kompilasi_kode_ke_kir(kode);
    let target = KonfigurasiTarget::untuk_host();

    let temp_dir = std::env::temp_dir();
    let bin_path = temp_dir.join("tes_komolang_biner_native");

    // Bangun biner executable native
    let hasil_bangun = bangun_executable_native(&modul_ir, &target, &bin_path);
    assert!(
        hasil_bangun.is_ok(),
        "Gagal membangun executable native: {:?}",
        hasil_bangun.err()
    );

    let executable = hasil_bangun.unwrap();
    assert!(executable.exists(), "Berkas biner executable harus ada di disk");

    // Eksekusi biner native yang dihasilkan
    let output = Command::new(&executable)
        .output()
        .expect("Biner executable native harus bisa dijalankan");

    let stdout_str = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout_str.contains("Memulai kalkulasi native..."),
        "Keluaran stdout: {}",
        stdout_str
    );
    assert!(
        stdout_str.contains("Kalkulasi selesai!"),
        "Keluaran stdout: {}",
        stdout_str
    );

    // Exit code harus sama dengan kembalian utama() yaitu 55
    let status_code = output.status.code().unwrap_or(-1);
    assert_eq!(status_code, 55, "Exit code harus 55");

    // Bersihkan biner sementara
    let _ = fs::remove_file(executable);
}

#[test]
fn tes_codegen_seluruh_contoh_resmi() {
    let berkas_contoh = vec![
        include_str!("../../examples/halo_dunia.komo"),
        include_str!("../../examples/perhitungan.komo"),
        include_str!("../../examples/sintaks_lengkap.komo"),
    ];

    let target = KonfigurasiTarget::untuk_host();

    for kode in berkas_contoh {
        let modul_ir = kompilasi_kode_ke_kir(kode);
        let hasil_objek = kompilasi_ke_objek(&modul_ir, &target);
        assert!(
            hasil_objek.is_ok(),
            "Gagal kompilasi objek native contoh resmi: {:?}",
            hasil_objek.err()
        );
        let obj = hasil_objek.unwrap();
        assert!(!obj.bytes_objek.is_empty());
    }
}
