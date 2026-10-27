// Copyright (c) kangpcode (Dhafa Nazula Permadi). Licensed under MIT or Apache-2.0.

//! Suite pengujian galat semantik untuk `komo_sema`.
//! Memastikan program yang salah ditolak dengan kode diagnostik yang tepat dan pesan informatif.

use komo_diag::kode;
use komo_parser::urai;
use komo_sema::periksa;

fn cek_semantik_galat(sumber: &str, kode_diharapkan: &'static str) {
    let hasil_urai = urai(sumber);
    assert!(
        hasil_urai.berhasil(),
        "Penguraian seharusnya berhasil sebelum analisis semantik:\n{:?}\nSumber:\n{}",
        hasil_urai.diagnostik,
        sumber
    );

    let hasil_sema = periksa(&hasil_urai.berkas);
    assert!(
        !hasil_sema.berhasil(),
        "Analisis semantik seharusnya gagal, tetapi berhasil! Sumber:\n{}",
        sumber
    );

    assert!(
        hasil_sema
            .diagnostik
            .iter()
            .any(|d| d.kode == kode_diharapkan),
        "Tidak menemukan kode diagnostik semantik '{}'. Ditemukan:\n{:?}",
        kode_diharapkan,
        hasil_sema.diagnostik
    );
}

#[test]
fn tes_sema_galat_tipe_anotasi_tidak_cocok() {
    let sumber = r#"
paket u

fungsi coba() {
    biar x: bulat = "teks"
}
"#;
    cek_semantik_galat(sumber, kode::TIPE_TIDAK_COCOK);
}

#[test]
fn tes_sema_galat_penugasan_tipe_salah() {
    let sumber = r#"
paket u

fungsi coba() {
    var x = 10
    x = "sepuluh"
}
"#;
    cek_semantik_galat(sumber, kode::TIPE_TIDAK_COCOK);
}

#[test]
fn tes_sema_galat_argumen_fungsi_tipe_salah() {
    let sumber = r#"
paket u

fungsi cetak_angka(n: bulat) {
}

fungsi coba() {
    cetak_angka("seratus")
}
"#;
    cek_semantik_galat(sumber, kode::TIPE_TIDAK_COCOK);
}

#[test]
fn tes_sema_galat_simbol_tidak_ditemukan() {
    let sumber = r#"
paket u

fungsi coba() {
    biar x = variabel_tidak_dikenal + 1
}
"#;
    cek_semantik_galat(sumber, kode::SIMBOL_TIDAK_DITEMUKAN);
}

#[test]
fn tes_sema_galat_simbol_duplikat_dalam_lingkup() {
    let sumber = r#"
paket u

fungsi coba() {
    biar x = 1
    biar x = 2
}
"#;
    cek_semantik_galat(sumber, kode::SIMBOL_DUPLIKAT);
}

#[test]
fn tes_sema_galat_mutasi_variabel_biar() {
    let sumber = r#"
paket u

fungsi coba() {
    biar total = 100
    total = 200
}
"#;
    cek_semantik_galat(sumber, kode::MUTASI_VARIABEL_IMUTABEL);
}

#[test]
fn tes_sema_galat_mutasi_biar_gabungan() {
    let sumber = r#"
paket u

fungsi coba() {
    biar total = 100
    total += 10
}
"#;
    cek_semantik_galat(sumber, kode::MUTASI_VARIABEL_IMUTABEL);
}

#[test]
fn tes_sema_galat_penggunaan_setelah_dipindah() {
    let sumber = r#"
paket u

tipe Kotak struktur {
    isi: bulat
}

fungsi proses(k: Kotak) {
}

fungsi coba() {
    var b = Kotak{isi: 10}
    proses(pindah b)
    biar baca = b.isi
}
"#;
    cek_semantik_galat(sumber, kode::PENGGUNAAN_SETELAH_PINDAH);
}

#[test]
fn tes_sema_galat_pinjam_var_pada_variabel_imutabel() {
    let sumber = r#"
paket u

tipe Titik struktur {
    x: bulat
}

fungsi coba() {
    biar t = Titik{x: 10}
    biar p = pinjam var t
}
"#;
    cek_semantik_galat(sumber, kode::MUTASI_VARIABEL_IMUTABEL);
}

#[test]
fn tes_sema_galat_cabang_tidak_eksaustif_hasil() {
    let sumber = r#"
paket u

fungsi coba(h: hasil<bulat, teks>) {
    cocok h {
        sukses(n): cetak.baris(n)
    }
}
"#;
    cek_semantik_galat(sumber, kode::CABANG_TIDAK_EKSAUSTIF);
}

