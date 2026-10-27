// Copyright (c) kangpcode (Dhafa Nazula Permadi). Licensed under MIT or Apache-2.0.

//! Blok dasar (Basic Block) pada representasi SSA KIR.
//! Blok dasar terdiri dari urutan linear instruksi tanpa percabangan internal,
//! dan selalu diakhiri tepat oleh satu terminator.

use crate::instruksi::{Instruksi, Terminator};
use crate::nilai::{IdBlok, IdNilai};

/// Blok dasar dalam fungsi KIR.
#[derive(Debug, Clone, PartialEq)]
pub struct BlokDasar {
    /// Identifikator unik blok dasar.
    pub id: IdBlok,
    /// Label deskriptif opsional untuk kemudahan pembacaan IR.
    pub label: Option<String>,
    /// Urutan instruksi linier di dalam blok. Pasangan `(Option<IdNilai>, Instruksi)`
    /// menandakan apakah instruksi menghasilkan nilai SSA baru atau tidak.
    pub instruksi: Vec<(Option<IdNilai>, Instruksi)>,
    /// Instruksi pengakhir blok (wajib ada jika blok sudah lengkap).
    pub terminator: Option<Terminator>,
}

impl BlokDasar {
    /// Membuat blok dasar baru dengan ID dan label opsional.
    pub fn baru(id: IdBlok, label: Option<String>) -> Self {
        Self {
            id,
            label,
            instruksi: Vec::new(),
            terminator: None,
        }
    }

    /// Menambahkan instruksi ke akhir blok.
    pub fn tambah_instruksi(&mut self, hasil: Option<IdNilai>, inst: Instruksi) {
        assert!(
            self.terminator.is_none(),
            "Tidak dapat menambahkan instruksi ke blok yang sudah memiliki terminator"
        );
        self.instruksi.push((hasil, inst));
    }

    /// Menetapkan terminator blok dasar.
    pub fn atur_terminator(&mut self, term: Terminator) {
        self.terminator = Some(term);
    }

    /// Memeriksa apakah blok sudah ditutup dengan terminator.
    pub fn apakah_ditutup(&self) -> bool {
        self.terminator.is_some()
    }
}
