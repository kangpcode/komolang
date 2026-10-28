// Copyright (c) kangpcode (Dhafa Nazula Permadi). Licensed under MIT or Apache-2.0.

//! Pipeline middleware berantai (Onion Architecture): CORS, Logger, Rate Limiter, Recovery, dan Session.

use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::Instant;

use crate::http::{KodeStatus, MetodeHttp, PermintaanHttp, ResponsHttp};

/// Trait untuk middleware pemrosesan permintaan dan respons berantai.
pub trait Middleware: Send + Sync {
    fn tangani(
        &self,
        permintaan: &mut PermintaanHttp,
        berikutnya: &dyn Fn(&mut PermintaanHttp) -> ResponsHttp,
    ) -> ResponsHttp;
}

/// Middleware penanganan Cross-Origin Resource Sharing (CORS).
pub struct CorsMiddleware {
    pub izinkan_asal: String,
    pub izinkan_metode: String,
    pub izinkan_header: String,
}

impl Default for CorsMiddleware {
    fn default() -> Self {
        Self {
            izinkan_asal: "*".into(),
            izinkan_metode: "GET, POST, PUT, DELETE, PATCH, OPTIONS".into(),
            izinkan_header: "Content-Type, Authorization, X-Requested-With".into(),
        }
    }
}

impl Middleware for CorsMiddleware {
    fn tangani(
        &self,
        permintaan: &mut PermintaanHttp,
        berikutnya: &dyn Fn(&mut PermintaanHttp) -> ResponsHttp,
    ) -> ResponsHttp {
        // Pre-flight request OPTIONS
        if permintaan.metode == MetodeHttp::Opsi {
            return ResponsHttp::baru(KodeStatus::TANPA_KONTEN)
                .dengan_header("Access-Control-Allow-Origin", &self.izinkan_asal)
                .dengan_header("Access-Control-Allow-Methods", &self.izinkan_metode)
                .dengan_header("Access-Control-Allow-Headers", &self.izinkan_header);
        }

        let mut res = berikutnya(permintaan);
        res.header
            .insert("Access-Control-Allow-Origin".into(), self.izinkan_asal.clone());
        res
    }
}

/// Middleware pencatat akses HTTP terstruktur (Structured Logger).
pub struct PencatatMiddleware;

impl Middleware for PencatatMiddleware {
    fn tangani(
        &self,
        permintaan: &mut PermintaanHttp,
        berikutnya: &dyn Fn(&mut PermintaanHttp) -> ResponsHttp,
    ) -> ResponsHttp {
        let mulai = Instant::now();
        let metode = permintaan.metode;
        let jalur = permintaan.jalur.clone();
        let klien = permintaan.alamat_klien.clone();

        let res = berikutnya(permintaan);
        let durasi = mulai.elapsed();

        println!(
            "[KomoWeb] {} {} -> {} ({:.2?}) [klien: {}]",
            metode, jalur, res.status.0, durasi, klien
        );

        res
    }
}

/// Middleware pembatas laju lalu lintas (Rate Limiter) berbasis Token Bucket per alamat IP.
pub struct PembatasLajuMiddleware {
    pub kapasitas_maks: usize,
    rekaman: Arc<Mutex<HashMap<String, usize>>>,
}

impl PembatasLajuMiddleware {
    pub fn baru(kapasitas_maks: usize) -> Self {
        Self {
            kapasitas_maks,
            rekaman: Arc::new(Mutex::new(HashMap::new())),
        }
    }
}

impl Middleware for PembatasLajuMiddleware {
    fn tangani(
        &self,
        permintaan: &mut PermintaanHttp,
        berikutnya: &dyn Fn(&mut PermintaanHttp) -> ResponsHttp,
    ) -> ResponsHttp {
        let mut rekaman = self.rekaman.lock().unwrap();
        let hitungan = rekaman
            .entry(permintaan.alamat_klien.clone())
            .or_insert(0);

        if *hitungan >= self.kapasitas_maks {
            return ResponsHttp::teks("429 Terlalu Banyak Permintaan. Silakan coba lagi nanti.")
                .status_kode(KodeStatus::TERLALU_BANYAK_PERMINTAAN);
        }

        *hitungan += 1;
        drop(rekaman);

        berikutnya(permintaan)
    }
}

