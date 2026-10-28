// Copyright (c) kangpcode (Dhafa Nazula Permadi). Licensed under MIT or Apache-2.0.

//! `komo_parser` mengubah token dari `komo_lexer` menjadi AST `komo_ast`.
//!
//! Teknik: recursive descent untuk deklarasi/pernyataan dan Pratt parser untuk ekspresi.
//! Batas pernyataan ditentukan oleh baris baru (tanpa titik koma). Parser memulihkan diri
//! dari kesalahan (error recovery) sehingga satu galat tidak menutupi galat lainnya.

use komo_ast::*;
use komo_diag::{kode, Diagnostik};
use komo_lexer::{pindai, BagianToken, JenisToken, Token};

/// Batas kedalaman penyarangan agar masukan jahat tidak menyebabkan stack overflow.
const BATAS_KEDALAMAN: usize = 100;

/// Penanda galat; diagnostik sudah dicatat sebelum galat ini dikembalikan.
struct Galat;
type Hasil<T> = Result<T, Galat>;

/// Hasil penguraian satu berkas: AST (mungkin parsial bila ada galat) dan semua diagnostik.
#[derive(Debug, Clone)]
pub struct HasilUrai {
    pub berkas: BerkasProgram,
    pub diagnostik: Vec<Diagnostik>,
}

impl HasilUrai {
    pub fn berhasil(&self) -> bool {
        self.diagnostik.is_empty()
    }
}

/// Memindai dan mengurai kode sumber sekaligus. Diagnostik leksikal dan sintaksis digabung
/// dan diurutkan menurut posisi.
pub fn urai(sumber: &str) -> HasilUrai {
    let hasil = pindai(sumber);
    let mut pengurai = Pengurai::baru(hasil.token);
    let berkas = pengurai.urai_berkas();
    let mut diagnostik = hasil.diagnostik;
    diagnostik.extend(pengurai.diagnostik);
    diagnostik.sort_by_key(|d| d.rentang.awal);
    HasilUrai { berkas, diagnostik }
}

/// Mengurai satu ekspresi saja (berguna untuk alat bantu dan tes).
pub fn urai_ekspresi_sumber(sumber: &str) -> Result<Ekspresi, Vec<Diagnostik>> {
    let hasil = pindai(sumber);
    let mut diagnostik = hasil.diagnostik;
    let mut pengurai = Pengurai::baru(hasil.token);
    let ekspresi = pengurai.urai_ekspresi();
    if ekspresi.is_ok() && !pengurai.adalah_akhir() {
        let t = pengurai.sekarang().clone();
        diagnostik.push(Diagnostik::galat(
            kode::DIHARAPKAN_BARIS_BARU,
            format!(
                "token tak terduga setelah ekspresi: {}",
                t.jenis.deskripsi()
            ),
            t.rentang,
        ));
    }
    diagnostik.extend(pengurai.diagnostik);
    diagnostik.sort_by_key(|d| d.rentang.awal);
    match ekspresi {
        Ok(e) if diagnostik.is_empty() => Ok(e),
        _ => Err(diagnostik),
    }
}

/// Parser berbasis token.
pub struct Pengurai {
    tokens: Vec<Token>,
    pos: usize,
    diagnostik: Vec<Diagnostik>,
    /// False pada posisi kondisi (`jika x {`) agar `x {` tidak dibaca sebagai literal struktur.
    struktur_diizinkan: bool,
    /// True di dalam `()`/`[]`/`{}` literal: baris baru tidak memutus ekspresi.
    abaikan_baris_baru: bool,
    kedalaman: usize,
}

enum Infiks {
    Biner(OperatorBiner),
    Rentang { inklusif: bool },
}

fn infiks(j: &JenisToken) -> Option<(Infiks, u8)> {
    use JenisToken as T;
    use OperatorBiner as O;
    Some(match j {
        T::Atau => (Infiks::Biner(O::Atau), 1),
        T::Dan => (Infiks::Biner(O::Dan), 2),
        T::SamaKetat => (Infiks::Biner(O::Sama), 3),
        T::TidakSama => (Infiks::Biner(O::TidakSama), 3),
        T::KurangDari => (Infiks::Biner(O::LebihKecil), 3),
        T::KurangDariSama => (Infiks::Biner(O::LebihKecilSama), 3),
        T::LebihDari => (Infiks::Biner(O::LebihBesar), 3),
        T::LebihDariSama => (Infiks::Biner(O::LebihBesarSama), 3),
        T::TitikTitik => (Infiks::Rentang { inklusif: false }, 4),
        T::TitikTitikSama => (Infiks::Rentang { inklusif: true }, 4),
        T::Tambah => (Infiks::Biner(O::Tambah), 5),
        T::Kurang => (Infiks::Biner(O::Kurang), 5),
        T::Kali => (Infiks::Biner(O::Kali), 6),
        T::Bagi => (Infiks::Biner(O::Bagi), 6),
        T::SisaBagi => (Infiks::Biner(O::Sisa), 6),
        _ => return None,
    })
}

fn operator_tugas(j: &JenisToken) -> Option<OperatorTugas> {
    Some(match j {
        JenisToken::SamaDengan => OperatorTugas::Tetap,
        JenisToken::TambahSama => OperatorTugas::Tambah,
        JenisToken::KurangSama => OperatorTugas::Kurang,
        JenisToken::KaliSama => OperatorTugas::Kali,
        JenisToken::BagiSama => OperatorTugas::Bagi,
        JenisToken::SisaSama => OperatorTugas::Sisa,
        _ => return None,
    })
}

fn awal_pernyataan(j: &JenisToken) -> bool {
    matches!(
        j,
        JenisToken::Biar
            | JenisToken::Var
            | JenisToken::Jika
            | JenisToken::Selagi
            | JenisToken::Untuk
            | JenisToken::Pilih
            | JenisToken::Cocok
            | JenisToken::Tunda
            | JenisToken::Jalankan
            | JenisToken::Kembali
            | JenisToken::TidakAman
    )
}

fn awal_deklarasi(j: &JenisToken) -> bool {
    matches!(
        j,
        JenisToken::Fungsi
            | JenisToken::Tipe
            | JenisToken::Konst
            | JenisToken::Biar
            | JenisToken::Var
            | JenisToken::Publik
            | JenisToken::Pribadi
            | JenisToken::Impor
    )
}

/// Padanan kata kunci bahasa Inggris yang sering salah ketik oleh pemula.
fn padanan_inggris(nama: &str) -> Option<&'static str> {
    Some(match nama {
        "func" | "function" | "fn" => "fungsi",
        "let" => "biar",
        "const" => "konst",
        "if" => "jika",
        "else" => "lainnya",
        "while" => "selagi",
        "for" => "untuk",
        "in" => "dalam",
        "return" => "kembali",
        "import" => "impor",
        "package" => "paket",
        "struct" => "struktur",
        "interface" => "antarmuka",
        "switch" => "pilih",
        "case" => "kasus",
        "match" => "cocok",
        "true" => "benar",
        "false" => "salah",
        "null" | "nil" => "kosong",
        "defer" => "tunda",
        "go" | "spawn" => "jalankan",
        "type" => "tipe",
        "public" | "pub" => "publik",
        "private" => "pribadi",
        "unsafe" => "tidak_aman",
        "and" => "dan",
        "or" => "atau",
        "as" => "sebagai",
        "borrow" => "pinjam",
        "move" => "pindah",
        "chan" | "channel" => "saluran",
        _ => return None,
    })
}

fn saran_padanan(t: &Token) -> Option<String> {
    if let JenisToken::Pengenal(n) = &t.jenis {
        padanan_inggris(n)
            .map(|k| format!("gunakan kata kunci Komolang '{}' sebagai ganti '{}'", k, n))
    } else {
        None
    }
}

fn teks_polos(bagian: &[BagianToken]) -> Option<String> {
    let mut s = String::new();
    for b in bagian {
        match b {
            BagianToken::Teks(t) => s.push_str(t),
            BagianToken::Ekspresi(_) => return None,
        }
    }
    Some(s)
}

impl Pengurai {
    pub fn baru(mut tokens: Vec<Token>) -> Self {
        let perlu_akhir = !matches!(
            tokens.last().map(|t| &t.jenis),
            Some(JenisToken::AkhirBerkas)
        );
        if perlu_akhir {
            let rentang = tokens
                .last()
                .map(|t| {
                    Rentang::baru(
                        t.rentang.akhir,
                        t.rentang.akhir,
                        t.rentang.baris,
                        t.rentang.kolom,
                    )
                })
                .unwrap_or_else(Rentang::titik_kosong);
            tokens.push(Token {
                jenis: JenisToken::AkhirBerkas,
                rentang,
                baris_baru_sebelum: true,
            });
        }
        Self {
            tokens,
            pos: 0,
            diagnostik: Vec::new(),
            struktur_diizinkan: true,
            abaikan_baris_baru: false,
            kedalaman: 0,
        }
    }

    pub fn ambil_diagnostik(self) -> Vec<Diagnostik> {
        self.diagnostik
    }

    // -----------------------------------------------------------------------
    // Utilitas token
    // -----------------------------------------------------------------------

