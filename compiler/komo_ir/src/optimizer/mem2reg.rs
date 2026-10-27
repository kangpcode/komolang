// Copyright (c) kangpcode (Dhafa Nazula Permadi). Licensed under MIT or Apache-2.0.

//! Lintasan optimisasi Mem2Reg (Memory to Register Promotion).
//! Mempromosikan alokasi variabel lokal stack (`alloca`) menjadi register nilai SSA murni,
//! mengeliminasi pasangan instruksi `simpan` dan `muat` yang berulang.

use std::collections::{HashMap, HashSet};
use crate::fungsi::FungsiIr;
use crate::instruksi::Instruksi;
use crate::nilai::IdNilai;

/// Menjalankan optimisasi mem2reg pada satu fungsi KIR.
/// Mengembalikan true jika ada promosi memori yang berhasil dilakukan.
pub fn jalankan_mem2reg(f: &mut FungsiIr) -> bool {
    let mut berubah = false;

    // 1. Identifikasi pointer alloca yang tidak lolos (tidak escape)
    let mut kandidat_alloca: HashSet<IdNilai> = HashSet::new();
    let mut alloca_bocor: HashSet<IdNilai> = HashSet::new();

    for b in &f.blok {
        for (hasil, inst) in &b.instruksi {
            if let (Some(h), Instruksi::Alloca(_)) = (hasil, inst) {
                kandidat_alloca.insert(*h);
            }
        }
    }

    // Periksa apakah pointer digunakan selain pada Muat dan Simpan
    for b in &f.blok {
        for (_, inst) in &b.instruksi {
            match inst {
                Instruksi::Muat { penunjuk, .. } => {
                    // Valid
                }
                Instruksi::Simpan { penunjuk, nilai } => {
                    // Jika pointer alloca disimpan sebagai nilai ke tempat lain, pointer bocor
                    if kandidat_alloca.contains(nilai) {
                        alloca_bocor.insert(*nilai);
                    }
                }
                _ => {
                    for u in inst.ambil_penggunaan() {
                        if kandidat_alloca.contains(&u) {
                            alloca_bocor.insert(u);
                        }
                    }
                }
            }
        }
    }

    let alloca_valid: HashSet<IdNilai> = kandidat_alloca
        .difference(&alloca_bocor)
        .copied()
        .collect();

    if alloca_valid.is_empty() {
        return false;
    }

    // 2. Lakukan promosi mem2reg blok per blok untuk variabel lokal non-escaping
    let mut substitusi_nilai: HashMap<IdNilai, IdNilai> = HashMap::new();
    let mut nilai_memori_terkini: HashMap<IdNilai, IdNilai> = HashMap::new();

    for b in &mut f.blok {
        let mut instruksi_baru = Vec::new();

        for (hasil, mut inst) in b.instruksi.drain(..) {
            // Terapkan substitusi pada operan instruksi saat ini
            terapkan_substitusi_instruksi(&mut inst, &substitusi_nilai);

            match &inst {
                Instruksi::Alloca(_) => {
                    if let Some(h) = hasil {
                        if alloca_valid.contains(&h) {
                            // Hapus alloca karena akan dipromosikan ke register
                            berubah = true;
                            continue;
                        }
                    }
                    instruksi_baru.push((hasil, inst));
                }

                Instruksi::Simpan { penunjuk, nilai } => {
                    if alloca_valid.contains(penunjuk) {
                        // Catat nilai terbaru yang disimpan ke slot memori ini
                        nilai_memori_terkini.insert(*penunjuk, *nilai);
                        berubah = true;
                        // Hapus instruksi simpan redundant
                        continue;
                    }
                    instruksi_baru.push((hasil, inst));
                }

                Instruksi::Muat { penunjuk, .. } => {
                    if alloca_valid.contains(penunjuk) {
                        if let Some(&nilai_tersimpan) = nilai_memori_terkini.get(penunjuk) {
                            if let Some(h) = hasil {
                                substitusi_nilai.insert(h, nilai_tersimpan);
                                berubah = true;
                                continue;
                            }
                        }
                    }
                    instruksi_baru.push((hasil, inst));
                }

                _ => {
                    instruksi_baru.push((hasil, inst));
                }
            }
        }

        b.instruksi = instruksi_baru;

        // Terapkan substitusi pada terminator blok
        if let Some(term) = &mut b.terminator {
            terapkan_substitusi_terminator(term, &substitusi_nilai);
        }
    }

    berubah
}

fn terapkan_substitusi_instruksi(inst: &mut Instruksi, sub: &HashMap<IdNilai, IdNilai>) {
    match inst {
        Instruksi::Biner { kiri, kanan, .. } => {
            if let Some(&g) = sub.get(kiri) {
                *kiri = g;
            }
            if let Some(&g) = sub.get(kanan) {
                *kanan = g;
            }
        }
        Instruksi::Unari { operan, .. } => {
            if let Some(&g) = sub.get(operan) {
                *operan = g;
            }
        }
        Instruksi::Muat { penunjuk, .. } => {
            if let Some(&g) = sub.get(penunjuk) {
                *penunjuk = g;
            }
        }
        Instruksi::Simpan { nilai, penunjuk } => {
            if let Some(&g) = sub.get(nilai) {
                *nilai = g;
            }
            if let Some(&g) = sub.get(penunjuk) {
                *penunjuk = g;
            }
        }
        Instruksi::AksesBidang { struktur, .. } => {
            if let Some(&g) = sub.get(struktur) {
                *struktur = g;
            }
        }
        Instruksi::AksesIndeks { target, indeks } => {
            if let Some(&g) = sub.get(target) {
                *target = g;
            }
            if let Some(&g) = sub.get(indeks) {
                *indeks = g;
            }
        }
        Instruksi::Panggil { argumen, .. } => {
            for a in argumen {
                if let Some(&g) = sub.get(a) {
                    *a = g;
                }
            }
        }
        Instruksi::Phi { masukan, .. } => {
            for (val, _) in masukan {
                if let Some(&g) = sub.get(val) {
                    *val = g;
                }
            }
        }
        Instruksi::Cast { asal, .. } => {
            if let Some(&g) = sub.get(asal) {
                *asal = g;
            }
        }
        Instruksi::Konst(_) | Instruksi::Alloca(_) => {}
    }
}

fn terapkan_substitusi_terminator(
    term: &mut crate::instruksi::Terminator,
    sub: &HashMap<IdNilai, IdNilai>,
) {
    match term {
        crate::instruksi::Terminator::Cabang { kondisi, .. } => {
            if let Some(&g) = sub.get(kondisi) {
                *kondisi = g;
            }
        }
        crate::instruksi::Terminator::Kembali(Some(val)) => {
            if let Some(&g) = sub.get(val) {
                *val = g;
            }
        }
        _ => {}
    }
}
