// Copyright (c) kangpcode (Dhafa Nazula Permadi). Licensed under MIT or Apache-2.0.

//! Suite pengujian semantik valid untuk `komo_sema`.
//! Memastikan program-program Komolang yang benar lolos seluruh analisis semantik tanpa galat.

use komo_parser::urai;
use komo_sema::periksa;

fn cek_semantik_valid(sumber: &str) {
    let hasil_urai = urai(sumber);
    assert!(
        hasil_urai.berhasil(),
        "Penguraian gagal:\n{:?}\nSumber:\n{}",
        hasil_urai.diagnostik,
        sumber
    );

    let hasil_sema = periksa(&hasil_urai.berkas);
    assert!(
        hasil_sema.berhasil(),
        "Analisis semantik gagal:\n{:?}\nSumber:\n{}",
        hasil_sema.diagnostik,
        sumber
    );
}

#[test]
fn tes_sema_variabel_dan_inferensi_tipe() {
    let sumber = r#"
paket u

fungsi coba() {
    biar angka = 42
    biar pecahan = 3.14
    biar kata = "halo"
    biar benar_kah = benar
    var counter = 0
    counter += 1
}
"#;
    cek_semantik_valid(sumber);
}

#[test]
fn tes_sema_aritmatika_dan_perbandingan() {
    let sumber = r#"
paket u

fungsi hitung(a: bulat, b: bulat) bulat {
    biar tambah = a + b
    biar kurang = a - b
    biar kali = a * b
    biar bagi = a / b
    biar sisa = a % b
    biar sama = (a == b)
    biar beda = (a != b)
    biar kecil = (a < b)
    biar besar = (a > b)
    kembali tambah
}
"#;
    cek_semantik_valid(sumber);
}

#[test]
fn tes_sema_koersi_numerik_bulat_ke_desimal() {
    let sumber = r#"
paket u

fungsi luas(radius: desimal) desimal {
    biar pi = 3.14159
    kembali pi * radius * radius
}

fungsi coba() {
    biar r = 10 // bulat
    biar l = luas(r) // terkoersi ke desimal
    biar banding = (r == 10.0)
}
"#;
    cek_semantik_valid(sumber);
}

#[test]
fn tes_sema_struktur_dan_akses_bidang() {
    let sumber = r#"
paket u

tipe Titik struktur {
    x: bulat
    y: bulat
}

fungsi coba() {
    biar t = Titik{x: 10, y: 20}
    biar total = t.x + t.y
}
"#;
    cek_semantik_valid(sumber);
}

#[test]
fn tes_sema_metode_dengan_penerima() {
    let sumber = r#"
paket u

tipe Pengguna struktur {
    nama: teks
    umur: bulat
}

fungsi (p: Pengguna) sapa() teks {
    kembali "Halo, " + p.nama
}

fungsi coba() {
    biar u = Pengguna{nama: "Budi", umur: 25}
    biar salam = u.sapa()
}
"#;
    cek_semantik_valid(sumber);
}

#[test]
fn tes_sema_metode_penerima_pinjam_var() {
    let sumber = r#"
paket u

tipe Penghitung struktur {
    nilai: bulat
}

fungsi (p: pinjam var Penghitung) tambah() {
    p.nilai += 1
}

fungsi coba() {
    var c = Penghitung{nilai: 0}
    c.tambah()
}
"#;
    cek_semantik_valid(sumber);
}

#[test]
fn tes_sema_pencocokan_cocok_eksaustif_hasil() {
    let sumber = r#"
paket u

fungsi coba(h: hasil<bulat, teks>) {
    cocok h {
        sukses(n): cetak.baris(n)
        gagal(e): cetak.baris(e)
    }
}
"#;
    cek_semantik_valid(sumber);
}

#[test]
fn tes_sema_pencocokan_cocok_eksaustif_mungkin() {
    let sumber = r#"
paket u

fungsi coba(m: mungkin<bulat>) {
    cocok m {
        ada(n): cetak.baris(n)
        kosong: cetak.baris("kosong")
    }
}
"#;
    cek_semantik_valid(sumber);
}

#[test]
fn tes_sema_pencocokan_cocok_dengan_wildcard() {
    let sumber = r#"
paket u

fungsi coba(angka: bulat) {
    cocok angka {
        0: cetak.baris("nol")
        _: cetak.baris("bukan nol")
    }
}
"#;
    cek_semantik_valid(sumber);
}

#[test]
fn tes_sema_pencocokan_cocok_enum_eksaustif() {
    let sumber = r#"
paket u

tipe Warna enum {
    Merah
    Hijau
    Biru
}

fungsi label(w: Warna) teks {
    cocok w {
        Merah: kembali "merah"
        Hijau: kembali "hijau"
        Biru: kembali "biru"
    }
}
"#;
    cek_semantik_valid(sumber);
}

#[test]
fn tes_sema_larik_dan_pengindeksan() {
    let sumber = r#"
paket u

fungsi coba() {
    biar daftar = [10, 20, 30]
    biar pertama = daftar[0]
    biar kedua = daftar[1]
}
"#;
    cek_semantik_valid(sumber);
}

#[test]
fn tes_sema_pengulangan_untuk_dan_selagi() {
    let sumber = r#"
paket u

fungsi coba() {
    var total = 0
    untuk i dalam 0..10 {
        total += i
    }

    selagi total > 0 {
        total -= 1
    }
}
"#;
    cek_semantik_valid(sumber);
}

#[test]
fn tes_sema_pilih_kasus_dan_lainnya() {
    let sumber = r#"
paket u

fungsi coba(x: bulat) {
    pilih x {
        kasus 1: cetak.baris("satu")
        kasus 2: cetak.baris("dua")
        lainnya: cetak.baris("lain")
    }
}
"#;
    cek_semantik_valid(sumber);
}

#[test]
fn tes_sema_saluran_dan_konkurensi() {
    let sumber = r#"
paket u

fungsi kirim_angka(s: saluran<bulat>) {
    s.kirim(42)
}

fungsi coba() {
    biar s = saluran<bulat>(1)
    jalankan kirim_angka(s)
    biar pesan = s.terima()
    tunda s.tutup()
}
"#;
    cek_semantik_valid(sumber);
}

#[test]
fn tes_sema_ownership_peminjaman_dan_pindah() {
    let sumber = r#"
paket u

tipe Data struktur {
    id: bulat
}

fungsi proses(d: Data) {
}

fungsi coba() {
    var d = Data{id: 1}
    biar ref1 = pinjam d
    biar ref2 = pinjam d
    var d2 = Data{id: 2}
    proses(pindah d2)
}
"#;
    cek_semantik_valid(sumber);
}

#[test]
fn tes_sema_seluruh_contoh_sintaks_lengkap() {
    let sumber = include_str!("../../examples/sintaks_lengkap.komo");
    cek_semantik_valid(sumber);
}
