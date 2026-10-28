// Copyright (c) kangpcode (Dhafa Nazula Permadi). Licensed under MIT or Apache-2.0.

//! Manajemen ruang kerja monorepo (workspace) Komolang dan resolusi grafik DAG.

use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};

use crate::manifes::ManifesPaket;

/// Representasi satu anggota paket dalam ruang kerja.
#[derive(Debug, Clone)]
pub struct AnggotaRuangKerja {
    pub nama: String,
    pub jalur: PathBuf,
    pub manifes: ManifesPaket,
    pub dependensi_anggota_lain: Vec<String>,
}

/// Struktur data ruang kerja monorepo Komolang.
#[derive(Debug, Clone)]
pub struct RuangKerja {
    pub jalur_akar: PathBuf,
    pub anggota: HashMap<String, AnggotaRuangKerja>,
}

impl RuangKerja {
    /// Memuat ruang kerja dari direktori akar yang berisi berkas `komo.toml` dengan blok `[ruang_kerja]`.
    pub fn muat(jalur_akar: &Path) -> Result<Self, String> {
        let toml_akar = jalur_akar.join("komo.toml");
        if !toml_akar.exists() {
            return Err(format!(
                "Berkas 'komo.toml' tidak ditemukan di akar ruang kerja '{:?}'",
                jalur_akar
            ));
        }

        let manifes_akar = ManifesPaket::muat_dari_berkas(&toml_akar)?;
        let rk = manifes_akar
            .ruang_kerja
            .ok_or_else(|| "Berkas komo.toml tidak memiliki blok '[ruang_kerja]'".to_string())?;

        let mut peta_anggota = HashMap::new();

        for pola in &rk.anggota {
            let jalur_anggota = jalur_akar.join(pola);
            let toml_anggota = jalur_anggota.join("komo.toml");
            if !toml_anggota.exists() {
                return Err(format!(
                    "Anggota ruang kerja '{}' tidak memiliki berkas komo.toml di '{:?}'",
                    pola, toml_anggota
                ));
            }

            let manifes = ManifesPaket::muat_dari_berkas(&toml_anggota)?;
            let info = manifes
                .paket
                .as_ref()
                .ok_or_else(|| format!("Anggota '{}' tidak memiliki blok [paket]", pola))?;

            peta_anggota.insert(
                info.nama.clone(),
                AnggotaRuangKerja {
                    nama: info.nama.clone(),
                    jalur: jalur_anggota,
                    manifes,
                    dependensi_anggota_lain: Vec::new(),
                },
            );
        }

        // Tautkan dependensi lokal antar anggota ruang kerja
        let nama_anggota_semua: HashSet<String> = peta_anggota.keys().cloned().collect();

        for anggota in peta_anggota.values_mut() {
            let mut dep_internal = Vec::new();
            for (nama_dep, _sumber) in &anggota.manifes.dependensi {
                if nama_anggota_semua.contains(nama_dep) {
                    dep_internal.push(nama_dep.clone());
                }
            }
            dep_internal.sort();
            dep_internal.dedup();
            anggota.dependensi_anggota_lain = dep_internal;
        }

        Ok(Self {
            jalur_akar: jalur_akar.to_path_buf(),
            anggota: peta_anggota,
        })
    }

    /// Menghitung urutan topologis kompilasi paket (Directed Acyclic Graph)
    /// Memastikan dependensi dibangun sebelum paket konsumennya.
    pub fn urutan_kompilasi_topologis(&self) -> Result<Vec<String>, String> {
        let mut urutan = Vec::new();
        let mut dikunjungi = HashSet::new();
        let mut dalam_tumpukan = HashSet::new();

        for nama in self.anggota.keys() {
            self.kunjungi_topologis(
                nama,
                &mut dikunjungi,
                &mut dalam_tumpukan,
                &mut urutan,
            )?;
        }

        Ok(urutan)
    }

    fn kunjungi_topologis(
        &self,
        nama: &str,
        dikunjungi: &mut HashSet<String>,
        dalam_tumpukan: &mut HashSet<String>,
        urutan: &mut Vec<String>,
    ) -> Result<(), String> {
        if dalam_tumpukan.contains(nama) {
            return Err(format!(
                "Terdeteksi siklus dependensi sirkuler yang melibatkan paket '{}'",
                nama
            ));
        }

        if dikunjungi.contains(nama) {
            return Ok(());
        }

        dalam_tumpukan.insert(nama.to_string());

        if let Some(anggota) = self.anggota.get(nama) {
            for dep in &anggota.dependensi_anggota_lain {
                self.kunjungi_topologis(dep, dikunjungi, dalam_tumpukan, urutan)?;
            }
        }

        dalam_tumpukan.remove(nama);
        dikunjungi.insert(nama.to_string());
        urutan.push(nama.to_string());

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn tes_ruang_kerja_topologis() {
        let temp_dir = std::env::temp_dir().join("komo_tes_ruang_kerja");
        let _ = fs::remove_dir_all(&temp_dir);

        // Struktur:
        // root/
        //   komo.toml ([ruang_kerja] anggota = ["krate_a", "krate_b", "krate_c"])
        //   krate_a/komo.toml
        //   krate_b/komo.toml (bergantung pada krate_a)
        //   krate_c/komo.toml (bergantung pada krate_b)

        fs::create_dir_all(temp_dir.join("krate_a")).unwrap();
        fs::create_dir_all(temp_dir.join("krate_b")).unwrap();
        fs::create_dir_all(temp_dir.join("krate_c")).unwrap();

        fs::write(
            temp_dir.join("komo.toml"),
            "[ruang_kerja]\nanggota = [\"krate_a\", \"krate_b\", \"krate_c\"]\n",
        )
        .unwrap();

        fs::write(
            temp_dir.join("krate_a/komo.toml"),
            "[paket]\nnama = \"krate_a\"\nversi = \"0.1.0\"\n",
        )
        .unwrap();

        fs::write(
            temp_dir.join("krate_b/komo.toml"),
            "[paket]\nnama = \"krate_b\"\nversi = \"0.1.0\"\n[dependensi]\n\"krate_a\" = { path = \"../krate_a\" }\n",
        )
        .unwrap();

        fs::write(
            temp_dir.join("krate_c/komo.toml"),
            "[paket]\nnama = \"krate_c\"\nversi = \"0.1.0\"\n[dependensi]\n\"krate_b\" = { path = \"../krate_b\" }\n",
        )
        .unwrap();

        let rk = RuangKerja::muat(&temp_dir).unwrap();
        assert_eq!(rk.anggota.len(), 3);

        let urutan = rk.urutan_kompilasi_topologis().unwrap();
        // krate_a harus sebelum krate_b, krate_b harus sebelum krate_c
        let pos_a = urutan.iter().position(|x| x == "krate_a").unwrap();
        let pos_b = urutan.iter().position(|x| x == "krate_b").unwrap();
        let pos_c = urutan.iter().position(|x| x == "krate_c").unwrap();

        assert!(pos_a < pos_b);
        assert!(pos_b < pos_c);

        let _ = fs::remove_dir_all(&temp_dir);
    }
}
