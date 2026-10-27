// Copyright (c) kangpcode (Dhafa Nazula Permadi). Licensed under MIT or Apache-2.0.

//! `komo_lexer` memecah kode sumber Komolang (UTF-8) menjadi token.
//!
//! Fitur: komentar `//` dan `/* */` (bersarang), semua bentuk literal angka
//! (desimal, `0x`, `0b`, `0o`, pecahan, eksponen, pemisah `_`), teks dengan escape dan
//! interpolasi `{ekspresi}`, literal karakter, serta pemulihan error: kesalahan
//! leksikal dilaporkan sebagai diagnostik dan pemindaian tetap berlanjut.

use komo_ast::Rentang;
use komo_diag::{kode, Diagnostik};

/// Jenis token leksikal Komolang sesuai spesifikasi v0.1, RFC 0001, dan addendum Tahap 1.
#[derive(Debug, Clone, PartialEq)]
pub enum JenisToken {
    // Kata kunci (RFC 0001)
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

    // Kata operator tercadang (addendum Tahap 1)
    Dan,
    Atau,
    Sebagai,

    // Pengenal dan literal
    GarisBawah,
    Pengenal(String),
    LiteralBulat(u64),
    LiteralDesimal(f64),
    LiteralTeks(Vec<BagianToken>),
    LiteralRune(char),

    // Operator
    Tambah,
    Kurang,
    Kali,
    Bagi,
    SisaBagi,
    SamaDengan,
    TambahSama,
    KurangSama,
    KaliSama,
    BagiSama,
    SisaSama,
    SamaKetat,
    TidakSama,
    KurangDari,
    KurangDariSama,
    LebihDari,
    LebihDariSama,
    BukanLogika,

    // Tanda baca
    TitikKoma,
    Titik,
    TitikDua,
    Koma,
    TitikTitik,
    TitikTitikSama,
    KurungBuka,
    KurungTutup,
    KurungKurawalBuka,
    KurungKurawalTutup,
    KurungSikuBuka,
    KurungSikuTutup,

    AkhirBerkas,
}

/// Bagian dari literal teks: potongan teks biasa atau token ekspresi interpolasi.
#[derive(Debug, Clone, PartialEq)]
pub enum BagianToken {
    Teks(String),
    Ekspresi(Vec<Token>),
}

impl JenisToken {
    /// Simbol/tulisan baku token (kosong untuk token berdata seperti pengenal dan literal).
    pub fn simbol(&self) -> &'static str {
        use JenisToken::*;
        match self {
            Paket => "paket",
            Impor => "impor",
            Fungsi => "fungsi",
            Kembali => "kembali",
            Biar => "biar",
            Var => "var",
            Konst => "konst",
            Jika => "jika",
            Lainnya => "lainnya",
            Selagi => "selagi",
            Untuk => "untuk",
            Dalam => "dalam",
            Pilih => "pilih",
            Kasus => "kasus",
            Cocok => "cocok",
            Tipe => "tipe",
            Struktur => "struktur",
            Antarmuka => "antarmuka",
            Enum => "enum",
            Jalankan => "jalankan",
            Saluran => "saluran",
            Tunda => "tunda",
            Benar => "benar",
            Salah => "salah",
            Kosong => "kosong",
            Pinjam => "pinjam",
            Pindah => "pindah",
            Publik => "publik",
            Pribadi => "pribadi",
            TidakAman => "tidak_aman",
            Dan => "dan",
            Atau => "atau",
            Sebagai => "sebagai",
            GarisBawah => "_",
            Pengenal(_) | LiteralBulat(_) | LiteralDesimal(_) | LiteralTeks(_)
            | LiteralRune(_) => "",
            Tambah => "+",
            Kurang => "-",
            Kali => "*",
            Bagi => "/",
            SisaBagi => "%",
            SamaDengan => "=",
            TambahSama => "+=",
            KurangSama => "-=",
            KaliSama => "*=",
            BagiSama => "/=",
            SisaSama => "%=",
            SamaKetat => "==",
            TidakSama => "!=",
            KurangDari => "<",
            KurangDariSama => "<=",
            LebihDari => ">",
            LebihDariSama => ">=",
            BukanLogika => "!",
            TitikKoma => ";",
            Titik => ".",
            TitikDua => ":",
            Koma => ",",
            TitikTitik => "..",
            TitikTitikSama => "..=",
            KurungBuka => "(",
            KurungTutup => ")",
            KurungKurawalBuka => "{",
            KurungKurawalTutup => "}",
            KurungSikuBuka => "[",
            KurungSikuTutup => "]",
            AkhirBerkas => "",
        }
    }

    /// True untuk 30 kata kunci RFC 0001 serta kata operator tercadang `dan`, `atau`, `sebagai`.
    pub fn adalah_kata_kunci(&self) -> bool {
        let s = self.simbol();
        !s.is_empty() && cocokkan_kata_kunci(s).is_some()
    }

    /// Deskripsi ramah untuk pesan galat, mis. `'fungsi'`, `pengenal 'x'`, `angka`.
    pub fn deskripsi(&self) -> String {
        match self {
            JenisToken::Pengenal(n) => format!("pengenal '{}'", n),
            JenisToken::LiteralBulat(_) | JenisToken::LiteralDesimal(_) => "angka".to_string(),
            JenisToken::LiteralTeks(_) => "teks".to_string(),
            JenisToken::LiteralRune(_) => "karakter".to_string(),
            JenisToken::AkhirBerkas => "akhir berkas".to_string(),
            lain => format!("'{}'", lain.simbol()),
        }
    }
}

