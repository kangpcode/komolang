// Copyright (c) kangpcode (Dhafa Nazula Permadi). Licensed under MIT or Apache-2.0.

//! Lintasan optimisasi Fungsi Inlining.
//! Menggantikan pemanggilan fungsi-fungsi kecil/sepele langsung ke dalam blok pemanggil,
//! mengeliminasi overhead pemanggilan fungsi.

use crate::fungsi::FungsiIr;
use crate::instruksi::{Instruksi, Terminator};
use crate::modul::ModulIr;
use crate::nilai::IdNilai;
use std::collections::HashMap;

/// Menjalankan optimisasi inlining pada seluruh modul KIR.
/// Mengembalikan true jika ada fungsi yang berhasil di-inline.
pub fn jalankan_inlining(modul: &mut ModulIr) -> bool {
    let mut berubah = false;

    // 1. Identifikasi fungsi kandidat inlining:
    // - Hanya memiliki 1 blok dasar (tanpa percabangan internal)
    // - Jumlah instruksi <= 6
    // - Bukan fungsi rekursif terhadap dirinya sendiri
    let mut kandidat_inline: HashMap<String, FungsiIr> = HashMap::new();

    for f in &modul.fungsi {
        if f.blok.len() == 1 {
            let blok = &f.blok[0];
            if blok.instruksi.len() <= 6 {
                let ada_rekursi = blok.instruksi.iter().any(|(_, inst)| {
                    if let Instruksi::Panggil { fungsi, .. } = inst {
                        fungsi == &f.nama
                    } else {
                        false
                    }
                });
                if !ada_rekursi {
                    kandidat_inline.insert(f.nama.clone(), f.clone());
                }
            }
        }
    }

    if kandidat_inline.is_empty() {
        return false;
    }

    // 2. Lakukan inlining pada pemanggil
    for f in &mut modul.fungsi {
        for b_idx in 0..f.blok.len() {
            let mut instruksi_lama = std::mem::take(&mut f.blok[b_idx].instruksi);
            let mut instruksi_baru = Vec::new();

            for (hasil_call, inst) in instruksi_lama.drain(..) {
                if let Instruksi::Panggil {
                    fungsi, argumen, ..
                } = &inst
                {
                    if let Some(target_f) = kandidat_inline.get(fungsi) {
                        if target_f.nama != f.nama {
                            let inline_res = inline_satu_blok(
                                &mut f.cacah_nilai,
                                &mut instruksi_baru,
                                target_f,
                                argumen,
                            );
                            if let (Some(h), Some(res_val)) = (hasil_call, inline_res) {
                                instruksi_baru.push((
                                    Some(h),
                                    Instruksi::Cast {
                                        asal: res_val,
                                        tipe_tujuan: target_f.tipe_kembali.clone(),
                                    },
                                ));
                            }
                            berubah = true;
                            continue;
                        }
                    }
                }
                instruksi_baru.push((hasil_call, inst));
            }

            f.blok[b_idx].instruksi = instruksi_baru;
        }
    }

    berubah
}

fn inline_satu_blok(
    cacah_nilai: &mut usize,
    inst_keluar: &mut Vec<(Option<IdNilai>, Instruksi)>,
    target: &FungsiIr,
    argumen: &[IdNilai],
) -> Option<IdNilai> {
    let mut peta_nilai: HashMap<IdNilai, IdNilai> = HashMap::new();

    for (param, &arg) in target.parameter.iter().zip(argumen.iter()) {
        peta_nilai.insert(param.nilai, arg);
    }

    let target_blok = &target.blok[0];

    for (target_hasil, target_inst) in &target_blok.instruksi {
        let mut inst_klon = target_inst.clone();
        remap_instruksi(&mut inst_klon, &peta_nilai);

        let hasil_baru = if target_hasil.is_some() {
            let id_baru = IdNilai(*cacah_nilai);
            *cacah_nilai += 1;
            peta_nilai.insert(target_hasil.unwrap(), id_baru);
            Some(id_baru)
        } else {
            None
        };

        inst_keluar.push((hasil_baru, inst_klon));
    }

    if let Some(Terminator::Kembali(Some(ret_val))) = &target_blok.terminator {
        let ret_terpetakan = peta_nilai.get(ret_val).copied().unwrap_or(*ret_val);
        Some(ret_terpetakan)
    } else {
        None
    }
}

fn remap_instruksi(inst: &mut Instruksi, peta: &HashMap<IdNilai, IdNilai>) {
    match inst {
        Instruksi::Biner { kiri, kanan, .. } => {
            if let Some(&g) = peta.get(kiri) {
                *kiri = g;
            }
            if let Some(&g) = peta.get(kanan) {
                *kanan = g;
            }
        }
        Instruksi::Unari { operan, .. } => {
            if let Some(&g) = peta.get(operan) {
                *operan = g;
            }
        }
        Instruksi::Muat { penunjuk, .. } => {
            if let Some(&g) = peta.get(penunjuk) {
                *penunjuk = g;
            }
        }
        Instruksi::Simpan { nilai, penunjuk } => {
            if let Some(&g) = peta.get(nilai) {
                *nilai = g;
            }
            if let Some(&g) = peta.get(penunjuk) {
                *penunjuk = g;
            }
        }
        Instruksi::AksesBidang { struktur, .. } => {
            if let Some(&g) = peta.get(struktur) {
                *struktur = g;
            }
        }
        Instruksi::AksesIndeks { target, indeks } => {
            if let Some(&g) = peta.get(target) {
                *target = g;
            }
            if let Some(&g) = peta.get(indeks) {
                *indeks = g;
            }
        }
        Instruksi::Panggil { argumen, .. } => {
            for a in argumen {
                if let Some(&g) = peta.get(a) {
                    *a = g;
                }
            }
        }
        Instruksi::Phi { masukan, .. } => {
            for (val, _) in masukan {
                if let Some(&g) = peta.get(val) {
                    *val = g;
                }
            }
        }
        Instruksi::Cast { asal, .. } => {
            if let Some(&g) = peta.get(asal) {
                *asal = g;
            }
        }
        Instruksi::Konst(_) | Instruksi::Alloca(_) => {}
    }
}
