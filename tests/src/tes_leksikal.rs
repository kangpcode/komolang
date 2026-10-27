// Copyright (c) kangpcode (Dhafa Nazula Permadi). Licensed under MIT or Apache-2.0.

//! Suite pengujian leksikal menyeluruh untuk `komo_lexer`.
//! Mencakup kata kunci, pengenal, literal (angka, teks, rune), interpolasi,
//! komentar, operator, tanda baca, UTF-8, dan diagnostik leksikal.

use komo_diag::kode;
use komo_lexer::{pindai, BagianToken, JenisToken};

fn jenis_token(sumber: &str) -> Vec<JenisToken> {
    let hasil = pindai(sumber);
    assert!(
        hasil.diagnostik.is_empty(),
        "Ditemukan diagnostik tak terduga: {:?}",
        hasil.diagnostik
    );
    hasil.token.into_iter().map(|t| t.jenis).collect()
}

fn jenis_token_tunggal(sumber: &str) -> JenisToken {
    let mut tokens = jenis_token(sumber);
    assert_eq!(
        tokens.pop(),
        Some(JenisToken::AkhirBerkas),
        "Harus berujung AkhirBerkas"
    );
    assert_eq!(
        tokens.len(),
        1,
        "Diharapkan tepat 1 token sebelum AkhirBerkas"
    );
    tokens.remove(0)
}

#[test]
fn tes_lex_kata_kunci_semua() {
    let kata = vec![
        ("paket", JenisToken::Paket),
        ("impor", JenisToken::Impor),
        ("fungsi", JenisToken::Fungsi),
        ("kembali", JenisToken::Kembali),
        ("biar", JenisToken::Biar),
        ("var", JenisToken::Var),
        ("konst", JenisToken::Konst),
        ("jika", JenisToken::Jika),
        ("lainnya", JenisToken::Lainnya),
        ("selagi", JenisToken::Selagi),
        ("untuk", JenisToken::Untuk),
        ("dalam", JenisToken::Dalam),
        ("pilih", JenisToken::Pilih),
        ("kasus", JenisToken::Kasus),
        ("cocok", JenisToken::Cocok),
        ("tipe", JenisToken::Tipe),
        ("struktur", JenisToken::Struktur),
        ("antarmuka", JenisToken::Antarmuka),
        ("enum", JenisToken::Enum),
        ("jalankan", JenisToken::Jalankan),
        ("saluran", JenisToken::Saluran),
        ("tunda", JenisToken::Tunda),
        ("benar", JenisToken::Benar),
        ("salah", JenisToken::Salah),
        ("kosong", JenisToken::Kosong),
        ("pinjam", JenisToken::Pinjam),
        ("pindah", JenisToken::Pindah),
        ("publik", JenisToken::Publik),
        ("pribadi", JenisToken::Pribadi),
        ("tidak_aman", JenisToken::TidakAman),
        ("dan", JenisToken::Dan),
        ("atau", JenisToken::Atau),
        ("sebagai", JenisToken::Sebagai),
    ];

    for (teks, diharapkan) in kata {
        assert_eq!(
            jenis_token_tunggal(teks),
            diharapkan,
            "Gagal untuk kata kunci '{}'",
            teks
        );
    }
}

#[test]
fn tes_lex_pengenal_biasa() {
    assert_eq!(
        jenis_token_tunggal("hitung"),
        JenisToken::Pengenal("hitung".into())
    );
    assert_eq!(
        jenis_token_tunggal("namaLengkap"),
        JenisToken::Pengenal("namaLengkap".into())
    );
    assert_eq!(
        jenis_token_tunggal("total_harga_2026"),
        JenisToken::Pengenal("total_harga_2026".into())
    );
}

#[test]
fn tes_lex_pengenal_garis_bawah() {
    assert_eq!(jenis_token_tunggal("_"), JenisToken::GarisBawah);
    assert_eq!(
        jenis_token_tunggal("_rahasia"),
        JenisToken::Pengenal("_rahasia".into())
    );
    assert_eq!(
        jenis_token_tunggal("__init__"),
        JenisToken::Pengenal("__init__".into())
    );
}

#[test]
fn tes_lex_pengenal_panjang_dan_angka() {
    assert_eq!(
        jenis_token_tunggal("kucing_lucu_123"),
        JenisToken::Pengenal("kucing_lucu_123".into())
    );
    assert_eq!(
        jenis_token_tunggal("TotalHarga_2026"),
        JenisToken::Pengenal("TotalHarga_2026".into())
    );
    assert_eq!(
        jenis_token_tunggal("variabel_indonesia"),
        JenisToken::Pengenal("variabel_indonesia".into())
    );
}