/// Token lengkap dengan lokasi. `baris_baru_sebelum` bernilai true bila ada
/// pemisah baris (atau awal/akhir berkas) di antara token ini dan token sebelumnya;
/// parser memakainya untuk menentukan batas pernyataan tanpa titik koma.
#[derive(Debug, Clone, PartialEq)]
pub struct Token {
    pub jenis: JenisToken,
    pub rentang: Rentang,
    pub baris_baru_sebelum: bool,
}

/// Mencocokkan kata dengan kata kunci Komolang (termasuk `dan`, `atau`, `sebagai`).
pub fn cocokkan_kata_kunci(kata: &str) -> Option<JenisToken> {
    Some(match kata {
        "paket" => JenisToken::Paket,
        "impor" => JenisToken::Impor,
        "fungsi" => JenisToken::Fungsi,
        "kembali" => JenisToken::Kembali,
        "biar" => JenisToken::Biar,
        "var" => JenisToken::Var,
        "konst" => JenisToken::Konst,
        "jika" => JenisToken::Jika,
        "lainnya" => JenisToken::Lainnya,
        "selagi" => JenisToken::Selagi,
        "untuk" => JenisToken::Untuk,
        "dalam" => JenisToken::Dalam,
        "pilih" => JenisToken::Pilih,
        "kasus" => JenisToken::Kasus,
        "cocok" => JenisToken::Cocok,
        "tipe" => JenisToken::Tipe,
        "struktur" => JenisToken::Struktur,
        "antarmuka" => JenisToken::Antarmuka,
        "enum" => JenisToken::Enum,
        "jalankan" => JenisToken::Jalankan,
        "saluran" => JenisToken::Saluran,
        "tunda" => JenisToken::Tunda,
        "benar" => JenisToken::Benar,
        "salah" => JenisToken::Salah,
        "kosong" => JenisToken::Kosong,
        "pinjam" => JenisToken::Pinjam,
        "pindah" => JenisToken::Pindah,
        "publik" => JenisToken::Publik,
        "pribadi" => JenisToken::Pribadi,
        "tidak_aman" => JenisToken::TidakAman,
        "dan" => JenisToken::Dan,
        "atau" => JenisToken::Atau,
        "sebagai" => JenisToken::Sebagai,
        _ => return None,
    })
}

/// Hasil pemindaian: token (selalu diakhiri `AkhirBerkas`) dan diagnostik leksikal.
#[derive(Debug, Clone)]
pub struct HasilPindai {
    pub token: Vec<Token>,
    pub diagnostik: Vec<Diagnostik>,
}

/// Memindai seluruh kode sumber. Tidak pernah panik; kesalahan menjadi diagnostik.
pub fn pindai(sumber: &str) -> HasilPindai {
    let mut pemindai = Pemindai::baru(sumber);
    let mut token = pemindai.pindai_deret();
    token.push(Token {
        jenis: JenisToken::AkhirBerkas,
        rentang: Rentang::baru(pemindai.pos, pemindai.pos, pemindai.baris, pemindai.kolom),
        baris_baru_sebelum: true,
    });
    HasilPindai {
        token,
        diagnostik: pemindai.diagnostik,
    }
}

struct Pemindai<'a> {
    sumber: &'a str,
    pos: usize,
    akhir: usize,
    baris: usize,
    kolom: usize,
    diagnostik: Vec<Diagnostik>,
}

impl<'a> Pemindai<'a> {
    fn baru(sumber: &'a str) -> Self {
        Self {
            sumber,
            pos: 0,
            akhir: sumber.len(),
            baris: 1,
            kolom: 1,
            diagnostik: Vec::new(),
        }
    }

    fn intip(&self) -> Option<char> {
        self.sumber[self.pos..self.akhir].chars().next()
    }

    fn intip_ke(&self, n: usize) -> Option<char> {
        self.sumber[self.pos..self.akhir].chars().nth(n)
    }

    fn maju(&mut self) -> Option<char> {
        let c = self.intip()?;
        self.pos += c.len_utf8();
        if c == '\n' {
            self.baris += 1;
            self.kolom = 1;
        } else {
            self.kolom += 1;
        }
        Some(c)
    }

    fn lapor(&mut self, kode: &'static str, pesan: impl Into<String>, rentang: Rentang) {
        self.diagnostik.push(Diagnostik::galat(kode, pesan, rentang));
    }

    fn lapor_saran(
        &mut self,
        kode: &'static str,
        pesan: impl Into<String>,
        rentang: Rentang,
        saran: &str,
    ) {
        self.diagnostik
            .push(Diagnostik::galat(kode, pesan, rentang).dengan_saran(saran));
    }

    /// Memindai token-token hingga `self.akhir`.
    fn pindai_deret(&mut self) -> Vec<Token> {
        let mut hasil = Vec::new();
        loop {
            let baris_baru = self.lewati_spasi_dan_komentar();
            if self.pos >= self.akhir {
                break;
            }
            if let Some(mut token) = self.pindai_satu() {
                token.baris_baru_sebelum = baris_baru;
                hasil.push(token);
            }
        }
        hasil
    }

