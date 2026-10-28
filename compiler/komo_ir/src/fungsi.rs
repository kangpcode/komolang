// Copyright (c) kangpcode (Dhafa Nazula Permadi). Licensed under MIT or Apache-2.0.

//! Representasi fungsi pada KIR.
//! Fungsi berisi parameter, blok awal, dan daftar blok dasar SSA.

use crate::blok::BlokDasar;
use crate::nilai::{IdBlok, IdNilai};
use crate::tipe::IrTipe;
use std::collections::HashMap;

/// Parameter fungsi pada tingkat IR.
#[derive(Debug, Clone, PartialEq)]
pub struct ParameterIr {
    /// Nilai register SSA untuk parameter ini.
    pub nilai: IdNilai,
    /// Nama parameter asli pada kode sumber.
    pub nama: String,
    /// Tipe data konkret parameter.
    pub tipe: IrTipe,
}

/// Fungsi dalam KIR.
#[derive(Debug, Clone, PartialEq)]
pub struct FungsiIr {
    /// Nama fungsi atau metode (misalnya `utama` atau `Penghitung.tambah`).
    pub nama: String,
    /// Daftar parameter formal fungsi.
    pub parameter: Vec<ParameterIr>,
    /// Tipe nilai kembalian.
    pub tipe_kembali: IrTipe,
    /// Seluruh blok dasar yang menyusun fungsi ini.
    pub blok: Vec<BlokDasar>,
    /// ID blok dasar entri/awal.
    pub blok_awal: IdBlok,
    /// Penghitung untuk menghasilkan ID nilai baru yang unik dalam fungsi ini.
    pub cacah_nilai: usize,
    /// Penghitung untuk menghasilkan ID blok baru yang unik.
    pub cacah_blok: usize,
}

impl FungsiIr {
    /// Membuat fungsi baru dengan nama dan tipe kembalian.
    pub fn baru(nama: String, tipe_kembali: IrTipe) -> Self {
        let blok_awal_id = IdBlok(0);
        let blok_awal = BlokDasar::baru(blok_awal_id, Some("entri".to_string()));
        Self {
            nama,
            parameter: Vec::new(),
            tipe_kembali,
            blok: vec![blok_awal],
            blok_awal: blok_awal_id,
            cacah_nilai: 0,
            cacah_blok: 1,
        }
    }

    /// Menghasilkan `IdNilai` unik baru untuk fungsi ini.
    pub fn alokasikan_nilai(&mut self) -> IdNilai {
        let id = IdNilai(self.cacah_nilai);
        self.cacah_nilai += 1;
        id
    }

    /// Menambahkan parameter baru ke fungsi dan mengalokasikan ID nilainya.
    pub fn tambah_parameter(&mut self, nama: String, tipe: IrTipe) -> IdNilai {
        let nilai = self.alokasikan_nilai();
        self.parameter.push(ParameterIr { nilai, nama, tipe });
        nilai
    }

    /// Membuat blok dasar baru dalam fungsi ini dan mengembalikan ID-nya.
    pub fn buat_blok(&mut self, label: Option<String>) -> IdBlok {
        let id = IdBlok(self.cacah_blok);
        self.cacah_blok += 1;
        self.blok.push(BlokDasar::baru(id, label));
        id
    }

    /// Mencari referensi ke blok dasar berdasarkan ID.
    pub fn cari_blok(&self, id: IdBlok) -> Option<&BlokDasar> {
        self.blok.iter().find(|b| b.id == id)
    }

    /// Mencari referensi mutabel ke blok dasar berdasarkan ID.
    pub fn cari_blok_mut(&mut self, id: IdBlok) -> Option<&mut BlokDasar> {
        self.blok.iter_mut().find(|b| b.id == id)
    }

    /// Menghitung relasi pendahulu (predecessors) untuk seluruh blok dasar dalam fungsi ini.
    pub fn hitung_pendahulu(&self) -> HashMap<IdBlok, Vec<IdBlok>> {
        let mut pendahulu: HashMap<IdBlok, Vec<IdBlok>> = HashMap::new();
        for b in &self.blok {
            pendahulu.entry(b.id).or_default();
            if let Some(term) = &b.terminator {
                for penerus in term.ambil_penerus() {
                    pendahulu.entry(penerus).or_default().push(b.id);
                }
            }
        }
        pendahulu
    }

    /// Mengambil tipe nilai SSA dari parameter atau instruksi penghasilnya.
    pub fn ambil_tipe_nilai(&self, id: IdNilai) -> Option<IrTipe> {
        for p in &self.parameter {
            if p.nilai == id {
                return Some(p.tipe.clone());
            }
        }
        for b in &self.blok {
            for (hasil_opt, inst) in &b.instruksi {
                if *hasil_opt == Some(id) {
                    return inst.tipe_hasil();
                }
            }
        }
        None
    }
}