#[test]
fn tes_lex_angka_bulat_desimal_dasar() {
    assert_eq!(jenis_token_tunggal("0"), JenisToken::LiteralBulat(0));
    assert_eq!(jenis_token_tunggal("42"), JenisToken::LiteralBulat(42));
    assert_eq!(
        jenis_token_tunggal("1000000"),
        JenisToken::LiteralBulat(1_000_000)
    );
}

#[test]
fn tes_lex_angka_bulat_pemisah_garis_bawah() {
    assert_eq!(
        jenis_token_tunggal("1_000_000"),
        JenisToken::LiteralBulat(1_000_000)
    );
    assert_eq!(
        jenis_token_tunggal("12_34_56"),
        JenisToken::LiteralBulat(123456)
    );
}

#[test]
fn tes_lex_angka_heksadesimal() {
    assert_eq!(jenis_token_tunggal("0x0"), JenisToken::LiteralBulat(0));
    assert_eq!(jenis_token_tunggal("0xFF"), JenisToken::LiteralBulat(255));
    assert_eq!(
        jenis_token_tunggal("0x1a3F"),
        JenisToken::LiteralBulat(0x1a3f)
    );
    assert_eq!(
        jenis_token_tunggal("0xCAFE_BABE"),
        JenisToken::LiteralBulat(0xCAFEBABE)
    );
}

#[test]
fn tes_lex_angka_biner() {
    assert_eq!(jenis_token_tunggal("0b0"), JenisToken::LiteralBulat(0));
    assert_eq!(jenis_token_tunggal("0b1010"), JenisToken::LiteralBulat(10));
    assert_eq!(
        jenis_token_tunggal("0b1111_0000"),
        JenisToken::LiteralBulat(0xF0)
    );
}

#[test]
fn tes_lex_angka_oktal() {
    assert_eq!(jenis_token_tunggal("0o0"), JenisToken::LiteralBulat(0));
    assert_eq!(jenis_token_tunggal("0o77"), JenisToken::LiteralBulat(63));
    assert_eq!(
        jenis_token_tunggal("0o755"),
        JenisToken::LiteralBulat(0o755)
    );
    assert_eq!(
        jenis_token_tunggal("0o12_34"),
        JenisToken::LiteralBulat(0o1234)
    );
}

#[test]
fn tes_lex_angka_desimal_pecahan() {
    match jenis_token_tunggal("0.0") {
        JenisToken::LiteralDesimal(d) => assert!((d - 0.0).abs() < f64::EPSILON),
        lain => panic!("diharapkan desimal, dapat {:?}", lain),
    }
    match jenis_token_tunggal("12.3456") {
        JenisToken::LiteralDesimal(d) => assert!((d - 12.3456).abs() < 1e-5),
        lain => panic!("diharapkan desimal, dapat {:?}", lain),
    }
    match jenis_token_tunggal("1_000.500_25") {
        JenisToken::LiteralDesimal(d) => assert!((d - 1000.50025).abs() < 1e-5),
        lain => panic!("diharapkan desimal, dapat {:?}", lain),
    }
}

#[test]
fn tes_lex_angka_desimal_eksponen() {
    match jenis_token_tunggal("1e10") {
        JenisToken::LiteralDesimal(d) => assert!((d - 1e10).abs() < 1.0),
        lain => panic!("diharapkan desimal, dapat {:?}", lain),
    }
    match jenis_token_tunggal("2.5e-3") {
        JenisToken::LiteralDesimal(d) => assert!((d - 0.0025).abs() < 1e-6),
        lain => panic!("diharapkan desimal, dapat {:?}", lain),
    }
    match jenis_token_tunggal("1.2E+4") {
        JenisToken::LiteralDesimal(d) => assert!((d - 12000.0).abs() < 1e-3),
        lain => panic!("diharapkan desimal, dapat {:?}", lain),
    }
}

#[test]
fn tes_lex_rentang_vs_desimal() {
    let tokens = jenis_token("0..5");
    assert_eq!(
        tokens,
        vec![
            JenisToken::LiteralBulat(0),
            JenisToken::TitikTitik,
            JenisToken::LiteralBulat(5),
            JenisToken::AkhirBerkas,
        ]
    );

    let tokens_inklusif = jenis_token("0..=5");
    assert_eq!(
        tokens_inklusif,
        vec![
            JenisToken::LiteralBulat(0),
            JenisToken::TitikTitikSama,
            JenisToken::LiteralBulat(5),
            JenisToken::AkhirBerkas,
        ]
    );
}

