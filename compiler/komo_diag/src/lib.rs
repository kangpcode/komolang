// Copyright (c) kangpcode (Dhafa Nazula Permadi). Licensed under MIT or Apache-2.0.

//! `komo_diag` menyediakan struktur diagnostik (galat/peringatan) beserta penyajiannya
//! dalam Bahasa Indonesia: nama berkas, baris, kolom, cuplikan kode, penanda `^^^`,
//! dan saran perbaikan.

use komo_ast::Rentang;
use std::fmt;

/// Tingkat keparahan diagnostik.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Tingkat {
    Galat,
    Peringatan,
}

impl Tingkat {
    pub fn label(self) -> &'static str {
        match self {
            Tingkat::Galat => "galat",
            Tingkat::Peringatan => "peringatan",
        }
    }
}

/// Kode diagnostik tetap (stabil) agar tes dan dokumentasi dapat merujuknya.
/// Awalan `L` = leksikal, `P` = parser.
pub mod kode {
    pub const KARAKTER_TAK_DIKENALI: &str = "L0001";
    pub const TEKS_TIDAK_DITUTUP: &str = "L0002";
    pub const KOMENTAR_TIDAK_DITUTUP: &str = "L0003";
    pub const ESCAPE_TIDAK_VALID: &str = "L0004";
    pub const ANGKA_TIDAK_VALID: &str = "L0005";
    pub const INTERPOLASI_TIDAK_DITUTUP: &str = "L0006";
    pub const KURUNG_KURAWAL_TUNGGAL: &str = "L0007";
    pub const RUNE_TIDAK_VALID: &str = "L0008";
    pub const INTERPOLASI_KOSONG: &str = "L0009";

    pub const DIHARAPKAN_TOKEN: &str = "P0001";
    pub const DIHARAPKAN_EKSPRESI: &str = "P0002";
    pub const DIHARAPKAN_NAMA: &str = "P0003";
    pub const DEKLARASI_TIDAK_DIKENAL: &str = "P0004";
    pub const TITIK_KOMA: &str = "P0005";
    pub const IMPOR_TERLAMBAT: &str = "P0006";
    pub const PANGGILAN_DIPERLUKAN: &str = "P0007";
    pub const DEKLARASI_DALAM_FUNGSI: &str = "P0008";
    pub const POLA_TIDAK_VALID: &str = "P0009";
    pub const BLOK_TIDAK_DITUTUP: &str = "P0010";
    pub const DIHARAPKAN_BARIS_BARU: &str = "P0011";
    pub const JALUR_IMPOR_TIDAK_VALID: &str = "P0012";
    pub const DIHARAPKAN_TIPE: &str = "P0013";
    pub const TARGET_TUGAS_TIDAK_VALID: &str = "P0014";
    pub const CABANG_GANDA: &str = "P0015";
    pub const RENTANG_BERANTAI: &str = "P0016";
    pub const SALURAN_TANPA_TIPE: &str = "P0017";
    pub const PAKET_HILANG: &str = "P0018";
    pub const TERLALU_DALAM: &str = "P0019";

    // Semantik (S = Semantik)
    pub const TIPE_TIDAK_COCOK: &str = "S0001";
    pub const SIMBOL_TIDAK_DITEMUKAN: &str = "S0002";
    pub const SIMBOL_DUPLIKAT: &str = "S0003";
    pub const MUTASI_VARIABEL_IMUTABEL: &str = "S0004";
    pub const PENGGUNAAN_SETELAH_PINDAH: &str = "S0005";
    pub const PEMINJAMAN_KONFLIK: &str = "S0006";
    pub const CABANG_TIDAK_EKSAUSTIF: &str = "S0007";
    pub const BUKAN_FUNGSI: &str = "S0008";
    pub const JUMLAH_ARGUMEN_SALAH: &str = "S0009";
    pub const BIDANG_TIDAK_DITEMUKAN: &str = "S0010";
    pub const BUKAN_STRUKTUR: &str = "S0011";
    pub const BUKAN_LARIK: &str = "S0012";
    pub const INDEKS_BUKAN_BULAT: &str = "S0013";
    pub const KONDISI_BUKAN_BOOL: &str = "S0014";
    pub const OPERATOR_TIDAK_DIDUKUNG: &str = "S0015";
    pub const RETURN_TIDAK_COCOK: &str = "S0016";
    pub const RETURN_DI_LUAR_FUNGSI: &str = "S0017";
    pub const ANTARMUKA_TIDAK_TERPENUHI: &str = "S0018";
    pub const VARIAN_ENUM_TIDAK_DITEMUKAN: &str = "S0019";
    pub const JUMLAH_ARGUMEN_VARIAN_SALAH: &str = "S0020";
    pub const AKSES_PRIVAT: &str = "S0021";
    pub const TIPE_TIDAK_DITEMUKAN: &str = "S0022";
    pub const JUMLAH_ARGUMEN_TIPE_SALAH: &str = "S0023";
    pub const JALUR_MODUL_TIDAK_VALID: &str = "S0024";
    pub const INFERENSI_GAGAL: &str = "S0025";
}

/// Satu pesan diagnostik dengan lokasi dan saran perbaikan opsional.
#[derive(Debug, Clone, PartialEq)]
pub struct Diagnostik {
    pub tingkat: Tingkat,
    pub kode: &'static str,
    pub pesan: String,
    pub rentang: Rentang,
    pub saran: Option<String>,
}

