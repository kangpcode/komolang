// Copyright (c) kangpcode (Dhafa Nazula Permadi). Licensed under MIT or Apache-2.0.

//! Representasi sistem tipe data internal Komolang untuk analisis semantik.
//! Mendukung tipe primitif, tipe komposit bawaan (larik, peta, mungkin, hasil, saluran),
//! tipe buatan (struktur, enum aljabar, antarmuka), tipe fungsi, dan generic.

use komo_ast::{JenisTipe, Tipe, Visibilitas};
use std::fmt;

/// Tipe data semantik Komolang yang sudah teresolusi.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SemTipe {
    // Tipe Primitif
    Bulat,
    Bulat8,
    Bulat16,
    Bulat32,
    Bulat64,
    TBulat,
    TBulat8,
    TBulat16,
    TBulat32,
    TBulat64,
    Desimal,
    Desimal32,
    Desimal64,
    Teks,
    Bool,
    Byte,
    Rune,
    Kosong,

    // Tipe Koleksi & Konkurensi Bawaan
    Larik(Box<SemTipe>),
    Peta {
        kunci: Box<SemTipe>,
        nilai: Box<SemTipe>,
    },
    Mungkin(Box<SemTipe>),
    Hasil {
        sukses: Box<SemTipe>,
        gagal: Box<SemTipe>,
    },
    Saluran(Box<SemTipe>),

    // Tipe Bentukan Pengguna (Nominal)
    Struktur {
        nama: String,
        generik: Vec<String>,
        bidang: Vec<(String, SemTipe, Visibilitas)>,
    },
    Enum {
        nama: String,
        generik: Vec<String>,
        varian: Vec<(String, Vec<SemTipe>)>,
    },
    Antarmuka {
        nama: String,
        generik: Vec<String>,
        metode: Vec<(String, Vec<SemTipe>, SemTipe)>,
    },

    // Peminjaman
    Pinjam {
        mutabel: bool,
        dalam: Box<SemTipe>,
    },

    // Tipe Fungsi: parameter -> kembalian
    Fungsi {
        parameter: Vec<SemTipe>,
        kembalian: Box<SemTipe>,
    },

    // Parameter Tipe Generik (seperti T, K, V)
    Generik(String),

    // Galat Semantik (penampung saat ada galat tipe agar tidak memicu galat beruntun)
    Galat,
}

impl SemTipe {
    /// Nama ramah Bahasa Indonesia untuk pesan diagnostik.
    pub fn nama(&self) -> String {
        match self {
            SemTipe::Bulat => "bulat".to_string(),
            SemTipe::Bulat8 => "bulat8".to_string(),
            SemTipe::Bulat16 => "bulat16".to_string(),
            SemTipe::Bulat32 => "bulat32".to_string(),
            SemTipe::Bulat64 => "bulat64".to_string(),
            SemTipe::TBulat => "tbulat".to_string(),
            SemTipe::TBulat8 => "tbulat8".to_string(),
            SemTipe::TBulat16 => "tbulat16".to_string(),
            SemTipe::TBulat32 => "tbulat32".to_string(),
            SemTipe::TBulat64 => "tbulat64".to_string(),
            SemTipe::Desimal => "desimal".to_string(),
            SemTipe::Desimal32 => "desimal32".to_string(),
            SemTipe::Desimal64 => "desimal64".to_string(),
            SemTipe::Teks => "teks".to_string(),
            SemTipe::Bool => "bool".to_string(),
            SemTipe::Byte => "byte".to_string(),
            SemTipe::Rune => "rune".to_string(),
            SemTipe::Kosong => "kosong".to_string(),
            SemTipe::Larik(item) => format!("larik<{}>", item.nama()),
            SemTipe::Peta { kunci, nilai } => format!("peta<{}, {}>", kunci.nama(), nilai.nama()),
            SemTipe::Mungkin(dalam) => format!("mungkin<{}>", dalam.nama()),
            SemTipe::Hasil { sukses, gagal } => {
                format!("hasil<{}, {}>", sukses.nama(), gagal.nama())
            }
            SemTipe::Saluran(dalam) => format!("saluran<{}>", dalam.nama()),
            SemTipe::Struktur { nama, generik, .. } => {
                if generik.is_empty() {
                    nama.clone()
                } else {
                    format!("{}<{}>", nama, generik.join(", "))
                }
            }
            SemTipe::Enum { nama, generik, .. } => {
                if generik.is_empty() {
                    nama.clone()
                } else {
                    format!("{}<{}>", nama, generik.join(", "))
                }
            }
            SemTipe::Antarmuka { nama, generik, .. } => {
                if generik.is_empty() {
                    nama.clone()
                } else {
                    format!("{}<{}>", nama, generik.join(", "))
                }
            }
            SemTipe::Pinjam { mutabel, dalam } => {
                format!(
                    "pinjam {}{}",
                    if *mutabel { "var " } else { "" },
                    dalam.nama()
                )
            }
            SemTipe::Fungsi {
                parameter,
                kembalian,
            } => {
                let params = parameter
                    .iter()
                    .map(|p| p.nama())
                    .collect::<Vec<_>>()
                    .join(", ");
                format!("fungsi({}) {}", params, kembalian.nama())
            }
            SemTipe::Generik(g) => g.clone(),
            SemTipe::Galat => "<galat-tipe>".to_string(),
        }
    }

