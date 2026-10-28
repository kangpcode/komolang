// Copyright (c) kangpcode (Dhafa Nazula Permadi). Licensed under MIT or Apache-2.0.

//! Format dan manajemen berkas kunci deterministik `komo.kunci`.

use std::collections::BTreeMap;
use std::fmt;
use std::fs;
use std::path::Path;

use crate::semver::Versi;

/// Entri spesifik sebuah paket yang dikunci pada versi dan checksum tertentu.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PaketTerkunci {
    pub nama: String,
    pub versi: Versi,
    pub sumber: String,
    pub checksum: String,
    pub dependensi: Vec<String>,
}

/// Struktur data isi lengkap berkas kunci `komo.kunci`.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct BerkasKunci {
    pub paket: BTreeMap<String, PaketTerkunci>,
}

impl BerkasKunci {
    pub fn baru() -> Self {
        Self {
            paket: BTreeMap::new(),
        }
    }

    /// Menambahkan atau memperbarui entri paket terkunci.
    pub fn pasang(&mut self, entri: PaketTerkunci) {
        self.paket.insert(entri.nama.clone(), entri);
    }

    /// Mengambil entri paket berdasarkan nama.
    pub fn ambil(&self, nama: &str) -> Option<&PaketTerkunci> {
        self.paket.get(nama)
    }

    /// Membaca berkas `komo.kunci` dari filesystem.
    pub fn muat_dari_berkas(jalur: &Path) -> Result<Self, String> {
        let konten = fs::read_to_string(jalur)
            .map_err(|e| format!("Gagal membaca '{:?}': {}", jalur, e))?;
        Self::urai(&konten)
    }

    /// Menyimpan berkas `komo.kunci` ke filesystem dengan urutan kanonik deterministik.
    pub fn simpan_ke_berkas(&self, jalur: &Path) -> Result<(), String> {
        let teks = self.ke_string();
        fs::write(jalur, teks).map_err(|e| format!("Gagal menulis '{:?}': {}", jalur, e))
    }

    /// Mengurai teks berkas kunci `komo.kunci`.
    pub fn urai(konten: &str) -> Result<Self, String> {
        let mut berkas_kunci = BerkasKunci::baru();

        let mut nama_opt = None;
        let mut versi_opt = None;
        let mut sumber_opt = None;
        let mut checksum_opt = None;
        let mut dependensi_list = Vec::new();

        let mut simpan_entri = |nama_opt: &mut Option<String>,
                               versi_opt: &mut Option<Versi>,
                               sumber_opt: &mut Option<String>,
                               checksum_opt: &mut Option<String>,
                               dependensi_list: &mut Vec<String>| {
            if let (Some(nama), Some(versi), Some(sumber), Some(checksum)) = (
                nama_opt.take(),
                versi_opt.take(),
                sumber_opt.take(),
                checksum_opt.take(),
            ) {
                berkas_kunci.pasang(PaketTerkunci {
                    nama,
                    versi,
                    sumber,
                    checksum,
                    dependensi: std::mem::take(dependensi_list),
                });
            }
        };

        for baris in konten.lines() {
            let baris = baris.trim();
            if baris.is_empty() || baris.starts_with('#') {
                continue;
            }

            if baris == "[[paket]]" {
                simpan_entri(
                    &mut nama_opt,
                    &mut versi_opt,
                    &mut sumber_opt,
                    &mut checksum_opt,
                    &mut dependensi_list,
                );
                continue;
            }

            if let Some(pos) = baris.find('=') {
                let kunci = baris[..pos].trim();
                let nilai = baris[pos + 1..].trim();

                match kunci {
                    "nama" => nama_opt = Some(bersihkan_string(nilai)),
                    "versi" => {
                        let s = bersihkan_string(nilai);
                        versi_opt = Some(Versi::urai(&s)?);
                    }
                    "sumber" => sumber_opt = Some(bersihkan_string(nilai)),
                    "checksum" => checksum_opt = Some(bersihkan_string(nilai)),
                    "dependensi" => {
                        dependensi_list = urai_larik_string(nilai);
                    }
                    _ => {}
                }
            }
        }

        simpan_entri(
            &mut nama_opt,
            &mut versi_opt,
            &mut sumber_opt,
            &mut checksum_opt,
            &mut dependensi_list,
        );

        Ok(berkas_kunci)
    }

    /// Menghasilkan format teks kanonik deterministik untuk `komo.kunci`.
    pub fn ke_string(&self) -> String {
        let mut out = String::new();
        out.push_str("# Berkas kunci dependensi Komolang v1 - JANGAN SUNTING SECARA MANUAL\n");
        out.push_str("# Format ini dihasilkan secara otomatis dan deterministik oleh komopaket.\n\n");

        for (_, p) in &self.paket {
            out.push_str("[[paket]]\n");
            out.push_str(&format!("nama = \"{}\"\n", p.nama));
            out.push_str(&format!("versi = \"{}\"\n", p.versi));
            out.push_str(&format!("sumber = \"{}\"\n", p.sumber));
            out.push_str(&format!("checksum = \"{}\"\n", p.checksum));

            if !p.dependensi.is_empty() {
                let deps_str = p
                    .dependensi
                    .iter()
                    .map(|d| format!("\"{}\"", d))
                    .collect::<Vec<_>>()
                    .join(", ");
                out.push_str(&format!("dependensi = [{}]\n", deps_str));
            }

            out.push('\n');
        }

        out
    }
}

impl fmt::Display for BerkasKunci {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.ke_string())
    }
}

fn bersihkan_string(s: &str) -> String {
    let s = s.trim();
    if s.starts_with('"') && s.ends_with('"') && s.len() >= 2 {
        s[1..s.len() - 1].to_string()
    } else {
        s.to_string()
    }
}

fn urai_larik_string(s: &str) -> Vec<String> {
    let s = s.trim();
    if !s.starts_with('[') || !s.ends_with(']') {
        return Vec::new();
    }
    let dalam = &s[1..s.len() - 1].trim();
    if dalam.is_empty() {
        return Vec::new();
    }

    dalam
        .split(',')
        .map(|item| bersihkan_string(item.trim()))
        .filter(|item| !item.is_empty())
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tes_berkas_kunci_roundtrip() {
        let mut kunci = BerkasKunci::baru();
        kunci.pasang(PaketTerkunci {
            nama: "jaringan".into(),
            versi: Versi::baru(1, 2, 0),
            sumber: "registry+https://komopaket.dev".into(),
            checksum: "a1b2c3d4e5f6".into(),
            dependensi: vec!["kripto".into(), "teks".into()],
        });

        kunci.pasang(PaketTerkunci {
            nama: "kripto".into(),
            versi: Versi::baru(0, 4, 1),
            sumber: "registry+https://komopaket.dev".into(),
            checksum: "fedcba987654".into(),
            dependensi: vec![],
        });

        let teks = kunci.ke_string();
        let kunci_kembali = BerkasKunci::urai(&teks).unwrap();
        assert_eq!(kunci, kunci_kembali);

        let p_jaringan = kunci_kembali.ambil("jaringan").unwrap();
        assert_eq!(p_jaringan.versi, Versi::baru(1, 2, 0));
        assert_eq!(p_jaringan.dependensi, vec!["kripto", "teks"]);
    }
}
