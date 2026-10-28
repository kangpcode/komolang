// Copyright (c) kangpcode (Dhafa Nazula Permadi). Licensed under MIT or Apache-2.0.

//! Klien registry paket `komopaket` dan mekanisme penerbitan paket (`komo terbit`).

use std::collections::HashMap;
use std::path::Path;

use crate::cache::kemas_arsip_komopaket;
use crate::manifes::ManifesPaket;
use crate::resolusi::InfoRilis;
use crate::semver::Versi;
use crate::sha256::sha256_hex;

/// Informasi hasil penerbitan paket ke registry `komopaket`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InfoPenerbitan {
    pub nama: String,
    pub versi: Versi,
    pub checksum_sha256: String,
    pub ukuran_byte: usize,
    pub url_paket: String,
}

/// Hasil pencarian paket dalam registry.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HasilCariPaket {
    pub nama: String,
    pub versi_terbaru: Versi,
    pub deskripsi: String,
}

/// Registry in-memory untuk pengujian deterministik dan server lokal.
#[derive(Debug, Clone, Default)]
pub struct RegistryMock {
    /// Token otentikasi yang valid -> nama pengguna
    pub token_valid: HashMap<String, String>,
    /// Nama paket -> Versi -> (Muatan biner .komopaket, InfoRilis)
    pub repositori: HashMap<String, HashMap<Versi, (Vec<u8>, InfoRilis)>>,
}

impl RegistryMock {
    pub fn baru() -> Self {
        let mut reg = Self::default();
        reg.token_valid.insert(
            "kmp_token_pengembang_sah_2026".to_string(),
            "kangpcode".to_string(),
        );
        reg
    }

    /// Menambahkan token otentikasi baru
    pub fn tambah_token(&mut self, token: &str, pengguna: &str) {
        self.token_valid.insert(token.to_string(), pengguna.to_string());
    }

    /// Menerbitkan paket baru ke dalam registry mock.
    pub fn terima_penerbitan(
        &mut self,
        nama: &str,
        versi: &Versi,
        muatan_arsip: Vec<u8>,
        rilis: InfoRilis,
        token: &str,
    ) -> Result<InfoPenerbitan, String> {
        if !self.token_valid.contains_key(token) {
            return Err("Otentikasi ditolak: token registry tidak valid atau kedaluwarsa".into());
        }

        let entri_paket = self.repositori.entry(nama.to_string()).or_default();
        if entri_paket.contains_key(versi) {
            return Err(format!(
                "Versi '{}.{}.{}' untuk paket '{}' sudah pernah diterbitkan (tidak dapat ditimpa)",
                versi.major, versi.minor, versi.patch, nama
            ));
        }

        let checksum = sha256_hex(&muatan_arsip);
        let ukuran = muatan_arsip.len();

        entri_paket.insert(versi.clone(), (muatan_arsip, rilis));

        Ok(InfoPenerbitan {
            nama: nama.to_string(),
            versi: versi.clone(),
            checksum_sha256: checksum,
            ukuran_byte: ukuran,
            url_paket: format!("https://komopaket.dev/paket/{}/{}", nama, versi),
        })
    }

    /// Mengunduh muatan arsip paket
    pub fn unduh_arsip(&self, nama: &str, versi: &Versi) -> Result<&[u8], String> {
        let entri_paket = self
            .repositori
            .get(nama)
            .ok_or_else(|| format!("Paket '{}' tidak ditemukan", nama))?;

        let (arsip, _) = entri_paket
            .get(versi)
            .ok_or_else(|| format!("Versi '{}' dari paket '{}' tidak ditemukan", versi, nama))?;

        Ok(arsip.as_slice())
    }

