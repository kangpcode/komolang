// Copyright (c) kangpcode (Dhafa Nazula Permadi). Licensed under MIT or Apache-2.0.

//! Lintasan optimisasi Dead Code Elimination (DCE).
//! Menghapus instruksi murni yang nilainya tidak pernah dibaca,
//! serta mengeliminasi blok dasar yang tidak terjangkau (unreachable).

use std::collections::{HashSet, VecDeque};
use crate::fungsi::FungsiIr;
use crate::instruksi::{Instruksi, Terminator};
use crate::nilai::{IdBlok, IdNilai};

/// Menjalankan DCE pada satu fungsi KIR.
/// Mengembalikan true jika ada kode mati yang berhasil dieliminasi.
pub fn jalankan_dce(f: &mut FungsiIr) -> bool {
    let mut berubah = false;

    // 1. Eliminasi blok yang tidak terjangkau dari blok awal
    if eliminasi_blok_tak_terjangkau(f) {
        berubah = true;
    }

    // 2. Eliminasi instruksi tak terpakai (loop sampai titik tetap)
    while eliminasi_instruksi_tak_terpakai(f) {
        berubah = true;
    }

    berubah
}

fn eliminasi_blok_tak_terjangkau(f: &mut FungsiIr) -> bool {
    let mut terjangkau = HashSet::new();
    let mut antrean = VecDeque::new();

    terjangkau.insert(f.blok_awal);
    antrean.push_back(f.blok_awal);

    while let Some(b_id) = antrean.pop_front() {
        if let Some(b) = f.cari_blok(b_id) {
            if let Some(term) = &b.terminator {
                for p in term.ambil_penerus() {
                    if terjangkau.insert(p) {
                        antrean.push_back(p);
                    }
                }
            }
        }
    }

    let awal_len = f.blok.len();
    f.blok.retain(|b| terjangkau.contains(&b.id));
    let ada_dihapus = f.blok.len() < awal_len;

    // Bersihkan masukan phi yang merujuk ke blok yang sudah dihapus
    if ada_dihapus {
        for b in &mut f.blok {
            for (_, inst) in &mut b.instruksi {
                if let Instruksi::Phi { masukan, .. } = inst {
                    masukan.retain(|(_, blk_asal)| terjangkau.contains(blk_asal));
                }
            }
        }
    }

    ada_dihapus
}

fn eliminasi_instruksi_tak_terpakai(f: &mut FungsiIr) -> bool {
    // Kumpulkan seluruh IdNilai yang digunakan oleh instruksi dan terminator
    let mut nilai_dipakai = HashSet::new();

    for b in &f.blok {
        for (_, inst) in &b.instruksi {
            for u in inst.ambil_penggunaan() {
                nilai_dipakai.insert(u);
            }
        }
        if let Some(term) = &b.terminator {
            for u in term.ambil_penggunaan() {
                nilai_dipakai.insert(u);
            }
        }
    }

    let mut ada_dihapus = false;

    for b in &mut f.blok {
        let awal_len = b.instruksi.len();
        b.instruksi.retain(|(hasil, inst)| {
            if inst.punya_efek_samping() {
                return true; // Jangan hapus instruksi yang memiliki efek samping
            }
            if let Some(h) = hasil {
                nilai_dipakai.contains(h)
            } else {
                false
            }
        });

        if b.instruksi.len() < awal_len {
            ada_dihapus = true;
        }
    }

    ada_dihapus
}
