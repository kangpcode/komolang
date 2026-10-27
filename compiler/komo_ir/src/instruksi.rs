// Copyright (c) kangpcode (Dhafa Nazula Permadi). Licensed under MIT or Apache-2.0.

//! Instruksi dan Terminator untuk Komo Intermediate Representation (KIR).
//! Setiap instruksi beroperasi pada nilai SSA, dan setiap blok diakhiri oleh satu terminator.

use crate::nilai::{IdBlok, IdNilai, Konstanta};
use crate::tipe::IrTipe;
use std::fmt;

/// Operator biner aritmatika, perbandingan, dan logika pada tingkat IR.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum OpBiner {
    Tambah,
    Kurang,
    Kali,
    Bagi,
    Sisa,
    SamaDengan,
    TidakSama,
    KurangDari,
    KurangSama,
    LebihDari,
    LebihSama,
    DanBit,
    AtauBit,
    XorBit,
    GeserKiri,
    GeserKanan,
}

impl fmt::Display for OpBiner {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            OpBiner::Tambah => write!(f, "tambah"),
            OpBiner::Kurang => write!(f, "kurang"),
            OpBiner::Kali => write!(f, "kali"),
            OpBiner::Bagi => write!(f, "bagi"),
            OpBiner::Sisa => write!(f, "sisa"),
            OpBiner::SamaDengan => write!(f, "sama"),
            OpBiner::TidakSama => write!(f, "beda"),
            OpBiner::KurangDari => write!(f, "kurang_dari"),
            OpBiner::KurangSama => write!(f, "kurang_sama"),
            OpBiner::LebihDari => write!(f, "lebih_dari"),
            OpBiner::LebihSama => write!(f, "lebih_sama"),
            OpBiner::DanBit => write!(f, "dan_bit"),
            OpBiner::AtauBit => write!(f, "atau_bit"),
            OpBiner::XorBit => write!(f, "xor_bit"),
            OpBiner::GeserKiri => write!(f, "shl"),
            OpBiner::GeserKanan => write!(f, "shr"),
        }
    }
}

/// Operator unari pada tingkat IR.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum OpUnari {
    Negasi,
    Bukan,
    BukanBit,
}

impl fmt::Display for OpUnari {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            OpUnari::Negasi => write!(f, "neg"),
            OpUnari::Bukan => write!(f, "bukan"),
            OpUnari::BukanBit => write!(f, "bukan_bit"),
        }
    }
}

/// Instruksi tunggal SSA dalam blok dasar KIR.
#[derive(Debug, Clone, PartialEq)]
pub enum Instruksi {
    /// Menghasilkan konstanta literal: `%val = konst <nilai>`
    Konst(Konstanta),
    /// Operasi biner: `%val = biner.<op> <kiri>, <kanan>`
    Biner {
        op: OpBiner,
        kiri: IdNilai,
        kanan: IdNilai,
        tipe: IrTipe,
    },
    /// Operasi unari: `%val = unari.<op> <operan>`
    Unari {
        op: OpUnari,
        operan: IdNilai,
        tipe: IrTipe,
    },
    /// Alokasi memori stack lokal: `%ptr = alloca <tipe>`
    Alloca(IrTipe),
    /// Memuat nilai dari penunjuk memori: `%val = muat <tipe>, <penunjuk>`
    Muat { tipe: IrTipe, penunjuk: IdNilai },
    /// Menyimpan nilai ke penunjuk memori: `simpan <nilai>, <penunjuk>`
    Simpan { nilai: IdNilai, penunjuk: IdNilai },
    /// Menghitung alamat bidang ke-N pada struktur: `%ptr = akses_bidang <struktur>, <indeks>`
    AksesBidang { struktur: IdNilai, indeks: usize },
    /// Menghitung alamat elemen larik berdasarkan indeks: `%ptr = akses_indeks <larik>, <indeks>`
    AksesIndeks { target: IdNilai, indeks: IdNilai },
    /// Pemanggilan fungsi: `%val = panggil @nama(<argumen>)`
    Panggil {
        fungsi: String,
        argumen: Vec<IdNilai>,
        tipe_kembali: IrTipe,
    },
    /// Node Phi SSA untuk memilih nilai berdasarkan blok asal pendahulu:
    /// `%val = phi <tipe> [val1, bb1], [val2, bb2]`
    Phi {
        tipe: IrTipe,
        masukan: Vec<(IdNilai, IdBlok)>,
    },
    /// Konversi tipe data konkret (misalnya i64 ke f64): `%val = cast <asal> ke <tipe>`
    Cast { asal: IdNilai, tipe_tujuan: IrTipe },
}

