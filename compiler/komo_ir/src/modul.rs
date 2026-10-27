// Copyright (c) kangpcode (Dhafa Nazula Permadi). Licensed under MIT or Apache-2.0.

//! Representasi Modul pada KIR.
//! Modul adalah unit kompilasi tingkat atas yang memuat variabel/data global dan fungsi.

use crate::fungsi::FungsiIr;
use crate::nilai::Konstanta;
use crate::tipe::IrTipe;

/// Definisi data atau konstanta global dalam modul.
#[derive(Debug, Clone, PartialEq)]
pub struct DataGlobal {
    /// Nama simbol global.
    pub nama: String,
    /// Tipe data konkret.
    pub tipe: IrTipe,
    /// Nilai inisialisasi awal jika ada.
    pub inisialisasi: Option<Konstanta>,
    /// Apakah data bersifat konstan (read-only)?
    pub konstan: bool,
}

/// Modul KIR tingkat atas.
#[derive(Debug, Clone, PartialEq)]
pub struct ModulIr {
    /// Nama paket atau modul.
    pub nama: String,
    /// Daftar data dan konstanta global.
    pub data_global: Vec<DataGlobal>,
    /// Daftar seluruh fungsi yang terdefinisi dalam modul.
    pub fungsi: Vec<FungsiIr>,
}

impl ModulIr {
    /// Membuat modul baru dengan nama yang diberikan.
    pub fn baru(nama: String) -> Self {
        Self {
            nama,
            data_global: Vec::new(),
            fungsi: Vec::new(),
        }
    }

    /// Menambahkan fungsi ke dalam modul.
    pub fn tambah_fungsi(&mut self, fungsi: FungsiIr) {
        self.fungsi.push(fungsi);
    }

    /// Menambahkan data global ke dalam modul.
    pub fn tambah_global(&mut self, global: DataGlobal) {
        self.data_global.push(global);
    }

    /// Mencari fungsi berdasarkan nama.
    pub fn cari_fungsi(&self, nama: &str) -> Option<&FungsiIr> {
        self.fungsi.iter().find(|f| f.nama == nama)
    }

    /// Mencari fungsi mutabel berdasarkan nama.
    pub fn cari_fungsi_mut(&mut self, nama: &str) -> Option<&mut FungsiIr> {
        self.fungsi.iter_mut().find(|f| f.nama == nama)
    }
}
