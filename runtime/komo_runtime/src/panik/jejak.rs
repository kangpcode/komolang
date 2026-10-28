// Copyright (c) kangpcode (Dhafa Nazula Permadi). Licensed under MIT or Apache-2.0.

//! Penjejak tumpukan panggilan (stack trace) berbahasa Indonesia saat terjadi kepanikan runtime.

use std::fmt::{self, Write};

/// Satu bingkai dalam jejak tumpukan pemanggilan fungsi.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BingkaiJejak {
    pub nama_fungsi: String,
    pub berkas: String,
    pub baris: usize,
    pub kolom: usize,
}

impl BingkaiJejak {
    pub fn baru(nama_fungsi: &str, berkas: &str, baris: usize, kolom: usize) -> Self {
        Self {
            nama_fungsi: nama_fungsi.to_string(),
            berkas: berkas.to_string(),
            baris,
            kolom,
        }
    }
}

/// Struktur jejak tumpukan lengkap yang merekam alur eksekusi saat kepanikan (panic).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct JejakTumpukan {
    pub bingkai: Vec<BingkaiJejak>,
}

impl JejakTumpukan {
    pub fn baru() -> Self {
        Self {
            bingkai: Vec::new(),
        }
    }

    pub fn tambah(&mut self, bingkai: BingkaiJejak) {
        self.bingkai.push(bingkai);
    }

    /// Menghasilkan representasi teks laporan kepanikan resmi Komolang berbahasa Indonesia.
    pub fn format_laporan(&self, alasan: &str, nama_thread: Option<&str>) -> String {
        let mut out = String::new();
        writeln!(out, "🚨 KEPANIKAN RUNTIME KOMOLANG (Panic)").unwrap();
        if let Some(t) = nama_thread {
            writeln!(out, "  Thread : '{}'", t).unwrap();
        }
        writeln!(out, "  Alasan : {}", alasan).unwrap();

        if let Some(pertama) = self.bingkai.first() {
            writeln!(
                out,
                "  Lokasi : {}:{}:{}",
                pertama.berkas, pertama.baris, pertama.kolom
            )
            .unwrap();
        }

        if !self.bingkai.is_empty() {
            writeln!(out, "  Jejak Tumpukan (Stack Trace):").unwrap();
            for (idx, b) in self.bingkai.iter().enumerate() {
                writeln!(
                    out,
                    "    [{}] {} ({}:{}:{})",
                    idx, b.nama_fungsi, b.berkas, b.baris, b.kolom
                )
                .unwrap();
            }
        }

        writeln!(
            out,
            "  Catatan: Pastikan validasi masukan atau gunakan 'tangkap_kepanikan' untuk penanganan mandiri."
        )
        .unwrap();

        out
    }
}

impl Default for JejakTumpukan {
    fn default() -> Self {
        Self::baru()
    }
}

impl fmt::Display for JejakTumpukan {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.format_laporan("Kepanikan tidak spesifik", None))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tes_format_jejak_tumpukan_indonesia() {
        let mut jejak = JejakTumpukan::baru();
        jejak.tambah(BingkaiJejak::baru("matematika.bagi", "src/matematika.komo", 18, 5));
        jejak.tambah(BingkaiJejak::baru("utama.hitung", "src/utama.komo", 42, 10));
        jejak.tambah(BingkaiJejak::baru("utama.main", "src/utama.komo", 5, 1));

        let teks = jejak.format_laporan("Pembagian dengan nol", Some("pekerja-0"));
        assert!(teks.contains("🚨 KEPANIKAN RUNTIME KOMOLANG"));
        assert!(teks.contains("Thread : 'pekerja-0'"));
        assert!(teks.contains("Alasan : Pembagian dengan nol"));
        assert!(teks.contains("[0] matematika.bagi (src/matematika.komo:18:5)"));
        assert!(teks.contains("[1] utama.hitung (src/utama.komo:42:10)"));
    }
}