    /// Melewati spasi dan komentar; mengembalikan true bila ada pemisah baris.
    fn lewati_spasi_dan_komentar(&mut self) -> bool {
        let mut baris_baru = false;
        loop {
            match self.intip() {
                Some(c) if c.is_whitespace() => {
                    if c == '\n' {
                        baris_baru = true;
                    }
                    self.maju();
                }
                Some('/') if self.intip_ke(1) == Some('/') => {
                    while let Some(c) = self.intip() {
                        if c == '\n' {
                            break;
                        }
                        self.maju();
                    }
                }
                Some('/') if self.intip_ke(1) == Some('*') => {
                    if self.lewati_komentar_blok() {
                        baris_baru = true;
                    }
                }
                _ => break,
            }
        }
        baris_baru
    }

    /// Melewati komentar blok bersarang; true bila mengandung baris baru.
    fn lewati_komentar_blok(&mut self) -> bool {
        let (awal, baris, kolom) = (self.pos, self.baris, self.kolom);
        self.maju();
        self.maju();
        let mut kedalaman = 1usize;
        let mut ada_baris = false;
        while kedalaman > 0 {
            match self.intip() {
                None => {
                    self.lapor_saran(
                        kode::KOMENTAR_TIDAK_DITUTUP,
                        "komentar blok tidak ditutup",
                        Rentang::baru(awal, awal + 2, baris, kolom),
                        "tutup komentar dengan '*/'",
                    );
                    return ada_baris;
                }
                Some('/') if self.intip_ke(1) == Some('*') => {
                    self.maju();
                    self.maju();
                    kedalaman += 1;
                }
                Some('*') if self.intip_ke(1) == Some('/') => {
                    self.maju();
                    self.maju();
                    kedalaman -= 1;
                }
                Some(c) => {
                    if c == '\n' {
                        ada_baris = true;
                    }
                    self.maju();
                }
            }
        }
        ada_baris
    }

    fn pindai_satu(&mut self) -> Option<Token> {
        let (awal, baris, kolom) = (self.pos, self.baris, self.kolom);
        let c = self.intip()?;
        let jenis = if c.is_ascii_alphabetic() || c == '_' {
            self.pindai_pengenal()
        } else if c.is_ascii_digit() {
            self.pindai_angka()
        } else if c == '"' {
            self.pindai_teks()
        } else if c == '\'' {
            self.pindai_rune()
        } else if let Some(j) = self.pindai_operator() {
            j
        } else {
            self.maju();
            let rentang = Rentang::baru(awal, self.pos, baris, kolom);
            self.lapor_saran(
                kode::KARAKTER_TAK_DIKENALI,
                format!("karakter '{}' tidak dikenali", c),
                rentang,
                "hapus karakter tersebut; pengenal Komolang hanya memakai huruf a-z, A-Z, angka, dan '_'",
            );
            return None;
        };
        Some(Token {
            jenis,
            rentang: Rentang::baru(awal, self.pos, baris, kolom),
            baris_baru_sebelum: false,
        })
    }

    fn pindai_pengenal(&mut self) -> JenisToken {
        let sumber = self.sumber;
        let awal = self.pos;
        while let Some(c) = self.intip() {
            if c.is_ascii_alphanumeric() || c == '_' {
                self.maju();
            } else {
                break;
            }
        }
        let kata = &sumber[awal..self.pos];
        if kata == "_" {
            return JenisToken::GarisBawah;
        }
        cocokkan_kata_kunci(kata).unwrap_or_else(|| JenisToken::Pengenal(kata.to_string()))
    }

    fn pindai_operator(&mut self) -> Option<JenisToken> {
        use JenisToken::*;
        let c = self.intip()?;
        let c2 = self.intip_ke(1);
        let c3 = self.intip_ke(2);
        let (jenis, panjang) = match (c, c2, c3) {
            ('.', Some('.'), Some('=')) => (TitikTitikSama, 3),
            ('.', Some('.'), _) => (TitikTitik, 2),
            ('.', _, _) => (Titik, 1),
            ('+', Some('='), _) => (TambahSama, 2),
            ('+', _, _) => (Tambah, 1),
            ('-', Some('='), _) => (KurangSama, 2),
            ('-', _, _) => (Kurang, 1),
            ('*', Some('='), _) => (KaliSama, 2),
            ('*', _, _) => (Kali, 1),
            ('/', Some('='), _) => (BagiSama, 2),
            ('/', _, _) => (Bagi, 1),
            ('%', Some('='), _) => (SisaSama, 2),
            ('%', _, _) => (SisaBagi, 1),
            ('=', Some('='), _) => (SamaKetat, 2),
            ('=', _, _) => (SamaDengan, 1),
            ('!', Some('='), _) => (TidakSama, 2),
            ('!', _, _) => (BukanLogika, 1),
            ('<', Some('='), _) => (KurangDariSama, 2),
            ('<', _, _) => (KurangDari, 1),
            ('>', Some('='), _) => (LebihDariSama, 2),
            ('>', _, _) => (LebihDari, 1),
            (':', _, _) => (TitikDua, 1),
            (',', _, _) => (Koma, 1),
            (';', _, _) => (TitikKoma, 1),
            ('(', _, _) => (KurungBuka, 1),
            (')', _, _) => (KurungTutup, 1),
            ('{', _, _) => (KurungKurawalBuka, 1),
            ('}', _, _) => (KurungKurawalTutup, 1),
            ('[', _, _) => (KurungSikuBuka, 1),
            (']', _, _) => (KurungSikuTutup, 1),
            _ => return None,
        };
        for _ in 0..panjang {
            self.maju();
        }
        Some(jenis)
    }