#[test]
fn tes_sema_galat_cabang_tidak_eksaustif_mungkin() {
    let sumber = r#"
paket u

fungsi coba(m: mungkin<bulat>) {
    cocok m {
        ada(n): cetak.baris(n)
    }
}
"#;
    cek_semantik_galat(sumber, kode::CABANG_TIDAK_EKSAUSTIF);
}

#[test]
fn tes_sema_galat_cabang_tidak_eksaustif_enum() {
    let sumber = r#"
paket u

tipe Status enum {
    Baru
    Proses
    Selesai
}

fungsi coba(s: Status) {
    cocok s {
        Baru: cetak.baris("baru")
        Proses: cetak.baris("proses")
    }
}
"#;
    cek_semantik_galat(sumber, kode::CABANG_TIDAK_EKSAUSTIF);
}

#[test]
fn tes_sema_galat_bukan_fungsi_dipanggil() {
    let sumber = r#"
paket u

fungsi coba() {
    biar x = 42
    x()
}
"#;
    cek_semantik_galat(sumber, kode::BUKAN_FUNGSI);
}

#[test]
fn tes_sema_galat_jumlah_argumen_kurang() {
    let sumber = r#"
paket u

fungsi tambah(a: bulat, b: bulat) bulat {
    kembali a + b
}

fungsi coba() {
    biar h = tambah(1)
}
"#;
    cek_semantik_galat(sumber, kode::JUMLAH_ARGUMEN_SALAH);
}

#[test]
fn tes_sema_galat_jumlah_argumen_lebih() {
    let sumber = r#"
paket u

fungsi tambah(a: bulat, b: bulat) bulat {
    kembali a + b
}

fungsi coba() {
    biar h = tambah(1, 2, 3)
}
"#;
    cek_semantik_galat(sumber, kode::JUMLAH_ARGUMEN_SALAH);
}

#[test]
fn tes_sema_galat_bidang_tidak_ditemukan() {
    let sumber = r#"
paket u

tipe Orang struktur {
    nama: teks
}

fungsi coba() {
    biar p = Orang{nama: "Budi"}
    biar umur = p.alamat_palsu
}
"#;
    cek_semantik_galat(sumber, kode::BIDANG_TIDAK_DITEMUKAN);
}

#[test]
fn tes_sema_galat_bukan_struktur_diakses_bidang() {
    let sumber = r#"
paket u

fungsi coba() {
    biar angka = 123
    biar b = angka.nama
}
"#;
    cek_semantik_galat(sumber, kode::BUKAN_STRUKTUR);
}

#[test]
fn tes_sema_galat_bukan_larik_diindeks() {
    let sumber = r#"
paket u

fungsi coba() {
    biar angka = 123
    biar nilai = angka[0]
}
"#;
    cek_semantik_galat(sumber, kode::BUKAN_LARIK);
}

#[test]
fn tes_sema_galat_indeks_larik_bukan_bulat() {
    let sumber = r#"
paket u

fungsi coba() {
    biar daftar = [1, 2, 3]
    biar nilai = daftar["nol"]
}
"#;
    cek_semantik_galat(sumber, kode::INDEKS_BUKAN_BULAT);
}

#[test]
fn tes_sema_galat_kondisi_jika_bukan_bool() {
    let sumber = r#"
paket u

fungsi coba() {
    jika 42 {
        cetak.baris("salah")
    }
}
"#;
    cek_semantik_galat(sumber, kode::KONDISI_BUKAN_BOOL);
}

#[test]
fn tes_sema_galat_kondisi_selagi_bukan_bool() {
    let sumber = r#"
paket u

fungsi coba() {
    selagi "terus" {
        cetak.baris("salah")
    }
}
"#;
    cek_semantik_galat(sumber, kode::KONDISI_BUKAN_BOOL);
}

#[test]
fn tes_sema_galat_operator_aritmatika_pada_bool() {
    let sumber = r#"
paket u

fungsi coba() {
    biar x = benar * salah
}
"#;
    cek_semantik_galat(sumber, kode::OPERATOR_TIDAK_DIDUKUNG);
}

#[test]
fn tes_sema_galat_return_tipe_tidak_cocok() {
    let sumber = r#"
paket u

fungsi ambil_angka() bulat {
    kembali "bukan angka"
}
"#;
    cek_semantik_galat(sumber, kode::RETURN_TIDAK_COCOK);
}
