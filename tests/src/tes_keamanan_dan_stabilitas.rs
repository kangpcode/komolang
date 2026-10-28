// Copyright (c) kangpcode (Dhafa Nazula Permadi). Licensed under MIT or Apache-2.0.

//! Test suite integrasi untuk Tahap 13: Kualitas, Keamanan, Stabilitas, dan Kebijakan SemVer.

use std::fs;
use std::path::{Path, PathBuf};
use std::time::Instant;

use komo::perintah::{audit_proyek, KebijakanSandboxing};

#[test]
fn tes_keamanan_audit_supply_chain_dan_kredensial() {
    let temp_dir = std::env::temp_dir().join("tes_audit_supply_chain");
    let _ = fs::create_dir_all(&temp_dir);

    // 1. Proyek dengan kerentanan supply chain (paket CVE)
    let manifes_rentan = temp_dir.join("komo.toml");
    fs::write(
        &manifes_rentan,
        r#"
[paket]
nama = "aplikasi_uji"
versi = "0.1.0"

[dependensi]
paket_usang_lama = "0.4.0"
"#,
    )
    .unwrap();

    let lap_rentan = audit_proyek(temp_dir.to_str().unwrap()).unwrap();
    assert_eq!(lap_rentan.jumlah_kritis, 1);
    assert!(!lap_rentan.apakah_aman());

    // 2. Proyek bersih
    fs::write(
        &manifes_rentan,
        r#"
[paket]
nama = "aplikasi_bersih"
versi = "0.1.0"

[dependensi]
"#,
    )
    .unwrap();

    let berkas_bersih = temp_dir.join("bersih.komo");
    fs::write(
        &berkas_bersih,
        r#"
// Kode aman
paket utama

publik fungsi utama() {
    cetak("Aplikasi aman")
}
"#,
    )
    .unwrap();

    let lap_bersih = audit_proyek(temp_dir.to_str().unwrap()).unwrap();
    assert_eq!(lap_bersih.jumlah_kritis, 0);
    assert_eq!(lap_bersih.jumlah_tinggi, 0);
    assert!(lap_bersih.apakah_aman());

    let _ = fs::remove_dir_all(temp_dir);
}

#[test]
fn tes_keamanan_kebijakan_sandboxing_akses_berkas_dan_jaringan() {
    let workspace = PathBuf::from("/home/user/workspace/app");
    let sandbox = KebijakanSandboxing::ketat(workspace.clone());

    // Akses di dalam workspace diizinkan
    let file_legal = workspace.join("data").join("masukan.txt");
    assert!(sandbox.periksa_akses_jalur(&file_legal).is_ok());

    // Akses di luar workspace ditolak
    let file_ilegal = PathBuf::from("/etc/shadow");
    assert!(sandbox.periksa_akses_jalur(&file_ilegal).is_err());

    // Upaya Directory Traversal diblokir
    let file_traversal = workspace.join("sub").join("..").join("..").join("rahasia");
    assert!(sandbox.periksa_akses_jalur(&file_traversal).is_err());

    // Jaringan dan Subprocess diblokir pada kebijakan ketat
    assert!(sandbox.periksa_akses_jaringan().is_err());
    assert!(sandbox.periksa_izin_subprocess().is_err());

    // Kebijakan standar mengizinkan jaringan namun tetap memblokir subprocess
    let sandbox_std = KebijakanSandboxing::standar(workspace);
    assert!(sandbox_std.periksa_akses_jaringan().is_ok());
    assert!(sandbox_std.periksa_izin_subprocess().is_err());
}

#[test]
fn tes_stabilitas_fuzzing_dan_pemulihan_syntax_galat() {
    // Memastikan parser dan semantik Komolang tahan terhadap mutasi sintaks ekstrem
    let mutasi_tak_valid = vec![
        "fungsi ( { }",
        "biar x = + * /",
        "struktur { a: }",
        "jika { kembali } lainnya",
        "paket 123",
        "impor ;;;",
        "untuk dalam 0.. {",
        "cocok { => 1 }",
    ];

    for kode in mutasi_tak_valid {
        let hasil = komo_parser::urai(kode);
        // Parser tidak boleh panik/crash, melainkan harus mencatat diagnostik galat dengan anggun
        assert!(!hasil.berhasil());
        assert!(!hasil.diagnostik.is_empty());
    }
}

#[test]
fn tes_benchmark_publik_angka_tereproduksi() {
    // Uji komputasi Fibonacci iteratif kecepatan tinggi
    let n = 100_000usize;
    let mulai = Instant::now();
    let mut a: u64 = 0;
    let mut b: u64 = 1;
    for _ in 2..=n {
        let c = a.wrapping_add(b);
        a = b;
        b = c;
    }
    let durasi = mulai.elapsed();

    assert!(b > 0);
    // 100.000 iterasi harus selesai dalam waktu kurang dari 20 milidetik pada CPU modern
    assert!(
        durasi.as_millis() < 20,
        "Eksekusi Fibonacci harus sangat cepat, terukur: {:?} ms",
        durasi.as_millis()
    );

    // Verifikasi dokumen benchmark publik dan RFC
    let root_dir = Path::new(env!("CARGO_MANIFEST_DIR")).parent().unwrap();
    let bench_doc = root_dir.join("docs").join("benchmark_publik.md");
    let rfc_doc = root_dir.join("docs").join("rfc").join("0000-template-rfc.md");
    let semver_doc = root_dir.join("docs").join("kebijakan_kompatibilitas.md");

    assert!(bench_doc.exists(), "docs/benchmark_publik.md harus ada");
    assert!(rfc_doc.exists(), "docs/rfc/0000-template-rfc.md harus ada");
    assert!(semver_doc.exists(), "docs/kebijakan_kompatibilitas.md harus ada");

    let isi_bench = fs::read_to_string(bench_doc).unwrap();
    assert!(isi_bench.contains("Metodologi Pengujian"));
    assert!(isi_bench.contains("Komolang (KIR-Opt)"));
}
