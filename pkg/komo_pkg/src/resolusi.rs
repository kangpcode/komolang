// Copyright (c) kangpcode (Dhafa Nazula Permadi). Licensed under MIT or Apache-2.0.

//! Algoritma solver resolusi dependensi semver deterministik.

use std::collections::{BTreeMap, HashMap};

use crate::kunci::{BerkasKunci, PaketTerkunci};
use crate::manifes::SumberDependensi;
use crate::semver::{KebutuhanVersi, Versi};

/// Metadata sebuah rilis versi paket yang tersedia di registry.
#[derive(Debug, Clone)]
pub struct InfoRilis {
    pub versi: Versi,
    pub dependensi: BTreeMap<String, KebutuhanVersi>,
    pub checksum: String,
}

/// Indeks basis data paket yang tersedia untuk proses resolusi.
#[derive(Debug, Clone, Default)]
pub struct IndeksPaket {
    pub daftar_rilis: HashMap<String, Vec<InfoRilis>>,
}

impl IndeksPaket {
    pub fn baru() -> Self {
        Self::default()
    }

    /// Menambahkan versi rilis paket baru ke dalam indeks.
    pub fn daftarkan_rilis(&mut self, nama: &str, rilis: InfoRilis) {
        let entri = self.daftar_rilis.entry(nama.to_string()).or_default();
        entri.push(rilis);
        // Urutkan rilis dari versi tertinggi ke terendah untuk pemilihan greedy optimal
        entri.sort_by(|a, b| b.versi.cmp(&a.versi));
    }
}

/// Hasil resolusi dependensi yang berhasil diselesaikan.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HasilResolusi {
    pub paket_terpilih: BTreeMap<String, (Versi, String, Vec<String>)>,
}

impl HasilResolusi {
    /// Mengonversi hasil resolusi dependensi menjadi representasi `BerkasKunci`.
    pub fn ke_berkas_kunci(&self, registry_url: &str) -> BerkasKunci {
        let mut kunci = BerkasKunci::baru();
        for (nama, (versi, checksum, deps)) in &self.paket_terpilih {
            kunci.pasang(PaketTerkunci {
                nama: nama.clone(),
                versi: versi.clone(),
                sumber: format!("registry+{}", registry_url),
                checksum: checksum.clone(),
                dependensi: deps.clone(),
            });
        }
        kunci
    }
}

/// Penyelesai dependensi (Dependency Solver).
pub struct PenyelesaiDependensi<'a> {
    indeks: &'a IndeksPaket,
}

impl<'a> PenyelesaiDependensi<'a> {
    pub fn baru(indeks: &'a IndeksPaket) -> Self {
        Self { indeks }
    }