impl Instruksi {
    /// Mengembalikan tipe hasil instruksi jika instruksi ini menghasilkan nilai register SSA.
    pub fn tipe_hasil(&self) -> Option<IrTipe> {
        match self {
            Instruksi::Konst(k) => Some(k.tipe()),
            Instruksi::Biner { tipe, op, .. } => match op {
                OpBiner::SamaDengan
                | OpBiner::TidakSama
                | OpBiner::KurangDari
                | OpBiner::KurangSama
                | OpBiner::LebihDari
                | OpBiner::LebihSama => Some(IrTipe::I1),
                _ => Some(tipe.clone()),
            },
            Instruksi::Unari { tipe, op, .. } => match op {
                OpUnari::Bukan => Some(IrTipe::I1),
                _ => Some(tipe.clone()),
            },
            Instruksi::Alloca(_) => Some(IrTipe::Ptr),
            Instruksi::Muat { tipe, .. } => Some(tipe.clone()),
            Instruksi::Simpan { .. } => None,
            Instruksi::AksesBidang { .. } => Some(IrTipe::Ptr),
            Instruksi::AksesIndeks { .. } => Some(IrTipe::Ptr),
            Instruksi::Panggil { tipe_kembali, .. } => {
                if *tipe_kembali == IrTipe::Void {
                    None
                } else {
                    Some(tipe_kembali.clone())
                }
            }
            Instruksi::Phi { tipe, .. } => Some(tipe.clone()),
            Instruksi::Cast { tipe_tujuan, .. } => Some(tipe_tujuan.clone()),
        }
    }

    /// Mengumpulkan seluruh ID nilai SSA yang dibaca/digunakan oleh instruksi ini.
    pub fn ambil_penggunaan(&self) -> Vec<IdNilai> {
        match self {
            Instruksi::Konst(_) | Instruksi::Alloca(_) => Vec::new(),
            Instruksi::Biner { kiri, kanan, .. } => vec![*kiri, *kanan],
            Instruksi::Unari { operan, .. } => vec![*operan],
            Instruksi::Muat { penunjuk, .. } => vec![*penunjuk],
            Instruksi::Simpan { nilai, penunjuk } => vec![*nilai, *penunjuk],
            Instruksi::AksesBidang { struktur, .. } => vec![*struktur],
            Instruksi::AksesIndeks { target, indeks } => vec![*target, *indeks],
            Instruksi::Panggil { argumen, .. } => argumen.clone(),
            Instruksi::Phi { masukan, .. } => masukan.iter().map(|(val, _)| *val).collect(),
            Instruksi::Cast { asal, .. } => vec![*asal],
        }
    }

    /// Apakah instruksi ini memiliki efek samping (misalnya simpan memori atau panggil fungsi)?
    pub fn punya_efek_samping(&self) -> bool {
        matches!(self, Instruksi::Simpan { .. } | Instruksi::Panggil { .. })
    }
}

/// Instruksi pengakhir (Terminator) yang wajib ada di akhir setiap blok dasar SSA.
#[derive(Debug, Clone, PartialEq)]
pub enum Terminator {
    /// Lompat tanpa syarat ke blok target: `lompat <blok>`
    Lompat(IdBlok),
    /// Percabangan kondisional: `cabang <kondisi>, <blok_benar>, <blok_salah>`
    Cabang {
        kondisi: IdNilai,
        jika_benar: IdBlok,
        jika_salah: IdBlok,
    },
    /// Kembali dari fungsi: `kembali <nilai>` atau `kembali`
    Kembali(Option<IdNilai>),
    /// Keadaan tak terjangkau atau jebakan panik: `tidak_bisa "<alasan>"`
    TidakBisa(String),
}

impl Terminator {
    /// Mengembalikan seluruh blok dasar tujuan berikutnya (successors) dari terminator ini.
    pub fn ambil_penerus(&self) -> Vec<IdBlok> {
        match self {
            Terminator::Lompat(target) => vec![*target],
            Terminator::Cabang {
                jika_benar,
                jika_salah,
                ..
            } => vec![*jika_benar, *jika_salah],
            Terminator::Kembali(_) | Terminator::TidakBisa(_) => Vec::new(),
        }
    }

    /// Mengumpulkan seluruh ID nilai SSA yang dibaca oleh terminator ini.
    pub fn ambil_penggunaan(&self) -> Vec<IdNilai> {
        match self {
            Terminator::Lompat(_) | Terminator::TidakBisa(_) => Vec::new(),
            Terminator::Cabang { kondisi, .. } => vec![*kondisi],
            Terminator::Kembali(Some(val)) => vec![*val],
            Terminator::Kembali(None) => Vec::new(),
        }
    }
}