    fn pindai_angka(&mut self) -> JenisToken {
        let sumber = self.sumber;
        let (awal, baris, kolom) = (self.pos, self.baris, self.kolom);

        let radix = if self.intip() == Some('0') {
            match self.intip_ke(1) {
                Some('x') | Some('X') => 16,
                Some('b') | Some('B') => 2,
                Some('o') | Some('O') => 8,
                _ => 10,
            }
        } else {
            10
        };

        if radix != 10 {
            self.maju();
            self.maju();
            let mulai = self.pos;
            while let Some(c) = self.intip() {
                if c.is_ascii_alphanumeric() || c == '_' {
                    self.maju();
                } else {
                    break;
                }
            }
            let digit: String = sumber[mulai..self.pos]
                .chars()
                .filter(|c| *c != '_')
                .collect();
            let rentang = Rentang::baru(awal, self.pos, baris, kolom);
            if digit.is_empty() {
                self.lapor_saran(
                    kode::ANGKA_TIDAK_VALID,
                    "angka tidak lengkap: tidak ada digit setelah awalan",
                    rentang,
                    "contoh: 0xFF, 0b1010, 0o755",
                );
                return JenisToken::LiteralBulat(0);
            }
            return match u64::from_str_radix(&digit, radix) {
                Ok(n) => JenisToken::LiteralBulat(n),
                Err(_) => {
                    self.lapor(
                        kode::ANGKA_TIDAK_VALID,
                        format!(
                            "angka basis {} tidak valid: berisi digit yang salah atau terlalu besar",
                            radix
                        ),
                        rentang,
                    );
                    JenisToken::LiteralBulat(0)
                }
            };
        }

        let mut desimal = false;
        self.lewati_digit();
        if self.intip() == Some('.') && matches!(self.intip_ke(1), Some(c) if c.is_ascii_digit()) {
            self.maju();
            self.lewati_digit();
            desimal = true;
        }
        if matches!(self.intip(), Some('e') | Some('E')) {
            let ada_eksponen = match self.intip_ke(1) {
                Some(c) if c.is_ascii_digit() => true,
                Some('+') | Some('-') => matches!(self.intip_ke(2), Some(c) if c.is_ascii_digit()),
                _ => false,
            };
            if ada_eksponen {
                self.maju();
                if matches!(self.intip(), Some('+') | Some('-')) {
                    self.maju();
                }
                self.lewati_digit();
                desimal = true;
            }
        }
        let teks: String = sumber[awal..self.pos]
            .chars()
            .filter(|c| *c != '_')
            .collect();

        if matches!(self.intip(), Some(c) if c.is_ascii_alphabetic() || c == '_') {
            while let Some(c) = self.intip() {
                if c.is_ascii_alphanumeric() || c == '_' {
                    self.maju();
                } else {
                    break;
                }
            }
            let rentang = Rentang::baru(awal, self.pos, baris, kolom);
            self.lapor_saran(
                kode::ANGKA_TIDAK_VALID,
                "angka tidak valid: ada huruf menempel di belakang angka",
                rentang,
                "pisahkan angka dari nama, atau periksa awalan basis (0x, 0b, 0o)",
            );
            return JenisToken::LiteralBulat(0);
        }

        let rentang = Rentang::baru(awal, self.pos, baris, kolom);
        if desimal {
            match teks.parse::<f64>() {
                Ok(f) if f.is_finite() => JenisToken::LiteralDesimal(f),
                _ => {
                    self.lapor(
                        kode::ANGKA_TIDAK_VALID,
                        "angka desimal tidak valid atau di luar jangkauan",
                        rentang,
                    );
                    JenisToken::LiteralDesimal(0.0)
                }
            }
        } else {
            match teks.parse::<u64>() {
                Ok(n) => JenisToken::LiteralBulat(n),
                Err(_) => {
                    self.lapor_saran(
                        kode::ANGKA_TIDAK_VALID,
                        "angka bulat terlalu besar (maksimum 18446744073709551615)",
                        rentang,
                        "gunakan angka yang lebih kecil",
                    );
                    JenisToken::LiteralBulat(0)
                }
            }
        }
    }

    fn lewati_digit(&mut self) {
        while let Some(c) = self.intip() {
            if c.is_ascii_digit() || c == '_' {
                self.maju();
            } else {
                break;
            }
        }
    }

    /// Membaca urutan escape yang diawali `\`. Mengembalikan None (disertai diagnostik) bila tidak valid.
    fn baca_escape(&mut self) -> Option<char> {
        let (awal, baris, kolom) = (self.pos, self.baris, self.kolom);
        self.maju();
        let c = match self.intip() {
            Some(c) => c,
            None => {
                self.lapor(
                    kode::ESCAPE_TIDAK_VALID,
                    "urutan escape tidak lengkap di akhir berkas",
                    Rentang::baru(awal, self.pos, baris, kolom),
                );
                return None;
            }
        };
        let nilai = match c {
            'n' => Some('\n'),
            't' => Some('\t'),
            'r' => Some('\r'),
            '0' => Some('\0'),
            '\\' => Some('\\'),
            '"' => Some('"'),
            '\'' => Some('\''),
            'u' => {
                self.maju();
                return self.baca_unicode(awal, baris, kolom);
            }
            _ => None,
        };
        self.maju();
        if nilai.is_none() {
            self.lapor_saran(
                kode::ESCAPE_TIDAK_VALID,
                format!("urutan escape '\\{}' tidak dikenal", c),
                Rentang::baru(awal, self.pos, baris, kolom),
                "escape yang tersedia: \\n \\t \\r \\0 \\\\ \\\" \\' dan \\u{HHHH}",
            );
        }
        nilai
    }

