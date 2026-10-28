// Copyright (c) kangpcode (Dhafa Nazula Permadi). Licensed under MIT or Apache-2.0.

//! Pohon elemen UI deklaratif berbasis komponen (Container, Text, Button, Input, Checkbox, Image).

use crate::tema::WarnaRgba;

/// Arah tata letak (Flex Direction).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ArahTataLetak {
    Kolom, // Vertikal
    Baris, // Horizontal
}

/// Perataan elemen pada sumbu utama atau silang (Alignment).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Perataan {
    Awal,
    Tengah,
    Akhir,
    Antara,
}

/// Gaya visual dan properti tata letak elemen UI.
#[derive(Debug, Clone, PartialEq)]
pub struct GayaLayout {
    pub arah: ArahTataLetak,
    pub perataan_utama: Perataan,
    pub perataan_silang: Perataan,
    pub lebar: Option<f32>,
    pub tinggi: Option<f32>,
    pub padding: f32,
    pub margin: f32,
    pub jarak_antar_anak: f32,
    pub warna_latar: Option<WarnaRgba>,
    pub warna_batas: Option<WarnaRgba>,
    pub tebal_batas: f32,
    pub radius_sudut: f32,
}

impl Default for GayaLayout {
    fn default() -> Self {
        Self {
            arah: ArahTataLetak::Kolom,
            perataan_utama: Perataan::Awal,
            perataan_silang: Perataan::Awal,
            lebar: None,
            tinggi: None,
            padding: 0.0,
            margin: 0.0,
            jarak_antar_anak: 8.0,
            warna_latar: None,
            warna_batas: None,
            tebal_batas: 0.0,
            radius_sudut: 0.0,
        }
    }
}

/// Representasi simpul elemen dalam pohon UI KomoUI.
#[derive(Debug, Clone, PartialEq)]
pub enum ElemenUi {
    Wadah {
        gaya: GayaLayout,
        anak: Vec<ElemenUi>,
    },
    Teks {
        isi: String,
        ukuran: f32,
        tebal: bool,
        warna: Option<WarnaRgba>,
    },
    Tombol {
        label: String,
        id: String,
        gaya: GayaLayout,
    },
    Masukan {
        nilai: String,
        placeholder: String,
        id: String,
        lebar: f32,
    },
    KotakCentang {
        tercentang: bool,
        label: String,
        id: String,
    },
    Gambar {
        sumber: String,
        lebar: f32,
        tinggi: f32,
    },
}

impl ElemenUi {
    /// Menghitung jumlah seluruh simpul dalam subpohon UI secara rekursif.
    pub fn jumlah_simpul(&self) -> usize {
        match self {
            ElemenUi::Wadah { anak, .. } => 1 + anak.iter().map(|a| a.jumlah_simpul()).sum::<usize>(),
            _ => 1,
        }
    }
}

// -----------------------------------------------------------------------------
// Fungsi Pembantu Deklaratif (Declarative Builders)
// -----------------------------------------------------------------------------

pub fn wadah_kolom(anak: Vec<ElemenUi>) -> ElemenUi {
    ElemenUi::Wadah {
        gaya: GayaLayout {
            arah: ArahTataLetak::Kolom,
            ..Default::default()
        },
        anak,
    }
}

pub fn wadah_baris(anak: Vec<ElemenUi>) -> ElemenUi {
    ElemenUi::Wadah {
        gaya: GayaLayout {
            arah: ArahTataLetak::Baris,
            ..Default::default()
        },
        anak,
    }
}

pub fn teks(isi: &str) -> ElemenUi {
    ElemenUi::Teks {
        isi: isi.to_string(),
        ukuran: 14.0,
        tebal: false,
        warna: None,
    }
}

pub fn judul(isi: &str) -> ElemenUi {
    ElemenUi::Teks {
        isi: isi.to_string(),
        ukuran: 22.0,
        tebal: true,
        warna: None,
    }
}

pub fn tombol(label: &str, id: &str) -> ElemenUi {
    ElemenUi::Tombol {
        label: label.to_string(),
        id: id.to_string(),
        gaya: GayaLayout {
            padding: 8.0,
            radius_sudut: 6.0,
            ..Default::default()
        },
    }
}

pub fn masukan(placeholder: &str, id: &str) -> ElemenUi {
    ElemenUi::Masukan {
        nilai: String::new(),
        placeholder: placeholder.to_string(),
        id: id.to_string(),
        lebar: 200.0,
    }
}

pub fn kotak_centang(tercentang: bool, label: &str, id: &str) -> ElemenUi {
    ElemenUi::KotakCentang {
        tercentang,
        label: label.to_string(),
        id: id.to_string(),
    }
}
