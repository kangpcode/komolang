// Copyright (c) kangpcode (Dhafa Nazula Permadi). Licensed under MIT or Apache-2.0.

//! `komo_lexer` bertanggung jawab untuk analisis leksikal kode sumber Komolang menjadi token.

use komo_ast::Rentang;

/// Jenis token leksikal Komolang sesuai spesifikasi v0.1 dan RFC 0001.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum JenisToken {
    // Kata Kunci (Keywords)
    Paket,
    Impor,
    Fungsi,
    Kembali,
    Biar,
    Var,
    Konst,
    Jika,
    Lainnya,
    Selagi,
    Untuk,
    Dalam,
    Pilih,
    Kasus,
    Cocok,
    Tipe,
    Struktur,
    Antarmuka,
    Enum,
    Jalankan,
    Saluran,
    Tunda,
    Benar,
    Salah,
    Kosong,
    Pinjam,
    Pindah,
    Publik,
    Pribadi,
    TidakAman,

    // Tipe bawaan baku
    TipeBulat,
    TipeDesimal,
    TipeTeks,
    TipeBool,
    TipeByte,
    TipeRune,

    // Identifier & Literal
    Pengenal(String),
    LiteralBulat(i64),
    LiteralDesimal(String),
    LiteralTeks(String),
    LiteralRune(char),

    // Operator dan Tanda Baca
    Tambah,         // +
    Kurang,         // -
    Kali,           // *
    Bagi,           // /
    SisaBagi,       // %
    SamaDengan,     // =
    SamaKetat,      // ==
    TidakSama,      // !=
    KurangDari,     // <
    KurangDariSama, // <=
    LebihDari,      // >
    LebihDariSama,  // >=
    DanLogika,      // dan
    AtauLogika,     // atau
    BukanLogika,    // !

    TitikKoma,  // ;
    Titik,      // .
    TitikDua,   // :
    Koma,       // ,
    PanahKanan, // ->
    TitikTitik, // ..

    KurungBuka,         // (
    KurungTutup,        // )
    KurungKurawalBuka,  // {
    KurungKurawalTutup, // }
    KurungSikuBuka,     // [
    KurungSikuTutup,    // ]

    AkhirBerkas, // EOF
}

/// Token lengkap dengan jenis dan rentang lokasinya dalam kode sumber.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Token {
    pub jenis: JenisToken,
    pub rentang: Rentang,
}

impl Token {
    pub fn baru(jenis: JenisToken, rentang: Rentang) -> Self {
        Self { jenis, rentang }
    }
}

/// Mencocokkan kata dengan kata kunci resmi Komolang.
pub fn cocokkan_kata_kunci(kata: &str) -> Option<JenisToken> {
    match kata {
        "paket" => Some(JenisToken::Paket),
        "impor" => Some(JenisToken::Impor),
        "fungsi" => Some(JenisToken::Fungsi),
        "kembali" => Some(JenisToken::Kembali),
        "biar" => Some(JenisToken::Biar),
        "var" => Some(JenisToken::Var),
        "konst" => Some(JenisToken::Konst),
        "jika" => Some(JenisToken::Jika),
        "lainnya" => Some(JenisToken::Lainnya),
        "selagi" => Some(JenisToken::Selagi),
        "untuk" => Some(JenisToken::Untuk),
        "dalam" => Some(JenisToken::Dalam),
        "pilih" => Some(JenisToken::Pilih),
        "kasus" => Some(JenisToken::Kasus),
        "cocok" => Some(JenisToken::Cocok),
        "tipe" => Some(JenisToken::Tipe),
        "struktur" => Some(JenisToken::Struktur),
        "antarmuka" => Some(JenisToken::Antarmuka),
        "enum" => Some(JenisToken::Enum),
        "jalankan" => Some(JenisToken::Jalankan),
        "saluran" => Some(JenisToken::Saluran),
        "tunda" => Some(JenisToken::Tunda),
        "benar" => Some(JenisToken::Benar),
        "salah" => Some(JenisToken::Salah),
        "kosong" => Some(JenisToken::Kosong),
        "pinjam" => Some(JenisToken::Pinjam),
        "pindah" => Some(JenisToken::Pindah),
        "publik" => Some(JenisToken::Publik),
        "pribadi" => Some(JenisToken::Pribadi),
        "tidak_aman" => Some(JenisToken::TidakAman),

        "dan" => Some(JenisToken::DanLogika),
        "atau" => Some(JenisToken::AtauLogika),

        "bulat" => Some(JenisToken::TipeBulat),
        "desimal" => Some(JenisToken::TipeDesimal),
        "teks" => Some(JenisToken::TipeTeks),
        "bool" => Some(JenisToken::TipeBool),
        "byte" => Some(JenisToken::TipeByte),
        "rune" => Some(JenisToken::TipeRune),
        _ => None,
    }
}

