// Copyright (c) kangpcode (Dhafa Nazula Permadi). Licensed under MIT or Apache-2.0.

//! Pengurai dan pembuat berkas manifes paket `komo.toml`.

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use crate::semver::{KebutuhanVersi, Versi};

/// Metadata inti paket dalam blok `[paket]` pada `komo.toml`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InfoPaket {
    pub nama: String,
    pub versi: Versi,
    pub penulis: Vec<String>,
    pub lisensi: Option<String>,
    pub deskripsi: Option<String>,
    pub titik_masuk: Option<String>,
}

/// Asal sumber sebuah dependensi paket.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SumberDependensi {
    /// Diunduh dari registry resmi `komopaket` berdasarkan batasan versi semver.
    Registry(KebutuhanVersi),
    /// Berasal dari jalur lokal filesystem.
    Jalur(PathBuf),
    /// Diambil langsung dari repositori Git.
    Git {
        url: String,
        cabang: Option<String>,
        tag: Option<String>,
        commit: Option<String>,
    },
}

/// Konfigurasi ruang kerja monorepo dalam blok `[ruang_kerja]`.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct KonfigurasiRuangKerja {
    pub anggota: Vec<String>,
}

/// Struktur data lengkap berkas manifes `komo.toml`.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ManifesPaket {
    pub paket: Option<InfoPaket>,
    pub dependensi: BTreeMap<String, SumberDependensi>,
    pub ruang_kerja: Option<KonfigurasiRuangKerja>,
}

impl ManifesPaket {
    /// Membuat manifes proyek baru dengan nilai default yang valid.
    pub fn baru(nama: &str, versi_str: &str) -> Result<Self, String> {
        let v = Versi::urai(versi_str)?;
        Ok(Self {
            paket: Some(InfoPaket {
                nama: nama.to_string(),
                versi: v,
                penulis: vec!["Pengembang Komolang".to_string()],
                lisensi: Some("MIT ATAU Apache-2.0".to_string()),
                deskripsi: Some(format!("Paket Komolang {}", nama)),
                titik_masuk: Some("src/utama.komo".to_string()),
            }),
            dependensi: BTreeMap::new(),
            ruang_kerja: None,
        })
    }

    /// Membaca dan mengurai berkas `komo.toml` dari jalur berkas.
    pub fn muat_dari_berkas(jalur: &Path) -> Result<Self, String> {
        let konten = std::fs::read_to_string(jalur)
            .map_err(|e| format!("Gagal membaca '{:?}': {}", jalur, e))?;
        Self::urai(&konten)
    }

    /// Mengurai representasi teks TOML `komo.toml`.
    pub fn urai(konten: &str) -> Result<Self, String> {
        let mut manifes = ManifesPaket::default();
        let mut blok_aktif = "";

        let mut nama_opt = None;
        let mut versi_opt = None;
        let mut penulis_list = Vec::new();
        let mut lisensi_opt = None;
        let mut deskripsi_opt = None;
        let mut titik_masuk_opt = None;
        let mut anggota_workspace = Vec::new();

        for baris in konten.lines() {
            let baris = baris.trim();
            if baris.is_empty() || baris.starts_with('#') {
                continue;
            }

            if baris.starts_with('[') && baris.ends_with(']') {
                blok_aktif = baris[1..baris.len() - 1].trim();
                continue;
            }

            match blok_aktif {
                "paket" => {
                    if let Some((kunci, nilai)) = pisah_kunci_nilai(baris) {
                        match kunci.as_str() {
                            "nama" => nama_opt = Some(bersihkan_string(&nilai)),
                            "versi" => {
                                let s = bersihkan_string(&nilai);
                                versi_opt = Some(Versi::urai(&s)?);
                            }
                            "penulis" => {
                                penulis_list = urai_larik_string(&nilai);
                            }
                            "lisensi" => lisensi_opt = Some(bersihkan_string(&nilai)),
                            "deskripsi" => deskripsi_opt = Some(bersihkan_string(&nilai)),
                            "titik_masuk" => titik_masuk_opt = Some(bersihkan_string(&nilai)),
                            _ => {}
                        }
                    }
                }
                "dependensi" => {
                    if let Some((kunci, nilai)) = pisah_kunci_nilai(baris) {
                        let nama_dep = bersihkan_string(&kunci);
                        let sumber = urai_sumber_dependensi(&nilai)?;
                        manifes.dependensi.insert(nama_dep, sumber);
                    }
                }
                "ruang_kerja" => {
                    if let Some((kunci, nilai)) = pisah_kunci_nilai(baris) {
                        if kunci == "anggota" {
                            anggota_workspace = urai_larik_string(&nilai);
                        }
                    }
                }
                _ => {}
            }
        }

        if let Some(nama) = nama_opt {
            let versi = versi_opt.unwrap_or_else(|| Versi::baru(0, 1, 0));
            manifes.paket = Some(InfoPaket {
                nama,
                versi,
                penulis: penulis_list,
                lisensi: lisensi_opt,
                deskripsi: deskripsi_opt,
                titik_masuk: titik_masuk_opt,
            });
        }

        if !anggota_workspace.is_empty() {
            manifes.ruang_kerja = Some(KonfigurasiRuangKerja {
                anggota: anggota_workspace,
            });
        }

        Ok(manifes)
    }

