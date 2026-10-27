// Copyright (c) kangpcode (Dhafa Nazula Permadi). Licensed under MIT or Apache-2.0.

//! Pretty-printer AST ke kode Komolang kanonik (indentasi 4 spasi, satu gaya baku).
//! Hasil cetak selalu dapat di-parse ulang menjadi AST yang setara (idempoten).

use crate::*;
use std::fmt;

/// Mencetak seluruh berkas sebagai kode Komolang kanonik.
pub fn cetak_berkas(berkas: &BerkasProgram) -> String {
    let mut pencetak = Pencetak {
        keluar: String::new(),
        indentasi: 0,
    };
    pencetak.berkas(berkas);
    pencetak.keluar
}

pub fn cetak_ekspresi(e: &Ekspresi) -> String {
    ekspresi_str(e)
}

pub fn cetak_tipe(t: &Tipe) -> String {
    tipe_str(t)
}

impl fmt::Display for BerkasProgram {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&cetak_berkas(self))
    }
}

struct Pencetak {
    keluar: String,
    indentasi: usize,
}

fn vis_str(v: Visibilitas) -> &'static str {
    match v {
        Visibilitas::Publik => "publik ",
        Visibilitas::Pribadi => "",
    }
}

fn generik_str(generik: &[Pengenal]) -> String {
    if generik.is_empty() {
        String::new()
    } else {
        format!(
            "<{}>",
            generik
                .iter()
                .map(|g| g.nama.as_str())
                .collect::<Vec<_>>()
                .join(", ")
        )
    }
}

fn parameter_str(parameter: &[Parameter]) -> String {
    parameter
        .iter()
        .map(|p| format!("{}: {}", p.nama.nama, tipe_str(&p.tipe)))
        .collect::<Vec<_>>()
        .join(", ")
}

fn tipe_str(t: &Tipe) -> String {
    match &t.jenis {
        JenisTipe::Nama { nama, argumen } => {
            if argumen.is_empty() {
                nama.nama.clone()
            } else {
                format!(
                    "{}<{}>",
                    nama.nama,
                    argumen.iter().map(tipe_str).collect::<Vec<_>>().join(", ")
                )
            }
        }
        JenisTipe::Pinjam { mutabel, dalam } => {
            format!(
                "pinjam {}{}",
                if *mutabel { "var " } else { "" },
                tipe_str(dalam)
            )
        }
    }
}

fn escape_teks(s: &str) -> String {
    let mut keluar = String::new();
    for c in s.chars() {
        match c {
            '\\' => keluar.push_str("\\\\"),
            '"' => keluar.push_str("\\\""),
            '\n' => keluar.push_str("\\n"),
            '\t' => keluar.push_str("\\t"),
            '\r' => keluar.push_str("\\r"),
            '\0' => keluar.push_str("\\0"),
            '{' => keluar.push_str("{{"),
            '}' => keluar.push_str("}}"),
            c if c.is_control() => keluar.push_str(&format!("\\u{{{:x}}}", c as u32)),
            c => keluar.push(c),
        }
    }
    keluar
}

fn escape_rune(c: char) -> String {
    match c {
        '\\' => "\\\\".to_string(),
        '\'' => "\\'".to_string(),
        '\n' => "\\n".to_string(),
        '\t' => "\\t".to_string(),
        '\r' => "\\r".to_string(),
        '\0' => "\\0".to_string(),
        c if c.is_control() => format!("\\u{{{:x}}}", c as u32),
        c => c.to_string(),
    }
}

fn bungkus_jika(e: &Ekspresi, perlu: bool) -> String {
    if perlu {
        format!("({})", ekspresi_str(e))
    } else {
        ekspresi_str(e)
    }
}

