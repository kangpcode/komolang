// Copyright (c) kangpcode (Dhafa Nazula Permadi). Licensed under MIT or Apache-2.0.

//! Test suite integrasi dan konformansi untuk Tahap 12: Lintas Platform & Distribusi.

use std::fs;
use std::path::Path;

use komo::perintah::{
    bangun_berkas_lintas_platform, buat_debian_control, buat_formula_homebrew, buat_manifest_scoop,
    buat_manifest_sha256sums, buat_manifest_winget, terapkan_pembaruan_biner, verifikasi_sha256sums,
    TargetPlatform,
};
use komo_pkg::sha256::hitung_sha256;

#[test]
fn tes_cakupan_seluruh_target_resmi() {
    let daftar_target = TargetPlatform::daftar_semua();
    assert_eq!(daftar_target.len(), 9);

    let uji_string = vec![
        ("linux-x64", TargetPlatform::LinuxX86_64, "x86_64-unknown-linux-gnu", ""),
        ("linux-arm64", TargetPlatform::LinuxAArch64, "aarch64-unknown-linux-gnu", ""),
        ("win-x64", TargetPlatform::WindowsX86_64, "x86_64-pc-windows-msvc", ".exe"),
        ("macos-x64", TargetPlatform::MacOsX86_64, "x86_64-apple-darwin", ""),
        ("macos-arm64", TargetPlatform::MacOsAArch64, "aarch64-apple-darwin", ""),
        ("wasm32", TargetPlatform::Wasm32, "wasm32-unknown-unknown", ".wasm"),
        ("android-arm64", TargetPlatform::AndroidAArch64, "aarch64-linux-android", ""),
        ("ios-arm64", TargetPlatform::IosAArch64, "aarch64-apple-ios", ""),
        ("riscv64", TargetPlatform::EmbeddedRiscv64, "riscv64gc-unknown-none-elf", ""),
    ];

    for (input, target_diharapkan, triple_diharapkan, ext_diharapkan) in uji_string {
        let t = TargetPlatform::urai(input).expect("Target harus dikenali");
        assert_eq!(t, target_diharapkan);
        assert_eq!(t.sebagai_triple(), triple_diharapkan);
        assert_eq!(t.ekstensi_biner(), ext_diharapkan);
    }
}

#[test]
fn tes_manifest_sha256sums_reproducible_build() {
    let biner_linux = b"ELF\x02\x01\x01\x00_komolang_biner_linux_reproducible";
    let biner_win = b"MZ\x90\x00_komolang_biner_win_reproducible";

    let manifest = buat_manifest_sha256sums(&[
        ("komolang-linux-x64", biner_linux),
        ("komolang-win-x64.exe", biner_win),
    ]);

    let sha_linux = hitung_sha256(biner_linux);
    let sha_win = hitung_sha256(biner_win);

    assert!(manifest.contains(&format!("{}  komolang-linux-x64", sha_linux)));
    assert!(manifest.contains(&format!("{}  komolang-win-x64.exe", sha_win)));

    let cek_sukses = verifikasi_sha256sums(
        &manifest,
        &[
            ("komolang-linux-x64", biner_linux),
            ("komolang-win-x64.exe", biner_win),
        ],
    );
    assert!(cek_sukses.is_ok());

    // Cek kegagalan deteksi jika biner dimanipulasi
    let biner_rusak = b"ELF\x02_data_rusak_tampered";
    let cek_gagal = verifikasi_sha256sums(&manifest, &[("komolang-linux-x64", biner_rusak)]);
    assert!(cek_gagal.is_err());
}