    /// Mencari paket berdasarkan kata kunci kueri.
    pub fn cari(&self, kueri: &str) -> Vec<HasilCariPaket> {
        let kueri_rendah = kueri.to_lowercase();
        let mut hasil = Vec::new();

        for (nama, daftar_versi) in &self.repositori {
            if nama.to_lowercase().contains(&kueri_rendah) {
                if let Some((versi_tertinggi, _)) =
                    daftar_versi.iter().max_by_key(|(v, _)| *v)
                {
                    hasil.push(HasilCariPaket {
                        nama: nama.clone(),
                        versi_terbaru: versi_tertinggi.clone(),
                        deskripsi: format!("Paket resmi {}", nama),
                    });
                }
            }
        }

        hasil.sort_by(|a, b| a.nama.cmp(&b.nama));
        hasil
    }
}

/// Menjalankan validasi dan penerbitan proyek Komolang ke registry.
pub fn terbitkan_paket_ke_registry(
    jalur_proyek: &Path,
    token: &str,
    registry: &mut RegistryMock,
) -> Result<InfoPenerbitan, String> {
    let jalur_toml = jalur_proyek.join("komo.toml");
    if !jalur_toml.exists() {
        return Err("Berkas manifes 'komo.toml' tidak ditemukan di proyek ini".into());
    }

    let manifes = ManifesPaket::muat_dari_berkas(&jalur_toml)?;
    let info = manifes
        .paket
        .ok_or_else(|| "Blok '[paket]' tidak ditemukan dalam komo.toml".to_string())?;

    if info.nama.trim().is_empty() {
        return Err("Nama paket tidak boleh kosong".into());
    }

    // 1. Kemas direktori proyek menjadi arsip komopaket
    let muatan = kemas_arsip_komopaket(jalur_proyek)?;
    let hash = sha256_hex(&muatan);

    // 2. Kumpulkan dependensi ke InfoRilis
    let mut deps_req = std::collections::BTreeMap::new();
    for (nama_dep, sumber) in &manifes.dependensi {
        if let crate::manifes::SumberDependensi::Registry(req) = sumber {
            deps_req.insert(nama_dep.clone(), req.clone());
        }
    }

    let rilis = InfoRilis {
        versi: info.versi.clone(),
        dependensi: deps_req,
        checksum: hash,
    };

    // 3. Daftarkan ke registry
    registry.terima_penerbitan(&info.nama, &info.versi, muatan, rilis, token)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn tes_alur_penerbitan_dan_unduhan_registry() {
        let mut registry = RegistryMock::baru();
        let token = "kmp_token_pengembang_sah_2026";

        let temp_dir = std::env::temp_dir().join("komo_tes_terbit");
        let _ = fs::remove_dir_all(&temp_dir);
        fs::create_dir_all(temp_dir.join("src")).unwrap();

        let toml_konten = r#"[paket]
nama = "algoritma_cepat"
versi = "1.0.0"
penulis = ["Dhafa <kangpcode@gmail.com>"]
deskripsi = "Pustaka algoritma cepat Komolang"
"#;
        fs::write(temp_dir.join("komo.toml"), toml_konten).unwrap();
        fs::write(temp_dir.join("src/utama.komo"), "paket utama\n").unwrap();

        // Terbitkan
        let hasil = terbitkan_paket_ke_registry(&temp_dir, token, &mut registry);
        assert!(hasil.is_ok(), "Penerbitan harus sukses: {:?}", hasil);

        let info = hasil.unwrap();
        assert_eq!(info.nama, "algoritma_cepat");
        assert_eq!(info.versi, Versi::baru(1, 0, 0));
        assert_eq!(info.checksum_sha256.len(), 64);

        // Cari paket
        let cari = registry.cari("algoritma");
        assert_eq!(cari.len(), 1);
        assert_eq!(cari[0].nama, "algoritma_cepat");

        // Unduh arsip
        let arsip = registry.unduh_arsip("algoritma_cepat", &Versi::baru(1, 0, 0));
        assert!(arsip.is_ok());

        // Coba terbitkan ulang versi yang sama (harus ditolak)
        let gagal = terbitkan_paket_ke_registry(&temp_dir, token, &mut registry);
        assert!(gagal.is_err(), "Penimpaan versi yang sama harus ditolak");

        let _ = fs::remove_dir_all(&temp_dir);
    }
}