#[test]
fn tes_lex_teks_kosong_dan_sederhana() {
    match jenis_token_tunggal("\"\"") {
        JenisToken::LiteralTeks(b) => {
            assert_eq!(b.len(), 1);
            assert_eq!(b[0], BagianToken::Teks(String::new()));
        }
        lain => panic!("diharapkan teks kosong, dapat {:?}", lain),
    }
    match jenis_token_tunggal("\"Halo Dunia\"") {
        JenisToken::LiteralTeks(b) => {
            assert_eq!(b.len(), 1);
            assert_eq!(b[0], BagianToken::Teks("Halo Dunia".into()));
        }
        lain => panic!("diharapkan teks, dapat {:?}", lain),
    }
}

#[test]
fn tes_lex_teks_escape_lengkap() {
    let sumber = r#""\n\t\r\\\"\0""#;
    match jenis_token_tunggal(sumber) {
        JenisToken::LiteralTeks(b) => {
            assert_eq!(b.len(), 1);
            assert_eq!(b[0], BagianToken::Teks("\n\t\r\\\"\0".into()));
        }
        lain => panic!("diharapkan teks escape, dapat {:?}", lain),
    }
}

#[test]
fn tes_lex_teks_kurung_kurawal_escape() {
    let sumber = r#""{{Halo}}""#;
    match jenis_token_tunggal(sumber) {
        JenisToken::LiteralTeks(b) => {
            assert_eq!(b.len(), 1);
            assert_eq!(b[0], BagianToken::Teks("{Halo}".into()));
        }
        lain => panic!("diharapkan kurung kurawal harfiah, dapat {:?}", lain),
    }
}

#[test]
fn tes_lex_teks_interpolasi_satu_ekspresi() {
    let sumber = r#""Halo, {nama}!""#;
    match jenis_token_tunggal(sumber) {
        JenisToken::LiteralTeks(b) => {
            assert_eq!(b.len(), 3);
            assert_eq!(b[0], BagianToken::Teks("Halo, ".into()));
            match &b[1] {
                BagianToken::Ekspresi(toks) => {
                    assert_eq!(toks.len(), 1);
                    assert_eq!(toks[0].jenis, JenisToken::Pengenal("nama".into()));
                }
                _ => panic!("diharapkan bagian ekspresi"),
            }
            assert_eq!(b[2], BagianToken::Teks("!".into()));
        }
        lain => panic!("diharapkan literal teks terinterpolasi, dapat {:?}", lain),
    }
}

#[test]
fn tes_lex_teks_interpolasi_majemuk() {
    let sumber = r#""a = {x + 1}, b = {y * 2}""#;
    match jenis_token_tunggal(sumber) {
        JenisToken::LiteralTeks(b) => {
            assert_eq!(b.len(), 4);
            assert_eq!(b[0], BagianToken::Teks("a = ".into()));
            assert!(matches!(b[1], BagianToken::Ekspresi(_)));
            assert_eq!(b[2], BagianToken::Teks(", b = ".into()));
            assert!(matches!(b[3], BagianToken::Ekspresi(_)));
        }
        lain => panic!("diharapkan teks majemuk, dapat {:?}", lain),
    }
}

#[test]
fn tes_lex_teks_multibaris_dan_unicode() {
    let sumber = "\"baris 1\nbaris 2\nkomodo 🦎\"";
    match jenis_token_tunggal(sumber) {
        JenisToken::LiteralTeks(b) => {
            assert_eq!(b.len(), 1);
            assert_eq!(
                b[0],
                BagianToken::Teks("baris 1\nbaris 2\nkomodo 🦎".into())
            );
        }
        lain => panic!("diharapkan teks multibaris, dapat {:?}", lain),
    }
}

#[test]
fn tes_lex_rune_karakter_biasa() {
    assert_eq!(jenis_token_tunggal("'a'"), JenisToken::LiteralRune('a'));
    assert_eq!(jenis_token_tunggal("'Z'"), JenisToken::LiteralRune('Z'));
    assert_eq!(jenis_token_tunggal("'9'"), JenisToken::LiteralRune('9'));
}

