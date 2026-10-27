// Copyright (c) kangpcode (Dhafa Nazula Permadi). Licensed under MIT or Apache-2.0.

//! Representasi tipe data konkret untuk Komo Intermediate Representation (KIR).
//! Pada tingkat IR, semua tipe bersifat konkret dan terukur ukurannya.

use komo_sema::SemTipe;
use std::fmt;

/// Tipe data konkret pada KIR.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum IrTipe {
    /// Tipe boolean 1-bit (`bool`).
    I1,
    /// Integer 8-bit bertanda / byte (`byte`).
    I8,
    /// Integer 16-bit bertanda.
    I16,
    /// Integer 32-bit bertanda.
    I32,
    /// Integer 64-bit bertanda (`bulat`).
    I64,
    /// Karakter UTF-32 (32-bit integer / `rune`).
    Char,
    /// Bilangan pecahan presisi tunggal 32-bit.
    F32,
    /// Bilangan pecahan presisi ganda 64-bit (`desimal`).
    F64,
    /// Penunjuk memori (pointer), ukuran 64-bit pada arsitektur 64-bit.
    Ptr,
    /// Tipe komposit struktur/rekaman dengan bidang-bidang terurut.
    Struktur(Vec<IrTipe>),
    /// Tipe larik berukuran tetap.
    Larik(Box<IrTipe>, usize),
    /// Tipe penunjuk fungsi.
    Fungsi {
        parameter: Vec<IrTipe>,
        kembali: Box<IrTipe>,
    },
    /// Tidak mengembalikan nilai / unit.
    Void,
}

impl IrTipe {
    /// Menghitung perkiraan ukuran tipe dalam bita pada target 64-bit.
    pub fn ukuran_bita(&self) -> usize {
        match self {
            IrTipe::I1 | IrTipe::I8 => 1,
            IrTipe::I16 => 2,
            IrTipe::I32 | IrTipe::Char | IrTipe::F32 => 4,
            IrTipe::I64 | IrTipe::F64 | IrTipe::Ptr => 8,
            IrTipe::Struktur(bidang) => bidang.iter().map(|b| b.ukuran_bita()).sum(),
            IrTipe::Larik(elemen, pjg) => elemen.ukuran_bita() * pjg,
            IrTipe::Fungsi { .. } => 8,
            IrTipe::Void => 0,
        }
    }

    /// Mengonversi tipe semantik (`SemTipe`) ke tipe konkret IR (`IrTipe`).
    pub fn dari_sem_tipe(sem: &SemTipe) -> Self {
        match sem {
            SemTipe::Bulat | SemTipe::Bulat64 | SemTipe::TBulat | SemTipe::TBulat64 => IrTipe::I64,
            SemTipe::Bulat32 | SemTipe::TBulat32 => IrTipe::I32,
            SemTipe::Bulat16 | SemTipe::TBulat16 => IrTipe::I16,
            SemTipe::Bulat8 | SemTipe::TBulat8 | SemTipe::Byte => IrTipe::I8,
            SemTipe::Desimal | SemTipe::Desimal64 => IrTipe::F64,
            SemTipe::Desimal32 => IrTipe::F32,
            SemTipe::Teks => IrTipe::Ptr,
            SemTipe::Bool => IrTipe::I1,
            SemTipe::Rune => IrTipe::Char,
            SemTipe::Kosong => IrTipe::Void,
            SemTipe::Pinjam { .. } => IrTipe::Ptr,
            SemTipe::Larik(elemen) => {
                let _ = elemen;
                // Representasi slice/larik dinamis: penunjuk data, panjang, kapasitas
                IrTipe::Struktur(vec![IrTipe::Ptr, IrTipe::I64, IrTipe::I64])
            }
            SemTipe::Peta { .. } => IrTipe::Ptr,
            SemTipe::Saluran(_) => IrTipe::Ptr,
            SemTipe::Mungkin(dalam) => {
                // Tag diskriminan (i8) + payload
                let dalam_ir = Self::dari_sem_tipe(dalam);
                IrTipe::Struktur(vec![IrTipe::I8, dalam_ir])
            }
            SemTipe::Hasil { sukses, gagal } => {
                let ok_ir = Self::dari_sem_tipe(sukses);
                let err_ir = Self::dari_sem_tipe(gagal);
                let maks_ukuran = ok_ir.ukuran_bita().max(err_ir.ukuran_bita());
                IrTipe::Struktur(vec![
                    IrTipe::I8,
                    IrTipe::Larik(Box::new(IrTipe::I8), maks_ukuran),
                ])
            }
            SemTipe::Struktur { bidang, .. } => {
                let mut bidang_ir = Vec::new();
                for (_, t, _) in bidang {
                    bidang_ir.push(Self::dari_sem_tipe(t));
                }
                IrTipe::Struktur(bidang_ir)
            }
            SemTipe::Enum { varian, .. } => {
                let mut maks_payload = 0;
                for (_, tipe_isi) in varian {
                    let uk: usize = tipe_isi
                        .iter()
                        .map(|t| Self::dari_sem_tipe(t).ukuran_bita())
                        .sum();
                    if uk > maks_payload {
                        maks_payload = uk;
                    }
                }
                IrTipe::Struktur(vec![
                    IrTipe::I32,
                    IrTipe::Larik(Box::new(IrTipe::I8), maks_payload),
                ])
            }
            SemTipe::Antarmuka { .. } => IrTipe::Struktur(vec![IrTipe::Ptr, IrTipe::Ptr]),
            SemTipe::Fungsi {
                parameter,
                kembalian,
            } => IrTipe::Fungsi {
                parameter: parameter.iter().map(Self::dari_sem_tipe).collect(),
                kembali: Box::new(Self::dari_sem_tipe(kembalian)),
            },
            SemTipe::Generik { .. } | SemTipe::Galat => IrTipe::I64,
        }
    }
}

impl fmt::Display for IrTipe {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            IrTipe::I1 => write!(f, "i1"),
            IrTipe::I8 => write!(f, "i8"),
            IrTipe::I16 => write!(f, "i16"),
            IrTipe::I32 => write!(f, "i32"),
            IrTipe::I64 => write!(f, "i64"),
            IrTipe::Char => write!(f, "char"),
            IrTipe::F32 => write!(f, "f32"),
            IrTipe::F64 => write!(f, "f64"),
            IrTipe::Ptr => write!(f, "ptr"),
            IrTipe::Struktur(bidang) => {
                write!(f, "{{")?;
                for (i, b) in bidang.iter().enumerate() {
                    if i > 0 {
                        write!(f, ", ")?;
                    }
                    write!(f, "{}", b)?;
                }
                write!(f, "}}")
            }
            IrTipe::Larik(elemen, pjg) => write!(f, "[{} x {}]", pjg, elemen),
            IrTipe::Fungsi { parameter, kembali } => {
                write!(f, "fn(")?;
                for (i, p) in parameter.iter().enumerate() {
                    if i > 0 {
                        write!(f, ", ")?;
                    }
                    write!(f, "{}", p)?;
                }
                write!(f, ") -> {}", kembali)
            }
            IrTipe::Void => write!(f, "void"),
        }
    }
}
