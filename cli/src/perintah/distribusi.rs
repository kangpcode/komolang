// Copyright (c) kangpcode (Dhafa Nazula Permadi). Licensed under MIT or Apache-2.0.

//! Manajemen target lintas platform, cross-compilation, paket distribusi, dan pembaruan otomatis (self-updater).

use std::fs;
use std::path::{Path, PathBuf};

use komo_codegen::{
    bangun_executable_native, KonfigurasiTarget, TargetArsitektur, TargetSistemOperasi,
};
use komo_pkg::sha256::hitung_sha256;

use crate::parsing_dan_lowering;

/// Platform target kompilasi yang didukung secara resmi oleh Komolang.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TargetPlatform {
    LinuxX86_64,
    LinuxAArch64,
    WindowsX86_64,
    MacOsX86_64,
    MacOsAArch64,
    Wasm32,
    AndroidAArch64,
    IosAArch64,
    EmbeddedRiscv64,
}

impl TargetPlatform {
    /// Mengurai string nama target atau triple LLVM menjadi `TargetPlatform`.
    pub fn urai(input: &str) -> Result<Self, String> {
        let bersih = input.trim().to_lowercase();
        match bersih.as_str() {
            "x86_64-unknown-linux-gnu" | "x86_64-linux" | "linux-x64" | "linux-x86_64" => {
                Ok(Self::LinuxX86_64)
            }
            "aarch64-unknown-linux-gnu" | "aarch64-linux" | "linux-arm64" | "arm64-linux" => {
                Ok(Self::LinuxAArch64)
            }
            "x86_64-pc-windows-msvc" | "x86_64-pc-windows-gnu" | "x86_64-windows" | "win-x64"
            | "windows-x86_64" | "windows" => Ok(Self::WindowsX86_64),
            "x86_64-apple-darwin" | "x86_64-macos" | "darwin-x64" | "macos-x64" => {
                Ok(Self::MacOsX86_64)
            }
            "aarch64-apple-darwin" | "aarch64-macos" | "darwin-arm64" | "macos-arm64"
            | "apple-silicon" | "m1" | "m2" | "m3" => Ok(Self::MacOsAArch64),
            "wasm32-unknown-unknown" | "wasm32-wasi" | "wasm32" | "wasm" => Ok(Self::Wasm32),
            "aarch64-linux-android" | "android-arm64" | "android" => Ok(Self::AndroidAArch64),
            "aarch64-apple-ios" | "ios-arm64" | "ios" => Ok(Self::IosAArch64),
            "riscv64gc-unknown-none-elf" | "riscv64" | "embedded-riscv" => {
                Ok(Self::EmbeddedRiscv64)
            }
            _ => Err(format!(
                "Target platform '{}' tidak dikenal. Target resmi: linux-x64, linux-arm64, win-x64, macos-x64, macos-arm64, wasm32, android, ios, riscv64",
                input
            )),
        }
    }

