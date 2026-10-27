// Copyright (c) kangpcode (Dhafa Nazula Permadi). Licensed under MIT or Apache-2.0.

//! `komo_ast` mendefinisikan representasi pohon sintaks abstrak (Abstract Syntax Tree / AST)
//! untuk bahasa pemrograman Komolang.

/// Menunjukkan lokasi span karakter dalam kode sumber (indeks byte awal dan akhir).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Rentang {
    pub awal: usize,
    pub akhir: usize,
    pub baris: usize,
    pub kolom: usize,
}

impl Rentang {
    pub fn baru(awal: usize, akhir: usize, baris: usize, kolom: usize) -> Self {
        Self {
            awal,
            akhir,
            baris,
            kolom,
        }
    }

    pub fn titik_kosong() -> Self {
        Self {
            awal: 0,
            akhir: 0,
            baris: 1,
            kolom: 1,
        }
    }
}

/// Identifikasi pengenal (identifier) beserta lokasinya.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Pengenal {
    pub nama: String,
    pub rentang: Rentang,
}

impl Pengenal {
    pub fn baru(nama: impl Into<String>, rentang: Rentang) -> Self {
        Self {
            nama: nama.into(),
            rentang,
        }
    }
}

/// Unit kompilasi lengkap (berkas `.komo`).
#[derive(Debug, Clone, PartialEq)]
pub struct BerkasProgram {
    pub paket: DeklarasiPaket,
    pub impor: Vec<DeklarasiImpor>,
    pub deklarasi: Vec<DeklarasiTingkatAtas>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DeklarasiPaket {
    pub nama: Pengenal,
    pub rentang: Rentang,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DeklarasiImpor {
    pub jalur: String,
    pub alias: Option<Pengenal>,
    pub rentang: Rentang,
}

#[derive(Debug, Clone, PartialEq)]
pub enum DeklarasiTingkatAtas {
    Fungsi(DeklarasiFungsi),
    Konstanta(DeklarasiKonstanta),
    Variabel(DeklarasiVariabel),
}

#[derive(Debug, Clone, PartialEq)]
pub struct DeklarasiFungsi {
    pub adalah_publik: bool,
    pub nama: Pengenal,
    pub parameter: Vec<ParameterFungsi>,
    pub tipe_kembalian: Option<String>,
    pub rentang: Rentang,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParameterFungsi {
    pub nama: Pengenal,
    pub tipe_data: String,
    pub adalah_pinjam: bool,
    pub adalah_mutabel: bool,
}

#[derive(Debug, Clone, PartialEq)]
pub struct DeklarasiKonstanta {
    pub adalah_publik: bool,
    pub nama: Pengenal,
    pub tipe_data: Option<String>,
    pub nilai_literal: String,
    pub rentang: Rentang,
}

#[derive(Debug, Clone, PartialEq)]
pub struct DeklarasiVariabel {
    pub adalah_mutabel: bool,
    pub nama: Pengenal,
    pub tipe_data: Option<String>,
    pub nilai_literal: String,
    pub rentang: Rentang,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tes_rentang_dan_pengenal() {
        let rentang = Rentang::baru(0, 5, 1, 1);
        let id = Pengenal::baru("utama", rentang);
        assert_eq!(id.nama, "utama");
        assert_eq!(id.rentang.baris, 1);
        assert_eq!(id.rentang.kolom, 1);
    }

    #[test]
    fn tes_berkas_program_kosong() {
        let paket = DeklarasiPaket {
            nama: Pengenal::baru("utama", Rentang::titik_kosong()),
            rentang: Rentang::titik_kosong(),
        };
        let berkas = BerkasProgram {
            paket,
            impor: Vec::new(),
            deklarasi: Vec::new(),
        };
        assert_eq!(berkas.paket.nama.nama, "utama");
        assert!(berkas.impor.is_empty());
        assert!(berkas.deklarasi.is_empty());
    }
}