    fn sekarang(&self) -> &Token {
        &self.tokens[self.pos]
    }

    fn intip(&self, n: usize) -> &Token {
        let i = (self.pos + n).min(self.tokens.len() - 1);
        &self.tokens[i]
    }

    fn maju(&mut self) -> Token {
        let t = self.tokens[self.pos].clone();
        if self.pos + 1 < self.tokens.len() {
            self.pos += 1;
        }
        t
    }

    fn adalah_akhir(&self) -> bool {
        matches!(self.sekarang().jenis, JenisToken::AkhirBerkas)
    }

    fn cek(&self, jenis: &JenisToken) -> bool {
        &self.sekarang().jenis == jenis
    }

    fn cocok(&mut self, jenis: &JenisToken) -> bool {
        if self.cek(jenis) {
            self.maju();
            true
        } else {
            false
        }
    }

    fn rentang_sejak(&self, awal: Rentang) -> Rentang {
        awal.gabung(self.tokens[self.pos.saturating_sub(1)].rentang)
    }

    fn baris_baru_memutus(&self) -> bool {
        !self.abaikan_baris_baru && self.sekarang().baris_baru_sebelum
    }

    fn galat<T>(&mut self, diagnostik: Diagnostik) -> Hasil<T> {
        self.diagnostik.push(diagnostik);
        Err(Galat)
    }

    fn masuk_dalam(&mut self) -> Hasil<()> {
        if self.kedalaman >= BATAS_KEDALAMAN {
            let rentang = self.sekarang().rentang;
            return self.galat(
                Diagnostik::galat(kode::TERLALU_DALAM, "penyarangan terlalu dalam", rentang)
                    .dengan_saran("pecah ekspresi atau blok menjadi bagian yang lebih sederhana"),
            );
        }
        self.kedalaman += 1;
        Ok(())
    }

    fn harap(&mut self, jenis: JenisToken, konteks: &str) -> Hasil<Token> {
        if self.cek(&jenis) {
            return Ok(self.maju());
        }
        let t = self.sekarang().clone();
        let mut d = Diagnostik::galat(
            kode::DIHARAPKAN_TOKEN,
            format!(
                "diharapkan {} {}, tetapi ditemukan {}",
                jenis.deskripsi(),
                konteks,
                t.jenis.deskripsi()
            ),
            t.rentang,
        );
        if let Some(s) = saran_padanan(&t) {
            d = d.dengan_saran(s);
        }
        self.galat(d)
    }

    fn harap_pengenal(&mut self, konteks: &str) -> Hasil<Pengenal> {
        let t = self.sekarang().clone();
        match &t.jenis {
            JenisToken::Pengenal(n) => {
                self.maju();
                Ok(Pengenal::baru(n.clone(), t.rentang))
            }
            JenisToken::Saluran => {
                self.maju();
                Ok(Pengenal::baru("saluran", t.rentang))
            }
            j => {
                let mut d = Diagnostik::galat(
                    kode::DIHARAPKAN_NAMA,
                    format!(
                        "diharapkan nama {}, tetapi ditemukan {}",
                        konteks,
                        j.deskripsi()
                    ),
                    t.rentang,
                );
                if j.adalah_kata_kunci() {
                    d = d.dengan_saran(format!(
                        "'{}' adalah kata kunci Komolang dan tidak dapat dipakai sebagai nama; pilih nama lain",
                        j.simbol()
                    ));
                }
                self.galat(d)
            }
        }
    }

    fn harap_penutup_blok(&mut self, buka: Rentang) -> Hasil<Token> {
        if self.cek(&JenisToken::KurungKurawalTutup) {
            return Ok(self.maju());
        }
        let t = self.sekarang().clone();
        self.galat(
            Diagnostik::galat(
                kode::BLOK_TIDAK_DITUTUP,
                format!(
                    "blok tidak ditutup: diharapkan '}}' tetapi ditemukan {}",
                    t.jenis.deskripsi()
                ),
                t.rentang,
            )
            .dengan_saran(format!(
                "blok dibuka pada baris {} kolom {}; tambahkan '}}' untuk menutupnya",
                buka.baris, buka.kolom
            )),
        )
    }

    fn lewati_titik_koma(&mut self) {
        while self.cek(&JenisToken::TitikKoma) {
            let t = self.maju();
            self.diagnostik.push(
                Diagnostik::galat(
                    kode::TITIK_KOMA,
                    "Komolang tidak memakai ';' sebagai pemisah pernyataan",
                    t.rentang,
                )
                .dengan_saran("hapus ';'; pernyataan dipisahkan oleh baris baru"),
            );
        }
    }

    fn akhiri_pernyataan(&mut self) -> Hasil<()> {
        if self.cek(&JenisToken::TitikKoma) {
            self.lewati_titik_koma();
            return Ok(());
        }
        let t = self.sekarang();
        if t.baris_baru_sebelum
            || matches!(
                t.jenis,
                JenisToken::KurungKurawalTutup | JenisToken::Kasus | JenisToken::Lainnya
            )
        {
            return Ok(());
        }
        let t = t.clone();
        let sebelumnya = self.tokens[self.pos.saturating_sub(1)].clone();
        let mut d = Diagnostik::galat(
            kode::DIHARAPKAN_BARIS_BARU,
            format!(
                "diharapkan baris baru setelah pernyataan, tetapi ditemukan {}",
                t.jenis.deskripsi()
            ),
            t.rentang,
        );
        if let Some(s) = saran_padanan(&sebelumnya) {
            d = d.dengan_saran(s);
        } else {
            d = d.dengan_saran("tulis satu pernyataan per baris");
        }
        self.galat(d)
    }

    fn akhiri_anggota(&mut self) -> Hasil<()> {
        let t = self.sekarang();
        if t.baris_baru_sebelum
            || matches!(t.jenis, JenisToken::KurungKurawalTutup | JenisToken::Koma)
        {
            return Ok(());
        }
        let t = t.clone();
        self.galat(
            Diagnostik::galat(
                kode::DIHARAPKAN_BARIS_BARU,
                format!(
                    "diharapkan baris baru atau ',' setelah anggota, tetapi ditemukan {}",
                    t.jenis.deskripsi()
                ),
                t.rentang,
            )
            .dengan_saran("tulis satu anggota per baris"),
        )
    }

    // -----------------------------------------------------------------------
    // Pemulihan galat
    // -----------------------------------------------------------------------

    fn sinkronkan_pernyataan(&mut self) {
        let mut kedalaman = 0usize;
        while !self.adalah_akhir() {
            let t = self.sekarang();
            match t.jenis {
                JenisToken::KurungKurawalBuka => kedalaman += 1,
                JenisToken::KurungKurawalTutup => {
                    if kedalaman == 0 {
                        return;
                    }
                    kedalaman -= 1;
                }
                _ => {
                    if kedalaman == 0
                        && t.baris_baru_sebelum
                        && (awal_pernyataan(&t.jenis)
                            || matches!(t.jenis, JenisToken::Kasus | JenisToken::Lainnya))
                    {
                        return;
                    }
                }
            }
            self.maju();
        }
    }

    fn sinkronkan_deklarasi(&mut self) {
        let mut kedalaman = 0usize;
        while !self.adalah_akhir() {
            let t = self.sekarang();
            match t.jenis {
                JenisToken::KurungKurawalBuka => kedalaman += 1,
                JenisToken::KurungKurawalTutup => kedalaman = kedalaman.saturating_sub(1),
                _ => {
                    if kedalaman == 0 && t.baris_baru_sebelum && awal_deklarasi(&t.jenis) {
                        return;
                    }
                }
            }
            self.maju();
        }
    }

    // -----------------------------------------------------------------------
    // Berkas dan deklarasi
    // -----------------------------------------------------------------------

    pub fn urai_berkas(&mut self) -> BerkasProgram {
        let paket = self.urai_paket();
        let mut impor = Vec::new();
        let mut deklarasi = Vec::new();
        loop {
            self.lewati_titik_koma();
            if self.adalah_akhir() {
                break;
            }
            let mulai = self.pos;
            if self.cek(&JenisToken::Impor) {
                if !deklarasi.is_empty() {
                    let rentang = self.sekarang().rentang;
                    self.diagnostik.push(
                        Diagnostik::galat(
                            kode::IMPOR_TERLAMBAT,
                            "'impor' harus ditulis sebelum deklarasi lain",
                            rentang,
                        )
                        .dengan_saran(
                            "pindahkan semua 'impor' ke bagian atas berkas, setelah 'paket'",
                        ),
                    );
                }
                match self.urai_impor() {
                    Ok(mut v) => impor.append(&mut v),
                    Err(Galat) => self.sinkronkan_deklarasi(),
                }
            } else {
                match self.urai_deklarasi() {
                    Ok(d) => deklarasi.push(d),
                    Err(Galat) => self.sinkronkan_deklarasi(),
                }
            }
            if self.pos == mulai {
                self.maju();
            }
        }
        BerkasProgram {
            paket,
            impor,
            deklarasi,
        }
    }