    fn baca_unicode(&mut self, awal: usize, baris: usize, kolom: usize) -> Option<char> {
        let sumber = self.sumber;
        if self.intip() != Some('{') {
            self.lapor_saran(
                kode::ESCAPE_TIDAK_VALID,
                "diharapkan '{' setelah \\u",
                Rentang::baru(awal, self.pos, baris, kolom),
                "bentuk yang benar: \\u{1F525}",
            );
            return None;
        }
        self.maju();
        let mulai = self.pos;
        while let Some(c) = self.intip() {
            if c.is_ascii_hexdigit() {
                self.maju();
            } else {
                break;
            }
        }
        let hex = &sumber[mulai..self.pos];
        let tutup = self.intip() == Some('}');
        if tutup {
            self.maju();
        }
        let rentang = Rentang::baru(awal, self.pos, baris, kolom);
        if !tutup || hex.is_empty() || hex.len() > 6 {
            self.lapor_saran(
                kode::ESCAPE_TIDAK_VALID,
                "escape unicode tidak valid",
                rentang,
                "gunakan 1 sampai 6 digit heksadesimal: \\u{41}",
            );
            return None;
        }
        match u32::from_str_radix(hex, 16).ok().and_then(char::from_u32) {
            Some(c) => Some(c),
            None => {
                self.lapor(
                    kode::ESCAPE_TIDAK_VALID,
                    "titik kode unicode tidak valid",
                    rentang,
                );
                None
            }
        }
    }

    fn pindai_rune(&mut self) -> JenisToken {
        let (awal, baris, kolom) = (self.pos, self.baris, self.kolom);
        self.maju();
        let nilai = match self.intip() {
            None | Some('\n') => {
                self.lapor(
                    kode::RUNE_TIDAK_VALID,
                    "literal karakter tidak ditutup",
                    Rentang::baru(awal, self.pos, baris, kolom),
                );
                return JenisToken::LiteralRune('\0');
            }
            Some('\'') => {
                self.maju();
                self.lapor_saran(
                    kode::RUNE_TIDAK_VALID,
                    "literal karakter kosong",
                    Rentang::baru(awal, self.pos, baris, kolom),
                    "tulis tepat satu karakter, mis. 'a'",
                );
                return JenisToken::LiteralRune('\0');
            }
            Some('\\') => self.baca_escape().unwrap_or('\0'),
            Some(c) => {
                self.maju();
                c
            }
        };
        if self.intip() == Some('\'') {
            self.maju();
        } else {
            while let Some(c) = self.intip() {
                if c == '\n' {
                    break;
                }
                self.maju();
                if c == '\'' {
                    break;
                }
            }
            self.lapor_saran(
                kode::RUNE_TIDAK_VALID,
                "literal karakter harus berisi tepat satu karakter dan ditutup dengan '",
                Rentang::baru(awal, self.pos, baris, kolom),
                "untuk teks yang lebih panjang gunakan tanda petik ganda: \"...\"",
            );
        }
        JenisToken::LiteralRune(nilai)
    }

    fn pindai_teks(&mut self) -> JenisToken {
        let (awal, baris, kolom) = (self.pos, self.baris, self.kolom);
        self.maju();
        let mut bagian: Vec<BagianToken> = Vec::new();
        let mut buf = String::new();
        let mut ditutup = false;
        while let Some(c) = self.intip() {
            match c {
                '"' => {
                    self.maju();
                    ditutup = true;
                    break;
                }
                '\\' => {
                    if let Some(ch) = self.baca_escape() {
                        buf.push(ch);
                    }
                }
                '{' => {
                    if self.intip_ke(1) == Some('{') {
                        self.maju();
                        self.maju();
                        buf.push('{');
                    } else {
                        self.pindai_interpolasi(&mut bagian, &mut buf);
                    }
                }
                '}' => {
                    if self.intip_ke(1) == Some('}') {
                        self.maju();
                        self.maju();
                        buf.push('}');
                    } else {
                        let (a, b, k) = (self.pos, self.baris, self.kolom);
                        self.maju();
                        self.lapor_saran(
                            kode::KURUNG_KURAWAL_TUNGGAL,
                            "'}' tunggal di dalam teks",
                            Rentang::baru(a, self.pos, b, k),
                            "gunakan '}}' untuk menulis kurung kurawal tutup secara harfiah",
                        );
                    }
                }
                _ => {
                    buf.push(c);
                    self.maju();
                }
            }
        }
        if !ditutup {
            self.lapor_saran(
                kode::TEKS_TIDAK_DITUTUP,
                "teks tidak ditutup",
                Rentang::baru(awal, self.pos, baris, kolom),
                "tambahkan tanda petik ganda \" untuk menutup teks",
            );
        }
        if !buf.is_empty() || bagian.is_empty() {
            bagian.push(BagianToken::Teks(buf));
        }
        JenisToken::LiteralTeks(bagian)
    }

