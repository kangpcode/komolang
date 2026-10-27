// Copyright (c) kangpcode (Dhafa Nazula Permadi). Licensed under MIT or Apache-2.0.

//! Algoritma Linear Scan Register Allocation untuk pemetaan nilai register SSA ke register mesin atau stack slot.
//! Mengimplementasikan pendekatan Poletto & Sarkar yang cepat dan deterministik.

use std::collections::{BTreeSet, HashMap};

use komo_ir::{FungsiIr, IdNilai};

/// Lokasi penyimpanan konkret sebuah nilai register SSA pada target arsitektur.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum LokasiNilai {
    /// Disimpan dalam register fisik mesin dengan indeks tertentu dalam pool register.
    Register(usize),
    /// Di-spill ke dalam tumpukan lokal (stack slot) dengan offset negatif dari frame pointer RBP/FP.
    Stack(i64),
}

/// Rentang hidup (liveness interval) untuk suatu nilai SSA dalam fungsi.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RentangHidup {
    pub nilai: IdNilai,
    pub mulai: usize,
    pub selesai: usize,
}

/// Hasil alokasi register untuk seluruh fungsi.
#[derive(Debug, Clone)]
pub struct HasilAlokasiRegister {
    pub peta_lokasi: HashMap<IdNilai, LokasiNilai>,
    pub ukuran_stack: usize,
}

/// Menghitung rentang hidup setiap `IdNilai` dalam urutan eksekusi linier fungsi.
pub fn hitung_rentang_hidup(fungsi: &FungsiIr) -> Vec<RentangHidup> {
    let mut batas_mulai: HashMap<IdNilai, usize> = HashMap::new();
    let mut batas_selesai: HashMap<IdNilai, usize> = HashMap::new();

    // Parameter fungsi mulai hidup pada titik 0
    for p in &fungsi.parameter {
        batas_mulai.insert(p.nilai, 0);
        batas_selesai.insert(p.nilai, 0);
    }

    let mut langkah = 1;

    for b in &fungsi.blok {
        for (hasil, inst) in &b.instruksi {
            if let Some(res) = hasil {
                batas_mulai.entry(*res).or_insert(langkah);
                batas_selesai.insert(*res, langkah);
            }
            for digunakan in inst.ambil_penggunaan() {
                batas_mulai.entry(digunakan).or_insert(0);
                batas_selesai.insert(digunakan, langkah);
            }
            langkah += 1;
        }

        if let Some(term) = &b.terminator {
            for digunakan in term.ambil_penggunaan() {
                batas_mulai.entry(digunakan).or_insert(0);
                batas_selesai.insert(digunakan, langkah);
            }
            langkah += 1;
        }
    }

    let mut hasil: Vec<RentangHidup> = batas_mulai
        .into_iter()
        .map(|(nilai, mulai)| {
            let selesai = batas_selesai.get(&nilai).copied().unwrap_or(mulai);
            RentangHidup {
                nilai,
                mulai,
                selesai: selesai.max(mulai),
            }
        })
        .collect();

    hasil.sort_by_key(|r| r.mulai);
    hasil
}

/// Menjalankan algoritma linear scan allocation dengan jumlah register fisik yang tersedia.
/// `cacah_register`: Jumlah register umum yang tersedia untuk alokasi variabel (misalnya 6 atau 12).
pub fn alokasikan_register_linear_scan(
    fungsi: &FungsiIr,
    cacah_register: usize,
) -> HasilAlokasiRegister {
    let mut rentang = hitung_rentang_hidup(fungsi);
    let mut peta_lokasi: HashMap<IdNilai, LokasiNilai> = HashMap::new();

    // Set register yang bebas dipakai
    let mut register_bebas: BTreeSet<usize> = (0..cacah_register).collect();

    // Daftar interval aktif: urut berdasarkan titik `selesai` (ascending)
    let mut aktif: Vec<(usize, usize, IdNilai)> = Vec::new(); // (selesai, reg_id, nilai)

    let mut offset_stack: i64 = 8;

    for interval in &mut rentang {
        // 1. Kadaluarsakan interval lama yang sudah selesai sebelum `interval.mulai`
        let mut i = 0;
        while i < aktif.len() {
            if aktif[i].0 < interval.mulai {
                let (_, reg, _) = aktif.remove(i);
                register_bebas.insert(reg);
            } else {
                i += 1;
            }
        }

        // 2. Alokasikan register jika tersedia
        if let Some(&reg) = register_bebas.iter().next() {
            register_bebas.remove(&reg);
            peta_lokasi.insert(interval.nilai, LokasiNilai::Register(reg));
            aktif.push((interval.selesai, reg, interval.nilai));
            aktif.sort_by_key(|a| a.0);
        } else {
            // 3. Jika tidak ada register bebas, lakukan spill
            if let Some(terakhir) = aktif.last() {
                if terakhir.0 > interval.selesai {
                    // Spill interval yang aktif dan berikan reg-nya ke interval sekarang
                    let (_, reg, nilai_lama) = aktif.pop().unwrap();
                    peta_lokasi.insert(nilai_lama, LokasiNilai::Stack(-offset_stack));
                    offset_stack += 8;

                    peta_lokasi.insert(interval.nilai, LokasiNilai::Register(reg));
                    aktif.push((interval.selesai, reg, interval.nilai));
                    aktif.sort_by_key(|a| a.0);
                } else {
                    // Spill interval sekarang langsung ke stack
                    peta_lokasi.insert(interval.nilai, LokasiNilai::Stack(-offset_stack));
                    offset_stack += 8;
                }
            } else {
                peta_lokasi.insert(interval.nilai, LokasiNilai::Stack(-offset_stack));
                offset_stack += 8;
            }
        }
    }

    // Alignment stack ke kelipatan 16 byte (standar System V ABI dan ARM64 AAPCS64)
    let mut ukuran_stack = offset_stack as usize;
    if ukuran_stack % 16 != 0 {
        ukuran_stack = (ukuran_stack + 15) & !15;
    }

    HasilAlokasiRegister {
        peta_lokasi,
        ukuran_stack,
    }
}