/// Tokenizer sederhana untuk memvalidasi token leksikal pada tahap fondasi.
pub struct Pemindai<'a> {
    sumber: &'a str,
    posisi: usize,
    baris: usize,
    kolom: usize,
}

impl<'a> Pemindai<'a> {
    pub fn baru(sumber: &'a str) -> Self {
        Self {
            sumber,
            posisi: 0,
            baris: 1,
            kolom: 1,
        }
    }

    pub fn pindai_semua(&mut self) -> Result<Vec<Token>, String> {
        let mut tokens = Vec::new();

        while self.posisi < self.sumber.len() {
            self.lewati_spasi_dan_komentar();
            if self.posisi >= self.sumber.len() {
                break;
            }

            let token = self.pindai_token_berikutnya()?;
            tokens.push(token);
        }

        tokens.push(Token::baru(
            JenisToken::AkhirBerkas,
            Rentang::baru(self.posisi, self.posisi, self.baris, self.kolom),
        ));

        Ok(tokens)
    }

    fn lewati_spasi_dan_komentar(&mut self) {
        let sisa = &self.sumber[self.posisi..];
        let mut chars = sisa.char_indices().peekable();

        while let Some((_, c)) = chars.peek().copied() {
            if c.is_whitespace() {
                if c == '\n' {
                    self.baris += 1;
                    self.kolom = 1;
                } else {
                    self.kolom += 1;
                }
                self.posisi += c.len_utf8();
                chars.next();
            } else if c == '/' {
                let mut temp = chars.clone();
                temp.next();
                if let Some((_, '/')) = temp.peek() {
                    // Komentar sebaris: lewati sampai \n
                    for (_, c2) in chars.by_ref() {
                        self.posisi += c2.len_utf8();
                        if c2 == '\n' {
                            self.baris += 1;
                            self.kolom = 1;
                            break;
                        }
                    }
                } else {
                    break;
                }
            } else {
                break;
            }
        }
    }