    /// Daftar seluruh target platform resmi tingkat Tier-1 dan Tier-2.
    pub fn daftar_semua() -> &'static [Self] {
        &[
            Self::LinuxX86_64,
            Self::LinuxAArch64,
            Self::WindowsX86_64,
            Self::MacOsX86_64,
            Self::MacOsAArch64,
            Self::Wasm32,
            Self::AndroidAArch64,
            Self::IosAArch64,
            Self::EmbeddedRiscv64,
        ]
    }

    /// Mengembalikan target triple standar LLVM.
    pub fn sebagai_triple(&self) -> &'static str {
        match self {
            Self::LinuxX86_64 => "x86_64-unknown-linux-gnu",
            Self::LinuxAArch64 => "aarch64-unknown-linux-gnu",
            Self::WindowsX86_64 => "x86_64-pc-windows-msvc",
            Self::MacOsX86_64 => "x86_64-apple-darwin",
            Self::MacOsAArch64 => "aarch64-apple-darwin",
            Self::Wasm32 => "wasm32-unknown-unknown",
            Self::AndroidAArch64 => "aarch64-linux-android",
            Self::IosAArch64 => "aarch64-apple-ios",
            Self::EmbeddedRiscv64 => "riscv64gc-unknown-none-elf",
        }
    }

    /// Ekstensi biner keluaran untuk platform bersangkutan.
    pub fn ekstensi_biner(&self) -> &'static str {
        match self {
            Self::WindowsX86_64 => ".exe",
            Self::Wasm32 => ".wasm",
            _ => "",
        }
    }

    /// Mengonversi ke `KonfigurasiTarget` backend native bila didukung.
    pub fn ke_konfigurasi_target(&self, optimasi: bool) -> Result<KonfigurasiTarget, String> {
        match self {
            Self::LinuxX86_64 => Ok(KonfigurasiTarget {
                arsitektur: TargetArsitektur::X86_64,
                sistem_operasi: TargetSistemOperasi::Linux,
                debug_info: true,
                optimasi,
            }),
            Self::LinuxAArch64 => Ok(KonfigurasiTarget {
                arsitektur: TargetArsitektur::AArch64,
                sistem_operasi: TargetSistemOperasi::Linux,
                debug_info: true,
                optimasi,
            }),
            Self::WindowsX86_64 => Ok(KonfigurasiTarget {
                arsitektur: TargetArsitektur::X86_64,
                sistem_operasi: TargetSistemOperasi::Windows,
                debug_info: true,
                optimasi,
            }),
            Self::MacOsX86_64 => Ok(KonfigurasiTarget {
                arsitektur: TargetArsitektur::X86_64,
                sistem_operasi: TargetSistemOperasi::MacOS,
                debug_info: true,
                optimasi,
            }),
            Self::MacOsAArch64 => Ok(KonfigurasiTarget {
                arsitektur: TargetArsitektur::AArch64,
                sistem_operasi: TargetSistemOperasi::MacOS,
                debug_info: true,
                optimasi,
            }),
            _ => Err(format!(
                "Backend native langsung belum mendukung emisi mandiri untuk target '{}'. Gunakan backend assembly atau wasm.",
                self.sebagai_triple()
            )),
        }
    }
}

/// Membangun biner untuk target platform tertentu (Cross-compilation).
pub fn bangun_berkas_lintas_platform(
    jalur_sumber: &str,
    output_dasar: &Path,
    target: TargetPlatform,
    optimasi: bool,
    pakai_asm: bool,
) -> Result<PathBuf, String> {
    let modul_ir = parsing_dan_lowering(jalur_sumber, optimasi)?;

    // Sesuaikan ekstensi berkas biner
    let mut jalur_keluar = output_dasar.to_path_buf();
    let ext = target.ekstensi_biner();
    if !ext.is_empty() {
        let nama = jalur_keluar
            .file_name()
            .unwrap_or_default()
            .to_string_lossy()
            .to_string();
        if !nama.ends_with(ext) {
            jalur_keluar.set_extension(ext.trim_start_matches('.'));
        }
    }

    if pakai_asm {
        match target {
            TargetPlatform::LinuxAArch64 | TargetPlatform::MacOsAArch64 => {
                komo_asm::bangun_executable_asm(&modul_ir, &jalur_keluar)
                    .map_err(|e| format!("Gagal kompilasi ARM64 ASM: {}", e))?;
            }
            _ => {
                komo_asm::bangun_executable_asm(&modul_ir, &jalur_keluar)
                    .map_err(|e| format!("Gagal kompilasi x86_64 ASM: {}", e))?;
            }
        }
    } else {
        let cfg = target.ke_konfigurasi_target(optimasi)?;
        bangun_executable_native(&modul_ir, &cfg, &jalur_keluar)
            .map_err(|e| format!("Gagal cross-compilation native: {}", e))?;
    }

    Ok(jalur_keluar)
}

// =============================================================================
// PEMBARUAN OTOMATIS (SELF-UPDATER) & REPRODUCIBLE BUILDS
// =============================================================================

/// Informasi metadata rilis dari server pembaruan resmi.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InfoRilis {
    pub versi: String,
    pub target_triple: String,
    pub url_unduh: String,
    pub sha256: String,
    pub tanggal: String,
}

/// Menghitung baris daftar SHA256SUMS kanonik untuk biner rilis.
pub fn buat_manifest_sha256sums(daftar_berkas: &[(&str, &[u8])]) -> String {
    let mut hasil = String::new();
    for (nama, bytes) in daftar_berkas {
        let sha = hitung_sha256(bytes);
        hasil.push_str(&format!("{}  {}\n", sha, nama));
    }
    hasil
}