impl Diagnostik {
    pub fn galat(kode: &'static str, pesan: impl Into<String>, rentang: Rentang) -> Self {
        Self {
            tingkat: Tingkat::Galat,
            kode,
            pesan: pesan.into(),
            rentang,
            saran: None,
        }
    }

    pub fn peringatan(kode: &'static str, pesan: impl Into<String>, rentang: Rentang) -> Self {
        Self {
            tingkat: Tingkat::Peringatan,
            kode,
            pesan: pesan.into(),
            rentang,
            saran: None,
        }
    }

    pub fn dengan_saran(mut self, saran: impl Into<String>) -> Self {
        self.saran = Some(saran.into());
        self
    }

    /// Menyajikan diagnostik dalam format berikut:
    ///
    /// ```text
    /// galat[P0001]: diharapkan ')' ...
    ///  --> berkas.komo:3:10
    ///   |
    /// 3 | fungsi halo( {
    ///   |              ^
    ///   = saran: ...
    /// ```
    pub fn render(&self, nama_berkas: &str, sumber: &str) -> String {
        let baris = self.rentang.baris.max(1);
        let kolom = self.rentang.kolom.max(1);
        let teks_baris = sumber
            .lines()
            .nth(baris - 1)
            .unwrap_or("")
            .trim_end_matches('\r');
        let lebar = baris.to_string().len();
        let pad = " ".repeat(lebar);

        let sisa_baris = teks_baris.chars().count().saturating_sub(kolom - 1);
        let panjang = match sumber.get(self.rentang.awal..self.rentang.akhir) {
            Some(potongan) if potongan.contains('\n') => sisa_baris.max(1),
            Some(potongan) if !potongan.is_empty() => potongan.chars().count(),
            _ => 1,
        }
        .max(1);

        let spasi: String = teks_baris
            .chars()
            .take(kolom - 1)
            .map(|c| if c == '\t' { '\t' } else { ' ' })
            .collect();

        let mut keluar = format!("{}[{}]: {}\n", self.tingkat.label(), self.kode, self.pesan);
        keluar.push_str(&format!("{pad}--> {nama_berkas}:{baris}:{kolom}\n"));
        keluar.push_str(&format!("{pad} |\n"));
        keluar.push_str(&format!("{baris} | {teks_baris}\n"));
        keluar.push_str(&format!("{pad} | {spasi}{}\n", "^".repeat(panjang)));
        if let Some(saran) = &self.saran {
            keluar.push_str(&format!("{pad} = saran: {saran}\n"));
        }
        keluar
    }
}

impl fmt::Display for Diagnostik {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}[{}]: {} (baris {}, kolom {})",
            self.tingkat.label(),
            self.kode,
            self.pesan,
            self.rentang.baris,
            self.rentang.kolom
        )
    }
}

/// Menyajikan banyak diagnostik sekaligus, dipisahkan baris kosong.
pub fn render_semua(diagnostik: &[Diagnostik], nama_berkas: &str, sumber: &str) -> String {
    diagnostik
        .iter()
        .map(|d| d.render(nama_berkas, sumber))
        .collect::<Vec<_>>()
        .join("\n")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tes_render_memuat_lokasi_dan_saran() {
        let sumber = "paket utama\nbiar x = #\n";
        let d = Diagnostik::galat(
            kode::KARAKTER_TAK_DIKENALI,
            "karakter '#' tidak dikenali",
            Rentang::baru(21, 22, 2, 10),
        )
        .dengan_saran("hapus karakter tersebut");
        let teks = d.render("uji.komo", sumber);
        assert!(teks.contains("galat[L0001]"));
        assert!(teks.contains("--> uji.komo:2:10"));
        assert!(teks.contains("2 | biar x = #"));
        assert!(teks.contains("^"));
        assert!(teks.contains("saran: hapus karakter tersebut"));
    }

    #[test]
    fn tes_render_tidak_panik_di_luar_batas() {
        let d = Diagnostik::galat("X", "uji", Rentang::baru(999, 1200, 50, 80));
        let teks = d.render("a.komo", "pendek");
        assert!(teks.contains("galat[X]"));
    }

    #[test]
    fn tes_render_rentang_multibaris() {
        let sumber = "abc\ndef\n";
        let d = Diagnostik::galat("X", "multi", Rentang::baru(1, 6, 1, 2));
        let teks = d.render("a.komo", sumber);
        assert!(teks.contains("^^"));
    }

    #[test]
    fn tes_display_ringkas() {
        let d = Diagnostik::peringatan("W1", "awas", Rentang::baru(0, 1, 3, 4));
        assert_eq!(d.to_string(), "peringatan[W1]: awas (baris 3, kolom 4)");
    }

    #[test]
    fn tes_render_semua_memisah() {
        let a = Diagnostik::galat("A", "satu", Rentang::baru(0, 1, 1, 1));
        let b = Diagnostik::galat("B", "dua", Rentang::baru(0, 1, 1, 1));
        let teks = render_semua(&[a, b], "x.komo", "x");
        assert!(teks.contains("galat[A]"));
        assert!(teks.contains("galat[B]"));
    }
}