fn ekspresi_str(e: &Ekspresi) -> String {
    match &e.jenis {
        JenisEkspresi::Bulat(n) => n.to_string(),
        JenisEkspresi::Desimal(f) => format!("{:?}", f),
        JenisEkspresi::Teks(bagian) => {
            let mut s = String::from("\"");
            for b in bagian {
                match b {
                    BagianTeks::Teks(t) => s.push_str(&escape_teks(t)),
                    BagianTeks::Ekspresi(x) => {
                        s.push('{');
                        s.push_str(&ekspresi_str(x));
                        s.push('}');
                    }
                }
            }
            s.push('"');
            s
        }
        JenisEkspresi::Rune(c) => format!("'{}'", escape_rune(*c)),
        JenisEkspresi::Bool(true) => "benar".to_string(),
        JenisEkspresi::Bool(false) => "salah".to_string(),
        JenisEkspresi::Kosong => "kosong".to_string(),
        JenisEkspresi::Pengenal(p) => p.nama.clone(),
        JenisEkspresi::Biner {
            operator,
            kiri,
            kanan,
        } => {
            let p = operator.prioritas();
            format!(
                "{} {} {}",
                bungkus_jika(kiri, kiri.prioritas() < p),
                operator.simbol(),
                bungkus_jika(kanan, kanan.prioritas() <= p)
            )
        }
        JenisEkspresi::Unari { operator, operan } => {
            format!(
                "{}{}",
                operator.simbol(),
                bungkus_jika(operan, operan.prioritas() < PRIORITAS_UNARI)
            )
        }
        JenisEkspresi::Panggil {
            fungsi,
            argumen_tipe,
            argumen,
        } => {
            let tipe = if argumen_tipe.is_empty() {
                String::new()
            } else {
                format!(
                    "<{}>",
                    argumen_tipe
                        .iter()
                        .map(tipe_str)
                        .collect::<Vec<_>>()
                        .join(", ")
                )
            };
            format!(
                "{}{}({})",
                bungkus_jika(fungsi, fungsi.prioritas() < PRIORITAS_POSTFIKS),
                tipe,
                argumen
                    .iter()
                    .map(ekspresi_str)
                    .collect::<Vec<_>>()
                    .join(", ")
            )
        }
        JenisEkspresi::Bidang { objek, nama } => {
            format!(
                "{}.{}",
                bungkus_jika(objek, objek.prioritas() < PRIORITAS_POSTFIKS),
                nama.nama
            )
        }
        JenisEkspresi::Indeks { objek, indeks } => {
            format!(
                "{}[{}]",
                bungkus_jika(objek, objek.prioritas() < PRIORITAS_POSTFIKS),
                ekspresi_str(indeks)
            )
        }
        JenisEkspresi::Struktur { nama, bidang } => {
            format!(
                "{}{{{}}}",
                nama.nama,
                bidang
                    .iter()
                    .map(|b| format!("{}: {}", b.nama.nama, ekspresi_str(&b.nilai)))
                    .collect::<Vec<_>>()
                    .join(", ")
            )
        }
        JenisEkspresi::Larik(item) => {
            format!(
                "[{}]",
                item.iter().map(ekspresi_str).collect::<Vec<_>>().join(", ")
            )
        }
        JenisEkspresi::Rentang {
            awal,
            akhir,
            inklusif,
        } => {
            format!(
                "{}{}{}",
                bungkus_jika(awal, awal.prioritas() <= PRIORITAS_RENTANG),
                if *inklusif { "..=" } else { ".." },
                bungkus_jika(akhir, akhir.prioritas() <= PRIORITAS_RENTANG)
            )
        }
    }
}

/// Literal struktur tidak boleh muncul telanjang pada posisi kondisi (`jika x {`),
/// sehingga ekspresi yang memuatnya dibungkus kurung.
fn memuat_struktur(e: &Ekspresi) -> bool {
    match &e.jenis {
        JenisEkspresi::Struktur { .. } => true,
        JenisEkspresi::Biner { kiri, kanan, .. } => memuat_struktur(kiri) || memuat_struktur(kanan),
        JenisEkspresi::Unari { operan, .. } => memuat_struktur(operan),
        JenisEkspresi::Panggil {
            fungsi, argumen, ..
        } => memuat_struktur(fungsi) || argumen.iter().any(memuat_struktur),
        JenisEkspresi::Bidang { objek, .. } => memuat_struktur(objek),
        JenisEkspresi::Indeks { objek, indeks } => memuat_struktur(objek) || memuat_struktur(indeks),
        JenisEkspresi::Larik(item) => item.iter().any(memuat_struktur),
        JenisEkspresi::Rentang { awal, akhir, .. } => memuat_struktur(awal) || memuat_struktur(akhir),
        JenisEkspresi::Teks(bagian) => bagian
            .iter()
            .any(|b| matches!(b, BagianTeks::Ekspresi(x) if memuat_struktur(x))),
        _ => false,
    }
}