/// Memvalidasi berkas terhadap baris manifest SHA256SUMS.
pub fn verifikasi_sha256sums(manifest: &str, data_berkas: &[(&str, &[u8])]) -> Result<bool, String> {
    let mut peta = std::collections::HashMap::new();
    for baris in manifest.lines() {
        let bagian: Vec<&str> = baris.split_whitespace().collect();
        if bagian.len() >= 2 {
            peta.insert(bagian[1], bagian[0]);
        }
    }

    for (nama, bytes) in data_berkas {
        if let Some(&sha_diharapkan) = peta.get(nama) {
            let sha_nyata = hitung_sha256(bytes);
            if sha_nyata != sha_diharapkan {
                return Err(format!(
                    "Integritas berkas '{}' tidak cocok! Diharapkan: {}, Ditemukan: {}",
                    nama, sha_diharapkan, sha_nyata
                ));
            }
        } else {
            return Err(format!("Berkas '{}' tidak terdaftar di manifest SHA256SUMS", nama));
        }
    }

    Ok(true)
}

/// Memperbarui biner executable yang sedang berjalan secara aman dan atomik.
pub fn terapkan_pembaruan_biner(
    jalur_biner_saat_ini: &Path,
    biner_baru_bytes: &[u8],
    sha256_diharapkan: &str,
) -> Result<(), String> {
    // 1. Verifikasi integritas hash SHA-256
    let sha_nyata = hitung_sha256(biner_baru_bytes);
    if !sha256_diharapkan.is_empty() && sha_nyata != sha256_diharapkan {
        return Err(format!(
            "Checksum SHA-256 gagal! Diharapkan: {}, Diperoleh: {}",
            sha256_diharapkan, sha_nyata
        ));
    }

    // 2. Simpan cadangan (backup) biner lama
    let jalur_cadangan = jalur_biner_saat_ini.with_extension("komo_lama.bak");
    if jalur_biner_saat_ini.exists() {
        let _ = fs::copy(jalur_biner_saat_ini, &jalur_cadangan);
    }

    // 3. Tulis biner baru ke file sementara lalu ganti secara atomik
    let jalur_sementara = jalur_biner_saat_ini.with_extension("komo_baru.tmp");
    fs::write(&jalur_sementara, biner_baru_bytes)
        .map_err(|e| format!("Gagal menulis biner baru: {}", e))?;

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let perms = fs::Permissions::from_mode(0o755);
        let _ = fs::set_permissions(&jalur_sementara, perms);
    }

    if let Err(e) = fs::rename(&jalur_sementara, jalur_biner_saat_ini) {
        // Rollback jika rename gagal
        if jalur_cadangan.exists() {
            let _ = fs::copy(&jalur_cadangan, jalur_biner_saat_ini);
        }
        return Err(format!("Gagal mengganti biner utama: {}", e));
    }

    // Hapus file cadangan jika sukses
    if jalur_cadangan.exists() {
        let _ = fs::remove_file(jalur_cadangan);
    }

    Ok(())
}

// =============================================================================
// GENERATOR FORMAT DISTRIBUSI (HOMEBREW, SCOOP, WINGET, DEBIAN)
// =============================================================================

/// Menghasilkan formula Ruby untuk package manager Homebrew (macOS & Linux).
pub fn buat_formula_homebrew(versi: &str, url_tar: &str, sha256: &str) -> String {
    format!(
        r#"# Formula resmi Homebrew untuk Komolang
class Komolang < Formula
  desc "Bahasa pemrograman kompilasi native asli Indonesia berkecepatan tinggi"
  homepage "https://komolang.org"
  url "{url_tar}"
  version "{versi}"
  sha256 "{sha256}"
  license "MIT"

  def install
    bin.install "komo"
    prefix.install "lib" if Dir.exist?("lib")
  end

  test do
    system bin/"komo", "versi"
  end
end
"#
    )
}

/// Menghasilkan manifest JSON untuk package manager Scoop (Windows).
pub fn buat_manifest_scoop(versi: &str, url_zip: &str, sha256: &str) -> String {
    format!(
        r#"{{
  "version": "{}",
  "description": "Bahasa pemrograman kompilasi native asli Indonesia",
  "homepage": "https://komolang.org",
  "license": "MIT",
  "url": "{}",
  "hash": "{}",
  "bin": "komo.exe",
  "checkver": "github",
  "autoupdate": {{
    "url": "https://github.com/kangpcode/komolang/releases/download/v$version/komolang-v$version-x86_64-pc-windows-msvc.zip"
  }}
}}
"#,
        versi, url_zip, sha256
    )
}

