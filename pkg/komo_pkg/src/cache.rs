// Copyright (c) kangpcode (Dhafa Nazula Permadi). Licensed under MIT or Apache-2.0.

//! Manajemen cache lokal paket Komolang dan verifikasi integritas checksum SHA-256.

use std::env;
use std::fs;
use std::path::{Path, PathBuf};

use crate::semver::Versi;
use crate::sha256::sha256_hex;

/// Pengelola penyimpanan lokal cache paket Komolang.
#[derive(Debug, Clone)]
pub struct ManajerCache {
    pub direktori_akar: PathBuf,
}

impl ManajerCache {
    /// Membuat manajer cache dengan direktori default (`$KOMO_HOME/cache` atau `~/.komo/cache`).
    pub fn bawaan() -> Self {
        let jalur = if let Ok(komo_home) = env::var("KOMO_HOME") {
            PathBuf::from(komo_home).join("cache")
        } else if let Ok(home) = env::var("HOME") {
            PathBuf::from(home).join(".komo").join("cache")
        } else {
            env::temp_dir().join("komo").join("cache")
        };

        Self {
            direktori_akar: jalur,
        }
    }

    /// Membuat manajer cache dengan jalur direktori kustom.
    pub fn di_jalur(jalur: PathBuf) -> Self {
        Self {
            direktori_akar: jalur,
        }
    }

    /// Direktori penyimpanan arsip `.komopaket`.
    pub fn direktori_arsip(&self) -> PathBuf {
        self.direktori_akar.join("paket")
    }

    /// Direktori hasil ekstraksi kode sumber paket.
    pub fn direktori_sumber(&self) -> PathBuf {
        self.direktori_akar.join("sumber")
    }

    /// Memastikan direktori cache siap digunakan di filesystem.
    pub fn inisialisasi(&self) -> Result<(), String> {
        fs::create_dir_all(self.direktori_arsip())
            .map_err(|e| format!("Gagal membuat direktori cache arsip: {}", e))?;
        fs::create_dir_all(self.direktori_sumber())
            .map_err(|e| format!("Gagal membuat direktori cache sumber: {}", e))?;
        Ok(())
    }

    /// Jalur berkas arsip terunduh untuk paket dan versi tertentu.
    pub fn jalur_arsip_paket(&self, nama: &str, versi: &Versi) -> PathBuf {
        let nama_aman = nama.replace('/', "_");
        self.direktori_arsip()
            .join(format!("{}-{}.komopaket", nama_aman, versi))
    }

    /// Jalur folder kode sumber hasil ekstraksi.
    pub fn jalur_sumber_paket(&self, nama: &str, versi: &Versi) -> PathBuf {
        let nama_aman = nama.replace('/', "_");
        self.direktori_sumber()
            .join(format!("{}-{}", nama_aman, versi))
    }

    /// Memeriksa apakah suatu versi paket sudah ada di cache lokal.
    pub fn apakah_tersedia(&self, nama: &str, versi: &Versi) -> bool {
        self.jalur_sumber_paket(nama, versi).exists()
    }

    /// Menyimpan dan memverifikasi arsip paket biner, kemudian mengekstraknya ke direktori sumber.
    pub fn pasang_dan_verifikasi(
        &self,
        nama: &str,
        versi: &Versi,
        muatan_arsip: &[u8],
        ekspektasi_checksum: &str,
    ) -> Result<PathBuf, String> {
        self.inisialisasi()?;

        // 1. Verifikasi Checksum SHA-256
        let hash_aktual = sha256_hex(muatan_arsip);
        if !ekspektasi_checksum.is_empty() && hash_aktual != ekspektasi_checksum {
            return Err(format!(
                "Integritas paket '{} v{}' rusak! Checksum tidak cocok.\n  Diharapkan: {}\n  Aktual:     {}",
                nama, versi, ekspektasi_checksum, hash_aktual
            ));
        }

        // 2. Simpan arsip mentah
        let jalur_arsip = self.jalur_arsip_paket(nama, versi);
        fs::write(&jalur_arsip, muatan_arsip)
            .map_err(|e| format!("Gagal menulis arsip paket: {}", e))?;

        // 3. Ekstraksi arsip ke direktori sumber
        let jalur_sumber = self.jalur_sumber_paket(nama, versi);
        if jalur_sumber.exists() {
            let _ = fs::remove_dir_all(&jalur_sumber);
        }
        fs::create_dir_all(&jalur_sumber)
            .map_err(|e| format!("Gagal membuat direktori sumber paket: {}", e))?;

        ekstrak_arsip_komopaket(muatan_arsip, &jalur_sumber)?;

        Ok(jalur_sumber)
    }
}