#[test]
fn tes_lex_rune_escape() {
    assert_eq!(jenis_token_tunggal(r"'\n'"), JenisToken::LiteralRune('\n'));
    assert_eq!(jenis_token_tunggal(r"'\t'"), JenisToken::LiteralRune('\t'));
    assert_eq!(jenis_token_tunggal(r"'\r'"), JenisToken::LiteralRune('\r'));
    assert_eq!(jenis_token_tunggal(r"'\\'"), JenisToken::LiteralRune('\\'));
    assert_eq!(jenis_token_tunggal(r"'\''"), JenisToken::LiteralRune('\''));
    assert_eq!(jenis_token_tunggal(r"'\0'"), JenisToken::LiteralRune('\0'));
}

#[test]
fn tes_lex_rune_unicode() {
    assert_eq!(jenis_token_tunggal("'α'"), JenisToken::LiteralRune('α'));
    assert_eq!(jenis_token_tunggal("'🦎'"), JenisToken::LiteralRune('🦎'));
}

#[test]
fn tes_lex_operator_aritmatika_dan_tugas() {
    let ops = vec![
        ("+", JenisToken::Tambah),
        ("-", JenisToken::Kurang),
        ("*", JenisToken::Kali),
        ("/", JenisToken::Bagi),
        ("%", JenisToken::SisaBagi),
        ("=", JenisToken::SamaDengan),
        ("+=", JenisToken::TambahSama),
        ("-=", JenisToken::KurangSama),
        ("*=", JenisToken::KaliSama),
        ("/=", JenisToken::BagiSama),
        ("%=", JenisToken::SisaSama),
    ];
    for (sumber, jenis) in ops {
        assert_eq!(jenis_token_tunggal(sumber), jenis);
    }
}

#[test]
fn tes_lex_operator_perbandingan_dan_logika() {
    let ops = vec![
        ("==", JenisToken::SamaKetat),
        ("!=", JenisToken::TidakSama),
        ("<", JenisToken::KurangDari),
        ("<=", JenisToken::KurangDariSama),
        (">", JenisToken::LebihDari),
        (">=", JenisToken::LebihDariSama),
        ("!", JenisToken::BukanLogika),
    ];
    for (sumber, jenis) in ops {
        assert_eq!(jenis_token_tunggal(sumber), jenis);
    }
}

#[test]
fn tes_lex_tanda_baca() {
    let tanda = vec![
        (".", JenisToken::Titik),
        (":", JenisToken::TitikDua),
        (",", JenisToken::Koma),
        (";", JenisToken::TitikKoma),
        ("(", JenisToken::KurungBuka),
        (")", JenisToken::KurungTutup),
        ("{", JenisToken::KurungKurawalBuka),
        ("}", JenisToken::KurungKurawalTutup),
        ("[", JenisToken::KurungSikuBuka),
        ("]", JenisToken::KurungSikuTutup),
    ];
    for (sumber, jenis) in tanda {
        assert_eq!(jenis_token_tunggal(sumber), jenis);
    }
}

#[test]
fn tes_lex_komentar_sebaris() {
    let sumber = "// ini komentar\n42";
    let toks = jenis_token(sumber);
    assert_eq!(
        toks,
        vec![JenisToken::LiteralBulat(42), JenisToken::AkhirBerkas]
    );
}

#[test]
fn tes_lex_komentar_blok() {
    let sumber = "/* komentar blok */ 42";
    let toks = jenis_token(sumber);
    assert_eq!(
        toks,
        vec![JenisToken::LiteralBulat(42), JenisToken::AkhirBerkas]
    );
}

#[test]
fn tes_lex_komentar_blok_bersarang_dalam() {
    let sumber = "/* satu /* dua /* tiga */ dua */ satu */ 100";
    let toks = jenis_token(sumber);
    assert_eq!(
        toks,
        vec![JenisToken::LiteralBulat(100), JenisToken::AkhirBerkas]
    );
}

#[test]
fn tes_lex_galat_karakter_tak_dikenali() {
    let hasil = pindai("@");
    assert_eq!(hasil.diagnostik.len(), 1);
    assert_eq!(hasil.diagnostik[0].kode, kode::KARAKTER_TAK_DIKENALI);
}

#[test]
fn tes_lex_galat_teks_tidak_ditutup() {
    let hasil = pindai("\"tidak ditutup");
    assert_eq!(hasil.diagnostik.len(), 1);
    assert_eq!(hasil.diagnostik[0].kode, kode::TEKS_TIDAK_DITUTUP);
}

#[test]
fn tes_lex_galat_komentar_tidak_ditutup() {
    let hasil = pindai("/* komentar terbuka");
    assert_eq!(hasil.diagnostik.len(), 1);
    assert_eq!(hasil.diagnostik[0].kode, kode::KOMENTAR_TIDAK_DITUTUP);
}