    fn pindai_interpolasi(&mut self, bagian: &mut Vec<BagianToken>, buf: &mut String) {
        let (awal, baris, kolom) = (self.pos, self.baris, self.kolom);
        match self.cari_penutup(self.pos + 1) {
            None => {
                self.maju();
                self.lapor_saran(
                    kode::INTERPOLASI_TIDAK_DITUTUP,
                    "interpolasi '{' tidak ditutup dengan '}'",
                    Rentang::baru(awal, awal + 1, baris, kolom),
                    "gunakan '{{' untuk menulis kurung kurawal buka secara harfiah",
                );
            }
            Some(akhir_dalam) => {
                if !buf.is_empty() {
                    bagian.push(BagianToken::Teks(std::mem::take(buf)));
                }
                self.maju();
                let akhir_lama = self.akhir;
                self.akhir = akhir_dalam;
                let token = self.pindai_deret();
                self.akhir = akhir_lama;
                self.maju();
                if token.is_empty() {
                    self.lapor_saran(
                        kode::INTERPOLASI_KOSONG,
                        "interpolasi kosong: tidak ada ekspresi di antara '{' dan '}'",
                        Rentang::baru(awal, self.pos, baris, kolom),
                        "tulis ekspresi, mis. {nama}, atau gunakan '{{}}' untuk kurung harfiah",
                    );
                } else {
                    bagian.push(BagianToken::Ekspresi(token));
                }
            }
        }
    }

    /// Mencari `}` penutup interpolasi mulai indeks byte `i` (tepat setelah `{`),
    /// memperhitungkan kurung bersarang, teks bersarang, dan literal karakter.
    fn cari_penutup(&self, mut i: usize) -> Option<usize> {
        let b = self.sumber.as_bytes();
        let batas = self.akhir;
        let mut kedalaman = 1usize;
        while i < batas {
            match b[i] {
                b'"' => {
                    i = self.lewati_teks(i + 1)?;
                    continue;
                }
                b'\'' => {
                    let mulai = if b.get(i + 1) == Some(&b'\\') {
                        i + 3
                    } else {
                        i + 2
                    };
                    let tutup = (mulai..batas.min(mulai + 8))
                        .take_while(|&j| b[j] != b'\n')
                        .find(|&j| b[j] == b'\'');
                    if let Some(j) = tutup {
                        i = j + 1;
                        continue;
                    }
                }
                b'{' => kedalaman += 1,
                b'}' => {
                    kedalaman -= 1;
                    if kedalaman == 0 {
                        return Some(i);
                    }
                }
                _ => {}
            }
            i += 1;
        }
        None
    }

    /// Melewati isi teks bersarang mulai `i` (setelah tanda petik pembuka);
    /// mengembalikan indeks tepat setelah tanda petik penutup.
    fn lewati_teks(&self, mut i: usize) -> Option<usize> {
        let b = self.sumber.as_bytes();
        let batas = self.akhir;
        while i < batas {
            match b[i] {
                b'\\' => i += 2,
                b'"' => return Some(i + 1),
                b'{' => {
                    if b.get(i + 1) == Some(&b'{') {
                        i += 2;
                    } else {
                        i = self.cari_penutup(i + 1)? + 1;
                    }
                }
                _ => i += 1,
            }
        }
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn jenis(src: &str) -> Vec<JenisToken> {
        let h = pindai(src);
        assert!(h.diagnostik.is_empty(), "diagnostik: {:?}", h.diagnostik);
        h.token
            .into_iter()
            .map(|t| t.jenis)
            .filter(|j| *j != JenisToken::AkhirBerkas)
            .collect()
    }

    fn kode_galat(src: &str) -> Vec<&'static str> {
        pindai(src).diagnostik.iter().map(|d| d.kode).collect()
    }

    fn id(n: &str) -> JenisToken {
        JenisToken::Pengenal(n.to_string())
    }

    #[test]
    fn tes_kata_kunci_indonesia() {
        let semua = [
            "paket", "impor", "fungsi", "kembali", "biar", "var", "konst", "jika", "lainnya",
            "selagi", "untuk", "dalam", "pilih", "kasus", "cocok", "tipe", "struktur",
            "antarmuka", "enum", "jalankan", "saluran", "tunda", "benar", "salah", "kosong",
            "pinjam", "pindah", "publik", "pribadi", "tidak_aman", "dan", "atau", "sebagai",
        ];
        for kata in semua {
            let j = cocokkan_kata_kunci(kata).unwrap_or_else(|| panic!("{kata} bukan kata kunci"));
            assert_eq!(j.simbol(), kata);
            assert!(j.adalah_kata_kunci());
            assert_eq!(jenis(kata), vec![j]);
        }
    }

    #[test]
    fn tes_bukan_kata_kunci() {
        assert!(cocokkan_kata_kunci("bulat").is_none());
        assert!(cocokkan_kata_kunci("teks").is_none());
        assert!(cocokkan_kata_kunci("Paket").is_none());
        assert_eq!(jenis("paketx"), vec![id("paketx")]);
    }

    #[test]
    fn tes_pengenal_dan_garis_bawah() {
        assert_eq!(jenis("a _x x_1 _"), vec![id("a"), id("_x"), id("x_1"), JenisToken::GarisBawah]);
    }