/// Header arsip komopaket: format biner portabel mandiri
/// Format arsip Komolang `.komopaket`:
/// [4 byte magic: 'KOMO'] [4 byte versi: 1]
/// Pengulangan berkas:
/// [4 byte panjang_jalur (BE)] [panjang_jalur byte path relatif UTF-8]
/// [8 byte panjang_konten (BE)] [panjang_konten byte isi berkas]
/// Berakhir saat EOF.
pub fn kemas_arsip_komopaket(direktori_sumber: &Path) -> Result<Vec<u8>, String> {
    let mut buffer = Vec::new();
    buffer.extend_from_slice(b"KOMO");
    buffer.extend_from_slice(&1u32.to_be_bytes());

    let mut daftar_berkas = Vec::new();
    kumpulkan_berkas_rekursif(direktori_sumber, direktori_sumber, &mut daftar_berkas)?;

    // Urutkan daftar berkas secara alfabetis untuk menghasilkan arsip deterministik
    daftar_berkas.sort();

    for (rel_path, abs_path) in daftar_berkas {
        let konten = fs::read(&abs_path)
            .map_err(|e| format!("Gagal membaca '{:?}': {}", abs_path, e))?;

        let rel_bytes = rel_path.as_bytes();
        buffer.extend_from_slice(&(rel_bytes.len() as u32).to_be_bytes());
        buffer.extend_from_slice(rel_bytes);

        buffer.extend_from_slice(&(konten.len() as u64).to_be_bytes());
        buffer.extend_from_slice(&konten);
    }

    Ok(buffer)
}

/// Mengekstrak muatan biner arsip `.komopaket` ke direktori tujuan.
pub fn ekstrak_arsip_komopaket(muatan: &[u8], tujuan: &Path) -> Result<(), String> {
    if muatan.len() < 8 {
        return Err("Arsip komopaket rusak: ukuran muatan terlalu kecil".into());
    }

    if &muatan[0..4] != b"KOMO" {
        return Err("Bukan arsip resmi komopaket (magic bytes tidak cocok)".into());
    }

    let mut cursor = 8;
    while cursor < muatan.len() {
        if cursor + 4 > muatan.len() {
            return Err("Arsip rusak pada pembacaan panjang jalur".into());
        }
        let panjang_jalur = u32::from_be_bytes([
            muatan[cursor],
            muatan[cursor + 1],
            muatan[cursor + 2],
            muatan[cursor + 3],
        ]) as usize;
        cursor += 4;

        if cursor + panjang_jalur > muatan.len() {
            return Err("Arsip rusak pada pembacaan nama jalur".into());
        }
        let jalur_str = std::str::from_utf8(&muatan[cursor..cursor + panjang_jalur])
            .map_err(|_| "Nama jalur bukan UTF-8 valid")?;
        cursor += panjang_jalur;

        if cursor + 8 > muatan.len() {
            return Err("Arsip rusak pada pembacaan ukuran konten".into());
        }
        let panjang_konten = u64::from_be_bytes([
            muatan[cursor],
            muatan[cursor + 1],
            muatan[cursor + 2],
            muatan[cursor + 3],
            muatan[cursor + 4],
            muatan[cursor + 5],
            muatan[cursor + 6],
            muatan[cursor + 7],
        ]) as usize;
        cursor += 8;

        if cursor + panjang_konten > muatan.len() {
            return Err("Arsip rusak pada pembacaan isi berkas".into());
        }
        let isi_berkas = &muatan[cursor..cursor + panjang_konten];
        cursor += panjang_konten;

        let target_berkas = tujuan.join(jalur_str);
        if let Some(parent) = target_berkas.parent() {
            let _ = fs::create_dir_all(parent);
        }
        fs::write(&target_berkas, isi_berkas)
            .map_err(|e| format!("Gagal mengekstrak '{:?}': {}", target_berkas, e))?;
    }

    Ok(())
}

fn kumpulkan_berkas_rekursif(
    akar: &Path,
    sekarang: &Path,
    daftar: &mut Vec<(String, PathBuf)>,
) -> Result<(), String> {
    let entri = match fs::read_dir(sekarang) {
        Ok(e) => e,
        Err(_) => return Ok(()),
    };

    for item in entri.flatten() {
        let path = item.path();
        let nama = path.file_name().and_then(|n| n.to_str()).unwrap_or("");

        // Abaikan folder target dan git
        if nama == "target" || nama == ".git" {
            continue;
        }

        if path.is_dir() {
            kumpulkan_berkas_rekursif(akar, &path, daftar)?;
        } else if path.is_file() {
            if let Ok(rel) = path.strip_prefix(akar) {
                daftar.push((rel.to_string_lossy().to_string(), path.clone()));
            }
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tes_pengemasan_dan_ekstraksi_arsip() {
        let temp_dir = env::temp_dir().join("komo_tes_arsip");
        let sumber = temp_dir.join("sumber");
        let tujuan = temp_dir.join("tujuan");

        let _ = fs::remove_dir_all(&temp_dir);
        fs::create_dir_all(sumber.join("src")).unwrap();

        fs::write(sumber.join("komo.toml"), "[paket]\nnama = \"tes\"\n").unwrap();
        fs::write(sumber.join("src/utama.komo"), "paket utama\n").unwrap();

        // Kemas
        let arsip = kemas_arsip_komopaket(&sumber).unwrap();
        assert!(!arsip.is_empty());
        assert_eq!(&arsip[0..4], b"KOMO");

        let hash = sha256_hex(&arsip);
        assert_eq!(hash.len(), 64);

        // Ekstrak
        fs::create_dir_all(&tujuan).unwrap();
        ekstrak_arsip_komopaket(&arsip, &tujuan).unwrap();

        assert!(tujuan.join("komo.toml").exists());
        assert!(tujuan.join("src/utama.komo").exists());

        let _ = fs::remove_dir_all(&temp_dir);
    }
}