#[test]
fn tes_lex_galat_escape_tidak_valid() {
    let hasil = pindai(r#""\q""#);
    assert_eq!(hasil.diagnostik.len(), 1);
    assert_eq!(hasil.diagnostik[0].kode, kode::ESCAPE_TIDAK_VALID);
}

#[test]
fn tes_lex_galat_angka_heks_kosong() {
    let hasil = pindai("0x");
    assert_eq!(hasil.diagnostik.len(), 1);
    assert_eq!(hasil.diagnostik[0].kode, kode::ANGKA_TIDAK_VALID);
}

#[test]
fn tes_lex_galat_angka_biner_kosong() {
    let hasil = pindai("0b");
    assert_eq!(hasil.diagnostik.len(), 1);
    assert_eq!(hasil.diagnostik[0].kode, kode::ANGKA_TIDAK_VALID);
}

#[test]
fn tes_lex_galat_angka_oktal_kosong() {
    let hasil = pindai("0o");
    assert_eq!(hasil.diagnostik.len(), 1);
    assert_eq!(hasil.diagnostik[0].kode, kode::ANGKA_TIDAK_VALID);
}

#[test]
fn tes_lex_galat_interpolasi_tidak_ditutup() {
    let hasil = pindai("\"nilai = {x\"");
    assert_eq!(hasil.diagnostik.len(), 1);
    assert_eq!(hasil.diagnostik[0].kode, kode::INTERPOLASI_TIDAK_DITUTUP);
}

#[test]
fn tes_lex_galat_kurung_kurawal_tunggal() {
    let hasil = pindai("\"halo { dunia\"");
    assert!(hasil.diagnostik.iter().any(
        |d| d.kode == kode::KURUNG_KURAWAL_TUNGGAL || d.kode == kode::INTERPOLASI_TIDAK_DITUTUP
    ));
}

#[test]
fn tes_lex_galat_interpolasi_kosong() {
    let hasil = pindai("\"halo {}\"");
    assert_eq!(hasil.diagnostik.len(), 1);
    assert_eq!(hasil.diagnostik[0].kode, kode::INTERPOLASI_KOSONG);
}

#[test]
fn tes_lex_galat_rune_kosong() {
    let hasil = pindai("''");
    assert_eq!(hasil.diagnostik.len(), 1);
    assert_eq!(hasil.diagnostik[0].kode, kode::RUNE_TIDAK_VALID);
}

#[test]
fn tes_lex_galat_rune_lebih_dari_satu() {
    let hasil = pindai("'ab'");
    assert_eq!(hasil.diagnostik.len(), 1);
    assert_eq!(hasil.diagnostik[0].kode, kode::RUNE_TIDAK_VALID);
}

#[test]
fn tes_lex_galat_rune_tidak_ditutup() {
    let hasil = pindai("'a");
    assert_eq!(hasil.diagnostik.len(), 1);
    assert_eq!(hasil.diagnostik[0].kode, kode::RUNE_TIDAK_VALID);
}

#[test]
fn tes_lex_pemulihan_banyak_galat() {
    let hasil = pindai("@ $ #");
    assert_eq!(hasil.diagnostik.len(), 3);
    assert_eq!(hasil.token.len(), 1); // hanya AkhirBerkas
    assert_eq!(hasil.token[0].jenis, JenisToken::AkhirBerkas);
}

#[test]
fn tes_lex_posisi_baris_dan_kolom_multibaris() {
    let sumber = "a\nb\n  c";
    let hasil = pindai(sumber);
    assert_eq!(hasil.token[0].rentang.baris, 1);
    assert_eq!(hasil.token[0].rentang.kolom, 1);
    assert_eq!(hasil.token[1].rentang.baris, 2);
    assert_eq!(hasil.token[1].rentang.kolom, 1);
    assert_eq!(hasil.token[2].rentang.baris, 3);
    assert_eq!(hasil.token[2].rentang.kolom, 3);
}

#[test]
fn tes_lex_baris_baru_sebelum_flag() {
    let sumber = "a\nb c";
    let hasil = pindai(sumber);
    assert!(!hasil.token[0].baris_baru_sebelum);
    assert!(hasil.token[1].baris_baru_sebelum); // b ada baris baru sebelum
    assert!(!hasil.token[2].baris_baru_sebelum); // c tidak ada baris baru sebelum
}
