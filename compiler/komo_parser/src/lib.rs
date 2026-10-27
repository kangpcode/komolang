// Copyright (c) kangpcode (Dhafa Nazula Permadi). Licensed under MIT or Apache-2.0.

//! `komo_parser` mem-parsing daftar token dari `komo_lexer` menjadi AST `komo_ast`.

use komo_ast::{
    BerkasProgram, DeklarasiFungsi, DeklarasiImpor, DeklarasiPaket, DeklarasiTingkatAtas, Pengenal,
};
use komo_lexer::{JenisToken, Token};

pub struct Pengurai {
    tokens: Vec<Token>,
    posisi: usize,
}

impl Pengurai {
    pub fn baru(tokens: Vec<Token>) -> Self {
        Self { tokens, posisi: 0 }
    }

    pub fn urai_berkas(&mut self) -> Result<BerkasProgram, String> {
        let paket = self.urai_paket()?;
        let mut impor = Vec::new();

        while self.cek(&JenisToken::Impor) {
            impor.push(self.urai_impor()?);
        }

        let mut deklarasi = Vec::new();
        while !self.adalah_akhir() {
            let item = self.urai_deklarasi_tingkat_atas()?;
            deklarasi.push(item);
        }

        Ok(BerkasProgram {
            paket,
            impor,
            deklarasi,
        })
    }

    fn urai_paket(&mut self) -> Result<DeklarasiPaket, String> {
        let token_paket = self.konsumsi(
            &JenisToken::Paket,
            "Diharapkan kata kunci 'paket' di awal berkas",
        )?;
        let token_nama = match self.ambil_sekarang() {
            Some(Token {
                jenis: JenisToken::Pengenal(nama),
                rentang,
            }) => {
                let id = Pengenal::baru(nama.clone(), *rentang);
                self.maju();
                id
            }
            _ => {
                return Err(format!(
                    "Diharapkan nama paket setelah 'paket' pada baris {}",
                    token_paket.rentang.baris
                ))
            }
        };

        Ok(DeklarasiPaket {
            nama: token_nama,
            rentang: token_paket.rentang,
        })
    }

    fn urai_impor(&mut self) -> Result<DeklarasiImpor, String> {
        let token_impor = self.konsumsi(&JenisToken::Impor, "Diharapkan kata kunci 'impor'")?;
        let jalur = match self.ambil_sekarang() {
            Some(Token {
                jenis: JenisToken::LiteralTeks(j),
                ..
            }) => {
                let j = j.clone();
                self.maju();
                j
            }
            _ => {
                return Err(format!(
                    "Diharapkan path string setelah 'impor' pada baris {}",
                    token_impor.rentang.baris
                ))
            }
        };

        Ok(DeklarasiImpor {
            jalur,
            alias: None,
            rentang: token_impor.rentang,
        })
    }

    fn urai_deklarasi_tingkat_atas(&mut self) -> Result<DeklarasiTingkatAtas, String> {
        let adalah_publik = if self.cek(&JenisToken::Publik) {
            self.maju();
            true
        } else {
            false
        };

        if self.cek(&JenisToken::Fungsi) {
            let fungsi = self.urai_fungsi(adalah_publik)?;
            return Ok(DeklarasiTingkatAtas::Fungsi(fungsi));
        }

        Err(format!(
            "Deklarasi tingkat atas tidak dikenali pada baris {}",
            self.ambil_sekarang().map(|t| t.rentang.baris).unwrap_or(1)
        ))
    }

    fn urai_fungsi(&mut self, adalah_publik: bool) -> Result<DeklarasiFungsi, String> {
        let token_fn = self.konsumsi(&JenisToken::Fungsi, "Diharapkan kata kunci 'fungsi'")?;
        let nama = match self.ambil_sekarang() {
            Some(Token {
                jenis: JenisToken::Pengenal(n),
                rentang,
            }) => {
                let id = Pengenal::baru(n.clone(), *rentang);
                self.maju();
                id
            }
            _ => {
                return Err(format!(
                    "Diharapkan nama fungsi setelah 'fungsi' pada baris {}",
                    token_fn.rentang.baris
                ))
            }
        };

        self.konsumsi(
            &JenisToken::KurungBuka,
            "Diharapkan '(' setelah nama fungsi",
        )?;
        self.konsumsi(
            &JenisToken::KurungTutup,
            "Diharapkan ')' menutup parameter fungsi",
        )?;

        // Penanganan tipe kembalian opsional
        let tipe_kembalian = if let Some(Token {
            jenis: JenisToken::Pengenal(tipe),
            ..
        }) = self.ambil_sekarang()
        {
            let t = tipe.clone();
            self.maju();
            Some(t)
        } else {
            None
        };

        // Blok kosong {}
        self.konsumsi(
            &JenisToken::KurungKurawalBuka,
            "Diharapkan '{' memulai tubuh fungsi",
        )?;
        self.konsumsi(
            &JenisToken::KurungKurawalTutup,
            "Diharapkan '}' menutup tubuh fungsi",
        )?;

        Ok(DeklarasiFungsi {
            adalah_publik,
            nama,
            parameter: Vec::new(),
            tipe_kembalian,
            rentang: token_fn.rentang,
        })
    }

    fn konsumsi(&mut self, jenis_target: &JenisToken, pesan_error: &str) -> Result<Token, String> {
        if let Some(token) = self.ambil_sekarang() {
            if &token.jenis == jenis_target {
                let t = token.clone();
                self.maju();
                return Ok(t);
            }
        }
        Err(pesan_error.to_string())
    }

    fn cek(&self, jenis: &JenisToken) -> bool {
        if let Some(token) = self.ambil_sekarang() {
            &token.jenis == jenis
        } else {
            false
        }
    }

    fn maju(&mut self) {
        if self.posisi < self.tokens.len() {
            self.posisi += 1;
        }
    }

    fn ambil_sekarang(&self) -> Option<&Token> {
        self.tokens.get(self.posisi)
    }

    fn adalah_akhir(&self) -> bool {
        match self.ambil_sekarang() {
            Some(t) => t.jenis == JenisToken::AkhirBerkas,
            None => true,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use komo_lexer::Pemindai;

    #[test]
    fn tes_urai_paket_dan_fungsi() {
        let sumber = r#"
            paket utama
            impor "std/cetak"

            publik fungsi halo() {
            }
        "#;
        let mut pemindai = Pemindai::baru(sumber);
        let tokens = pemindai.pindai_semua().expect("Gagal memindai");

        let mut pengurai = Pengurai::baru(tokens);
        let berkas = pengurai.urai_berkas().expect("Gagal mengurai");

        assert_eq!(berkas.paket.nama.nama, "utama");
        assert_eq!(berkas.impor.len(), 1);
        assert_eq!(berkas.impor[0].jalur, "std/cetak");
        assert_eq!(berkas.deklarasi.len(), 1);

        if let DeklarasiTingkatAtas::Fungsi(f) = &berkas.deklarasi[0] {
            assert!(f.adalah_publik);
            assert_eq!(f.nama.nama, "halo");
        } else {
            panic!("Diharapkan DeklarasiFungsi");
        }
    }
}
