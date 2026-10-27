// Copyright (c) kangpcode (Dhafa Nazula Permadi). Licensed under MIT or Apache-2.0.

//! `komo_ast` mendefinisikan pohon sintaks abstrak (AST) bahasa Komolang v0.1,
//! lengkap dengan lokasi sumber (`Rentang`) pada setiap simpul, serta pretty-printer
//! kanonik di modul [`cetak`].

pub mod cetak;

/// Prioritas (binding power) simpul ekspresi; dipakai parser dan pretty-printer.
pub const PRIORITAS_RENTANG: u8 = 4;
pub const PRIORITAS_UNARI: u8 = 7;
pub const PRIORITAS_POSTFIKS: u8 = 8;
pub const PRIORITAS_PRIMER: u8 = 9;

/// Lokasi dalam kode sumber: offset byte `[awal, akhir)` serta baris/kolom awal (basis 1,
/// kolom dihitung dalam karakter).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Rentang {
    pub awal: usize,
    pub akhir: usize,
    pub baris: usize,
    pub kolom: usize,
}

impl Rentang {
    pub fn baru(awal: usize, akhir: usize, baris: usize, kolom: usize) -> Self {
        Self {
            awal,
            akhir,
            baris,
            kolom,
        }
    }

    pub fn titik_kosong() -> Self {
        Self {
            awal: 0,
            akhir: 0,
            baris: 1,
            kolom: 1,
        }
    }

    /// Menggabungkan dua rentang: dimulai dari `self`, berakhir di akhir `lain`.
    pub fn gabung(self, lain: Rentang) -> Rentang {
        Rentang {
            awal: self.awal,
            akhir: lain.akhir.max(self.akhir),
            baris: self.baris,
            kolom: self.kolom,
        }
    }
}

/// Pengenal (nama) beserta lokasinya.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Pengenal {
    pub nama: String,
    pub rentang: Rentang,
}