/// Middleware penangkap panik (Panic Recovery) agar server tetap stabil.
pub struct PemulihanMiddleware;

impl Middleware for PemulihanMiddleware {
    fn tangani(
        &self,
        permintaan: &mut PermintaanHttp,
        berikutnya: &dyn Fn(&mut PermintaanHttp) -> ResponsHttp,
    ) -> ResponsHttp {
        let hasil = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            berikutnya(permintaan)
        }));

        match hasil {
            Ok(res) => res,
            Err(_) => ResponsHttp::kesalahan_internal("Terjadi kegagalan panik yang tertangkap oleh PemulihanMiddleware"),
        }
    }
}

/// Middleware sesi stateful berbasis cookie `KOMO_SESI_ID`.
#[derive(Clone, Default)]
pub struct SesiMiddleware {
    penyimpanan: Arc<Mutex<HashMap<String, HashMap<String, String>>>>,
}

impl SesiMiddleware {
    pub fn baru() -> Self {
        Self::default()
    }

    pub fn ambil_nilai(&self, id_sesi: &str, kunci: &str) -> Option<String> {
        let store = self.penyimpanan.lock().unwrap();
        store.get(id_sesi).and_then(|m| m.get(kunci).cloned())
    }

    pub fn simpan_nilai(&self, id_sesi: &str, kunci: &str, nilai: &str) {
        let mut store = self.penyimpanan.lock().unwrap();
        let entri = store.entry(id_sesi.to_string()).or_default();
        entri.insert(kunci.to_string(), nilai.to_string());
    }
}

impl Middleware for SesiMiddleware {
    fn tangani(
        &self,
        permintaan: &mut PermintaanHttp,
        berikutnya: &dyn Fn(&mut PermintaanHttp) -> ResponsHttp,
    ) -> ResponsHttp {
        let mut id_sesi = None;
        if let Some(cookie) = permintaan.ambil_header("cookie") {
            for bagian in cookie.split(';') {
                let bagian = bagian.trim();
                if let Some(val) = bagian.strip_prefix("KOMO_SESI_ID=") {
                    id_sesi = Some(val.to_string());
                    break;
                }
            }
        }

        let buat_baru = id_sesi.is_none();
        let sesi_aktif = id_sesi.unwrap_or_else(|| {
            format!("sesi_{}", std::time::SystemTime::now().elapsed().unwrap().as_nanos())
        });

        permintaan
            .header
            .insert("X-Komo-Sesi-Id".into(), sesi_aktif.clone());

        let mut res = berikutnya(permintaan);

        if buat_baru {
            res.header.insert(
                "Set-Cookie".into(),
                format!("KOMO_SESI_ID={}; Path=/; HttpOnly; SameSite=Lax", sesi_aktif),
            );
        }

        res
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tes_cors_dan_rate_limiter() {
        let cors = CorsMiddleware::default();
        let limiter = PembatasLajuMiddleware::baru(2);

        let mut req = PermintaanHttp::baru(MetodeHttp::Dapatkan, "/api");
        let handler = |_req: &mut PermintaanHttp| ResponsHttp::teks("OK");

        // Request 1: harus sukses
        let r1 = cors.tangani(&mut req, &|r| limiter.tangani(r, &handler));
        assert_eq!(r1.status, KodeStatus::OK);
        assert_eq!(r1.header.get("Access-Control-Allow-Origin").unwrap(), "*");

        // Request 2: harus sukses
        let r2 = cors.tangani(&mut req, &|r| limiter.tangani(r, &handler));
        assert_eq!(r2.status, KodeStatus::OK);

        // Request 3: melebihi kapasitas (maks 2), harus 429
        let r3 = cors.tangani(&mut req, &|r| limiter.tangani(r, &handler));
        assert_eq!(r3.status, KodeStatus::TERLALU_BANYAK_PERMINTAAN);
    }

    #[test]
    fn tes_pemulihan_panik() {
        let pemulihan = PemulihanMiddleware;
        let mut req = PermintaanHttp::baru(MetodeHttp::Dapatkan, "/panik");

        let res = pemulihan.tangani(&mut req, &|_| {
            panic!("Kesalahan tak terduga!");
        });

        assert_eq!(res.status, KodeStatus::KESALAHAN_SERVER_INTERNAL);
        assert!(res.teks_muatan().unwrap().contains("PemulihanMiddleware"));
    }
}