/// Menghasilkan manifest YAML untuk Windows Package Manager (WinGet).
pub fn buat_manifest_winget(versi: &str, url_installer: &str, sha256: &str) -> String {
    format!(
        r#"# yaml-language-server: $schema=https://aka.ms/winget-manifest.singleton.1.6.0.schema.json
PackageIdentifier: kangpcode.komolang
PackageVersion: {versi}
PackageName: Komolang
Publisher: Dhafa Nazula Permadi (kangpcode)
License: MIT
ShortDescription: Kompilator bahasa pemrograman modern berbasis performa tinggi
Installers:
  - Architecture: x64
    InstallerType: zip
    InstallerUrl: {url_installer}
    InstallerSha256: {sha256}
    NestedInstallerType: portable
    NestedInstallerFiles:
      - RelativeFilePath: komo.exe
        PortableCommandAlias: komo
ManifestType: singleton
ManifestVersion: 1.6.0
"#
    )
}

/// Menghasilkan berkas kontrol packaging Debian / Ubuntu (.deb).
pub fn buat_debian_control(versi: &str, arsitektur_deb: &str) -> String {
    format!(
        r#"Package: komolang
Version: {versi}
Section: devel
Priority: optional
Architecture: {arsitektur_deb}
Maintainer: Dhafa Nazula Permadi <kangpcode@gmail.com>
Description: Bahasa pemrograman kompilasi native berperforma tinggi asli Indonesia.
 Komolang menggabungkan kecepatan eksekusi setara C/Rust dengan kemudahan
 sintaks deklaratif berbahasa Indonesia yang elegan.
"#
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tes_urai_target_platform() {
        assert_eq!(
            TargetPlatform::urai("linux-x64").unwrap(),
            TargetPlatform::LinuxX86_64
        );
        assert_eq!(
            TargetPlatform::urai("linux-arm64").unwrap(),
            TargetPlatform::LinuxAArch64
        );
        assert_eq!(
            TargetPlatform::urai("win-x64").unwrap(),
            TargetPlatform::WindowsX86_64
        );
        assert_eq!(
            TargetPlatform::urai("macos-arm64").unwrap(),
            TargetPlatform::MacOsAArch64
        );
        assert_eq!(
            TargetPlatform::urai("wasm32").unwrap(),
            TargetPlatform::Wasm32
        );
        assert!(TargetPlatform::urai("target-palsu-123").is_err());
    }

    #[test]
    fn tes_manifest_sha256sums_dan_verifikasi() {
        let file_a = ("komo-linux-x64", b"konten biner linux x64" as &[u8]);
        let file_b = ("komo-win-x64.exe", b"konten biner windows x64" as &[u8]);

        let manifest = buat_manifest_sha256sums(&[file_a, file_b]);
        assert!(manifest.contains("komo-linux-x64"));
        assert!(manifest.contains("komo-win-x64.exe"));

        let verif_ok = verifikasi_sha256sums(&manifest, &[file_a, file_b]);
        assert!(verif_ok.is_ok());

        // Uji data palsu
        let file_palsu = ("komo-linux-x64", b"konten telah dirusak" as &[u8]);
        let verif_err = verifikasi_sha256sums(&manifest, &[file_palsu]);
        assert!(verif_err.is_err());
    }

    #[test]
    fn tes_generator_manifest_distribusi() {
        let brew = buat_formula_homebrew("0.1.0", "https://example.com/komo.tar.gz", "abcdef123456");
        assert!(brew.contains("class Komolang < Formula"));
        assert!(brew.contains("url \"https://example.com/komo.tar.gz\""));

        let scoop = buat_manifest_scoop("0.1.0", "https://example.com/komo.zip", "abcdef123456");
        assert!(scoop.contains("\"bin\": \"komo.exe\""));

        let deb = buat_debian_control("0.1.0", "amd64");
        assert!(deb.contains("Package: komolang"));
        assert!(deb.contains("Architecture: amd64"));
    }

    #[test]
    fn tes_terapkan_pembaruan_biner() {
        let temp_bin = std::env::temp_dir().join("tes_komo_updater_target");
        let _ = fs::write(&temp_bin, b"biner versi 0.1.0 lama");

        let biner_baru = b"biner versi 0.1.1 baru diperbarui";
        let sha_baru = hitung_sha256(biner_baru);

        let hasil = terapkan_pembaruan_biner(&temp_bin, biner_baru, &sha_baru);
        assert!(hasil.is_ok());

        let konten_baru = fs::read(&temp_bin).unwrap();
        assert_eq!(konten_baru, biner_baru);

        let _ = fs::remove_file(temp_bin);
    }
}