impl Pengenal {
    pub fn baru(nama: impl Into<String>, rentang: Rentang) -> Self {
        Self {
            nama: nama.into(),
            rentang,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Visibilitas {
    Pribadi,
    Publik,
}

// ---------------------------------------------------------------------------
// Unit kompilasi dan deklarasi
// ---------------------------------------------------------------------------

/// Unit kompilasi lengkap (satu berkas `.komo`).
#[derive(Debug, Clone, PartialEq)]
pub struct BerkasProgram {
    pub paket: DeklarasiPaket,
    pub impor: Vec<DeklarasiImpor>,
    pub deklarasi: Vec<Deklarasi>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DeklarasiPaket {
    pub nama: Pengenal,
    pub rentang: Rentang,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DeklarasiImpor {
    pub jalur: String,
    pub alias: Option<Pengenal>,
    pub rentang: Rentang,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Deklarasi {
    Fungsi(DeklarasiFungsi),
    Tipe(DeklarasiTipe),
    Konstanta(DeklarasiKonstanta),
    Variabel(DeklarasiVariabel),
}

#[derive(Debug, Clone, PartialEq)]
pub struct DeklarasiFungsi {
    pub visibilitas: Visibilitas,
    pub penerima: Option<Parameter>,
    pub nama: Pengenal,
    pub generik: Vec<Pengenal>,
    pub parameter: Vec<Parameter>,
    pub tipe_kembalian: Option<Tipe>,
    pub badan: Blok,
    pub rentang: Rentang,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Parameter {
    pub nama: Pengenal,
    pub tipe: Tipe,
}

#[derive(Debug, Clone, PartialEq)]
pub struct DeklarasiTipe {
    pub visibilitas: Visibilitas,
    pub nama: Pengenal,
    pub generik: Vec<Pengenal>,
    pub isi: IsiTipe,
    pub rentang: Rentang,
}

#[derive(Debug, Clone, PartialEq)]
pub enum IsiTipe {
    Struktur(Vec<Bidang>),
    Antarmuka(Vec<TandaTanganMetode>),
    Enum(Vec<Varian>),
    Alias(Tipe),
}

#[derive(Debug, Clone, PartialEq)]
pub struct Bidang {
    pub visibilitas: Visibilitas,
    pub nama: Pengenal,
    pub tipe: Tipe,
    pub rentang: Rentang,
}

#[derive(Debug, Clone, PartialEq)]
pub struct TandaTanganMetode {
    pub nama: Pengenal,
    pub parameter: Vec<Parameter>,
    pub tipe_kembalian: Option<Tipe>,
    pub rentang: Rentang,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Varian {
    pub nama: Pengenal,
    pub tipe: Vec<Tipe>,
    pub rentang: Rentang,
}

#[derive(Debug, Clone, PartialEq)]
pub struct DeklarasiKonstanta {
    pub visibilitas: Visibilitas,
    pub nama: Pengenal,
    pub tipe: Option<Tipe>,
    pub nilai: Ekspresi,
    pub rentang: Rentang,
}

#[derive(Debug, Clone, PartialEq)]
pub struct DeklarasiVariabel {
    pub visibilitas: Visibilitas,
    pub mutabel: bool,
    pub nama: Pengenal,
    pub tipe: Option<Tipe>,
    pub nilai: Ekspresi,
    pub rentang: Rentang,
}

// ---------------------------------------------------------------------------
// Tipe data
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, PartialEq)]
pub struct Tipe {
    pub jenis: JenisTipe,
    pub rentang: Rentang,
}

#[derive(Debug, Clone, PartialEq)]
pub enum JenisTipe {
    /// Nama tipe, mis. `bulat`, `larik<teks>`, `hasil<T, E>`, `saluran<bulat>`.
    Nama { nama: Pengenal, argumen: Vec<Tipe> },
    /// `pinjam T` atau `pinjam var T`.
    Pinjam { mutabel: bool, dalam: Box<Tipe> },
}

impl Tipe {
    pub fn baru(jenis: JenisTipe, rentang: Rentang) -> Self {
        Self { jenis, rentang }
    }
}

// ---------------------------------------------------------------------------
// Pernyataan
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, PartialEq)]
pub struct Blok {
    pub pernyataan: Vec<Pernyataan>,
    pub rentang: Rentang,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Pernyataan {
    pub jenis: JenisPernyataan,
    pub rentang: Rentang,
}

impl Pernyataan {
    pub fn baru(jenis: JenisPernyataan, rentang: Rentang) -> Self {
        Self { jenis, rentang }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OperatorTugas {
    Tetap,
    Tambah,
    Kurang,
    Kali,
    Bagi,
    Sisa,
}

impl OperatorTugas {
    pub fn simbol(self) -> &'static str {
        match self {
            OperatorTugas::Tetap => "=",
            OperatorTugas::Tambah => "+=",
            OperatorTugas::Kurang => "-=",
            OperatorTugas::Kali => "*=",
            OperatorTugas::Bagi => "/=",
            OperatorTugas::Sisa => "%=",
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum JenisPernyataan {
    Variabel {
        mutabel: bool,
        nama: Pengenal,
        tipe: Option<Tipe>,
        nilai: Ekspresi,
    },
    Tugas {
        target: Ekspresi,
        operator: OperatorTugas,
        nilai: Ekspresi,
    },
    /// `lainnya` berisi pernyataan `Jika` (else-if) atau `Blok` (else).
    Jika {
        kondisi: Ekspresi,
        maka: Blok,
        lainnya: Option<Box<Pernyataan>>,
    },
    Selagi {
        kondisi: Ekspresi,
        badan: Blok,
    },
    Untuk {
        variabel: Pengenal,
        koleksi: Ekspresi,
        badan: Blok,
    },
    Pilih {
        nilai: Ekspresi,
        kasus: Vec<CabangKasus>,
        lainnya: Option<Vec<Pernyataan>>,
    },
    Cocok {
        nilai: Ekspresi,
        cabang: Vec<CabangPola>,
    },
    Tunda(Ekspresi),
    TundaBlok(Blok),
    Jalankan(Ekspresi),
    Kembali(Option<Ekspresi>),
    TidakAman(Blok),
    Blok(Blok),
    Ekspresi(Ekspresi),
}

#[derive(Debug, Clone, PartialEq)]
pub struct CabangKasus {
    pub nilai: Ekspresi,
    pub badan: Vec<Pernyataan>,
    pub rentang: Rentang,
}

#[derive(Debug, Clone, PartialEq)]
pub struct CabangPola {
    pub pola: Pola,
    pub badan: Pernyataan,
    pub rentang: Rentang,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Pola {
    pub jenis: JenisPola,
    pub rentang: Rentang,
}

impl Pola {
    pub fn baru(jenis: JenisPola, rentang: Rentang) -> Self {
        Self { jenis, rentang }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum JenisPola {
    /// `_`
    Wildcard,
    /// Literal angka, teks, karakter, `benar`, `salah`, `kosong` (angka boleh negatif).
    Literal(Ekspresi),
    /// Nama saja: pengikat variabel atau varian tanpa data (mis. `tidak_ada`).
    Pengenal(Pengenal),
    /// Varian berdata, mis. `sukses(n)`.
    Varian { nama: Pengenal, argumen: Vec<Pola> },
}

// ---------------------------------------------------------------------------
// Ekspresi
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, PartialEq)]
pub struct Ekspresi {
    pub jenis: JenisEkspresi,
    pub rentang: Rentang,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OperatorBiner {
    Tambah,
    Kurang,
    Kali,
    Bagi,
    Sisa,
    Sama,
    TidakSama,
    LebihKecil,
    LebihKecilSama,
    LebihBesar,
    LebihBesarSama,
    Dan,
    Atau,
}

impl OperatorBiner {
    pub fn simbol(self) -> &'static str {
        match self {
            OperatorBiner::Tambah => "+",
            OperatorBiner::Kurang => "-",
            OperatorBiner::Kali => "*",
            OperatorBiner::Bagi => "/",
            OperatorBiner::Sisa => "%",
            OperatorBiner::Sama => "==",
            OperatorBiner::TidakSama => "!=",
            OperatorBiner::LebihKecil => "<",
            OperatorBiner::LebihKecilSama => "<=",
            OperatorBiner::LebihBesar => ">",
            OperatorBiner::LebihBesarSama => ">=",
            OperatorBiner::Dan => "dan",
            OperatorBiner::Atau => "atau",
        }
    }

    /// Prioritas: `atau` < `dan` < relasi < rentang(4) < `+ -` < `* / %`.
    pub fn prioritas(self) -> u8 {
        match self {
            OperatorBiner::Atau => 1,
            OperatorBiner::Dan => 2,
            OperatorBiner::Sama
            | OperatorBiner::TidakSama
            | OperatorBiner::LebihKecil
            | OperatorBiner::LebihKecilSama
            | OperatorBiner::LebihBesar
            | OperatorBiner::LebihBesarSama => 3,
            OperatorBiner::Tambah | OperatorBiner::Kurang => 5,
            OperatorBiner::Kali | OperatorBiner::Bagi | OperatorBiner::Sisa => 6,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OperatorUnari {
    Negatif,
    Bukan,
    Pinjam,
    PinjamMutabel,
    Pindah,
}

impl OperatorUnari {
    /// Simbol awalan; operator berupa kata menyertakan spasi di akhir.
    pub fn simbol(self) -> &'static str {
        match self {
            OperatorUnari::Negatif => "-",
            OperatorUnari::Bukan => "!",
            OperatorUnari::Pinjam => "pinjam ",
            OperatorUnari::PinjamMutabel => "pinjam var ",
            OperatorUnari::Pindah => "pindah ",
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum BagianTeks {
    Teks(String),
    Ekspresi(Box<Ekspresi>),
}

#[derive(Debug, Clone, PartialEq)]
pub struct BidangInisialisasi {
    pub nama: Pengenal,
    pub nilai: Ekspresi,
}

#[derive(Debug, Clone, PartialEq)]
pub enum JenisEkspresi {
    Bulat(u64),
    Desimal(f64),
    /// Teks, dengan interpolasi `{ekspresi}` sebagai bagian terpisah.
    Teks(Vec<BagianTeks>),
    Rune(char),
    Bool(bool),
    Kosong,
    Pengenal(Pengenal),
    Biner {
        operator: OperatorBiner,
        kiri: Box<Ekspresi>,
        kanan: Box<Ekspresi>,
    },
    Unari {
        operator: OperatorUnari,
        operan: Box<Ekspresi>,
    },
    Panggil {
        fungsi: Box<Ekspresi>,
        argumen_tipe: Vec<Tipe>,
        argumen: Vec<Ekspresi>,
    },
    Bidang {
        objek: Box<Ekspresi>,
        nama: Pengenal,
    },
    Indeks {
        objek: Box<Ekspresi>,
        indeks: Box<Ekspresi>,
    },
    Struktur {
        nama: Pengenal,
        bidang: Vec<BidangInisialisasi>,
    },
    Larik(Vec<Ekspresi>),
    Rentang {
        awal: Box<Ekspresi>,
        akhir: Box<Ekspresi>,
        inklusif: bool,
    },
}

impl Ekspresi {
    pub fn baru(jenis: JenisEkspresi, rentang: Rentang) -> Self {
        Self { jenis, rentang }
    }

    /// Prioritas simpul ini; lebih tinggi berarti lebih erat mengikat.
    pub fn prioritas(&self) -> u8 {
        match &self.jenis {
            JenisEkspresi::Biner { operator, .. } => operator.prioritas(),
            JenisEkspresi::Rentang { .. } => PRIORITAS_RENTANG,
            JenisEkspresi::Unari { .. } => PRIORITAS_UNARI,
            JenisEkspresi::Panggil { .. }
            | JenisEkspresi::Bidang { .. }
            | JenisEkspresi::Indeks { .. } => PRIORITAS_POSTFIKS,
            _ => PRIORITAS_PRIMER,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn r() -> Rentang {
        Rentang::titik_kosong()
    }

    fn id(n: &str) -> Pengenal {
        Pengenal::baru(n, r())
    }

    fn bulat(n: u64) -> Ekspresi {
        Ekspresi::baru(JenisEkspresi::Bulat(n), r())
    }

    #[test]
    fn tes_rentang_dan_pengenal() {
        let rentang = Rentang::baru(0, 5, 1, 1);
        let p = Pengenal::baru("utama", rentang);
        assert_eq!(p.nama, "utama");
        assert_eq!(p.rentang.baris, 1);
        assert_eq!(p.rentang.kolom, 1);
    }

    #[test]
    fn tes_gabung_rentang() {
        let a = Rentang::baru(2, 4, 1, 3);
        let b = Rentang::baru(10, 15, 2, 1);
        let g = a.gabung(b);
        assert_eq!((g.awal, g.akhir, g.baris, g.kolom), (2, 15, 1, 3));
    }

    #[test]
    fn tes_prioritas_operator() {
        assert!(OperatorBiner::Kali.prioritas() > OperatorBiner::Tambah.prioritas());
        assert!(OperatorBiner::Tambah.prioritas() > PRIORITAS_RENTANG);
        assert!(PRIORITAS_RENTANG > OperatorBiner::Sama.prioritas());
        assert!(OperatorBiner::Dan.prioritas() > OperatorBiner::Atau.prioritas());
    }

    #[test]
    fn tes_prioritas_ekspresi() {
        let e = Ekspresi::baru(
            JenisEkspresi::Biner {
                operator: OperatorBiner::Tambah,
                kiri: Box::new(bulat(1)),
                kanan: Box::new(bulat(2)),
            },
            r(),
        );
        assert_eq!(e.prioritas(), 5);
        assert_eq!(bulat(1).prioritas(), PRIORITAS_PRIMER);
    }

    #[test]
    fn tes_berkas_program_kosong() {
        let berkas = BerkasProgram {
            paket: DeklarasiPaket {
                nama: id("utama"),
                rentang: r(),
            },
            impor: Vec::new(),
            deklarasi: Vec::new(),
        };
        assert_eq!(berkas.paket.nama.nama, "utama");
        assert!(berkas.impor.is_empty());
        assert!(berkas.deklarasi.is_empty());
    }

    #[test]
    fn tes_simbol_operator() {
        assert_eq!(OperatorTugas::Tambah.simbol(), "+=");
        assert_eq!(OperatorBiner::Dan.simbol(), "dan");
        assert_eq!(OperatorUnari::PinjamMutabel.simbol(), "pinjam var ");
    }
}