    fn urai_paket(&mut self) -> DeklarasiPaket {
        let t = self.sekarang().clone();
        if self.cek(&JenisToken::Paket) {
            self.maju();
            if let Ok(nama) = self.harap_pengenal("paket") {
                let rentang = t.rentang.gabung(nama.rentang);
                return DeklarasiPaket { nama, rentang };
            }
        } else {
            self.diagnostik.push(
                Diagnostik::galat(
                    kode::PAKET_HILANG,
                    format!(
                        "berkas harus diawali dengan deklarasi paket, tetapi ditemukan {}",
                        t.jenis.deskripsi()
                    ),
                    t.rentang,
                )
                .dengan_saran("tambahkan 'paket utama' di baris pertama berkas"),
            );
        }
        DeklarasiPaket {
            nama: Pengenal::baru(
                "",
                Rentang::baru(
                    t.rentang.awal,
                    t.rentang.awal,
                    t.rentang.baris,
                    t.rentang.kolom,
                ),
            ),
            rentang: t.rentang,
        }
    }

    fn urai_impor(&mut self) -> Hasil<Vec<DeklarasiImpor>> {
        let awal = self.maju();
        let mut hasil = Vec::new();
        if self.cocok(&JenisToken::KurungBuka) {
            while !self.cek(&JenisToken::KurungTutup) && !self.adalah_akhir() {
                hasil.push(self.urai_satu_impor(awal.rentang)?);
            }
            self.harap(JenisToken::KurungTutup, "untuk menutup daftar impor")?;
        } else {
            hasil.push(self.urai_satu_impor(awal.rentang)?);
        }
        self.akhiri_pernyataan()?;
        Ok(hasil)
    }

    fn urai_satu_impor(&mut self, awal: Rentang) -> Hasil<DeklarasiImpor> {
        let t = self.sekarang().clone();
        let jalur = match &t.jenis {
            JenisToken::LiteralTeks(bagian) => match teks_polos(bagian) {
                Some(s) => {
                    self.maju();
                    s
                }
                None => {
                    return self.galat(
                        Diagnostik::galat(
                            kode::JALUR_IMPOR_TIDAK_VALID,
                            "jalur impor tidak boleh memakai interpolasi",
                            t.rentang,
                        )
                        .dengan_saran("tulis jalur sebagai teks biasa, mis. impor \"std/cetak\""),
                    )
                }
            },
            j => {
                return self.galat(
                    Diagnostik::galat(
                        kode::JALUR_IMPOR_TIDAK_VALID,
                        format!(
                            "diharapkan jalur impor berupa teks, tetapi ditemukan {}",
                            j.deskripsi()
                        ),
                        t.rentang,
                    )
                    .dengan_saran("contoh: impor \"std/cetak\""),
                )
            }
        };
        let alias = if self.cocok(&JenisToken::Sebagai) {
            Some(self.harap_pengenal("alias impor")?)
        } else {
            None
        };
        Ok(DeklarasiImpor {
            jalur,
            alias,
            rentang: self.rentang_sejak(awal),
        })
    }

    fn urai_visibilitas(&mut self) -> Visibilitas {
        if self.cocok(&JenisToken::Publik) {
            Visibilitas::Publik
        } else {
            self.cocok(&JenisToken::Pribadi);
            Visibilitas::Pribadi
        }
    }

    fn urai_deklarasi(&mut self) -> Hasil<Deklarasi> {
        let vis = self.urai_visibilitas();
        let t = self.sekarang().clone();
        match &t.jenis {
            JenisToken::Fungsi => self.urai_fungsi(vis).map(Deklarasi::Fungsi),
            JenisToken::Tipe => self.urai_tipe(vis).map(Deklarasi::Tipe),
            JenisToken::Konst => self.urai_konstanta(vis).map(Deklarasi::Konstanta),
            JenisToken::Biar | JenisToken::Var => {
                self.urai_variabel_global(vis).map(Deklarasi::Variabel)
            }
            j => {
                let mut d = Diagnostik::galat(
                    kode::DEKLARASI_TIDAK_DIKENAL,
                    format!(
                        "diharapkan deklarasi ('fungsi', 'tipe', 'konst', 'biar' atau 'var') di tingkat atas, tetapi ditemukan {}",
                        j.deskripsi()
                    ),
                    t.rentang,
                );
                if let Some(s) = saran_padanan(&t) {
                    d = d.dengan_saran(s);
                }
                self.galat(d)
            }
        }
    }

    fn urai_generik_opsional(&mut self) -> Hasil<Vec<Pengenal>> {
        let mut v = Vec::new();
        if self.cocok(&JenisToken::KurangDari) {
            loop {
                v.push(self.harap_pengenal("parameter generik")?);
                if !self.cocok(&JenisToken::Koma) {
                    break;
                }
            }
            self.harap(JenisToken::LebihDari, "untuk menutup parameter generik")?;
        }
        Ok(v)
    }

    fn urai_daftar_parameter(&mut self) -> Hasil<Vec<Parameter>> {
        let mut v = Vec::new();
        while !self.cek(&JenisToken::KurungTutup) && !self.adalah_akhir() {
            let nama = self.harap_pengenal("parameter")?;
            self.harap(JenisToken::TitikDua, "setelah nama parameter")?;
            let tipe = self.urai_tipe_data()?;
            v.push(Parameter { nama, tipe });
            if !self.cocok(&JenisToken::Koma) {
                break;
            }
        }
        Ok(v)
    }

    fn urai_fungsi(&mut self, vis: Visibilitas) -> Hasil<DeklarasiFungsi> {
        let awal = self.harap(JenisToken::Fungsi, "untuk memulai fungsi")?;
        let penerima = if self.cocok(&JenisToken::KurungBuka) {
            let nama = self.harap_pengenal("penerima metode")?;
            self.harap(JenisToken::TitikDua, "setelah nama penerima")?;
            let tipe = self.urai_tipe_data()?;
            self.harap(JenisToken::KurungTutup, "untuk menutup penerima metode")?;
            Some(Parameter { nama, tipe })
        } else {
            None
        };
        let nama = self.harap_pengenal("fungsi")?;
        let generik = self.urai_generik_opsional()?;
        self.harap(JenisToken::KurungBuka, "setelah nama fungsi")?;
        let parameter = self.urai_daftar_parameter()?;
        self.harap(JenisToken::KurungTutup, "untuk menutup daftar parameter")?;
        let tipe_kembalian = if self.cek(&JenisToken::KurungKurawalBuka) {
            None
        } else {
            Some(self.urai_tipe_data()?)
        };
        let badan = self.urai_blok()?;
        Ok(DeklarasiFungsi {
            visibilitas: vis,
            penerima,
            nama,
            generik,
            parameter,
            tipe_kembalian,
            badan,
            rentang: self.rentang_sejak(awal.rentang),
        })
    }

    fn urai_tipe(&mut self, vis: Visibilitas) -> Hasil<DeklarasiTipe> {
        let awal = self.harap(JenisToken::Tipe, "untuk memulai deklarasi tipe")?;
        let nama = self.harap_pengenal("tipe")?;
        let generik = self.urai_generik_opsional()?;
        let _ = self.cocok(&JenisToken::SamaDengan);
        let isi = match self.sekarang().jenis {
            JenisToken::Struktur => {
                self.maju();
                IsiTipe::Struktur(self.urai_bidang_struktur()?)
            }
            JenisToken::Antarmuka => {
                self.maju();
                IsiTipe::Antarmuka(self.urai_metode_antarmuka()?)
            }
            JenisToken::Enum => {
                self.maju();
                IsiTipe::Enum(self.urai_varian_enum()?)
            }
            _ => IsiTipe::Alias(self.urai_tipe_data()?),
        };
        Ok(DeklarasiTipe {
            visibilitas: vis,
            nama,
            generik,
            isi,
            rentang: self.rentang_sejak(awal.rentang),
        })
    }

    fn urai_bidang_struktur(&mut self) -> Hasil<Vec<Bidang>> {
        let buka = self.harap(JenisToken::KurungKurawalBuka, "untuk memulai isi struktur")?;
        let mut v = Vec::new();
        loop {
            while self.cocok(&JenisToken::Koma) {}
            if self.cek(&JenisToken::KurungKurawalTutup) || self.adalah_akhir() {
                break;
            }
            let awal = self.sekarang().rentang;
            let visibilitas = self.urai_visibilitas();
            let nama = self.harap_pengenal("bidang")?;
            self.harap(JenisToken::TitikDua, "setelah nama bidang")?;
            let tipe = self.urai_tipe_data()?;
            let rentang = awal.gabung(tipe.rentang);
            v.push(Bidang {
                visibilitas,
                nama,
                tipe,
                rentang,
            });
            self.akhiri_anggota()?;
        }
        self.harap_penutup_blok(buka.rentang)?;
        Ok(v)
    }

