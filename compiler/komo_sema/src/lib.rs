// Copyright (c) kangpcode (Dhafa Nazula Permadi). Licensed under MIT or Apache-2.0.

//! `komo_sema` melakukan analisis semantik, resolusi nama/simbol, pemeriksaan tipe,
//! pengecekan exhaustiveness pada `cocok`, dan aturan peminjaman/ownership sederhana.

pub mod eksaustif;
pub mod ownership;
pub mod pemeriksa;
pub mod simbol;
pub mod tipe;

pub use pemeriksa::PemeriksaSemantik;
pub use simbol::{JenisSimbol, Simbol, TabelSimbol};
pub use tipe::{tipe_dari_ast, SemTipe};

use komo_ast::{BerkasProgram, Rentang};
use komo_diag::Diagnostik;
use std::collections::HashMap;

/// Hasil analisis semantik dari sebuah program Komolang.
#[derive(Debug, Clone)]
pub struct HasilSemantik {
    pub tabel: TabelSimbol,
    pub diagnostik: Vec<Diagnostik>,
    pub tipe_ekspresi: HashMap<Rentang, SemTipe>,
}

impl HasilSemantik {
    pub fn berhasil(&self) -> bool {
        self.diagnostik.is_empty()
    }
}

/// Menjalankan analisis semantik menyeluruh pada AST program Komolang.
pub fn periksa(berkas: &BerkasProgram) -> HasilSemantik {
    let mut pemeriksa = PemeriksaSemantik::baru();
    pemeriksa.periksa_berkas(berkas);
    pemeriksa.diagnostik.sort_by_key(|d| d.rentang.awal);
    HasilSemantik {
        tabel: pemeriksa.tabel,
        diagnostik: pemeriksa.diagnostik,
        tipe_ekspresi: pemeriksa.tipe_ekspresi,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use komo_ast::*;

    fn r() -> Rentang {
        Rentang::titik_kosong()
    }

    fn id(n: &str) -> Pengenal {
        Pengenal::baru(n, r())
    }

    #[test]
    fn tes_program_minimal_valid() {
        let berkas = BerkasProgram {
            paket: DeklarasiPaket {
                nama: id("utama"),
                rentang: r(),
            },
            impor: Vec::new(),
            deklarasi: vec![Deklarasi::Konstanta(DeklarasiKonstanta {
                visibilitas: Visibilitas::Publik,
                nama: id("A"),
                tipe: None,
                nilai: Ekspresi::baru(JenisEkspresi::Bulat(42), r()),
                rentang: r(),
            })],
        };

        let hasil = periksa(&berkas);
        assert!(hasil.berhasil(), "{:?}", hasil.diagnostik);
        assert!(hasil.tabel.cari_simbol("A").is_some());
    }

    #[test]
    fn tes_deteksi_simbol_duplikat() {
        let berkas = BerkasProgram {
            paket: DeklarasiPaket {
                nama: id("utama"),
                rentang: r(),
            },
            impor: Vec::new(),
            deklarasi: vec![
                Deklarasi::Konstanta(DeklarasiKonstanta {
                    visibilitas: Visibilitas::Publik,
                    nama: id("X"),
                    tipe: None,
                    nilai: Ekspresi::baru(JenisEkspresi::Bulat(1), r()),
                    rentang: r(),
                }),
                Deklarasi::Konstanta(DeklarasiKonstanta {
                    visibilitas: Visibilitas::Publik,
                    nama: id("X"),
                    tipe: None,
                    nilai: Ekspresi::baru(JenisEkspresi::Bulat(2), r()),
                    rentang: r(),
                }),
            ],
        };

        let hasil = periksa(&berkas);
        assert!(!hasil.berhasil());
        assert_eq!(hasil.diagnostik[0].kode, komo_diag::kode::SIMBOL_DUPLIKAT);
    }
}