fn kondisi_str(e: &Ekspresi) -> String {
    if memuat_struktur(e) {
        format!("({})", ekspresi_str(e))
    } else {
        ekspresi_str(e)
    }
}

fn pola_str(p: &Pola) -> String {
    match &p.jenis {
        JenisPola::Wildcard => "_".to_string(),
        JenisPola::Literal(e) => ekspresi_str(e),
        JenisPola::Pengenal(n) => n.nama.clone(),
        JenisPola::Varian { nama, argumen } => format!(
            "{}({})",
            nama.nama,
            argumen.iter().map(pola_str).collect::<Vec<_>>().join(", ")
        ),
    }
}

impl Pencetak {
    fn tulis(&mut self, s: &str) {
        self.keluar.push_str(s);
    }

    fn indent(&mut self) {
        for _ in 0..self.indentasi {
            self.keluar.push_str("    ");
        }
    }

    fn berkas(&mut self, b: &BerkasProgram) {
        self.tulis(&format!("paket {}\n", b.paket.nama.nama));
        if !b.impor.is_empty() {
            self.tulis("\n");
            for i in &b.impor {
                self.tulis(&format!("impor \"{}\"", escape_teks(&i.jalur)));
                if let Some(alias) = &i.alias {
                    self.tulis(&format!(" sebagai {}", alias.nama));
                }
                self.tulis("\n");
            }
        }
        for d in &b.deklarasi {
            self.tulis("\n");
            self.deklarasi(d);
        }
    }

    fn deklarasi(&mut self, d: &Deklarasi) {
        match d {
            Deklarasi::Fungsi(f) => self.fungsi(f),
            Deklarasi::Tipe(t) => self.tipe(t),
            Deklarasi::Konstanta(k) => {
                self.tulis(vis_str(k.visibilitas));
                self.tulis(&format!("konst {}", k.nama.nama));
                if let Some(t) = &k.tipe {
                    self.tulis(&format!(": {}", tipe_str(t)));
                }
                self.tulis(&format!(" = {}\n", ekspresi_str(&k.nilai)));
            }
            Deklarasi::Variabel(v) => {
                self.tulis(vis_str(v.visibilitas));
                self.tulis(if v.mutabel { "var " } else { "biar " });
                self.tulis(&v.nama.nama);
                if let Some(t) = &v.tipe {
                    self.tulis(&format!(": {}", tipe_str(t)));
                }
                self.tulis(&format!(" = {}\n", ekspresi_str(&v.nilai)));
            }
        }
    }

    fn fungsi(&mut self, f: &DeklarasiFungsi) {
        self.tulis(vis_str(f.visibilitas));
        self.tulis("fungsi ");
        if let Some(p) = &f.penerima {
            self.tulis(&format!("({}: {}) ", p.nama.nama, tipe_str(&p.tipe)));
        }
        self.tulis(&f.nama.nama);
        self.tulis(&generik_str(&f.generik));
        self.tulis(&format!("({})", parameter_str(&f.parameter)));
        if let Some(t) = &f.tipe_kembalian {
            self.tulis(&format!(" {}", tipe_str(t)));
        }
        self.tulis(" ");
        self.blok(&f.badan);
        self.tulis("\n");
    }

    fn tipe(&mut self, t: &DeklarasiTipe) {
        self.tulis(vis_str(t.visibilitas));
        self.tulis(&format!("tipe {}{}", t.nama.nama, generik_str(&t.generik)));
        match &t.isi {
            IsiTipe::Alias(a) => {
                self.tulis(&format!(" {}\n", tipe_str(a)));
            }
            IsiTipe::Struktur(bidang) => {
                self.tulis(" struktur {\n");
                self.indentasi += 1;
                for b in bidang {
                    self.indent();
                    self.tulis(vis_str(b.visibilitas));
                    self.tulis(&format!("{}: {}\n", b.nama.nama, tipe_str(&b.tipe)));
                }
                self.indentasi -= 1;
                self.tulis("}\n");
            }
            IsiTipe::Antarmuka(metode) => {
                self.tulis(" antarmuka {\n");
                self.indentasi += 1;
                for m in metode {
                    self.indent();
                    self.tulis(&format!("{}({})", m.nama.nama, parameter_str(&m.parameter)));
                    if let Some(r) = &m.tipe_kembalian {
                        self.tulis(&format!(" {}", tipe_str(r)));
                    }
                    self.tulis("\n");
                }
                self.indentasi -= 1;
                self.tulis("}\n");
            }
            IsiTipe::Enum(varian) => {
                self.tulis(" enum {\n");
                self.indentasi += 1;
                for v in varian {
                    self.indent();
                    self.tulis(&v.nama.nama);
                    if !v.tipe.is_empty() {
                        self.tulis(&format!(
                            "({})",
                            v.tipe.iter().map(tipe_str).collect::<Vec<_>>().join(", ")
                        ));
                    }
                    self.tulis("\n");
                }
                self.indentasi -= 1;
                self.tulis("}\n");
            }
        }
    }