    fn urai_metode_antarmuka(&mut self) -> Hasil<Vec<TandaTanganMetode>> {
        let buka = self.harap(JenisToken::KurungKurawalBuka, "untuk memulai isi antarmuka")?;
        let mut v = Vec::new();
        loop {
            while self.cocok(&JenisToken::Koma) {}
            if self.cek(&JenisToken::KurungKurawalTutup) || self.adalah_akhir() {
                break;
            }
            let nama = self.harap_pengenal("metode")?;
            self.harap(JenisToken::KurungBuka, "setelah nama metode")?;
            let parameter = self.urai_daftar_parameter()?;
            self.harap(JenisToken::KurungTutup, "untuk menutup daftar parameter")?;
            let t = self.sekarang();
            let tipe_kembalian = if !t.baris_baru_sebelum
                && !matches!(t.jenis, JenisToken::KurungKurawalTutup | JenisToken::Koma)
            {
                Some(self.urai_tipe_data()?)
            } else {
                None
            };
            let rentang = self.rentang_sejak(nama.rentang);
            v.push(TandaTanganMetode {
                nama,
                parameter,
                tipe_kembalian,
                rentang,
            });
            self.akhiri_anggota()?;
        }
        self.harap_penutup_blok(buka.rentang)?;
        Ok(v)
    }

    fn urai_varian_enum(&mut self) -> Hasil<Vec<Varian>> {
        let buka = self.harap(JenisToken::KurungKurawalBuka, "untuk memulai isi enum")?;
        let mut v = Vec::new();
        loop {
            while self.cocok(&JenisToken::Koma) {}
            if self.cek(&JenisToken::KurungKurawalTutup) || self.adalah_akhir() {
                break;
            }
            let nama = self.harap_pengenal("varian")?;
            let mut tipe = Vec::new();
            if self.cek(&JenisToken::KurungBuka) && !self.sekarang().baris_baru_sebelum {
                self.maju();
                while !self.cek(&JenisToken::KurungTutup) && !self.adalah_akhir() {
                    tipe.push(self.urai_tipe_data()?);
                    if !self.cocok(&JenisToken::Koma) {
                        break;
                    }
                }
                self.harap(JenisToken::KurungTutup, "untuk menutup data varian")?;
            }
            let rentang = self.rentang_sejak(nama.rentang);
            v.push(Varian {
                nama,
                tipe,
                rentang,
            });
            self.akhiri_anggota()?;
        }
        self.harap_penutup_blok(buka.rentang)?;
        Ok(v)
    }

    fn urai_konstanta(&mut self, vis: Visibilitas) -> Hasil<DeklarasiKonstanta> {
        let awal = self.harap(JenisToken::Konst, "untuk memulai konstanta")?;
        let nama = self.harap_pengenal("konstanta")?;
        let tipe = if self.cocok(&JenisToken::TitikDua) {
            Some(self.urai_tipe_data()?)
        } else {
            None
        };
        self.harap(JenisToken::SamaDengan, "setelah nama konstanta")?;
        let nilai = self.urai_ekspresi()?;
        self.akhiri_pernyataan()?;
        Ok(DeklarasiKonstanta {
            visibilitas: vis,
            nama,
            tipe,
            nilai,
            rentang: self.rentang_sejak(awal.rentang),
        })
    }

    #[allow(clippy::type_complexity)]
    fn urai_ikatan(&mut self) -> Hasil<(Token, Pengenal, Option<Tipe>, Ekspresi)> {
        let awal = self.maju();
        let nama = self.harap_pengenal("variabel")?;
        let tipe = if self.cocok(&JenisToken::TitikDua) {
            Some(self.urai_tipe_data()?)
        } else {
            None
        };
        if !self.cek(&JenisToken::SamaDengan) {
            let t = self.sekarang().clone();
            return self.galat(
                Diagnostik::galat(
                    kode::DIHARAPKAN_TOKEN,
                    format!(
                        "diharapkan '=' setelah nama variabel '{}', tetapi ditemukan {}",
                        nama.nama,
                        t.jenis.deskripsi()
                    ),
                    t.rentang,
                )
                .dengan_saran("variabel harus diberi nilai awal, contoh: biar x = 0"),
            );
        }
        self.maju();
        let nilai = self.urai_ekspresi()?;
        self.akhiri_pernyataan()?;
        Ok((awal, nama, tipe, nilai))
    }

    fn urai_variabel_global(&mut self, vis: Visibilitas) -> Hasil<DeklarasiVariabel> {
        let (awal, nama, tipe, nilai) = self.urai_ikatan()?;
        Ok(DeklarasiVariabel {
            visibilitas: vis,
            mutabel: awal.jenis == JenisToken::Var,
            nama,
            tipe,
            nilai,
            rentang: self.rentang_sejak(awal.rentang),
        })
    }

    // -----------------------------------------------------------------------
    // Tipe data
    // -----------------------------------------------------------------------

    fn urai_tipe_data(&mut self) -> Hasil<Tipe> {
        self.masuk_dalam()?;
        let h = self.urai_tipe_data_inti();
        self.kedalaman -= 1;
        h
    }

    fn urai_tipe_data_inti(&mut self) -> Hasil<Tipe> {
        let t = self.sekarang().clone();
        match &t.jenis {
            JenisToken::Pinjam => {
                self.maju();
                let mutabel = self.cocok(&JenisToken::Var);
                let dalam = self.urai_tipe_data()?;
                let rentang = t.rentang.gabung(dalam.rentang);
                Ok(Tipe::baru(
                    JenisTipe::Pinjam {
                        mutabel,
                        dalam: Box::new(dalam),
                    },
                    rentang,
                ))
            }
            JenisToken::KurungSikuBuka => {
                self.maju();
                let elemen = self.urai_tipe_data()?;
                let tutup = self.harap(JenisToken::KurungSikuTutup, "untuk menutup tipe larik ']'")?;
                let rentang = t.rentang.gabung(tutup.rentang);
                Ok(Tipe::baru(
                    JenisTipe::Nama {
                        nama: Pengenal::baru("larik".to_string(), t.rentang),
                        argumen: vec![elemen],
                    },
                    rentang,
                ))
            }
            JenisToken::Pengenal(_) | JenisToken::Saluran => {
                self.maju();
                let mut nama_gabung = match &t.jenis {
                    JenisToken::Pengenal(n) => n.clone(),
                    _ => "saluran".to_string(),
                };
                while self.cocok(&JenisToken::Titik) {
                    let sub = self.harap_pengenal("nama tipe setelah '.'")?;
                    nama_gabung.push('.');
                    nama_gabung.push_str(&sub.nama);
                }
                let nama = Pengenal::baru(nama_gabung, self.rentang_sejak(t.rentang));
                let mut argumen = Vec::new();
                if self.cocok(&JenisToken::KurangDari) {
                    loop {
                        argumen.push(self.urai_tipe_data()?);
                        if !self.cocok(&JenisToken::Koma) {
                            break;
                        }
                    }
                    self.harap(JenisToken::LebihDari, "untuk menutup argumen tipe")?;
                }
                let rentang = self.rentang_sejak(t.rentang);
                Ok(Tipe::baru(JenisTipe::Nama { nama, argumen }, rentang))
            }
            j => self.galat(
                Diagnostik::galat(
                    kode::DIHARAPKAN_TIPE,
                    format!("diharapkan tipe data, tetapi ditemukan {}", j.deskripsi()),
                    t.rentang,
                )
                .dengan_saran("contoh tipe: bulat, teks, larik<bulat>, hasil<bulat, teks>"),
            ),
        }
    }

    // -----------------------------------------------------------------------
    // Blok dan pernyataan
    // -----------------------------------------------------------------------

    fn urai_blok(&mut self) -> Hasil<Blok> {
        let buka = self.harap(JenisToken::KurungKurawalBuka, "untuk memulai blok")?;
        let lama = (self.struktur_diizinkan, self.abaikan_baris_baru);
        self.struktur_diizinkan = true;
        self.abaikan_baris_baru = false;
        let mut pernyataan = Vec::new();
        loop {
            self.lewati_titik_koma();
            if self.cek(&JenisToken::KurungKurawalTutup) || self.adalah_akhir() {
                break;
            }
            let mulai = self.pos;
            match self.urai_pernyataan() {
                Ok(p) => pernyataan.push(p),
                Err(Galat) => self.sinkronkan_pernyataan(),
            }
            if self.pos == mulai {
                self.maju();
            }
        }
        self.struktur_diizinkan = lama.0;
        self.abaikan_baris_baru = lama.1;
        let tutup = self.harap_penutup_blok(buka.rentang)?;
        Ok(Blok {
            pernyataan,
            rentang: buka.rentang.gabung(tutup.rentang),
        })
    }

    fn urai_pernyataan(&mut self) -> Hasil<Pernyataan> {
        self.masuk_dalam()?;
        let h = self.urai_pernyataan_inti();
        self.kedalaman -= 1;
        h
    }

