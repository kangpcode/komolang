// Copyright (c) kangpcode (Dhafa Nazula Permadi). Licensed under MIT or Apache-2.0.

//! Router berbasis Radix Tree tanpa alokasi berlebih dengan dukungan parameter dinamis dan wildcard.

use std::collections::HashMap;
use std::sync::Arc;

use crate::http::{MetodeHttp, PermintaanHttp, ResponsHttp};

/// Tipe penangan fungsi HTTP untuk melayani permintaan.
pub type PenanganRute = Arc<dyn Fn(&PermintaanHttp) -> ResponsHttp + Send + Sync>;

/// Jenis simpul dalam pohon perutean Radix.
#[derive(Debug, Clone, PartialEq, Eq)]
enum JenisSimpul {
    Statis(String),
    Parameter(String), // :id
    Wildcard(String),  // *jalur
}

/// Satu simpul dalam Radix Tree perutean.
struct SimpulRute {
    jenis: JenisSimpul,
    anak: Vec<SimpulRute>,
    penangan: HashMap<MetodeHttp, PenanganRute>,
}

impl SimpulRute {
    fn baru(jenis: JenisSimpul) -> Self {
        Self {
            jenis,
            anak: Vec::new(),
            penangan: HashMap::new(),
        }
    }
}

/// Router HTTP utama KomoWeb.
pub struct Router {
    akar: SimpulRute,
}

impl Default for Router {
    fn default() -> Self {
        Self::baru()
    }
}

impl Router {
    pub fn baru() -> Self {
        Self {
            akar: SimpulRute::baru(JenisSimpul::Statis(String::new())),
        }
    }

    /// Mendaftarkan rute dengan metode HTTP dan pola jalur.
    pub fn daftarkan(
        &mut self,
        metode: MetodeHttp,
        jalur: &str,
        penangan: impl Fn(&PermintaanHttp) -> ResponsHttp + Send + Sync + 'static,
    ) {
        let segmen = pisah_segmen(jalur);
        let mut saat_ini = &mut self.akar;

        for s in segmen {
            let jenis = if let Some(param) = s.strip_prefix(':') {
                JenisSimpul::Parameter(param.to_string())
            } else if let Some(wild) = s.strip_prefix('*') {
                JenisSimpul::Wildcard(wild.to_string())
            } else {
                JenisSimpul::Statis(s.to_string())
            };

            let pos = saat_ini.anak.iter().position(|child| child.jenis == jenis);
            let idx = match pos {
                Some(i) => i,
                None => {
                    saat_ini.anak.push(SimpulRute::baru(jenis));
                    saat_ini.anak.len() - 1
                }
            };
            saat_ini = &mut saat_ini.anak[idx];
        }

        saat_ini.penangan.insert(metode, Arc::new(penangan));
    }

    pub fn dapatkan(
        &mut self,
        jalur: &str,
        penangan: impl Fn(&PermintaanHttp) -> ResponsHttp + Send + Sync + 'static,
    ) {
        self.daftarkan(MetodeHttp::Dapatkan, jalur, penangan);
    }

    pub fn pos(
        &mut self,
        jalur: &str,
        penangan: impl Fn(&PermintaanHttp) -> ResponsHttp + Send + Sync + 'static,
    ) {
        self.daftarkan(MetodeHttp::Pos, jalur, penangan);
    }

    pub fn taruh(
        &mut self,
        jalur: &str,
        penangan: impl Fn(&PermintaanHttp) -> ResponsHttp + Send + Sync + 'static,
    ) {
        self.daftarkan(MetodeHttp::Taruh, jalur, penangan);
    }

    pub fn hapus(
        &mut self,
        jalur: &str,
        penangan: impl Fn(&PermintaanHttp) -> ResponsHttp + Send + Sync + 'static,
    ) {
        self.daftarkan(MetodeHttp::Hapus, jalur, penangan);
    }

    /// Mencari penangan yang cocok dan mengekstrak parameter rute.
    pub fn cocokkan(
        &self,
        metode: MetodeHttp,
        jalur: &str,
    ) -> Option<(PenanganRute, HashMap<String, String>)> {
        let segmen = pisah_segmen(jalur);
        let mut parameter = HashMap::new();

        if let Some(penangan) = self.cocokkan_rekursif(&self.akar, &segmen, 0, metode, &mut parameter) {
            Some((penangan, parameter))
        } else {
            None
        }
    }

