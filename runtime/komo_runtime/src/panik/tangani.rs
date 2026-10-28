// Copyright (c) kangpcode (Dhafa Nazula Permadi). Licensed under MIT or Apache-2.0.

//! Penanganan kepanikan runtime, pemulihan (catch unwind), serta deteksi overflow aritmatika dan batas memori.

use std::fmt;
use std::panic::{catch_unwind, AssertUnwindSafe};

use crate::panik::jejak::{BingkaiJejak, JejakTumpukan};

/// Jenis kesalahan runtime fatal yang memicu kepanikan (panic).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum KesalahanRuntime {
    PembagianDenganNol,
    OverflowAritmatika(String),
    IndeksDiLuarBatas { indeks: usize, panjang: usize },
    AksesPenunjukNol,
    KepanikanEksplisit(String),
}

impl fmt::Display for KesalahanRuntime {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            KesalahanRuntime::PembagianDenganNol => write!(f, "Pembagian atau modulus dengan angka nol"),
            KesalahanRuntime::OverflowAritmatika(ket) => write!(f, "Luapan nilai aritmatika (arithmetic overflow): {}", ket),
            KesalahanRuntime::IndeksDiLuarBatas { indeks, panjang } => {
                write!(f, "Indeks {} di luar batas panjang larik {}", indeks, panjang)
            }
            KesalahanRuntime::AksesPenunjukNol => write!(f, "Dereferensi penunjuk memori kosong / nol (null pointer dereference)"),
            KesalahanRuntime::KepanikanEksplisit(alasan) => write!(f, "{}", alasan),
        }
    }
}

/// Memeriksa apakah pembagi integer adalah nol.
#[inline(always)]
pub fn periksa_pembagian_nol_bulat(pembagi: i64) -> Result<(), KesalahanRuntime> {
    if pembagi == 0 {
        Err(KesalahanRuntime::PembagianDenganNol)
    } else {
        Ok(())
    }
}

/// Menghitung penjumlahan bertanda dengan deteksi overflow.
#[inline(always)]
pub fn periksa_overflow_tambah(a: i64, b: i64) -> Result<i64, KesalahanRuntime> {
    a.checked_add(b).ok_or_else(|| {
        KesalahanRuntime::OverflowAritmatika(format!("penjumlahan {} + {} meluap dari batas i64", a, b))
    })
}

/// Menghitung pengurangan bertanda dengan deteksi overflow.
#[inline(always)]
pub fn periksa_overflow_kurang(a: i64, b: i64) -> Result<i64, KesalahanRuntime> {
    a.checked_sub(b).ok_or_else(|| {
        KesalahanRuntime::OverflowAritmatika(format!("pengurangan {} - {} meluap dari batas i64", a, b))
    })
}

/// Menghitung perkalian bertanda dengan deteksi overflow.
#[inline(always)]
pub fn periksa_overflow_kali(a: i64, b: i64) -> Result<i64, KesalahanRuntime> {
    a.checked_mul(b).ok_or_else(|| {
        KesalahanRuntime::OverflowAritmatika(format!("perkalian {} * {} meluap dari batas i64", a, b))
    })
}

/// Memeriksa validitas batas indeks larik.
#[inline(always)]
pub fn periksa_indeks_larik(indeks: usize, panjang: usize) -> Result<(), KesalahanRuntime> {
    if indeks >= panjang {
        Err(KesalahanRuntime::IndeksDiLuarBatas { indeks, panjang })
    } else {
        Ok(())
    }
}

/// Menjalankan closure berpotensi panik dan menangkap kepanikan secara terisolasi tanpa mematikan proses utama.
pub fn tangkap_kepanikan<F, R>(aksi: F) -> Result<R, String>
where
    F: FnOnce() -> R,
{
    match catch_unwind(AssertUnwindSafe(aksi)) {
        Ok(hasil) => Ok(hasil),
        Err(err) => {
            let alasan = if let Some(s) = err.downcast_ref::<&str>() {
                s.to_string()
            } else if let Some(s) = err.downcast_ref::<String>() {
                s.clone()
            } else {
                "Kepanikan tidak teridentifikasi".to_string()
            };
            Err(alasan)
        }
    }
}

/// Memformat dan mencetak laporan kepanikan runtime Komolang ke stderr.
pub fn laporkan_kepanikan(
    alasan: &str,
    nama_fungsi: &str,
    berkas: &str,
    baris: usize,
    kolom: usize,
) -> String {
    let mut jejak = JejakTumpukan::baru();
    jejak.tambah(BingkaiJejak::baru(nama_fungsi, berkas, baris, kolom));
    jejak.format_laporan(alasan, thread_current_name().as_deref())
}

fn thread_current_name() -> Option<String> {
    std::thread::current().name().map(|s| s.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tes_deteksi_overflow_aritmatika() {
        assert!(periksa_overflow_tambah(i64::MAX, 1).is_err());
        assert!(periksa_overflow_kurang(i64::MIN, 1).is_err());
        assert!(periksa_overflow_kali(i64::MAX, 2).is_err());
        assert_eq!(periksa_overflow_tambah(10, 20).unwrap(), 30);
    }

    #[test]
    fn tes_deteksi_pembagian_nol() {
        assert_eq!(
            periksa_pembagian_nol_bulat(0),
            Err(KesalahanRuntime::PembagianDenganNol)
        );
        assert!(periksa_pembagian_nol_bulat(5).is_ok());
    }

    #[test]
    fn tes_deteksi_indeks_larik() {
        assert!(periksa_indeks_larik(0, 5).is_ok());
        assert!(periksa_indeks_larik(4, 5).is_ok());
        assert!(periksa_indeks_larik(5, 5).is_err());
    }

    #[test]
    fn tes_tangkap_kepanikan_aman() {
        let hasil_panik = tangkap_kepanikan(|| {
            panic!("Kepanikan uji coba!");
        });
        assert!(hasil_panik.is_err());
        assert!(hasil_panik.unwrap_err().contains("Kepanikan uji coba!"));

        let hasil_sukses = tangkap_kepanikan(|| 100 + 200);
        assert_eq!(hasil_sukses, Ok(300));
    }
}