    fn urai_pernyataan_inti(&mut self) -> Hasil<Pernyataan> {
        let t = self.sekarang().clone();
        match &t.jenis {
            JenisToken::Biar | JenisToken::Var => self.urai_variabel_lokal(),
            JenisToken::Jika => self.urai_jika(),
            JenisToken::Selagi => self.urai_selagi(),
            JenisToken::Untuk => self.urai_untuk(),
            JenisToken::Pilih => self.urai_pilih(),
            JenisToken::Cocok => self.urai_cocok(),
            JenisToken::Tunda => self.urai_tunda(),
            JenisToken::Jalankan => {
                self.maju();
                let e = self.urai_panggilan_wajib("jalankan")?;
                self.akhiri_pernyataan()?;
                Ok(Pernyataan::baru(
                    JenisPernyataan::Jalankan(e),
                    self.rentang_sejak(t.rentang),
                ))
            }
            JenisToken::Kembali => self.urai_kembali(),
            JenisToken::TidakAman => {
                self.maju();
                let b = self.urai_blok()?;
                Ok(Pernyataan::baru(
                    JenisPernyataan::TidakAman(b),
                    self.rentang_sejak(t.rentang),
                ))
            }
            JenisToken::KurungKurawalBuka => {
                let b = self.urai_blok()?;
                let r = b.rentang;
                Ok(Pernyataan::baru(JenisPernyataan::Blok(b), r))
            }
            JenisToken::Fungsi
            | JenisToken::Tipe
            | JenisToken::Konst
            | JenisToken::Impor
            | JenisToken::Paket
            | JenisToken::Publik
            | JenisToken::Pribadi => {
                let saran = match &t.jenis {
                    JenisToken::Impor | JenisToken::Paket => {
                        "pindahkan ke bagian atas berkas".to_string()
                    }
                    _ => "pindahkan deklarasi ke tingkat atas berkas, di luar fungsi".to_string(),
                };
                self.galat(
                    Diagnostik::galat(
                        kode::DEKLARASI_DALAM_FUNGSI,
                        format!(
                            "deklarasi {} tidak dapat berada di dalam fungsi",
                            t.jenis.deskripsi()
                        ),
                        t.rentang,
                    )
                    .dengan_saran(saran),
                )
            }
            _ => self.urai_ekspresi_atau_tugas(),
        }
    }

    fn urai_ekspresi_atau_tugas(&mut self) -> Hasil<Pernyataan> {
        let awal = self.sekarang().rentang;
        let ekspresi = self.urai_ekspresi()?;
        if let Some(operator) = operator_tugas(&self.sekarang().jenis) {
            self.maju();
            let sah = matches!(
                ekspresi.jenis,
                JenisEkspresi::Pengenal(_)
                    | JenisEkspresi::Bidang { .. }
                    | JenisEkspresi::Indeks { .. }
            );
            if !sah {
                return self.galat(
                    Diagnostik::galat(
                        kode::TARGET_TUGAS_TIDAK_VALID,
                        "target penugasan tidak valid: hanya nama, bidang, atau elemen indeks yang dapat diberi nilai",
                        ekspresi.rentang,
                    )
                    .dengan_saran("contoh: x = 1, objek.bidang = 1, larik[0] = 1"),
                );
            }
            let nilai = self.urai_ekspresi()?;
            self.akhiri_pernyataan()?;
            return Ok(Pernyataan::baru(
                JenisPernyataan::Tugas {
                    target: ekspresi,
                    operator,
                    nilai,
                },
                self.rentang_sejak(awal),
            ));
        }
        self.akhiri_pernyataan()?;
        Ok(Pernyataan::baru(
            JenisPernyataan::Ekspresi(ekspresi),
            self.rentang_sejak(awal),
        ))
    }

    fn urai_variabel_lokal(&mut self) -> Hasil<Pernyataan> {
        let (awal, nama, tipe, nilai) = self.urai_ikatan()?;
        Ok(Pernyataan::baru(
            JenisPernyataan::Variabel {
                mutabel: awal.jenis == JenisToken::Var,
                nama,
                tipe,
                nilai,
            },
            self.rentang_sejak(awal.rentang),
        ))
    }

    fn urai_kondisi(&mut self) -> Hasil<Ekspresi> {
        let lama = self.struktur_diizinkan;
        self.struktur_diizinkan = false;
        let h = self.urai_ekspresi();
        self.struktur_diizinkan = lama;
        h
    }

    fn urai_jika(&mut self) -> Hasil<Pernyataan> {
        let awal = self.maju();
        let kondisi = self.urai_kondisi()?;
        let maka = self.urai_blok()?;
        let lainnya = if self.cek(&JenisToken::Lainnya)
            && matches!(
                self.intip(1).jenis,
                JenisToken::Jika | JenisToken::KurungKurawalBuka
            ) {
            self.maju();
            if self.cek(&JenisToken::Jika) {
                Some(Box::new(self.urai_jika()?))
            } else {
                let b = self.urai_blok()?;
                let r = b.rentang;
                Some(Box::new(Pernyataan::baru(JenisPernyataan::Blok(b), r)))
            }
        } else {
            None
        };
        Ok(Pernyataan::baru(
            JenisPernyataan::Jika {
                kondisi,
                maka,
                lainnya,
            },
            self.rentang_sejak(awal.rentang),
        ))
    }

    fn urai_selagi(&mut self) -> Hasil<Pernyataan> {
        let awal = self.maju();
        let kondisi = self.urai_kondisi()?;
        let badan = self.urai_blok()?;
        Ok(Pernyataan::baru(
            JenisPernyataan::Selagi { kondisi, badan },
            self.rentang_sejak(awal.rentang),
        ))
    }

    fn urai_untuk(&mut self) -> Hasil<Pernyataan> {
        let awal = self.maju();
        let variabel = if self.cek(&JenisToken::GarisBawah) {
            let t = self.maju();
            Pengenal::baru("_", t.rentang)
        } else {
            self.harap_pengenal("variabel perulangan")?
        };
        self.harap(JenisToken::Dalam, "setelah variabel perulangan")?;
        let koleksi = self.urai_kondisi()?;
        let badan = self.urai_blok()?;
        Ok(Pernyataan::baru(
            JenisPernyataan::Untuk {
                variabel,
                koleksi,
                badan,
            },
            self.rentang_sejak(awal.rentang),
        ))
    }

    fn urai_pilih(&mut self) -> Hasil<Pernyataan> {
        let awal = self.maju();
        let nilai = self.urai_kondisi()?;
        let buka = self.harap(
            JenisToken::KurungKurawalBuka,
            "untuk memulai cabang 'pilih'",
        )?;
        let mut kasus = Vec::new();
        let mut lainnya: Option<Vec<Pernyataan>> = None;
        loop {
            self.lewati_titik_koma();
            if self.cek(&JenisToken::KurungKurawalTutup) || self.adalah_akhir() {
                break;
            }
            let mulai = self.pos;
            let t = self.sekarang().clone();
            match &t.jenis {
                JenisToken::Kasus => match self.urai_cabang_kasus() {
                    Ok(k) => kasus.push(k),
                    Err(Galat) => self.sinkronkan_pernyataan(),
                },
                JenisToken::Lainnya => {
                    self.maju();
                    if self
                        .harap(JenisToken::TitikDua, "setelah 'lainnya'")
                        .is_err()
                    {
                        self.sinkronkan_pernyataan();
                    } else {
                        let badan = self.urai_badan_kasus();
                        if lainnya.is_some() {
                            self.diagnostik.push(
                                Diagnostik::galat(
                                    kode::CABANG_GANDA,
                                    "cabang 'lainnya' hanya boleh ada satu dalam 'pilih'",
                                    t.rentang,
                                )
                                .dengan_saran("gabungkan isi kedua cabang 'lainnya'"),
                            );
                        } else {
                            lainnya = Some(badan);
                        }
                    }
                }
                j => {
                    self.diagnostik.push(
                        Diagnostik::galat(
                            kode::DIHARAPKAN_TOKEN,
                            format!(
                                "diharapkan 'kasus' atau 'lainnya' di dalam 'pilih', tetapi ditemukan {}",
                                j.deskripsi()
                            ),
                            t.rentang,
                        )
                        .dengan_saran("setiap cabang diawali 'kasus nilai:' atau 'lainnya:'"),
                    );
                    self.sinkronkan_pernyataan();
                }
            }
            if self.pos == mulai {
                self.maju();
            }
        }
        self.harap_penutup_blok(buka.rentang)?;
        Ok(Pernyataan::baru(
            JenisPernyataan::Pilih {
                nilai,
                kasus,
                lainnya,
            },
            self.rentang_sejak(awal.rentang),
        ))
    }

    fn urai_cabang_kasus(&mut self) -> Hasil<CabangKasus> {
        let awal = self.maju();
        let nilai = self.urai_kondisi()?;
        self.harap(JenisToken::TitikDua, "setelah nilai 'kasus'")?;
        let badan = self.urai_badan_kasus();
        Ok(CabangKasus {
            nilai,
            badan,
            rentang: self.rentang_sejak(awal.rentang),
        })
    }

    fn urai_badan_kasus(&mut self) -> Vec<Pernyataan> {
        let mut v = Vec::new();
        loop {
            self.lewati_titik_koma();
            if matches!(
                self.sekarang().jenis,
                JenisToken::Kasus | JenisToken::Lainnya | JenisToken::KurungKurawalTutup
            ) || self.adalah_akhir()
            {
                break;
            }
            let mulai = self.pos;
            match self.urai_pernyataan() {
                Ok(p) => v.push(p),
                Err(Galat) => self.sinkronkan_pernyataan(),
            }
            if self.pos == mulai {
                self.maju();
            }
        }
        v
    }