    /// Apakah tipe ini kompatibel / dapat diberikan ke tipe target?
    pub fn cocok_dengan(&self, target: &SemTipe) -> bool {
        if *self == SemTipe::Galat || *target == SemTipe::Galat {
            return true;
        }

        if self == target {
            return true;
        }

        // Peminjaman: `pinjam var T` dapat diberikan ke `pinjam T` (koersi mutabel ke imutabel)
        if let (
            SemTipe::Pinjam {
                mutabel: true,
                dalam: d1,
            },
            SemTipe::Pinjam {
                mutabel: false,
                dalam: d2,
            },
        ) = (self, target)
        {
            return d1.cocok_dengan(d2);
        }

        // Generik: parameter generik cocok dengan tipe konkret saat inferensi / substitusi
        if matches!(self, SemTipe::Generik(_)) || matches!(target, SemTipe::Generik(_)) {
            return true;
        }

        false
    }

    /// Apakah tipe ini berupa bilangan bulat bertanda atau tak bertanda?
    pub fn adalah_bulat(&self) -> bool {
        matches!(
            self,
            SemTipe::Bulat
                | SemTipe::Bulat8
                | SemTipe::Bulat16
                | SemTipe::Bulat32
                | SemTipe::Bulat64
                | SemTipe::TBulat
                | SemTipe::TBulat8
                | SemTipe::TBulat16
                | SemTipe::TBulat32
                | SemTipe::TBulat64
                | SemTipe::Byte
        )
    }

    /// Apakah tipe ini berupa desimal pecahan?
    pub fn adalah_desimal(&self) -> bool {
        matches!(
            self,
            SemTipe::Desimal | SemTipe::Desimal32 | SemTipe::Desimal64
        )
    }

    /// Apakah tipe ini berupa angka numerik?
    pub fn adalah_angka(&self) -> bool {
        self.adalah_bulat() || self.adalah_desimal()
    }

    /// Apakah tipe ini memiliki semantik Copy sederhana (tidak perlu invalidasi move)?
    pub fn adalah_copy(&self) -> bool {
        self.adalah_angka()
            || matches!(
                self,
                SemTipe::Bool | SemTipe::Rune | SemTipe::Byte | SemTipe::Kosong
            )
    }

    /// Substitusi parameter generik dengan tipe argumen konkret.
    pub fn substitusi(&self, nama_generik: &str, tipe_konkret: &SemTipe) -> SemTipe {
        match self {
            SemTipe::Generik(g) if g == nama_generik => tipe_konkret.clone(),
            SemTipe::Larik(item) => {
                SemTipe::Larik(Box::new(item.substitusi(nama_generik, tipe_konkret)))
            }
            SemTipe::Peta { kunci, nilai } => SemTipe::Peta {
                kunci: Box::new(kunci.substitusi(nama_generik, tipe_konkret)),
                nilai: Box::new(nilai.substitusi(nama_generik, tipe_konkret)),
            },
            SemTipe::Mungkin(dalam) => {
                SemTipe::Mungkin(Box::new(dalam.substitusi(nama_generik, tipe_konkret)))
            }
            SemTipe::Hasil { sukses, gagal } => SemTipe::Hasil {
                sukses: Box::new(sukses.substitusi(nama_generik, tipe_konkret)),
                gagal: Box::new(gagal.substitusi(nama_generik, tipe_konkret)),
            },
            SemTipe::Saluran(dalam) => {
                SemTipe::Saluran(Box::new(dalam.substitusi(nama_generik, tipe_konkret)))
            }
            SemTipe::Pinjam { mutabel, dalam } => SemTipe::Pinjam {
                mutabel: *mutabel,
                dalam: Box::new(dalam.substitusi(nama_generik, tipe_konkret)),
            },
            SemTipe::Fungsi {
                parameter,
                kembalian,
            } => SemTipe::Fungsi {
                parameter: parameter
                    .iter()
                    .map(|p| p.substitusi(nama_generik, tipe_konkret))
                    .collect(),
                kembalian: Box::new(kembalian.substitusi(nama_generik, tipe_konkret)),
            },
            lain => lain.clone(),
        }
    }
}

