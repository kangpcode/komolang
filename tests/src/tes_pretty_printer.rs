// Copyright (c) kangpcode (Dhafa Nazula Permadi). Licensed under MIT or Apache-2.0.

//! Suite pengujian pretty-printer AST (`komo_ast::cetak`) dan idempotensi penguraian ulang.
//! Setiap program valid yang dicetak harus dapat diurai kembali menghasilkan AST yang setara.

use komo_ast::cetak::{cetak_berkas, cetak_ekspresi};
use komo_parser::{urai, urai_ekspresi_sumber};

fn cek_idempoten(sumber: &str) {
    let hasil1 = urai(sumber);
    assert!(
        hasil1.berhasil(),
        "Penguraian awal gagal:\n{:?}\nSumber:\n{}",
        hasil1.diagnostik,
        sumber
    );

    let cetak1 = cetak_berkas(&hasil1.berkas);
    let hasil2 = urai(&cetak1);
    assert!(
        hasil2.berhasil(),
        "Penguraian kode tercetak gagal:\n{:?}\nKode Tercetak:\n{}",
        hasil2.diagnostik,
        cetak1
    );

    let cetak2 = cetak_berkas(&hasil2.berkas);
    assert_eq!(
        cetak1, cetak2,
        "Pencetakan tidak idempoten!\nCetak 1:\n{}\nCetak 2:\n{}",
        cetak1, cetak2
    );
}

#[test]
fn tes_cetak_idempoten_paket_dan_impor() {
    let sumber = r#"paket demo

impor "std/cetak"
impor "std/web" sebagai web
"#;
    cek_idempoten(sumber);
}

#[test]
fn tes_cetak_idempoten_konstanta_dan_variabel() {
    let sumber = r#"paket demo

konst PI = 3.14159
publik konst MAKS: bulat = 1000
biar nama = "Komodo"
publik var hitung: bulat = 0
"#;
    cek_idempoten(sumber);
}

#[test]
fn tes_cetak_idempoten_struktur_dan_metode() {
    let sumber = r#"paket demo

tipe Titik struktur {
    publik x: bulat
    publik y: bulat
}

fungsi (t: Titik) jarak() bulat {
    kembali t.x + t.y
}
"#;
    cek_idempoten(sumber);
}

#[test]
fn tes_cetak_idempoten_antarmuka_dan_enum() {
    let sumber = r#"paket demo

tipe Hewan antarmuka {
    suara() teks
    makan(porsi: bulat) bool
}

tipe Opsi enum {
    Ada(teks)
    Kosong
}
"#;
    cek_idempoten(sumber);
}

#[test]
fn tes_cetak_idempoten_fungsi_generik() {
    let sumber = r#"paket demo

fungsi tukar<T>(nilai: T) T {
    kembali nilai
}
"#;
    cek_idempoten(sumber);
}

#[test]
fn tes_cetak_idempoten_kondisi_jika_lainnya() {
    let sumber = r#"paket demo

fungsi periksa(x: bulat) teks {
    jika x > 0 {
        kembali "positif"
    } lainnya jika x < 0 {
        kembali "negatif"
    } lainnya {
        kembali "nol"
    }
}
"#;
    cek_idempoten(sumber);
}

#[test]
fn tes_cetak_idempoten_pengulangan_selagi_dan_untuk() {
    let sumber = r#"paket demo

fungsi hitung() {
    var i = 0
    selagi i < 10 {
        i += 1
    }
    untuk n dalam 0..5 {
        cetak.baris(n)
    }
}
"#;
    cek_idempoten(sumber);
}

#[test]
fn tes_cetak_idempoten_pilih_kasus() {
    let sumber = r#"paket demo

fungsi periksa(n: bulat) {
    pilih n {
        kasus 1:
            cetak.baris("satu")
        kasus 2:
            cetak.baris("dua")
        lainnya:
            cetak.baris("lain")
    }
}
"#;
    cek_idempoten(sumber);
}

#[test]
fn tes_cetak_idempoten_cocok_pola() {
    let sumber = r#"paket demo

fungsi tangani(h: hasil<bulat, teks>) {
    cocok h {
        sukses(n): cetak.baris(n)
        gagal(e): cetak.baris(e)
        _: cetak.baris("tak dikenal")
    }
}
"#;
    cek_idempoten(sumber);
}

#[test]
fn tes_cetak_idempoten_konkurensi_dan_tunda() {
    let sumber = r#"paket demo

fungsi tugas(s: saluran<bulat>) {
    jalankan s.kirim(42)
    tunda s.tutup()
    tunda {
        cetak.baris("selesai")
    }
}
"#;
    cek_idempoten(sumber);
}

#[test]
fn tes_cetak_idempoten_blok_tidak_aman() {
    let sumber = r#"paket demo

fungsi operasi() {
    tidak_aman {
        biar x = 0
    }
}
"#;
    cek_idempoten(sumber);
}

#[test]
fn tes_cetak_idempoten_template_lengkap() {
    let berkas_contoh = include_str!("../../examples/sintaks_lengkap.komo");
    cek_idempoten(berkas_contoh);
}

#[test]
fn tes_cetak_ekspresi_presedensi_aritmatika() {
    let e = urai_ekspresi_sumber("1 + 2 * 3").unwrap();
    assert_eq!(cetak_ekspresi(&e), "1 + 2 * 3");

    let e2 = urai_ekspresi_sumber("(1 + 2) * 3").unwrap();
    assert_eq!(cetak_ekspresi(&e2), "(1 + 2) * 3");
}

#[test]
fn tes_cetak_ekspresi_presedensi_logika() {
    let e = urai_ekspresi_sumber("a atau b dan c").unwrap();
    assert_eq!(cetak_ekspresi(&e), "a atau b dan c");

    let e2 = urai_ekspresi_sumber("(a atau b) dan c").unwrap();
    assert_eq!(cetak_ekspresi(&e2), "(a atau b) dan c");
}

#[test]
fn tes_cetak_ekspresi_interpolasi_teks() {
    let e = urai_ekspresi_sumber(r#""angka: {x + 1}""#).unwrap();
    assert_eq!(cetak_ekspresi(&e), "\"angka: {x + 1}\"");
}

#[test]
fn tes_cetak_ekspresi_struktur_dalam_kondisi_terbungkus() {
    let sumber = r#"paket demo

fungsi cek() {
    jika (Titik{x: 1, y: 2}.x > 0) {
        cetak.baris("positif")
    }
}
"#;
    cek_idempoten(sumber);
}
