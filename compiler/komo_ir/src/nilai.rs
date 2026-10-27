// Copyright (c) kangpcode (Dhafa Nazula Permadi). Licensed under MIT or Apache-2.0.

//! Identifikator nilai SSA, blok dasar, dan konstanta literal pada KIR.

use std::fmt;
use crate::tipe::IrTipe;

/// Identifikator unik untuk nilai register SSA (misalnya `%0`, `%1`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct IdNilai(pub usize);

impl fmt::Display for IdNilai {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "%{}", self.0)
    }
}

/// Identifikator unik untuk blok dasar (misalnya `bb0`, `bb1`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct IdBlok(pub usize);

impl fmt::Display for IdBlok {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "bb{}", self.0)
    }
}

/// Nilai konstanta yang diketahui pada saat kompilasi.
#[derive(Debug, Clone, PartialEq)]
pub enum Konstanta {
    /// Nilai integer 64-bit bertanda.
    Bulat(i64),
    /// Nilai pecahan presisi ganda 64-bit.
    Desimal(f64),
    /// Nilai kebenaran boolean (`benar` / `salah`).
    Bool(bool),
    /// Nilai karakter UTF-32.
    Karakter(char),
    /// Nilai byte 8-bit.
    Bita(u8),
    /// Nilai string teks literal.
    Teks(String),
    /// Nilai penunjuk nol (null pointer).
    Nol,
    /// Nilai kosong / void.
    Kosong,
}

impl Konstanta {
    /// Mengembalikan tipe IR dari konstanta ini.
    pub fn tipe(&self) -> IrTipe {
        match self {
            Konstanta::Bulat(_) => IrTipe::I64,
            Konstanta::Desimal(_) => IrTipe::F64,
            Konstanta::Bool(_) => IrTipe::I1,
            Konstanta::Karakter(_) => IrTipe::Char,
            Konstanta::Bita(_) => IrTipe::I8,
            Konstanta::Teks(_) => IrTipe::Ptr,
            Konstanta::Nol => IrTipe::Ptr,
            Konstanta::Kosong => IrTipe::Void,
        }
    }
}

impl fmt::Display for Konstanta {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Konstanta::Bulat(n) => write!(f, "{}", n),
            Konstanta::Desimal(d) => {
                if d.fract() == 0.0 {
                    write!(f, "{:.1}", d)
                } else {
                    write!(f, "{}", d)
                }
            }
            Konstanta::Bool(b) => write!(f, "{}", if *b { "benar" } else { "salah" }),
            Konstanta::Karakter(c) => write!(f, "'{}'", c),
            Konstanta::Bita(b) => write!(f, "0x{:02x}", b),
            Konstanta::Teks(s) => write!(f, "\"{}\"", s.replace('"', "\\\"")),
            Konstanta::Nol => write!(f, "null"),
            Konstanta::Kosong => write!(f, "kosong"),
        }
    }
}
