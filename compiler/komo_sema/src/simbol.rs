// Copyright (c) kangpcode (Dhafa Nazula Permadi). Licensed under MIT or Apache-2.0.

//! Tabel simbol dan manajemen lingkup (lexical scoping) untuk Komolang.
//! Mengelola nama variabel, konstanta, fungsi, tipe data, dan metode.

use crate::tipe::SemTipe;
use komo_ast::{Rentang, Visibilitas};
use komo_diag::{kode, Diagnostik};
use std::collections::HashMap;

/// Klasifikasi jenis entri simbol dalam tabel simbol.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum JenisSimbol {
    Variabel {
        tipe: SemTipe,
        mutabel: bool,
        sudah_dipindah: bool,
        jumlah_pinjam: usize,
        pinjam_mutabel: bool,
    },
    Konstanta {
        tipe: SemTipe,
    },
    Fungsi {
        tipe: SemTipe,
        visibilitas: Visibilitas,
    },
    Metode {
        tipe_penerima: SemTipe,
        tipe: SemTipe,
        visibilitas: Visibilitas,
    },
    Tipe {
        tipe: SemTipe,
        visibilitas: Visibilitas,
    },
    Modul {
        nama: String,
    },
}

/// Satu entri simbol dengan nama, informasi jenis, visibilitas, dan lokasi rentang definisinya.
#[derive(Debug, Clone)]
pub struct Simbol {
    pub nama: String,
    pub jenis: JenisSimbol,
    pub rentang: Rentang,
}

impl Simbol {
    pub fn tipe(&self) -> SemTipe {
        match &self.jenis {
            JenisSimbol::Variabel { tipe, .. } => tipe.clone(),
            JenisSimbol::Konstanta { tipe } => tipe.clone(),
            JenisSimbol::Fungsi { tipe, .. } => tipe.clone(),
            JenisSimbol::Metode { tipe, .. } => tipe.clone(),
            JenisSimbol::Tipe { tipe, .. } => tipe.clone(),
            JenisSimbol::Modul { .. } => SemTipe::Kosong,
        }
    }
}

/// Satu lapisan lingkup leksikal (scope).
#[derive(Debug, Clone)]
pub struct Lingkup {
    pub simbol: HashMap<String, Simbol>,
    pub adalah_fungsi: bool,
    pub tipe_kembalian_fungsi: Option<SemTipe>,
}

impl Lingkup {
    pub fn baru() -> Self {
        Self {
            simbol: HashMap::new(),
            adalah_fungsi: false,
            tipe_kembalian_fungsi: None,
        }
    }

    pub fn fungsi(kembalian: SemTipe) -> Self {
        Self {
            simbol: HashMap::new(),
            adalah_fungsi: true,
            tipe_kembalian_fungsi: Some(kembalian),
        }
    }
}

/// Tabel simbol berbasis tumpukan (stack) untuk mendukung scoping leksikal.
#[derive(Debug, Clone)]
pub struct TabelSimbol {
    lingkup: Vec<Lingkup>,
}

impl TabelSimbol {
    pub fn baru() -> Self {
        let mut t = Self {
            lingkup: vec![Lingkup::baru()],
        };
        t.daftarkan_bawaan();
        t
    }