    fn pindai_token_berikutnya(&mut self) -> Result<Token, String> {
        let sisa = &self.sumber[self.posisi..];
        let c = sisa.chars().next().ok_or("Akhir berkas tak terduga")?;
        let awal_pos = self.posisi;
        let awal_baris = self.baris;
        let awal_kolom = self.kolom;

        if c.is_alphabetic() || c == '_' {
            // Identifier atau Kata Kunci
            let mut len = 0;
            for ch in sisa.chars() {
                if ch.is_alphanumeric() || ch == '_' {
                    len += ch.len_utf8();
                } else {
                    break;
                }
            }
            let kata = &sisa[..len];
            self.posisi += len;
            self.kolom += kata.chars().count();

            let jenis =
                cocokkan_kata_kunci(kata).unwrap_or_else(|| JenisToken::Pengenal(kata.to_string()));
            let rentang = Rentang::baru(awal_pos, self.posisi, awal_baris, awal_kolom);
            return Ok(Token::baru(jenis, rentang));
        }

        if c.is_ascii_digit() {
            // Literal Angka sederhana
            let mut len = 0;
            for ch in sisa.chars() {
                if ch.is_ascii_digit() || ch == '_' {
                    len += ch.len_utf8();
                } else {
                    break;
                }
            }
            let angka_str = sisa[..len].replace('_', "");
            self.posisi += len;
            self.kolom += len;
            let angka: i64 = angka_str
                .parse()
                .map_err(|e| format!("Angka tidak valid: {}", e))?;
            let rentang = Rentang::baru(awal_pos, self.posisi, awal_baris, awal_kolom);
            return Ok(Token::baru(JenisToken::LiteralBulat(angka), rentang));
        }

        if c == '"' {
            // Literal Teks sederhana
            let mut len = c.len_utf8();
            let mut ditutup = false;
            let mut teks_konten = String::new();

            for ch in sisa[len..].chars() {
                len += ch.len_utf8();
                if ch == '"' {
                    ditutup = true;
                    break;
                } else {
                    teks_konten.push(ch);
                }
            }

            if !ditutup {
                return Err(format!("String tidak ditutup pada baris {}", awal_baris));
            }

            self.posisi += len;
            self.kolom += len;
            let rentang = Rentang::baru(awal_pos, self.posisi, awal_baris, awal_kolom);
            return Ok(Token::baru(JenisToken::LiteralTeks(teks_konten), rentang));
        }

        // Simbol tunggal atau ganda
        let jenis = match c {
            '+' => JenisToken::Tambah,
            '-' => JenisToken::Kurang,
            '*' => JenisToken::Kali,
            '/' => JenisToken::Bagi,
            '%' => JenisToken::SisaBagi,
            '=' => {
                if sisa.starts_with("==") {
                    self.posisi += 2;
                    self.kolom += 2;
                    let rentang = Rentang::baru(awal_pos, self.posisi, awal_baris, awal_kolom);
                    return Ok(Token::baru(JenisToken::SamaKetat, rentang));
                }
                JenisToken::SamaDengan
            }
            '<' => {
                if sisa.starts_with("<=") {
                    self.posisi += 2;
                    self.kolom += 2;
                    let rentang = Rentang::baru(awal_pos, self.posisi, awal_baris, awal_kolom);
                    return Ok(Token::baru(JenisToken::KurangDariSama, rentang));
                }
                JenisToken::KurangDari
            }
            '>' => {
                if sisa.starts_with(">=") {
                    self.posisi += 2;
                    self.kolom += 2;
                    let rentang = Rentang::baru(awal_pos, self.posisi, awal_baris, awal_kolom);
                    return Ok(Token::baru(JenisToken::LebihDariSama, rentang));
                }
                JenisToken::LebihDari
            }
            '!' => {
                if sisa.starts_with("!=") {
                    self.posisi += 2;
                    self.kolom += 2;
                    let rentang = Rentang::baru(awal_pos, self.posisi, awal_baris, awal_kolom);
                    return Ok(Token::baru(JenisToken::TidakSama, rentang));
                }
                JenisToken::BukanLogika
            }
            '.' => {
                if sisa.starts_with("..") {
                    self.posisi += 2;
                    self.kolom += 2;
                    let rentang = Rentang::baru(awal_pos, self.posisi, awal_baris, awal_kolom);
                    return Ok(Token::baru(JenisToken::TitikTitik, rentang));
                }
                JenisToken::Titik
            }
            ':' => JenisToken::TitikDua,
            ';' => JenisToken::TitikKoma,
            ',' => JenisToken::Koma,
            '(' => JenisToken::KurungBuka,
            ')' => JenisToken::KurungTutup,
            '{' => JenisToken::KurungKurawalBuka,
            '}' => JenisToken::KurungKurawalTutup,
            '[' => JenisToken::KurungSikuBuka,
            ']' => JenisToken::KurungSikuTutup,
            _ => {
                return Err(format!(
                    "Karakter tidak dikenali '{}' di baris {}, kolom {}",
                    c, awal_baris, awal_kolom
                ))
            }
        };

        self.posisi += c.len_utf8();
        self.kolom += 1;
        let rentang = Rentang::baru(awal_pos, self.posisi, awal_baris, awal_kolom);
        Ok(Token::baru(jenis, rentang))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tes_kata_kunci_indonesia() {
        assert_eq!(cocokkan_kata_kunci("paket"), Some(JenisToken::Paket));
        assert_eq!(cocokkan_kata_kunci("fungsi"), Some(JenisToken::Fungsi));
        assert_eq!(cocokkan_kata_kunci("biar"), Some(JenisToken::Biar));
        assert_eq!(cocokkan_kata_kunci("kembali"), Some(JenisToken::Kembali));
        assert_eq!(cocokkan_kata_kunci("cocok"), Some(JenisToken::Cocok));
        assert_eq!(cocokkan_kata_kunci("tunda"), Some(JenisToken::Tunda));
        assert_eq!(cocokkan_kata_kunci("jalankan"), Some(JenisToken::Jalankan));
        assert_eq!(
            cocokkan_kata_kunci("tidak_aman"),
            Some(JenisToken::TidakAman)
        );
    }

    #[test]
    fn tes_pindai_program_sederhana() {
        let sumber = r#"
            paket utama
            // komentar
            biar pesan = "Halo Komodo"
        "#;
        let mut pemindai = Pemindai::baru(sumber);
        let tokens = pemindai.pindai_semua().expect("Gagal memindai");

        assert_eq!(tokens[0].jenis, JenisToken::Paket);
        assert_eq!(tokens[1].jenis, JenisToken::Pengenal("utama".to_string()));
        assert_eq!(tokens[2].jenis, JenisToken::Biar);
        assert_eq!(tokens[3].jenis, JenisToken::Pengenal("pesan".to_string()));
        assert_eq!(tokens[4].jenis, JenisToken::SamaDengan);
        assert_eq!(
            tokens[5].jenis,
            JenisToken::LiteralTeks("Halo Komodo".to_string())
        );
        assert_eq!(tokens[6].jenis, JenisToken::AkhirBerkas);
    }
}