impl fmt::Display for SemTipe {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.nama())
    }
}

/// Mengonversi tipe AST ke tipe semantik `SemTipe`.
pub fn tipe_dari_ast<F>(ast_tipe: &Tipe, resolusi: &F) -> SemTipe
where
    F: Fn(&str) -> Option<SemTipe>,
{
    match &ast_tipe.jenis {
        JenisTipe::Nama { nama, argumen } => {
            let n = nama.nama.as_str();
            match n {
                "bulat" => SemTipe::Bulat,
                "bulat8" => SemTipe::Bulat8,
                "bulat16" => SemTipe::Bulat16,
                "bulat32" => SemTipe::Bulat32,
                "bulat64" => SemTipe::Bulat64,
                "tbulat" => SemTipe::TBulat,
                "tbulat8" => SemTipe::TBulat8,
                "tbulat16" => SemTipe::TBulat16,
                "tbulat32" => SemTipe::TBulat32,
                "tbulat64" => SemTipe::TBulat64,
                "desimal" => SemTipe::Desimal,
                "desimal32" => SemTipe::Desimal32,
                "desimal64" => SemTipe::Desimal64,
                "teks" => SemTipe::Teks,
                "bool" => SemTipe::Bool,
                "byte" => SemTipe::Byte,
                "rune" => SemTipe::Rune,
                "kosong" => SemTipe::Kosong,
                "larik" => {
                    let item = if !argumen.is_empty() {
                        tipe_dari_ast(&argumen[0], resolusi)
                    } else {
                        SemTipe::Galat
                    };
                    SemTipe::Larik(Box::new(item))
                }
                "peta" => {
                    let (k, v) = if argumen.len() >= 2 {
                        (
                            tipe_dari_ast(&argumen[0], resolusi),
                            tipe_dari_ast(&argumen[1], resolusi),
                        )
                    } else {
                        (SemTipe::Galat, SemTipe::Galat)
                    };
                    SemTipe::Peta {
                        kunci: Box::new(k),
                        nilai: Box::new(v),
                    }
                }
                "mungkin" => {
                    let dalam = if !argumen.is_empty() {
                        tipe_dari_ast(&argumen[0], resolusi)
                    } else {
                        SemTipe::Galat
                    };
                    SemTipe::Mungkin(Box::new(dalam))
                }
                "hasil" => {
                    let (s, g) = if argumen.len() >= 2 {
                        (
                            tipe_dari_ast(&argumen[0], resolusi),
                            tipe_dari_ast(&argumen[1], resolusi),
                        )
                    } else {
                        (SemTipe::Galat, SemTipe::Galat)
                    };
                    SemTipe::Hasil {
                        sukses: Box::new(s),
                        gagal: Box::new(g),
                    }
                }
                "saluran" => {
                    let dalam = if !argumen.is_empty() {
                        tipe_dari_ast(&argumen[0], resolusi)
                    } else {
                        SemTipe::Galat
                    };
                    SemTipe::Saluran(Box::new(dalam))
                }
                kustom => {
                    if let Some(t) = resolusi(kustom) {
                        t
                    } else {
                        SemTipe::Generik(kustom.to_string())
                    }
                }
            }
        }
        JenisTipe::Pinjam { mutabel, dalam } => SemTipe::Pinjam {
            mutabel: *mutabel,
            dalam: Box::new(tipe_dari_ast(dalam, resolusi)),
        },
    }
}