    /// Menyelesaikan grafik dependensi dari himpunan kebutuhan akar.
    pub fn selesaikan(
        &self,
        akar: &BTreeMap<String, SumberDependensi>,
    ) -> Result<HasilResolusi, String> {
        let mut batasan_kumulatif: BTreeMap<String, Vec<KebutuhanVersi>> = BTreeMap::new();

        // 1. Kumpulkan batasan dari paket akar
        for (nama, sumber) in akar {
            match sumber {
                SumberDependensi::Registry(req) => {
                    batasan_kumulatif
                        .entry(nama.clone())
                        .or_default()
                        .push(req.clone());
                }
                SumberDependensi::Jalur(_) | SumberDependensi::Git { .. } => {
                    // Sumber lokal/git ditangani khusus
                }
            }
        }

        let mut terpilih: BTreeMap<String, (Versi, String, Vec<String>)> = BTreeMap::new();
        let mut antrean: Vec<String> = batasan_kumulatif.keys().cloned().collect();

        while let Some(nama_paket) = antrean.pop() {
            if terpilih.contains_key(&nama_paket) {
                continue;
            }

            let daftar_rilis = self
                .indeks
                .daftar_rilis
                .get(&nama_paket)
                .ok_or_else(|| format!("Paket '{}' tidak ditemukan dalam indeks registry", nama_paket))?;

            let batasan = batasan_kumulatif.get(&nama_paket).unwrap();

            // Cari versi tertinggi yang memenuhi semua batasan
            let rilis_cocok = daftar_rilis
                .iter()
                .find(|r| batasan.iter().all(|req| req.cocok_dengan(&r.versi)));

            let rilis_terpilih = match rilis_cocok {
                Some(r) => r,
                None => {
                    let req_str = batasan
                        .iter()
                        .map(|b| b.to_string())
                        .collect::<Vec<_>>()
                        .join(", ");
                    return Err(format!(
                        "Konflik resolusi: tidak ada versi '{}' yang memenuhi batasan [{}]",
                        nama_paket, req_str
                    ));
                }
            };

            let mut dep_anak = Vec::new();
            for (anak_nama, anak_req) in &rilis_terpilih.dependensi {
                dep_anak.push(anak_nama.clone());
                let batasan_anak = batasan_kumulatif.entry(anak_nama.clone()).or_default();
                batasan_anak.push(anak_req.clone());

                if let Some((versi_sudah, _, _)) = terpilih.get(anak_nama) {
                    if !anak_req.cocok_dengan(versi_sudah) {
                        return Err(format!(
                            "Konflik versi dependensi transisi: paket '{}' membutuhkan '{} {}', tetapi versi '{}' sudah terpilih sebelumnya",
                            nama_paket, anak_nama, anak_req, versi_sudah
                        ));
                    }
                } else if !antrean.contains(anak_nama) {
                    antrean.push(anak_nama.clone());
                }
            }

            terpilih.insert(
                nama_paket,
                (
                    rilis_terpilih.versi.clone(),
                    rilis_terpilih.checksum.clone(),
                    dep_anak,
                ),
            );
        }

        Ok(HasilResolusi {
            paket_terpilih: terpilih,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tes_resolusi_dependensi_sukses() {
        let mut indeks = IndeksPaket::baru();

        // Pustaka A
        indeks.daftarkan_rilis(
            "pustaka_a",
            InfoRilis {
                versi: Versi::baru(1, 0, 0),
                dependensi: BTreeMap::from([("pustaka_b".into(), KebutuhanVersi::urai("^1.0").unwrap())]),
                checksum: "hash_a_100".into(),
            },
        );
        indeks.daftarkan_rilis(
            "pustaka_a",
            InfoRilis {
                versi: Versi::baru(1, 2, 0),
                dependensi: BTreeMap::from([("pustaka_b".into(), KebutuhanVersi::urai("^1.1").unwrap())]),
                checksum: "hash_a_120".into(),
            },
        );

        // Pustaka B
        indeks.daftarkan_rilis(
            "pustaka_b",
            InfoRilis {
                versi: Versi::baru(1, 0, 5),
                dependensi: BTreeMap::new(),
                checksum: "hash_b_105".into(),
            },
        );
        indeks.daftarkan_rilis(
            "pustaka_b",
            InfoRilis {
                versi: Versi::baru(1, 1, 2),
                dependensi: BTreeMap::new(),
                checksum: "hash_b_112".into(),
            },
        );

        let solver = PenyelesaiDependensi::baru(&indeks);
        let akar = BTreeMap::from([(
            "pustaka_a".into(),
            SumberDependensi::Registry(KebutuhanVersi::urai("^1.0").unwrap()),
        )]);

        let hasil = solver.selesaikan(&akar).unwrap();
        let (v_a, _, _) = hasil.paket_terpilih.get("pustaka_a").unwrap();
        let (v_b, _, _) = hasil.paket_terpilih.get("pustaka_b").unwrap();

        assert_eq!(*v_a, Versi::baru(1, 2, 0));
        assert_eq!(*v_b, Versi::baru(1, 1, 2));
    }

    #[test]
    fn tes_resolusi_deteksi_konflik() {
        let mut indeks = IndeksPaket::baru();
        indeks.daftarkan_rilis(
            "pustaka_x",
            InfoRilis {
                versi: Versi::baru(1, 0, 0),
                dependensi: BTreeMap::new(),
                checksum: "x100".into(),
            },
        );

        let solver = PenyelesaiDependensi::baru(&indeks);
        let akar = BTreeMap::from([(
            "pustaka_x".into(),
            SumberDependensi::Registry(KebutuhanVersi::urai("^2.0").unwrap()),
        )]);

        let hasil = solver.selesaikan(&akar);
        assert!(hasil.is_err(), "Harus mendeteksi konflik bila tidak ada versi yang cocok");
    }
}