    /// Menulis `{ ... }` tanpa indentasi awal dan tanpa baris baru akhir.
    fn blok(&mut self, b: &Blok) {
        self.tulis("{\n");
        self.indentasi += 1;
        for p in &b.pernyataan {
            self.pernyataan(p);
        }
        self.indentasi -= 1;
        self.indent();
        self.tulis("}");
    }

    fn pernyataan(&mut self, p: &Pernyataan) {
        self.indent();
        self.isi(p);
        self.tulis("\n");
    }

    /// Isi pernyataan tanpa indentasi awal dan tanpa baris baru akhir.
    fn isi(&mut self, p: &Pernyataan) {
        match &p.jenis {
            JenisPernyataan::Variabel {
                mutabel,
                nama,
                tipe,
                nilai,
            } => {
                self.tulis(if *mutabel { "var " } else { "biar " });
                self.tulis(&nama.nama);
                if let Some(t) = tipe {
                    self.tulis(&format!(": {}", tipe_str(t)));
                }
                self.tulis(&format!(" = {}", ekspresi_str(nilai)));
            }
            JenisPernyataan::Tugas {
                target,
                operator,
                nilai,
            } => {
                self.tulis(&format!(
                    "{} {} {}",
                    ekspresi_str(target),
                    operator.simbol(),
                    ekspresi_str(nilai)
                ));
            }
            JenisPernyataan::Jika {
                kondisi,
                maka,
                lainnya,
            } => {
                self.tulis(&format!("jika {} ", kondisi_str(kondisi)));
                self.blok(maka);
                if let Some(l) = lainnya {
                    self.tulis(" lainnya ");
                    self.isi(l);
                }
            }
            JenisPernyataan::Selagi { kondisi, badan } => {
                self.tulis(&format!("selagi {} ", kondisi_str(kondisi)));
                self.blok(badan);
            }
            JenisPernyataan::Untuk {
                variabel,
                koleksi,
                badan,
            } => {
                self.tulis(&format!(
                    "untuk {} dalam {} ",
                    variabel.nama,
                    kondisi_str(koleksi)
                ));
                self.blok(badan);
            }
            JenisPernyataan::Pilih {
                nilai,
                kasus,
                lainnya,
            } => {
                self.tulis(&format!("pilih {} {{\n", kondisi_str(nilai)));
                self.indentasi += 1;
                for k in kasus {
                    self.indent();
                    self.tulis(&format!("kasus {}:\n", kondisi_str(&k.nilai)));
                    self.indentasi += 1;
                    for s in &k.badan {
                        self.pernyataan(s);
                    }
                    self.indentasi -= 1;
                }
                if let Some(l) = lainnya {
                    self.indent();
                    self.tulis("lainnya:\n");
                    self.indentasi += 1;
                    for s in l {
                        self.pernyataan(s);
                    }
                    self.indentasi -= 1;
                }
                self.indentasi -= 1;
                self.indent();
                self.tulis("}");
            }
            JenisPernyataan::Cocok { nilai, cabang } => {
                self.tulis(&format!("cocok {} {{\n", kondisi_str(nilai)));
                self.indentasi += 1;
                for c in cabang {
                    self.indent();
                    self.tulis(&format!("{}: ", pola_str(&c.pola)));
                    self.isi(&c.badan);
                    self.tulis("\n");
                }
                self.indentasi -= 1;
                self.indent();
                self.tulis("}");
            }
            JenisPernyataan::Tunda(e) => self.tulis(&format!("tunda {}", ekspresi_str(e))),
            JenisPernyataan::TundaBlok(b) => {
                self.tulis("tunda ");
                self.blok(b);
            }
            JenisPernyataan::Jalankan(e) => self.tulis(&format!("jalankan {}", ekspresi_str(e))),
            JenisPernyataan::Kembali(None) => self.tulis("kembali"),
            JenisPernyataan::Kembali(Some(e)) => {
                self.tulis(&format!("kembali {}", ekspresi_str(e)))
            }
            JenisPernyataan::TidakAman(b) => {
                self.tulis("tidak_aman ");
                self.blok(b);
            }
            JenisPernyataan::Blok(b) => self.blok(b),
            JenisPernyataan::Ekspresi(e) => self.tulis(&ekspresi_str(e)),
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

    fn pengenal(n: &str) -> Ekspresi {
        Ekspresi::baru(JenisEkspresi::Pengenal(id(n)), r())
    }

    fn bulat(n: u64) -> Ekspresi {
        Ekspresi::baru(JenisEkspresi::Bulat(n), r())
    }

    fn biner(op: OperatorBiner, a: Ekspresi, b: Ekspresi) -> Ekspresi {
        Ekspresi::baru(
            JenisEkspresi::Biner {
                operator: op,
                kiri: Box::new(a),
                kanan: Box::new(b),
            },
            r(),
        )
    }

    #[test]
    fn tes_kurung_mengikuti_prioritas() {
        let e = biner(
            OperatorBiner::Kali,
            biner(OperatorBiner::Tambah, bulat(1), bulat(2)),
            bulat(3),
        );
        assert_eq!(cetak_ekspresi(&e), "(1 + 2) * 3");
    }

    #[test]
    fn tes_tanpa_kurung_berlebih() {
        let e = biner(
            OperatorBiner::Tambah,
            bulat(1),
            biner(OperatorBiner::Kali, bulat(2), bulat(3)),
        );
        assert_eq!(cetak_ekspresi(&e), "1 + 2 * 3");
    }

    #[test]
    fn tes_asosiatif_kiri_kanan() {
        let kanan = biner(
            OperatorBiner::Kurang,
            bulat(1),
            biner(OperatorBiner::Kurang, bulat(2), bulat(3)),
        );
        assert_eq!(cetak_ekspresi(&kanan), "1 - (2 - 3)");
        let kiri = biner(
            OperatorBiner::Kurang,
            biner(OperatorBiner::Kurang, bulat(1), bulat(2)),
            bulat(3),
        );
        assert_eq!(cetak_ekspresi(&kiri), "1 - 2 - 3");
    }

    #[test]
    fn tes_escape_teks_dan_interpolasi() {
        let e = Ekspresi::baru(
            JenisEkspresi::Teks(vec![
                BagianTeks::Teks("a\"b{c}\n".to_string()),
                BagianTeks::Ekspresi(Box::new(pengenal("x"))),
            ]),
            r(),
        );
        assert_eq!(cetak_ekspresi(&e), "\"a\\\"b{{c}}\\n{x}\"");
    }

    #[test]
    fn tes_desimal_tetap_bisa_dibaca_ulang() {
        let e = Ekspresi::baru(JenisEkspresi::Desimal(2.0), r());
        assert_eq!(cetak_ekspresi(&e), "2.0");
    }

    #[test]
    fn tes_cetak_berkas_minimal() {
        let b = BerkasProgram {
            paket: DeklarasiPaket {
                nama: id("utama"),
                rentang: r(),
            },
            impor: vec![DeklarasiImpor {
                jalur: "std/cetak".to_string(),
                alias: None,
                rentang: r(),
            }],
            deklarasi: Vec::new(),
        };
        assert_eq!(cetak_berkas(&b), "paket utama\n\nimpor \"std/cetak\"\n");
        assert_eq!(b.to_string(), cetak_berkas(&b));
    }

    #[test]
    fn tes_struktur_dalam_kondisi_dibungkus() {
        let s = Ekspresi::baru(
            JenisEkspresi::Struktur {
                nama: id("P"),
                bidang: Vec::new(),
            },
            r(),
        );
        assert_eq!(kondisi_str(&s), "(P{})");
        assert_eq!(kondisi_str(&pengenal("x")), "x");
    }
}