    /// Menghasilkan format string TOML kanonik.
    pub fn ke_toml(&self) -> String {
        let mut toml = String::new();

        if let Some(ref p) = self.paket {
            toml.push_str("[paket]\n");
            toml.push_str(&format!("nama = \"{}\"\n", p.nama));
            toml.push_str(&format!("versi = \"{}\"\n", p.versi));

            let penulis_str = p
                .penulis
                .iter()
                .map(|a| format!("\"{}\"", a))
                .collect::<Vec<_>>()
                .join(", ");
            toml.push_str(&format!("penulis = [{}]\n", penulis_str));

            if let Some(ref l) = p.lisensi {
                toml.push_str(&format!("lisensi = \"{}\"\n", l));
            }
            if let Some(ref d) = p.deskripsi {
                toml.push_str(&format!("deskripsi = \"{}\"\n", d));
            }
            if let Some(ref tm) = p.titik_masuk {
                toml.push_str(&format!("titik_masuk = \"{}\"\n", tm));
            }
            toml.push('\n');
        }

        if let Some(ref rk) = self.ruang_kerja {
            toml.push_str("[ruang_kerja]\n");
            let anggota_str = rk
                .anggota
                .iter()
                .map(|a| format!("\"{}\"", a))
                .collect::<Vec<_>>()
                .join(", ");
            toml.push_str(&format!("anggota = [{}]\n\n", anggota_str));
        }

        toml.push_str("[dependensi]\n");
        for (nama, sumber) in &self.dependensi {
            match sumber {
                SumberDependensi::Registry(kebutuhan) => {
                    toml.push_str(&format!("\"{}\" = \"{}\"\n", nama, kebutuhan));
                }
                SumberDependensi::Jalur(p) => {
                    toml.push_str(&format!("\"{}\" = {{ path = \"{}\" }}\n", nama, p.display()));
                }
                SumberDependensi::Git {
                    url,
                    cabang,
                    tag,
                    commit,
                } => {
                    let mut extra = String::new();
                    if let Some(c) = cabang {
                        extra.push_str(&format!(", cabang = \"{}\"", c));
                    }
                    if let Some(t) = tag {
                        extra.push_str(&format!(", tag = \"{}\"", t));
                    }
                    if let Some(cm) = commit {
                        extra.push_str(&format!(", commit = \"{}\"", cm));
                    }
                    toml.push_str(&format!(
                        "\"{}\" = {{ git = \"{}\"{} }}\n",
                        nama, url, extra
                    ));
                }
            }
        }

        toml
    }
}

fn pisah_kunci_nilai(baris: &str) -> Option<(String, String)> {
    if let Some(pos) = baris.find('=') {
        let kunci = baris[..pos].trim().to_string();
        let nilai = baris[pos + 1..].trim().to_string();
        Some((kunci, nilai))
    } else {
        None
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

fn urai_sumber_dependensi(nilai: &str) -> Result<SumberDependensi, String> {
    let nilai = nilai.trim();
    // 1. Literal string versi semver (registry)
    if nilai.starts_with('"') && nilai.ends_with('"') {
        let bersih = bersihkan_string(nilai);
        let req = KebutuhanVersi::urai(&bersih)?;
        return Ok(SumberDependensi::Registry(req));
    }

    // 2. Format tabel sebaris { path = "..." } atau { git = "...", cabang = "..." }
    if nilai.starts_with('{') && nilai.ends_with('}') {
        let dalam = &nilai[1..nilai.len() - 1];
        let entri: Vec<&str> = dalam.split(',').collect();
        let mut map = BTreeMap::new();
        for e in entri {
            if let Some((k, v)) = pisah_kunci_nilai(e) {
                map.insert(k, bersihkan_string(&v));
            }
        }

        if let Some(path) = map.get("path") {
            return Ok(SumberDependensi::Jalur(PathBuf::from(path)));
        }

        if let Some(git) = map.get("git") {
            return Ok(SumberDependensi::Git {
                url: git.clone(),
                cabang: map.get("cabang").cloned(),
                tag: map.get("tag").cloned(),
                commit: map.get("commit").cloned(),
            });
        }
    }

    Err(format!("Format sumber dependensi tidak dikenali: '{}'", nilai))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tes_manifes_roundtrip() {
        let toml_asli = r#"[paket]
nama = "alat_hebat"
versi = "1.0.4"
penulis = ["Dhafa <dhafa@example.com>"]
lisensi = "MIT"
deskripsi = "Pustaka serba bisa"
titik_masuk = "src/lib.komo"

[ruang_kerja]
anggota = ["submodul_a", "submodul_b"]

[dependensi]
"std/jaringan" = "^0.1.0"
"sub_lokal" = { path = "../sub_lokal" }
"#;

        let manifes = ManifesPaket::urai(toml_asli).unwrap();
        assert_eq!(manifes.paket.as_ref().unwrap().nama, "alat_hebat");
        assert_eq!(manifes.paket.as_ref().unwrap().versi, Versi::baru(1, 0, 4));
        assert_eq!(manifes.dependensi.len(), 2);
        assert!(manifes.ruang_kerja.is_some());

        let cetak = manifes.ke_toml();
        let manifes_ulang = ManifesPaket::urai(&cetak).unwrap();
        assert_eq!(manifes, manifes_ulang);
    }
}