    #[test]
    fn tes_angka_bulat() {
        assert_eq!(jenis("0 42 1_000"), vec![
            JenisToken::LiteralBulat(0),
            JenisToken::LiteralBulat(42),
            JenisToken::LiteralBulat(1000),
        ]);
    }

    #[test]
    fn tes_angka_basis_lain() {
        assert_eq!(jenis("0xFF 0x1A_3F 0b1010_0110 0o755"), vec![
            JenisToken::LiteralBulat(255),
            JenisToken::LiteralBulat(0x1A3F),
            JenisToken::LiteralBulat(0b1010_0110),
            JenisToken::LiteralBulat(0o755),
        ]);
    }

    #[test]
    fn tes_angka_desimal_dan_eksponen() {
        assert_eq!(jenis("3.14 1.0e-5 6.022e23 6e2 2E+3"), vec![
            JenisToken::LiteralDesimal(3.14),
            JenisToken::LiteralDesimal(1.0e-5),
            JenisToken::LiteralDesimal(6.022e23),
            JenisToken::LiteralDesimal(600.0),
            JenisToken::LiteralDesimal(2000.0),
        ]);
    }

    #[test]
    fn tes_rentang_bukan_desimal() {
        assert_eq!(jenis("0..5"), vec![
            JenisToken::LiteralBulat(0),
            JenisToken::TitikTitik,
            JenisToken::LiteralBulat(5),
        ]);
        assert_eq!(jenis("0..=5"), vec![
            JenisToken::LiteralBulat(0),
            JenisToken::TitikTitikSama,
            JenisToken::LiteralBulat(5),
        ]);
    }

    #[test]
    fn tes_bulat_maksimum_u64() {
        assert_eq!(jenis("18446744073709551615"), vec![JenisToken::LiteralBulat(u64::MAX)]);
        assert_eq!(kode_galat("18446744073709551616"), vec![kode::ANGKA_TIDAK_VALID]);
    }

    #[test]
    fn tes_teks_sederhana_dan_kosong() {
        assert_eq!(jenis("\"halo\""), vec![JenisToken::LiteralTeks(vec![BagianToken::Teks("halo".into())])]);
        assert_eq!(jenis("\"\""), vec![JenisToken::LiteralTeks(vec![BagianToken::Teks(String::new())])]);
    }