#[test]
fn tes_self_updater_atomik_dan_rollback() {
    let temp_dir = std::env::temp_dir();
    let jalur_biner = temp_dir.join("tes_komo_executable_asli");

    let konten_asli = b"KOMO_V0_1_0_NATIVE_EXEC";
    fs::write(&jalur_biner, konten_asli).expect("Harus dapat membuat file biner asli");

    let biner_baru = b"KOMO_V0_2_0_NATIVE_EXEC_UPDATED";
    let sha_baru = hitung_sha256(biner_baru);

    // 1. Uji pembaruan berhasil
    let res = terapkan_pembaruan_biner(&jalur_biner, biner_baru, &sha_baru);
    assert!(res.is_ok());
    let konten_setelah_update = fs::read(&jalur_biner).unwrap();
    assert_eq!(konten_setelah_update, biner_baru);

    // 2. Uji proteksi jika checksum sha256 salah
    let res_salah = terapkan_pembaruan_biner(&jalur_biner, biner_baru, "sha256_salah_tidak_cocok");
    assert!(res_salah.is_err());
    // Biner tidak boleh rusak
    let konten_setelah_gagal = fs::read(&jalur_biner).unwrap();
    assert_eq!(konten_setelah_gagal, biner_baru);

    let _ = fs::remove_file(jalur_biner);
}

#[test]
fn tes_cross_compilation_bangun_target() {
    let contoh_dir = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .join("examples");
    let sumber = contoh_dir.join("perhitungan.komo");
    assert!(sumber.exists());

    let temp_out = std::env::temp_dir().join("tes_cross_build_bin");

    // Kompilasi ke target Windows x64 dengan ekstensi .exe
    let hasil_win = bangun_berkas_lintas_platform(
        sumber.to_str().unwrap(),
        &temp_out,
        TargetPlatform::WindowsX86_64,
        false,
        true, // Gunakan assembler mandiri
    );
    assert!(hasil_win.is_ok());
    let out_win = hasil_win.unwrap();
    assert!(out_win.to_string_lossy().ends_with(".exe"));
    let _ = fs::remove_file(out_win);

    // Kompilasi ke target Linux x64
    let hasil_linux = bangun_berkas_lintas_platform(
        sumber.to_str().unwrap(),
        &temp_out,
        TargetPlatform::LinuxX86_64,
        false,
        true,
    );
    assert!(hasil_linux.is_ok());
    let out_linux = hasil_linux.unwrap();
    assert!(out_linux.exists());
    let _ = fs::remove_file(out_linux);
}

#[test]
fn tes_berkas_paket_distribusi_installer() {
    // 1. Validasi Homebrew
    let brew = buat_formula_homebrew("0.1.0", "https://komolang.org/download/komo.tar.gz", "12345");
    assert!(brew.contains("class Komolang < Formula"));
    assert!(brew.contains("bin.install \"komo\""));

    // 2. Validasi Scoop
    let scoop = buat_manifest_scoop("0.1.0", "https://komolang.org/download/komo.zip", "12345");
    assert!(scoop.contains("\"bin\": \"komo.exe\""));

    // 3. Validasi WinGet
    let winget = buat_manifest_winget("0.1.0", "https://komolang.org/download/komo.zip", "12345");
    assert!(winget.contains("PackageIdentifier: kangpcode.komolang"));

    // 4. Validasi Debian Control
    let deb = buat_debian_control("0.1.0", "arm64");
    assert!(deb.contains("Architecture: arm64"));

    // 5. Pastikan berkas install.sh dan install.ps1 di root proyek valid
    let root_dir = Path::new(env!("CARGO_MANIFEST_DIR")).parent().unwrap();
    let install_sh = root_dir.join("install.sh");
    let install_ps1 = root_dir.join("install.ps1");
    let dockerfile = root_dir.join("Dockerfile");
    let ci_yml = root_dir.join(".github").join("workflows").join("ci_lintas_platform.yml");

    assert!(install_sh.exists(), "install.sh harus tersedia di root");
    assert!(install_ps1.exists(), "install.ps1 harus tersedia di root");
    assert!(dockerfile.exists(), "Dockerfile harus tersedia di root");
    assert!(ci_yml.exists(), "ci_lintas_platform.yml harus tersedia");
}