    fn cocokkan_rekursif(
        &self,
        simpul: &SimpulRute,
        segmen: &[&str],
        indeks: usize,
        metode: MetodeHttp,
        parameter: &mut HashMap<String, String>,
    ) -> Option<PenanganRute> {
        if indeks == segmen.len() {
            return simpul.penangan.get(&metode).cloned();
        }

        let s = segmen[indeks];

        // 1. Coba anak statis yang cocok persis
        for anak in &simpul.anak {
            if let JenisSimpul::Statis(ref nama) = anak.jenis {
                if nama == s {
                    if let Some(h) =
                        self.cocokkan_rekursif(anak, segmen, indeks + 1, metode, parameter)
                    {
                        return Some(h);
                    }
                }
            }
        }

        // 2. Coba anak parameter dinamis (:param)
        for anak in &simpul.anak {
            if let JenisSimpul::Parameter(ref nama_param) = anak.jenis {
                parameter.insert(nama_param.clone(), s.to_string());
                if let Some(h) =
                    self.cocokkan_rekursif(anak, segmen, indeks + 1, metode, parameter)
                {
                    return Some(h);
                }
                parameter.remove(nama_param);
            }
        }

        // 3. Coba anak wildcard (*sisa)
        for anak in &simpul.anak {
            if let JenisSimpul::Wildcard(ref nama_wild) = anak.jenis {
                let sisa_jalur = segmen[indeks..].join("/");
                parameter.insert(nama_wild.clone(), sisa_jalur);
                if let Some(h) = anak.penangan.get(&metode) {
                    return Some(h.clone());
                }
                parameter.remove(nama_wild);
            }
        }

        None
    }
}

fn pisah_segmen(jalur: &str) -> Vec<&str> {
    jalur
        .split('/')
        .map(|s| s.trim())
        .filter(|s| !s.is_empty())
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tes_perutean_dan_parameter() {
        let mut router = Router::baru();

        router.dapatkan("/halo", |_| ResponsHttp::teks("Halo Dunia"));
        router.dapatkan("/pengguna/:id", |req| {
            let id = req.parameter_jalur.get("id").cloned().unwrap_or_default();
            ResponsHttp::teks(&format!("Pengguna #{}", id))
        });
        router.dapatkan("/arsip/*jalur", |req| {
            let jalur = req.parameter_jalur.get("jalur").cloned().unwrap_or_default();
            ResponsHttp::teks(&format!("Berkas: {}", jalur))
        });

        // 1. Statis
        let (h1, _) = router.cocokkan(MetodeHttp::Dapatkan, "/halo").unwrap();
        let req1 = PermintaanHttp::baru(MetodeHttp::Dapatkan, "/halo");
        assert_eq!(h1(&req1).teks_muatan().unwrap(), "Halo Dunia");

        // 2. Parameter dinamis
        let (h2, param2) = router.cocokkan(MetodeHttp::Dapatkan, "/pengguna/42").unwrap();
        assert_eq!(param2.get("id").unwrap(), "42");
        let mut req2 = PermintaanHttp::baru(MetodeHttp::Dapatkan, "/pengguna/42");
        req2.parameter_jalur = param2;
        assert_eq!(h2(&req2).teks_muatan().unwrap(), "Pengguna #42");

        // 3. Wildcard
        let (h3, param3) = router.cocokkan(MetodeHttp::Dapatkan, "/arsip/dokumen/2026/laporan.pdf").unwrap();
        assert_eq!(param3.get("jalur").unwrap(), "dokumen/2026/laporan.pdf");
        let mut req3 = PermintaanHttp::baru(MetodeHttp::Dapatkan, "/arsip/dokumen/2026/laporan.pdf");
        req3.parameter_jalur = param3;
        assert_eq!(h3(&req3).teks_muatan().unwrap(), "Berkas: dokumen/2026/laporan.pdf");

        // 4. Tidak cocok
        assert!(router.cocokkan(MetodeHttp::Dapatkan, "/tak_ada").is_none());
    }
}
