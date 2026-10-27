// Copyright (c) kangpcode (Dhafa Nazula Permadi). Licensed under MIT or Apache-2.0.

//! Pengoptimal KIR (Komo IR Optimizer).
//! Mengatur eksekusi lintasan optimisasi: Inlining, Mem2Reg, Constant Folding, dan Dead Code Elimination.

pub mod constant_folding;
pub mod dce;
pub mod inlining;
pub mod mem2reg;

use crate::fungsi::FungsiIr;
use crate::modul::ModulIr;

/// Manajer lintasan optimisasi untuk modul dan fungsi KIR.
#[derive(Debug, Clone)]
pub struct Pengoptimal {
    /// Tingkat optimasi (0 = mati, 1 = dasar, 2 = penuh).
    pub tingkat: usize,
    /// Apakah inlining diaktifkan.
    pub aktif_inlining: bool,
    /// Apakah mem2reg diaktifkan.
    pub aktif_mem2reg: bool,
    /// Apakah constant folding diaktifkan.
    pub aktif_constant_folding: bool,
    /// Apakah dead code elimination diaktifkan.
    pub aktif_dce: bool,
}

impl Default for Pengoptimal {
    fn default() -> Self {
        Self {
            tingkat: 2,
            aktif_inlining: true,
            aktif_mem2reg: true,
            aktif_constant_folding: true,
            aktif_dce: true,
        }
    }
}

impl Pengoptimal {
    /// Membuat pengoptimal baru dengan tingkat optimasi tertentu.
    pub fn baru(tingkat: usize) -> Self {
        if tingkat == 0 {
            Self {
                tingkat: 0,
                aktif_inlining: false,
                aktif_mem2reg: false,
                aktif_constant_folding: false,
                aktif_dce: false,
            }
        } else {
            Self {
                tingkat,
                ..Default::default()
            }
        }
    }

    /// Mengoptimalkan seluruh modul KIR.
    pub fn optimalkan_modul(&self, modul: &mut ModulIr) {
        if self.tingkat == 0 {
            return;
        }

        // 1. Inlining tingkat modul
        if self.aktif_inlining {
            inlining::jalankan_inlining(modul);
        }

        // 2. Optimasi tiap fungsi
        for f in &mut modul.fungsi {
            self.optimalkan_fungsi(f);
        }
    }

    /// Mengoptimalkan satu fungsi KIR sampai mencapai titik tetap atau batas iterasi.
    pub fn optimalkan_fungsi(&self, f: &mut FungsiIr) {
        if self.tingkat == 0 {
            return;
        }

        let mut iterasi = 0;
        let maks_iterasi = 8;

        loop {
            let mut berubah = false;

            if self.aktif_mem2reg && mem2reg::jalankan_mem2reg(f) {
                berubah = true;
            }

            if self.aktif_constant_folding && constant_folding::jalankan_constant_folding(f) {
                berubah = true;
            }

            if self.aktif_dce && dce::jalankan_dce(f) {
                berubah = true;
            }

            iterasi += 1;
            if !berubah || iterasi >= maks_iterasi {
                break;
            }
        }
    }
}