    #[test]
    fn tes_teks_escape() {
        assert_eq!(
            jenis(r#""a\n\t\"\\\0\u{41}\u{1F525}""#),
            vec![JenisToken::LiteralTeks(vec![BagianToken::Teks(
                "a\n\t\"\\\0A🔥".into()
            )])]
        );
    }

    #[test]
    fn tes_teks_kurung_harfiah() {
        assert_eq!(
            jenis("\"{{x}}\""),
            vec![JenisToken::LiteralTeks(vec![BagianToken::Teks("{x}".into())])]
        );
    }

    #[test]
    fn tes_teks_interpolasi_struktur_token() {
        let t = jenis("\"a {b + 1} c\"");
        match &t[0] {
            JenisToken::LiteralTeks(bagian) => {
                assert_eq!(bagian.len(), 3);
                assert_eq!(bagian[0], BagianToken::Teks("a ".into()));
                match &bagian[1] {
                    BagianToken::Ekspresi(tok) => {
                        assert_eq!(tok.len(), 3);
                        assert_eq!(tok[0].jenis, id("b"));
                        assert_eq!(tok[1].jenis, JenisToken::Tambah);
                    }
                    lain => panic!("bukan ekspresi: {:?}", lain),
                }
                assert_eq!(bagian[2], BagianToken::Teks(" c".into()));
            }
            lain => panic!("bukan teks: {:?}", lain),
        }
    }

    #[test]
    fn tes_interpolasi_bersarang() {
        let t = jenis(r#""x {f("y {z}")} w""#);
        assert!(matches!(&t[0], JenisToken::LiteralTeks(b) if b.len() == 3));
    }

    #[test]
    fn tes_teks_unicode_dan_multibaris() {
        let t = jenis("\"Halo 🔥 ü\nbaris dua\"");
        assert!(matches!(&t[0], JenisToken::LiteralTeks(b) if b.len() == 1));
    }

    #[test]
    fn tes_rune() {
        assert_eq!(jenis(r"'a' '\n' '\'' '🔥' '\u{41}'"), vec![
            JenisToken::LiteralRune('a'),
            JenisToken::LiteralRune('\n'),
            JenisToken::LiteralRune('\''),
            JenisToken::LiteralRune('🔥'),
            JenisToken::LiteralRune('A'),
        ]);
    }

    #[test]
    fn tes_operator_satu_dan_dua_karakter() {
        use JenisToken::*;
        assert_eq!(
            jenis("+ - * / % = += -= *= /= %= == != < <= > >= !"),
            vec![
                Tambah, Kurang, Kali, Bagi, SisaBagi, SamaDengan, TambahSama, KurangSama, KaliSama,
                BagiSama, SisaSama, SamaKetat, TidakSama, KurangDari, KurangDariSama, LebihDari,
                LebihDariSama, BukanLogika
            ]
        );
    }

    #[test]
    fn tes_tanda_baca() {
        use JenisToken::*;
        assert_eq!(
            jenis("; . : , .. ..= ( ) { } [ ]"),
            vec![
                TitikKoma, Titik, TitikDua, Koma, TitikTitik, TitikTitikSama, KurungBuka,
                KurungTutup, KurungKurawalBuka, KurungKurawalTutup, KurungSikuBuka, KurungSikuTutup
            ]
        );
    }

    #[test]
    fn tes_generik_tidak_menjadi_geser() {
        use JenisToken::*;
        assert_eq!(
            jenis("a<b<c>>"),
            vec![id("a"), KurangDari, id("b"), KurangDari, id("c"), LebihDari, LebihDari]
        );
    }

    #[test]
    fn tes_komentar_sebaris_dan_blok() {
        assert_eq!(jenis("a // komentar\nb /* x */ c"), vec![id("a"), id("b"), id("c")]);
    }

    #[test]
    fn tes_komentar_blok_bersarang() {
        assert_eq!(jenis("a /* x /* y */ z */ b"), vec![id("a"), id("b")]);
    }

    #[test]
    fn tes_komentar_unicode() {
        assert_eq!(jenis("a // 🔥 ü\nb"), vec![id("a"), id("b")]);
    }

    #[test]
    fn tes_lokasi_baris_dan_kolom() {
        let h = pindai("ab\n  cd");
        assert_eq!((h.token[0].rentang.baris, h.token[0].rentang.kolom), (1, 1));
        assert_eq!((h.token[1].rentang.baris, h.token[1].rentang.kolom), (2, 3));
        assert_eq!(h.token[1].rentang.awal, 5);
        assert_eq!(h.token[1].rentang.akhir, 7);
    }

    #[test]
    fn tes_kolom_dihitung_karakter() {
        let h = pindai("\"🔥\" x");
        assert_eq!(h.token[1].rentang.kolom, 5);
    }

    #[test]
    fn tes_penanda_baris_baru() {
        let h = pindai("a b\nc");
        assert!(!h.token[1].baris_baru_sebelum);
        assert!(h.token[2].baris_baru_sebelum);
        assert!(h.token.last().unwrap().baris_baru_sebelum);
    }

    #[test]
    fn tes_komentar_blok_multibaris_menandai_baris_baru() {
        let h = pindai("a /* x\ny */ b");
        assert!(h.token[1].baris_baru_sebelum);
    }

    #[test]
    fn tes_selalu_diakhiri_akhir_berkas() {
        for src in ["", "   ", "// x", "a"] {
            let h = pindai(src);
            assert_eq!(h.token.last().unwrap().jenis, JenisToken::AkhirBerkas);
        }
    }

    #[test]
    fn tes_pemulihan_karakter_tak_dikenal() {
        let h = pindai("a # b");
        assert_eq!(h.diagnostik.len(), 1);
        assert_eq!(h.diagnostik[0].kode, kode::KARAKTER_TAK_DIKENALI);
        let nama: Vec<_> = h.token.iter().map(|t| t.jenis.clone()).collect();
        assert_eq!(nama, vec![id("a"), id("b"), JenisToken::AkhirBerkas]);
    }

    #[test]
    fn tes_semua_galat_leksikal_dilaporkan() {
        let h = pindai("# @ $");
        assert_eq!(h.diagnostik.len(), 3);
    }

    #[test]
    fn tes_kode_galat_leksikal() {
        assert_eq!(kode_galat("\"abc"), vec![kode::TEKS_TIDAK_DITUTUP]);
        assert_eq!(kode_galat("/* x"), vec![kode::KOMENTAR_TIDAK_DITUTUP]);
        assert_eq!(kode_galat(r#""\q""#), vec![kode::ESCAPE_TIDAK_VALID]);
        assert_eq!(kode_galat("0x"), vec![kode::ANGKA_TIDAK_VALID]);
        assert_eq!(kode_galat("12abc"), vec![kode::ANGKA_TIDAK_VALID]);
        assert_eq!(kode_galat("\"a } b\""), vec![kode::KURUNG_KURAWAL_TUNGGAL]);
        assert_eq!(kode_galat("\"a {} b\""), vec![kode::INTERPOLASI_KOSONG]);
        assert_eq!(kode_galat("\"a {b\""), vec![kode::INTERPOLASI_TIDAK_DITUTUP]);
        assert_eq!(kode_galat("''"), vec![kode::RUNE_TIDAK_VALID]);
        assert_eq!(kode_galat("'ab'"), vec![kode::RUNE_TIDAK_VALID]);
    }

    #[test]
    fn tes_deskripsi_token() {
        assert_eq!(JenisToken::Fungsi.deskripsi(), "'fungsi'");
        assert_eq!(id("x").deskripsi(), "pengenal 'x'");
        assert_eq!(JenisToken::LiteralBulat(1).deskripsi(), "angka");
        assert_eq!(JenisToken::AkhirBerkas.deskripsi(), "akhir berkas");
        assert!(!id("x").adalah_kata_kunci());
    }

    #[test]
    fn tes_rentang_token_valid_pada_sumber() {
        let src = "paket utama\nbiar s = \"Halo 🔥 {nama}\" // ü\n";
        let h = pindai(src);
        for t in &h.token {
            assert!(t.rentang.awal <= t.rentang.akhir);
            assert!(src.get(t.rentang.awal..t.rentang.akhir).is_some());
        }
    }
}
