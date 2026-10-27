// Copyright (c) kangpcode (Dhafa Nazula Permadi). Licensed under MIT or Apache-2.0.

//! Lintasan optimisasi Constant Folding dan Penyederhanaan Aljabar.
//! Menghitung nilai instruksi dengan operan konstan pada saat kompilasi
//! dan menyederhanakan percabangan berkondisi konstan.

use crate::fungsi::FungsiIr;
use crate::instruksi::{Instruksi, OpBiner, OpUnari, Terminator};
use crate::nilai::{IdNilai, Konstanta};
use std::collections::HashMap;

/// Menjalankan optimisasi constant folding pada satu fungsi KIR.
/// Mengembalikan true jika ada perubahan yang dilakukan.
pub fn jalankan_constant_folding(f: &mut FungsiIr) -> bool {
    let mut berubah = false;

    // 1. Kumpulkan seluruh konstanta yang terdefinisi
    let mut nilai_konstan: HashMap<IdNilai, Konstanta> = HashMap::new();

    for b in &f.blok {
        for (hasil, inst) in &b.instruksi {
            if let Some(h) = hasil {
                if let Instruksi::Konst(k) = inst {
                    nilai_konstan.insert(*h, k.clone());
                }
            }
        }
    }

    // 2. Evaluasi instruksi biner dan unari dengan operan konstan
    for b in &mut f.blok {
        for (hasil, inst) in &mut b.instruksi {
            let hasil_id = match hasil {
                Some(id) => *id,
                None => continue,
            };

            let mut pengganti_konst = None;

            match inst {
                Instruksi::Biner {
                    op, kiri, kanan, ..
                } => {
                    let k_kiri = nilai_konstan.get(kiri);
                    let k_kanan = nilai_konstan.get(kanan);

                    if let (Some(l), Some(r)) = (k_kiri, k_kanan) {
                        if let Some(eval) = lipat_biner(*op, l, r) {
                            pengganti_konst = Some(eval);
                        }
                    } else if let Some(r) = k_kanan {
                        // Penyederhanaan aljabar: x + 0 = x, x * 1 = x, x * 0 = 0
                        match (op, r) {
                            (OpBiner::Tambah, Konstanta::Bulat(0))
                            | (OpBiner::Kurang, Konstanta::Bulat(0)) => {
                                // x + 0 => x (bisa dipropagasikan jika x konstan)
                            }
                            (OpBiner::Kali, Konstanta::Bulat(0)) => {
                                pengganti_konst = Some(Konstanta::Bulat(0));
                            }
                            _ => {}
                        }
                    }
                }

                Instruksi::Unari { op, operan, .. } => {
                    if let Some(k) = nilai_konstan.get(operan) {
                        if let Some(eval) = lipat_unari(*op, k) {
                            pengganti_konst = Some(eval);
                        }
                    }
                }

                _ => {}
            }

            if let Some(k) = pengganti_konst {
                *inst = Instruksi::Konst(k.clone());
                nilai_konstan.insert(hasil_id, k);
                berubah = true;
            }
        }

        // 3. Lipat percabangan jika kondisinya sudah pasti konstan
        if let Some(Terminator::Cabang {
            kondisi,
            jika_benar,
            jika_salah,
        }) = &b.terminator
        {
            if let Some(Konstanta::Bool(b_val)) = nilai_konstan.get(kondisi) {
                let target = if *b_val { *jika_benar } else { *jika_salah };
                b.terminator = Some(Terminator::Lompat(target));
                berubah = true;
            }
        }
    }

    berubah
}

fn lipat_biner(op: OpBiner, l: &Konstanta, r: &Konstanta) -> Option<Konstanta> {
    match (l, r) {
        (Konstanta::Bulat(a), Konstanta::Bulat(b)) => match op {
            OpBiner::Tambah => Some(Konstanta::Bulat(a.wrapping_add(*b))),
            OpBiner::Kurang => Some(Konstanta::Bulat(a.wrapping_sub(*b))),
            OpBiner::Kali => Some(Konstanta::Bulat(a.wrapping_mul(*b))),
            OpBiner::Bagi if *b != 0 => Some(Konstanta::Bulat(a.wrapping_div(*b))),
            OpBiner::Sisa if *b != 0 => Some(Konstanta::Bulat(a.wrapping_rem(*b))),
            OpBiner::SamaDengan => Some(Konstanta::Bool(a == b)),
            OpBiner::TidakSama => Some(Konstanta::Bool(a != b)),
            OpBiner::KurangDari => Some(Konstanta::Bool(a < b)),
            OpBiner::KurangSama => Some(Konstanta::Bool(a <= b)),
            OpBiner::LebihDari => Some(Konstanta::Bool(a > b)),
            OpBiner::LebihSama => Some(Konstanta::Bool(a >= b)),
            OpBiner::DanBit => Some(Konstanta::Bulat(a & b)),
            OpBiner::AtauBit => Some(Konstanta::Bulat(a | b)),
            OpBiner::XorBit => Some(Konstanta::Bulat(a ^ b)),
            OpBiner::GeserKiri => Some(Konstanta::Bulat(a.wrapping_shl(*b as u32))),
            OpBiner::GeserKanan => Some(Konstanta::Bulat(a.wrapping_shr(*b as u32))),
            _ => None,
        },

        (Konstanta::Desimal(a), Konstanta::Desimal(b)) => match op {
            OpBiner::Tambah => Some(Konstanta::Desimal(a + b)),
            OpBiner::Kurang => Some(Konstanta::Desimal(a - b)),
            OpBiner::Kali => Some(Konstanta::Desimal(a * b)),
            OpBiner::Bagi if *b != 0.0 => Some(Konstanta::Desimal(a / b)),
            OpBiner::SamaDengan => Some(Konstanta::Bool((a - b).abs() < f64::EPSILON)),
            OpBiner::TidakSama => Some(Konstanta::Bool((a - b).abs() >= f64::EPSILON)),
            OpBiner::KurangDari => Some(Konstanta::Bool(a < b)),
            OpBiner::KurangSama => Some(Konstanta::Bool(a <= b)),
            OpBiner::LebihDari => Some(Konstanta::Bool(a > b)),
            OpBiner::LebihSama => Some(Konstanta::Bool(a >= b)),
            _ => None,
        },

        (Konstanta::Bool(a), Konstanta::Bool(b)) => match op {
            OpBiner::SamaDengan => Some(Konstanta::Bool(a == b)),
            OpBiner::TidakSama => Some(Konstanta::Bool(a != b)),
            OpBiner::DanBit => Some(Konstanta::Bool(*a && *b)),
            OpBiner::AtauBit => Some(Konstanta::Bool(*a || *b)),
            OpBiner::XorBit => Some(Konstanta::Bool(*a ^ *b)),
            _ => None,
        },

        _ => None,
    }
}

fn lipat_unari(op: OpUnari, val: &Konstanta) -> Option<Konstanta> {
    match val {
        Konstanta::Bulat(n) => match op {
            OpUnari::Negasi => Some(Konstanta::Bulat(n.wrapping_neg())),
            OpUnari::BukanBit => Some(Konstanta::Bulat(!n)),
            _ => None,
        },
        Konstanta::Desimal(d) => match op {
            OpUnari::Negasi => Some(Konstanta::Desimal(-d)),
            _ => None,
        },
        Konstanta::Bool(b) => match op {
            OpUnari::Bukan => Some(Konstanta::Bool(!b)),
            _ => None,
        },
        _ => None,
    }
}