    /// Mendaftarkan tipe data dan fungsi bawaan bahasa Komolang.
    fn daftarkan_bawaan(&mut self) {
        let r = Rentang::titik_kosong();
        let tipe_dasar = vec![
            ("bulat", SemTipe::Bulat),
            ("bulat8", SemTipe::Bulat8),
            ("bulat16", SemTipe::Bulat16),
            ("bulat32", SemTipe::Bulat32),
            ("bulat64", SemTipe::Bulat64),
            ("tbulat", SemTipe::TBulat),
            ("tbulat8", SemTipe::TBulat8),
            ("tbulat16", SemTipe::TBulat16),
            ("tbulat32", SemTipe::TBulat32),
            ("tbulat64", SemTipe::TBulat64),
            ("desimal", SemTipe::Desimal),
            ("desimal32", SemTipe::Desimal32),
            ("desimal64", SemTipe::Desimal64),
            ("teks", SemTipe::Teks),
            ("bool", SemTipe::Bool),
            ("byte", SemTipe::Byte),
            ("rune", SemTipe::Rune),
            ("kosong", SemTipe::Kosong),
        ];

        for (nama, tipe) in tipe_dasar {
            let _ = self.tambah_simbol(Simbol {
                nama: nama.to_string(),
                jenis: JenisSimbol::Tipe {
                    tipe,
                    visibilitas: Visibilitas::Publik,
                },
                rentang: r,
            });
        }

        // Modul bawaan: "std/cetak" (alias 'cetak')
        let _ = self.tambah_simbol(Simbol {
            nama: "cetak".to_string(),
            jenis: JenisSimbol::Modul {
                nama: "std/cetak".to_string(),
            },
            rentang: r,
        });

        // Modul bawaan: "std/web" (alias 'web')
        let _ = self.tambah_simbol(Simbol {
            nama: "web".to_string(),
            jenis: JenisSimbol::Modul {
                nama: "std/web".to_string(),
            },
            rentang: r,
        });
    }

    /// Masuk ke lingkup baru (misal blok `{ ... }`).
    pub fn masuk_lingkup(&mut self) {
        self.lingkup.push(Lingkup::baru());
    }

    /// Masuk ke lingkup badan fungsi baru.
    pub fn masuk_lingkup_fungsi(&mut self, kembalian: SemTipe) {
        self.lingkup.push(Lingkup::fungsi(kembalian));
    }

    /// Keluar dari lingkup terluar.
    pub fn keluar_lingkup(&mut self) {
        if self.lingkup.len() > 1 {
            self.lingkup.pop();
        }
    }

    /// Menambahkan simbol ke lingkup saat ini. Melaporkan galat jika ada nama duplikat pada lingkup yang sama.
    pub fn tambah_simbol(&mut self, simbol: Simbol) -> Result<(), Diagnostik> {
        let sekarang = self.lingkup.last_mut().expect("lingkup tidak boleh kosong");
        if let Some(lama) = sekarang.simbol.get(&simbol.nama) {
            return Err(Diagnostik::galat(
                kode::SIMBOL_DUPLIKAT,
                format!(
                    "simbol '{}' sudah dideklarasikan sebelumnya dalam lingkup ini",
                    simbol.nama
                ),
                simbol.rentang,
            )
            .dengan_saran(format!(
                "sebelumnya dideklarasikan pada baris {}, kolom {}; gunakan nama lain",
                lama.rentang.baris, lama.rentang.kolom
            )));
        }
        sekarang.simbol.insert(simbol.nama.clone(), simbol);
        Ok(())
    }

    /// Menemukan simbol dengan mencarinya dari lingkup terdalam hingga lingkup terluar.
    pub fn cari_simbol(&self, nama: &str) -> Option<&Simbol> {
        for l in self.lingkup.iter().rev() {
            if let Some(s) = l.simbol.get(nama) {
                return Some(s);
            }
        }
        None
    }

    /// Menemukan simbol secara mutabel untuk pembaruan status mutasi/ownership.
    pub fn cari_simbol_mut(&mut self, nama: &str) -> Option<&mut Simbol> {
        for l in self.lingkup.iter_mut().rev() {
            if let Some(s) = l.simbol.get_mut(nama) {
                return Some(s);
            }
        }
        None
    }

    /// Mencari definisi tipe data berdasarkan nama.
    pub fn cari_tipe(&self, nama: &str) -> Option<SemTipe> {
        if let Some(s) = self.cari_simbol(nama) {
            if let JenisSimbol::Tipe { tipe, .. } = &s.jenis {
                return Some(tipe.clone());
            }
        }
        None
    }

    /// Menemukan tipe kembalian fungsi aktif saat ini.
    pub fn tipe_kembalian_fungsi_aktif(&self) -> Option<SemTipe> {
        for l in self.lingkup.iter().rev() {
            if l.adalah_fungsi {
                return l.tipe_kembalian_fungsi.clone();
            }
        }
        None
    }

    /// Apakah saat ini berada di dalam sebuah fungsi?
    pub fn dalam_fungsi(&self) -> bool {
        self.lingkup.iter().any(|l| l.adalah_fungsi)
    }
}