    fn urai_cocok(&mut self) -> Hasil<Pernyataan> {
        let awal = self.maju();
        let nilai = self.urai_kondisi()?;
        let buka = self.harap(
            JenisToken::KurungKurawalBuka,
            "untuk memulai cabang 'cocok'",
        )?;
        let mut cabang = Vec::new();
        loop {
            self.lewati_titik_koma();
            if self.cek(&JenisToken::KurungKurawalTutup) || self.adalah_akhir() {
                break;
            }
            let mulai = self.pos;
            match self.urai_cabang_pola() {
                Ok(c) => cabang.push(c),
                Err(Galat) => self.sinkronkan_pernyataan(),
            }
            if self.pos == mulai {
                self.maju();
            }
        }
        self.harap_penutup_blok(buka.rentang)?;
        Ok(Pernyataan::baru(
            JenisPernyataan::Cocok { nilai, cabang },
            self.rentang_sejak(awal.rentang),
        ))
    }

    fn urai_cabang_pola(&mut self) -> Hasil<CabangPola> {
        let awal = self.sekarang().rentang;
        let pola = self.urai_pola()?;
        self.harap(JenisToken::TitikDua, "setelah pola")?;
        let badan = if self.cek(&JenisToken::KurungKurawalBuka) {
            let b = self.urai_blok()?;
            let r = b.rentang;
            Pernyataan::baru(JenisPernyataan::Blok(b), r)
        } else {
            self.urai_pernyataan()?
        };
        Ok(CabangPola {
            pola,
            badan,
            rentang: self.rentang_sejak(awal),
        })
    }

    fn urai_pola(&mut self) -> Hasil<Pola> {
        self.masuk_dalam()?;
        let h = self.urai_pola_inti();
        self.kedalaman -= 1;
        h
    }

    fn urai_pola_inti(&mut self) -> Hasil<Pola> {
        let t = self.sekarang().clone();
        match &t.jenis {
            JenisToken::GarisBawah => {
                self.maju();
                Ok(Pola::baru(JenisPola::Wildcard, t.rentang))
            }
            JenisToken::Pengenal(nama) => {
                self.maju();
                let id = Pengenal::baru(nama.clone(), t.rentang);
                if self.cek(&JenisToken::KurungBuka) && !self.sekarang().baris_baru_sebelum {
                    self.maju();
                    let mut argumen = Vec::new();
                    while !self.cek(&JenisToken::KurungTutup) && !self.adalah_akhir() {
                        argumen.push(self.urai_pola()?);
                        if !self.cocok(&JenisToken::Koma) {
                            break;
                        }
                    }
                    let tutup = self.harap(JenisToken::KurungTutup, "untuk menutup pola")?;
                    Ok(Pola::baru(
                        JenisPola::Varian { nama: id, argumen },
                        t.rentang.gabung(tutup.rentang),
                    ))
                } else {
                    Ok(Pola::baru(JenisPola::Pengenal(id), t.rentang))
                }
            }
            JenisToken::LiteralBulat(_)
            | JenisToken::LiteralDesimal(_)
            | JenisToken::LiteralTeks(_)
            | JenisToken::LiteralRune(_)
            | JenisToken::Benar
            | JenisToken::Salah
            | JenisToken::Kosong => {
                let e = self.urai_primer()?;
                let r = e.rentang;
                Ok(Pola::baru(JenisPola::Literal(e), r))
            }
            JenisToken::Kurang => {
                self.maju();
                if matches!(
                    self.sekarang().jenis,
                    JenisToken::LiteralBulat(_) | JenisToken::LiteralDesimal(_)
                ) {
                    let e = self.urai_primer()?;
                    let r = t.rentang.gabung(e.rentang);
                    let neg = Ekspresi::baru(
                        JenisEkspresi::Unari {
                            operator: OperatorUnari::Negatif,
                            operan: Box::new(e),
                        },
                        r,
                    );
                    Ok(Pola::baru(JenisPola::Literal(neg), r))
                } else {
                    let s = self.sekarang().clone();
                    self.galat(Diagnostik::galat(
                        kode::POLA_TIDAK_VALID,
                        format!(
                            "pola tidak valid: setelah '-' diharapkan angka, tetapi ditemukan {}",
                            s.jenis.deskripsi()
                        ),
                        s.rentang,
                    ))
                }
            }
            j => self.galat(
                Diagnostik::galat(
                    kode::POLA_TIDAK_VALID,
                    format!("pola tidak valid: ditemukan {}", j.deskripsi()),
                    t.rentang,
                )
                .dengan_saran(
                    "pola dapat berupa '_', nama, varian seperti sukses(x), atau literal",
                ),
            ),
        }
    }

    fn urai_tunda(&mut self) -> Hasil<Pernyataan> {
        let awal = self.maju();
        if self.cek(&JenisToken::KurungKurawalBuka) {
            let b = self.urai_blok()?;
            return Ok(Pernyataan::baru(
                JenisPernyataan::TundaBlok(b),
                self.rentang_sejak(awal.rentang),
            ));
        }
        let e = self.urai_panggilan_wajib("tunda")?;
        self.akhiri_pernyataan()?;
        Ok(Pernyataan::baru(
            JenisPernyataan::Tunda(e),
            self.rentang_sejak(awal.rentang),
        ))
    }

    fn urai_panggilan_wajib(&mut self, kata: &str) -> Hasil<Ekspresi> {
        let e = self.urai_ekspresi()?;
        if matches!(e.jenis, JenisEkspresi::Panggil { .. }) {
            Ok(e)
        } else {
            self.galat(
                Diagnostik::galat(
                    kode::PANGGILAN_DIPERLUKAN,
                    format!("'{}' membutuhkan pemanggilan fungsi", kata),
                    e.rentang,
                )
                .dengan_saran(format!("contoh: {} nama_fungsi()", kata)),
            )
        }
    }

    fn urai_kembali(&mut self) -> Hasil<Pernyataan> {
        let awal = self.maju();
        let t = self.sekarang();
        let ada_nilai = !t.baris_baru_sebelum
            && !matches!(
                t.jenis,
                JenisToken::KurungKurawalTutup
                    | JenisToken::Kasus
                    | JenisToken::Lainnya
                    | JenisToken::TitikKoma
                    | JenisToken::AkhirBerkas
            );
        let nilai = if ada_nilai {
            Some(self.urai_ekspresi()?)
        } else {
            None
        };
        self.akhiri_pernyataan()?;
        Ok(Pernyataan::baru(
            JenisPernyataan::Kembali(nilai),
            self.rentang_sejak(awal.rentang),
        ))
    }

    // -----------------------------------------------------------------------
    // Ekspresi (Pratt)
    // -----------------------------------------------------------------------

    fn urai_ekspresi(&mut self) -> Hasil<Ekspresi> {
        self.urai_ekspresi_bp(0)
    }

    fn urai_ekspresi_bp(&mut self, min_bp: u8) -> Hasil<Ekspresi> {
        self.masuk_dalam()?;
        let h = self.urai_ekspresi_bp_inti(min_bp);
        self.kedalaman -= 1;
        h
    }

    fn urai_ekspresi_bp_inti(&mut self, min_bp: u8) -> Hasil<Ekspresi> {
        let mut kiri = self.urai_unari()?;
        loop {
            let t = self.sekarang();
            let (op, bp) = match infiks(&t.jenis) {
                Some(x) => x,
                None => break,
            };
            if bp < min_bp {
                break;
            }
            if matches!(t.jenis, JenisToken::Kurang) && self.baris_baru_memutus() {
                break;
            }
            self.maju();
            let kanan = self.urai_ekspresi_bp(bp + 1)?;
            let rentang = kiri.rentang.gabung(kanan.rentang);
            kiri = match op {
                Infiks::Biner(operator) => Ekspresi::baru(
                    JenisEkspresi::Biner {
                        operator,
                        kiri: Box::new(kiri),
                        kanan: Box::new(kanan),
                    },
                    rentang,
                ),
                Infiks::Rentang { inklusif } => {
                    if matches!(
                        self.sekarang().jenis,
                        JenisToken::TitikTitik | JenisToken::TitikTitikSama
                    ) {
                        let lagi = self.sekarang().rentang;
                        return self.galat(
                            Diagnostik::galat(
                                kode::RENTANG_BERANTAI,
                                "operator rentang tidak dapat dirantai",
                                lagi,
                            )
                            .dengan_saran("gunakan kurung atau pecah menjadi beberapa rentang"),
                        );
                    }
                    Ekspresi::baru(
                        JenisEkspresi::Rentang {
                            awal: Box::new(kiri),
                            akhir: Box::new(kanan),
                            inklusif,
                        },
                        rentang,
                    )
                }
            };
        }
        Ok(kiri)
    }

    fn urai_unari(&mut self) -> Hasil<Ekspresi> {
        self.masuk_dalam()?;
        let h = self.urai_unari_inti();
        self.kedalaman -= 1;
        h
    }

    fn urai_unari_inti(&mut self) -> Hasil<Ekspresi> {
        let t = self.sekarang().clone();
        let operator = match &t.jenis {
            JenisToken::Kurang => {
                self.maju();
                OperatorUnari::Negatif
            }
            JenisToken::BukanLogika => {
                self.maju();
                OperatorUnari::Bukan
            }
            JenisToken::Pinjam => {
                self.maju();
                if self.cocok(&JenisToken::Var) {
                    OperatorUnari::PinjamMutabel
                } else {
                    OperatorUnari::Pinjam
                }
            }
            JenisToken::Pindah => {
                self.maju();
                OperatorUnari::Pindah
            }
            _ => return self.urai_postfiks(),
        };
        let operan = self.urai_unari()?;
        let rentang = t.rentang.gabung(operan.rentang);
        Ok(Ekspresi::baru(
            JenisEkspresi::Unari {
                operator,
                operan: Box::new(operan),
            },
            rentang,
        ))
    }

    fn dengan_konteks_bebas<T>(&mut self, f: impl FnOnce(&mut Self) -> Hasil<T>) -> Hasil<T> {
        let lama = (self.struktur_diizinkan, self.abaikan_baris_baru);
        self.struktur_diizinkan = true;
        self.abaikan_baris_baru = true;
        let h = f(self);
        self.struktur_diizinkan = lama.0;
        self.abaikan_baris_baru = lama.1;
        h
    }

    fn urai_argumen_hingga(
        &mut self,
        penutup: JenisToken,
        konteks: &str,
    ) -> Hasil<(Vec<Ekspresi>, Token)> {
        let mut v = Vec::new();
        while !self.cek(&penutup) && !self.adalah_akhir() {
            v.push(self.urai_ekspresi()?);
            if !self.cocok(&JenisToken::Koma) {
                break;
            }
        }
        let tutup = self.harap(penutup, konteks)?;
        Ok((v, tutup))
    }

    fn urai_postfiks(&mut self) -> Hasil<Ekspresi> {
        let mut e = self.urai_primer()?;
        loop {
            let t = self.sekarang().clone();
            match &t.jenis {
                JenisToken::KurungBuka if !self.baris_baru_memutus() => {
                    self.maju();
                    let (argumen, tutup) = self.dengan_konteks_bebas(|p| {
                        p.urai_argumen_hingga(
                            JenisToken::KurungTutup,
                            "untuk menutup pemanggilan fungsi",
                        )
                    })?;
                    let rentang = e.rentang.gabung(tutup.rentang);
                    e = Ekspresi::baru(
                        JenisEkspresi::Panggil {
                            fungsi: Box::new(e),
                            argumen_tipe: Vec::new(),
                            argumen,
                        },
                        rentang,
                    );
                }
                JenisToken::KurungSikuBuka if !self.baris_baru_memutus() => {
                    self.maju();
                    let (indeks, tutup) = self.dengan_konteks_bebas(|p| {
                        let i = p.urai_ekspresi()?;
                        let tutup = p.harap(JenisToken::KurungSikuTutup, "untuk menutup indeks")?;
                        Ok((i, tutup))
                    })?;
                    let rentang = e.rentang.gabung(tutup.rentang);
                    e = Ekspresi::baru(
                        JenisEkspresi::Indeks {
                            objek: Box::new(e),
                            indeks: Box::new(indeks),
                        },
                        rentang,
                    );
                }
                JenisToken::Titik => {
                    self.maju();
                    let nama = self.harap_pengenal("bidang atau metode setelah '.'")?;
                    let rentang = e.rentang.gabung(nama.rentang);
                    e = Ekspresi::baru(
                        JenisEkspresi::Bidang {
                            objek: Box::new(e),
                            nama,
                        },
                        rentang,
                    );
                }
                _ => break,
            }
        }
        Ok(e)
    }

    fn ubah_bagian_teks(&mut self, bagian: &[BagianToken]) -> Vec<BagianTeks> {
        let mut hasil = Vec::new();
        for b in bagian {
            match b {
                BagianToken::Teks(t) => hasil.push(BagianTeks::Teks(t.clone())),
                BagianToken::Ekspresi(tokens) => {
                    let mut sub = Pengurai::baru(tokens.clone());
                    sub.abaikan_baris_baru = true;
                    sub.kedalaman = self.kedalaman;
                    let e = sub.urai_ekspresi();
                    if e.is_ok() && !sub.adalah_akhir() {
                        let t = sub.sekarang().clone();
                        sub.diagnostik.push(
                            Diagnostik::galat(
                                kode::DIHARAPKAN_TOKEN,
                                format!(
                                    "token tak terduga dalam interpolasi: {}",
                                    t.jenis.deskripsi()
                                ),
                                t.rentang,
                            )
                            .dengan_saran("interpolasi hanya boleh berisi satu ekspresi"),
                        );
                    }
                    self.diagnostik.append(&mut sub.diagnostik);
                    if let Ok(e) = e {
                        hasil.push(BagianTeks::Ekspresi(Box::new(e)));
                    }
                }
            }
        }
        hasil
    }

    fn urai_primer(&mut self) -> Hasil<Ekspresi> {
        let t = self.sekarang().clone();
        match &t.jenis {
            JenisToken::LiteralBulat(n) => {
                self.maju();
                Ok(Ekspresi::baru(JenisEkspresi::Bulat(*n), t.rentang))
            }
            JenisToken::LiteralDesimal(f) => {
                self.maju();
                Ok(Ekspresi::baru(JenisEkspresi::Desimal(*f), t.rentang))
            }
            JenisToken::LiteralTeks(bagian) => {
                self.maju();
                let bagian = self.ubah_bagian_teks(bagian);
                Ok(Ekspresi::baru(JenisEkspresi::Teks(bagian), t.rentang))
            }
            JenisToken::LiteralRune(c) => {
                self.maju();
                Ok(Ekspresi::baru(JenisEkspresi::Rune(*c), t.rentang))
            }
            JenisToken::Benar => {
                self.maju();
                Ok(Ekspresi::baru(JenisEkspresi::Bool(true), t.rentang))
            }
            JenisToken::Salah => {
                self.maju();
                Ok(Ekspresi::baru(JenisEkspresi::Bool(false), t.rentang))
            }
            JenisToken::Kosong => {
                self.maju();
                Ok(Ekspresi::baru(JenisEkspresi::Kosong, t.rentang))
            }
            JenisToken::Pengenal(nama) => {
                self.maju();
                let id = Pengenal::baru(nama.clone(), t.rentang);
                self.urai_setelah_pengenal(id)
            }
            JenisToken::Saluran => {
                self.maju();
                let id = Pengenal::baru("saluran", t.rentang);
                if self.cek(&JenisToken::KurangDari)
                    && self.sekarang().rentang.awal == id.rentang.akhir
                {
                    if let Some(argumen_tipe) = self.coba_argumen_tipe() {
                        return self.selesaikan_panggilan_generik(id, argumen_tipe);
                    }
                }
                self.urai_setelah_pengenal(id)
            }
            JenisToken::KurungBuka => {
                self.maju();
                self.dengan_konteks_bebas(|p| {
                    let e = p.urai_ekspresi()?;
                    p.harap(JenisToken::KurungTutup, "untuk menutup ekspresi berkurung")?;
                    Ok(e)
                })
            }
            JenisToken::KurungSikuBuka => {
                self.maju();
                let (item, tutup) = self.dengan_konteks_bebas(|p| {
                    p.urai_argumen_hingga(
                        JenisToken::KurungSikuTutup,
                        "untuk menutup literal larik",
                    )
                })?;
                Ok(Ekspresi::baru(
                    JenisEkspresi::Larik(item),
                    t.rentang.gabung(tutup.rentang),
                ))
            }
            j => {
                let mut d = Diagnostik::galat(
                    kode::DIHARAPKAN_EKSPRESI,
                    format!("diharapkan ekspresi, tetapi ditemukan {}", j.deskripsi()),
                    t.rentang,
                );
                if j.adalah_kata_kunci() {
                    d = d.dengan_saran(format!(
                        "'{}' adalah kata kunci dan tidak dapat dipakai di sini",
                        j.simbol()
                    ));
                }
                self.galat(d)
            }
        }
    }

    fn urai_setelah_pengenal(&mut self, id: Pengenal) -> Hasil<Ekspresi> {
        if self.cek(&JenisToken::KurangDari) && self.sekarang().rentang.awal == id.rentang.akhir {
            if let Some(argumen_tipe) = self.coba_argumen_tipe() {
                return self.selesaikan_panggilan_generik(id, argumen_tipe);
            }
        }
        if self.struktur_diizinkan
            && self.cek(&JenisToken::KurungKurawalBuka)
            && !self.baris_baru_memutus()
            && self.mirip_literal_struktur()
        {
            return self.urai_literal_struktur(id);
        }
        let r = id.rentang;
        Ok(Ekspresi::baru(JenisEkspresi::Pengenal(id), r))
    }

    /// Literal struktur dikenali bila `{` diikuti `}` atau `nama:`.
    fn mirip_literal_struktur(&self) -> bool {
        match &self.intip(1).jenis {
            JenisToken::KurungKurawalTutup => true,
            JenisToken::Pengenal(_) => matches!(self.intip(2).jenis, JenisToken::TitikDua),
            _ => false,
        }
    }

    fn urai_literal_struktur(&mut self, nama: Pengenal) -> Hasil<Ekspresi> {
        self.maju();
        let (bidang, tutup) = self.dengan_konteks_bebas(|p| {
            let mut v = Vec::new();
            while !p.cek(&JenisToken::KurungKurawalTutup) && !p.adalah_akhir() {
                while p.cocok(&JenisToken::Koma) {}
                if p.cek(&JenisToken::KurungKurawalTutup) || p.adalah_akhir() {
                    break;
                }
                let n = p.harap_pengenal("bidang")?;
                p.harap(JenisToken::TitikDua, "setelah nama bidang")?;
                let nilai = p.urai_ekspresi()?;
                v.push(BidangInisialisasi { nama: n, nilai });
                let _ = p.cocok(&JenisToken::Koma);
            }
            let tutup = p.harap(
                JenisToken::KurungKurawalTutup,
                "untuk menutup literal struktur",
            )?;
            Ok((v, tutup))
        })?;
        let rentang = nama.rentang.gabung(tutup.rentang);
        Ok(Ekspresi::baru(
            JenisEkspresi::Struktur { nama, bidang },
            rentang,
        ))
    }

    /// Mencoba membaca `<T, ...>` yang langsung diikuti `(`; bila gagal, posisi dipulihkan
    /// sehingga `<` diperlakukan sebagai operator perbandingan.
    fn coba_argumen_tipe(&mut self) -> Option<Vec<Tipe>> {
        let simpan_pos = self.pos;
        let simpan_diag = self.diagnostik.len();
        let simpan_kedalaman = self.kedalaman;
        self.maju();
        let mut argumen = Vec::new();
        let mut ok = loop {
            match self.urai_tipe_data() {
                Ok(t) => argumen.push(t),
                Err(Galat) => break false,
            }
            if self.cocok(&JenisToken::Koma) {
                continue;
            }
            break true;
        };
        if ok {
            if self.cek(&JenisToken::LebihDari) {
                let tutup = self.maju();
                ok = self.cek(&JenisToken::KurungBuka)
                    && self.sekarang().rentang.awal == tutup.rentang.akhir;
            } else {
                ok = false;
            }
        }
        if ok {
            return Some(argumen);
        }
        self.pos = simpan_pos;
        self.kedalaman = simpan_kedalaman;
        self.diagnostik.truncate(simpan_diag);
        None
    }

    fn selesaikan_panggilan_generik(
        &mut self,
        id: Pengenal,
        argumen_tipe: Vec<Tipe>,
    ) -> Hasil<Ekspresi> {
        self.maju();
        let (argumen, tutup) = self.dengan_konteks_bebas(|p| {
            p.urai_argumen_hingga(JenisToken::KurungTutup, "untuk menutup pemanggilan fungsi")
        })?;
        let rentang = id.rentang.gabung(tutup.rentang);
        let fungsi = Ekspresi::baru(JenisEkspresi::Pengenal(id.clone()), id.rentang);
        Ok(Ekspresi::baru(
            JenisEkspresi::Panggil {
                fungsi: Box::new(fungsi),
                argumen_tipe,
                argumen,
            },
            rentang,
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use komo_ast::cetak::cetak_berkas;

    #[test]
    fn tes_urai_paket_dan_fungsi() {
        let sumber = "paket utama\nimpor \"std/cetak\"\n\npublik fungsi halo() {\n}\n";
        let hasil = urai(sumber);
        assert!(hasil.berhasil(), "{:?}", hasil.diagnostik);
        let berkas = hasil.berkas;
        assert_eq!(berkas.paket.nama.nama, "utama");
        assert_eq!(berkas.impor.len(), 1);
        assert_eq!(berkas.impor[0].jalur, "std/cetak");
        assert_eq!(berkas.deklarasi.len(), 1);
        match &berkas.deklarasi[0] {
            Deklarasi::Fungsi(f) => {
                assert_eq!(f.visibilitas, Visibilitas::Publik);
                assert_eq!(f.nama.nama, "halo");
            }
            lain => panic!("diharapkan fungsi: {:?}", lain),
        }
    }

    #[test]
    fn tes_presedensi_ekspresi() {
        let e = urai_ekspresi_sumber("1 + 2 * 3").unwrap();
        assert_eq!(komo_ast::cetak::cetak_ekspresi(&e), "1 + 2 * 3");
        match e.jenis {
            JenisEkspresi::Biner {
                operator, kanan, ..
            } => {
                assert_eq!(operator, OperatorBiner::Tambah);
                assert!(matches!(
                    kanan.jenis,
                    JenisEkspresi::Biner {
                        operator: OperatorBiner::Kali,
                        ..
                    }
                ));
            }
            lain => panic!("bukan biner: {:?}", lain),
        }
    }

    #[test]
    fn tes_asosiatif_kiri() {
        let e = urai_ekspresi_sumber("1 - 2 - 3").unwrap();
        match e.jenis {
            JenisEkspresi::Biner { kiri, .. } => {
                assert!(matches!(kiri.jenis, JenisEkspresi::Biner { .. }));
            }
            lain => panic!("bukan biner: {:?}", lain),
        }
    }

    #[test]
    fn tes_unari_lebih_erat_dari_kali() {
        let e = urai_ekspresi_sumber("-a * b").unwrap();
        match e.jenis {
            JenisEkspresi::Biner { kiri, operator, .. } => {
                assert_eq!(operator, OperatorBiner::Kali);
                assert!(matches!(kiri.jenis, JenisEkspresi::Unari { .. }));
            }
            lain => panic!("bukan biner: {:?}", lain),
        }
    }

    #[test]
    fn tes_panggilan_generik_vs_perbandingan() {
        let g = urai_ekspresi_sumber("f<bulat>(1)").unwrap();
        assert!(
            matches!(g.jenis, JenisEkspresi::Panggil { ref argumen_tipe, .. } if argumen_tipe.len() == 1)
        );
        let p = urai_ekspresi_sumber("a < b").unwrap();
        assert!(matches!(
            p.jenis,
            JenisEkspresi::Biner {
                operator: OperatorBiner::LebihKecil,
                ..
            }
        ));
    }

    #[test]
    fn tes_interpolasi_menjadi_ekspresi() {
        let e = urai_ekspresi_sumber("\"a {x + 1} b\"").unwrap();
        match e.jenis {
            JenisEkspresi::Teks(bagian) => {
                assert_eq!(bagian.len(), 3);
                assert!(
                    matches!(&bagian[1], BagianTeks::Ekspresi(x) if matches!(x.jenis, JenisEkspresi::Biner { .. }))
                );
            }
            lain => panic!("bukan teks: {:?}", lain),
        }
    }

    #[test]
    fn tes_baris_baru_memisah_pernyataan() {
        let h = urai("paket u\nfungsi f() {\n    a\n    (b)\n    -c\n}\n");
        assert!(h.berhasil(), "{:?}", h.diagnostik);
        match &h.berkas.deklarasi[0] {
            Deklarasi::Fungsi(f) => assert_eq!(f.badan.pernyataan.len(), 3),
            _ => panic!(),
        }
    }

    #[test]
    fn tes_pemulihan_galat_melanjutkan_ke_fungsi_berikutnya() {
        let h = urai("paket u\nfungsi a( {\n}\nfungsi b() {\n}\n");
        assert!(!h.berhasil());
        let nama: Vec<_> = h
            .berkas
            .deklarasi
            .iter()
            .map(|d| match d {
                Deklarasi::Fungsi(f) => f.nama.nama.clone(),
                _ => String::new(),
            })
            .collect();
        assert!(nama.contains(&"b".to_string()), "{:?}", nama);
    }

    #[test]
    fn tes_banyak_galat_dalam_satu_blok() {
        let h = urai("paket u\nfungsi f() {\n    biar = 1\n    biar y = 2\n    biar = 3\n}\n");
        assert_eq!(h.diagnostik.len(), 2, "{:?}", h.diagnostik);
        match &h.berkas.deklarasi[0] {
            Deklarasi::Fungsi(f) => assert_eq!(f.badan.pernyataan.len(), 1),
            _ => panic!(),
        }
    }

    #[test]
    fn tes_cetak_ulang_idempoten() {
        let h = urai("paket u\nfungsi f(a: bulat) bulat {\n    kembali a+1\n}\n");
        assert!(h.berhasil());
        let p1 = cetak_berkas(&h.berkas);
        let h2 = urai(&p1);
        assert!(h2.berhasil(), "{:?}", h2.diagnostik);
        assert_eq!(cetak_berkas(&h2.berkas), p1);
    }

    #[test]
    fn tes_batas_kedalaman_tidak_overflow() {
        let dalam = format!(
            "paket u\nbiar x = {}1{}\n",
            "(".repeat(2000),
            ")".repeat(2000)
        );
        let h = urai(&dalam);
        assert!(h.diagnostik.iter().any(|d| d.kode == kode::TERLALU_DALAM));
    }

    #[test]
    fn tes_ekspresi_sumber_menolak_sisa() {
        assert!(urai_ekspresi_sumber("1 2").is_err());
        assert!(urai_ekspresi_sumber("1 +").is_err());
        assert!(urai_ekspresi_sumber("a.b(c)[d]").is_ok());
    }
}
