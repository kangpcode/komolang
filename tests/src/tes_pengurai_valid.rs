// Copyright (c) kangpcode (Dhafa Nazula Permadi). Licensed under MIT or Apache-2.0.

//! Suite pengujian sintaks valid menyeluruh untuk `komo_parser`.
//! Menguji setiap jenis deklarasi, pernyataan, ekspresi, dan kombinasinya.

use komo_ast::*;
use komo_parser::{urai, urai_ekspresi_sumber};

fn urai_valid(sumber: &str) -> BerkasProgram {
    let hasil = urai(sumber);
    assert!(
        hasil.berhasil(),
        "Penguraian seharusnya berhasil tetapi menghasilkan galat:\n{:?}\nSumber:\n{}",
        hasil.diagnostik,
        sumber
    );
    hasil.berkas
}

#[test]
fn tes_urai_paket_sederhana() {
    let berkas = urai_valid("paket coba_saja\n");
    assert_eq!(berkas.paket.nama.nama, "coba_saja");
    assert!(berkas.impor.is_empty());
    assert!(berkas.deklarasi.is_empty());
}

#[test]
fn tes_urai_banyak_impor_dan_alias() {
    let sumber = r#"
paket aplikasi

impor "std/cetak"
impor "std/jaringan/http" sebagai klien_http
impor "lokal/pustaka"
"#;
    let berkas = urai_valid(sumber);
    assert_eq!(berkas.impor.len(), 3);
    assert_eq!(berkas.impor[0].jalur, "std/cetak");
    assert_eq!(berkas.impor[0].alias, None);
    assert_eq!(berkas.impor[1].jalur, "std/jaringan/http");
    assert_eq!(
        berkas.impor[1].alias.as_ref().map(|a| a.nama.as_str()),
        Some("klien_http")
    );
    assert_eq!(berkas.impor[2].jalur, "lokal/pustaka");
}

#[test]
fn tes_urai_konstanta_dan_variabel_tingkat_atas() {
    let sumber = r#"
paket modul

konst BATAS = 100
publik konst VERSI: teks = "1.0.0"
biar nama = "Komolang"
publik var penghitung: bulat = 0
"#;
    let berkas = urai_valid(sumber);
    assert_eq!(berkas.deklarasi.len(), 4);

    match &berkas.deklarasi[0] {
        Deklarasi::Konstanta(k) => {
            assert_eq!(k.nama.nama, "BATAS");
            assert_eq!(k.visibilitas, Visibilitas::Pribadi);
            assert!(k.tipe.is_none());
        }
        _ => panic!(),
    }

    match &berkas.deklarasi[1] {
        Deklarasi::Konstanta(k) => {
            assert_eq!(k.nama.nama, "VERSI");
            assert_eq!(k.visibilitas, Visibilitas::Publik);
            assert!(k.tipe.is_some());
        }
        _ => panic!(),
    }

    match &berkas.deklarasi[2] {
        Deklarasi::Variabel(v) => {
            assert_eq!(v.nama.nama, "nama");
            assert!(!v.mutabel);
        }
        _ => panic!(),
    }

    match &berkas.deklarasi[3] {
        Deklarasi::Variabel(v) => {
            assert_eq!(v.nama.nama, "penghitung");
            assert!(v.mutabel);
            assert_eq!(v.visibilitas, Visibilitas::Publik);
        }
        _ => panic!(),
    }
}

#[test]
fn tes_urai_fungsi_tanpa_parameter_dan_kembalian() {
    let berkas = urai_valid("paket u\nfungsi aksi_kosong() {}\n");
    match &berkas.deklarasi[0] {
        Deklarasi::Fungsi(f) => {
            assert_eq!(f.nama.nama, "aksi_kosong");
            assert!(f.parameter.is_empty());
            assert!(f.tipe_kembalian.is_none());
            assert!(f.badan.pernyataan.is_empty());
        }
        _ => panic!(),
    }
}

#[test]
fn tes_urai_fungsi_banyak_parameter_dan_kembalian() {
    let sumber = r#"
paket u
fungsi tambah(a: bulat, b: bulat) bulat {
    kembali a + b
}
"#;
    let berkas = urai_valid(sumber);
    match &berkas.deklarasi[0] {
        Deklarasi::Fungsi(f) => {
            assert_eq!(f.parameter.len(), 2);
            assert_eq!(f.parameter[0].nama.nama, "a");
            assert_eq!(f.parameter[1].nama.nama, "b");
            assert!(f.tipe_kembalian.is_some());
            assert_eq!(f.badan.pernyataan.len(), 1);
        }
        _ => panic!(),
    }
}

#[test]
fn tes_urai_fungsi_generik() {
    let sumber = r#"
paket u
fungsi tukar<T>(nilai: T) T {
    kembali nilai
}
"#;
    let berkas = urai_valid(sumber);
    match &berkas.deklarasi[0] {
        Deklarasi::Fungsi(f) => {
            assert_eq!(f.generik.len(), 1);
            assert_eq!(f.generik[0].nama, "T");
        }
        _ => panic!(),
    }
}

#[test]
fn tes_urai_fungsi_banyak_generik() {
    let sumber = r#"
paket u
fungsi pasang<K, V>(kunci: K, nilai: V) {
}
"#;
    let berkas = urai_valid(sumber);
    match &berkas.deklarasi[0] {
        Deklarasi::Fungsi(f) => {
            assert_eq!(f.generik.len(), 2);
            assert_eq!(f.generik[0].nama, "K");
            assert_eq!(f.generik[1].nama, "V");
        }
        _ => panic!(),
    }
}

#[test]
fn tes_urai_metode_pada_penerima() {
    let sumber = r#"
paket u
fungsi (p: Orang) salam() teks {
    kembali "Halo"
}
"#;
    let berkas = urai_valid(sumber);
    match &berkas.deklarasi[0] {
        Deklarasi::Fungsi(f) => {
            let p = f.penerima.as_ref().unwrap();
            assert_eq!(p.nama.nama, "p");
            assert_eq!(f.nama.nama, "salam");
        }
        _ => panic!(),
    }
}

#[test]
fn tes_urai_metode_penerima_pinjam_var() {
    let sumber = r#"
paket u
fungsi (self: pinjam var Benda) ubah() {
}
"#;
    let berkas = urai_valid(sumber);
    match &berkas.deklarasi[0] {
        Deklarasi::Fungsi(f) => {
            let p = f.penerima.as_ref().unwrap();
            assert!(matches!(
                p.tipe.jenis,
                JenisTipe::Pinjam { mutabel: true, .. }
            ));
        }
        _ => panic!(),
    }
}

#[test]
fn tes_urai_tipe_struktur_lengkap() {
    let sumber = r#"
paket u
tipe Pengguna struktur {
    publik id: bulat
    nama: teks
    publik umur: bulat
}
"#;
    let berkas = urai_valid(sumber);
    match &berkas.deklarasi[0] {
        Deklarasi::Tipe(t) => {
            assert_eq!(t.nama.nama, "Pengguna");
            match &t.isi {
                IsiTipe::Struktur(bidang) => {
                    assert_eq!(bidang.len(), 3);
                    assert_eq!(bidang[0].visibilitas, Visibilitas::Publik);
                    assert_eq!(bidang[0].nama.nama, "id");
                    assert_eq!(bidang[1].visibilitas, Visibilitas::Pribadi);
                    assert_eq!(bidang[1].nama.nama, "nama");
                }
                _ => panic!(),
            }
        }
        _ => panic!(),
    }
}

#[test]
fn tes_urai_tipe_struktur_generik() {
    let sumber = r#"
paket u
tipe Kotak<T> struktur {
    isi: T
}
"#;
    let berkas = urai_valid(sumber);
    match &berkas.deklarasi[0] {
        Deklarasi::Tipe(t) => {
            assert_eq!(t.generik.len(), 1);
            assert_eq!(t.generik[0].nama, "T");
        }
        _ => panic!(),
    }
}

#[test]
fn tes_urai_tipe_antarmuka() {
    let sumber = r#"
paket u
tipe Hewan antarmuka {
    suara() teks
    makan(makanan: teks) bool
}
"#;
    let berkas = urai_valid(sumber);
    match &berkas.deklarasi[0] {
        Deklarasi::Tipe(t) => match &t.isi {
            IsiTipe::Antarmuka(metode) => {
                assert_eq!(metode.len(), 2);
                assert_eq!(metode[0].nama.nama, "suara");
                assert_eq!(metode[1].nama.nama, "makan");
                assert_eq!(metode[1].parameter.len(), 1);
            }
            _ => panic!(),
        },
        _ => panic!(),
    }
}

#[test]
fn tes_urai_tipe_enum_aljabar() {
    let sumber = r#"
paket u
tipe Bentuk enum {
    Titik
    Lingkaran(desimal)
    PersegiPanjang(desimal, desimal)
}
"#;
    let berkas = urai_valid(sumber);
    match &berkas.deklarasi[0] {
        Deklarasi::Tipe(t) => match &t.isi {
            IsiTipe::Enum(varian) => {
                assert_eq!(varian.len(), 3);
                assert_eq!(varian[0].nama.nama, "Titik");
                assert!(varian[0].tipe.is_empty());
                assert_eq!(varian[1].nama.nama, "Lingkaran");
                assert_eq!(varian[1].tipe.len(), 1);
                assert_eq!(varian[2].nama.nama, "PersegiPanjang");
                assert_eq!(varian[2].tipe.len(), 2);
            }
            _ => panic!(),
        },
        _ => panic!(),
    }
}

#[test]
fn tes_urai_tipe_alias() {
    let sumber = r#"
paket u
tipe Angka bulat
tipe DaftarTeks larik<teks>
tipe Kamus peta<teks, bulat>
tipe OpsiNama mungkin<teks>
tipe HasilBagi hasil<desimal, teks>
"#;
    let berkas = urai_valid(sumber);
    assert_eq!(berkas.deklarasi.len(), 5);
}

#[test]
fn tes_urai_pernyataan_jika_dan_lainnya() {
    let sumber = r#"
paket u
fungsi tes(x: bulat) {
    jika x > 0 {
        cetak(1)
    } lainnya jika x < 0 {
        cetak(-1)
    } lainnya {
        cetak(0)
    }
}
"#;
    let berkas = urai_valid(sumber);
    match &berkas.deklarasi[0] {
        Deklarasi::Fungsi(f) => {
            assert_eq!(f.badan.pernyataan.len(), 1);
            assert!(matches!(
                f.badan.pernyataan[0].jenis,
                JenisPernyataan::Jika { .. }
            ));
        }
        _ => panic!(),
    }
}

#[test]
fn tes_urai_pernyataan_selagi() {
    let sumber = r#"
paket u
fungsi hitung_mundur() {
    var n = 10
    selagi n > 0 {
        n -= 1
    }
}
"#;
    let berkas = urai_valid(sumber);
    match &berkas.deklarasi[0] {
        Deklarasi::Fungsi(f) => {
            assert_eq!(f.badan.pernyataan.len(), 2);
            assert!(matches!(
                f.badan.pernyataan[1].jenis,
                JenisPernyataan::Selagi { .. }
            ));
        }
        _ => panic!(),
    }
}

#[test]
fn tes_urai_pernyataan_untuk_dalam_rentang() {
    let sumber = r#"
paket u
fungsi ulang() {
    untuk i dalam 0..10 {
        cetak(i)
    }
    untuk j dalam 0..=5 {
        cetak(j)
    }
}
"#;
    let berkas = urai_valid(sumber);
    match &berkas.deklarasi[0] {
        Deklarasi::Fungsi(f) => {
            assert_eq!(f.badan.pernyataan.len(), 2);
            assert!(matches!(
                f.badan.pernyataan[0].jenis,
                JenisPernyataan::Untuk { .. }
            ));
            assert!(matches!(
                f.badan.pernyataan[1].jenis,
                JenisPernyataan::Untuk { .. }
            ));
        }
        _ => panic!(),
    }
}

#[test]
fn tes_urai_pernyataan_pilih_kasus() {
    let sumber = r#"
paket u
fungsi evaluasi(angka: bulat) {
    pilih angka {
        kasus 1: cetak("satu")
        kasus 2:
            cetak("dua")
            cetak("lanjut")
        lainnya: cetak("lain")
    }
}
"#;
    let berkas = urai_valid(sumber);
    match &berkas.deklarasi[0] {
        Deklarasi::Fungsi(f) => {
            assert_eq!(f.badan.pernyataan.len(), 1);
            match &f.badan.pernyataan[0].jenis {
                JenisPernyataan::Pilih { kasus, lainnya, .. } => {
                    assert_eq!(kasus.len(), 2);
                    assert!(lainnya.is_some());
                }
                _ => panic!(),
            }
        }
        _ => panic!(),
    }
}

#[test]
fn tes_urai_pernyataan_cocok_pola() {
    let sumber = r#"
paket u
fungsi periksa(h: hasil<bulat, teks>) {
    cocok h {
        sukses(nilai): cetak(nilai)
        gagal(pesan): cetak(pesan)
        _: cetak("wildcard")
    }
}
"#;
    let berkas = urai_valid(sumber);
    match &berkas.deklarasi[0] {
        Deklarasi::Fungsi(f) => match &f.badan.pernyataan[0].jenis {
            JenisPernyataan::Cocok { cabang, .. } => {
                assert_eq!(cabang.len(), 3);
            }
            _ => panic!(),
        },
        _ => panic!(),
    }
}

#[test]
fn tes_urai_tunda_dan_jalankan() {
    let sumber = r#"
paket u
fungsi proses() {
    tunda cetak("selesai")
    tunda {
        cetak("blok 1")
        cetak("blok 2")
    }
    jalankan tugas_latar()
}
"#;
    let berkas = urai_valid(sumber);
    match &berkas.deklarasi[0] {
        Deklarasi::Fungsi(f) => {
            assert_eq!(f.badan.pernyataan.len(), 3);
            assert!(matches!(
                f.badan.pernyataan[0].jenis,
                JenisPernyataan::Tunda(_)
            ));
            assert!(matches!(
                f.badan.pernyataan[1].jenis,
                JenisPernyataan::TundaBlok(_)
            ));
            assert!(matches!(
                f.badan.pernyataan[2].jenis,
                JenisPernyataan::Jalankan(_)
            ));
        }
        _ => panic!(),
    }
}

#[test]
fn tes_urai_blok_tidak_aman() {
    let sumber = r#"
paket u
fungsi operasi_sistem() {
    tidak_aman {
        biar p = 0
    }
}
"#;
    let berkas = urai_valid(sumber);
    match &berkas.deklarasi[0] {
        Deklarasi::Fungsi(f) => {
            assert!(matches!(
                f.badan.pernyataan[0].jenis,
                JenisPernyataan::TidakAman(_)
            ));
        }
        _ => panic!(),
    }
}

#[test]
fn tes_urai_ekspresi_biner_semua_operator() {
    let ops = vec![
        "1 + 2", "3 - 4", "5 * 6", "7 / 8", "9 % 10", "a == b", "a != b", "a < b", "a <= b",
        "a > b", "a >= b", "x dan y", "x atau y",
    ];
    for op in ops {
        let e = urai_ekspresi_sumber(op).expect(op);
        assert!(matches!(e.jenis, JenisEkspresi::Biner { .. }));
    }
}

#[test]
fn tes_urai_ekspresi_unari() {
    let e1 = urai_ekspresi_sumber("-42").unwrap();
    assert!(matches!(
        e1.jenis,
        JenisEkspresi::Unari {
            operator: OperatorUnari::Negatif,
            ..
        }
    ));

    let e2 = urai_ekspresi_sumber("!benar").unwrap();
    assert!(matches!(
        e2.jenis,
        JenisEkspresi::Unari {
            operator: OperatorUnari::Bukan,
            ..
        }
    ));
}

#[test]
fn tes_urai_literal_larik() {
    let e1 = urai_ekspresi_sumber("[]").unwrap();
    match e1.jenis {
        JenisEkspresi::Larik(item) => assert!(item.is_empty()),
        _ => panic!(),
    }

    let e2 = urai_ekspresi_sumber("[1, 2, 3, 4]").unwrap();
    match e2.jenis {
        JenisEkspresi::Larik(item) => assert_eq!(item.len(), 4),
        _ => panic!(),
    }
}

#[test]
fn tes_urai_literal_struktur_inisialisasi() {
    let e = urai_ekspresi_sumber("Titik{x: 10, y: 20}").unwrap();
    match e.jenis {
        JenisEkspresi::Struktur { nama, bidang } => {
            assert_eq!(nama.nama, "Titik");
            assert_eq!(bidang.len(), 2);
            assert_eq!(bidang[0].nama.nama, "x");
            assert_eq!(bidang[1].nama.nama, "y");
        }
        _ => panic!(),
    }
}

#[test]
fn tes_urai_indeks_dan_bidang() {
    let e1 = urai_ekspresi_sumber("daftar[0]").unwrap();
    assert!(matches!(e1.jenis, JenisEkspresi::Indeks { .. }));

    let e2 = urai_ekspresi_sumber("pengguna.profil.nama").unwrap();
    assert!(matches!(e2.jenis, JenisEkspresi::Bidang { .. }));
}

#[test]
fn tes_urai_pemanggilan_berantai() {
    let e = urai_ekspresi_sumber("objek.satu().dua(42).tiga()").unwrap();
    assert!(matches!(e.jenis, JenisEkspresi::Panggil { .. }));
}

#[test]
fn tes_urai_pemanggilan_fungsi_generik() {
    let e = urai_ekspresi_sumber("konversi<teks>(123)").unwrap();
    match e.jenis {
        JenisEkspresi::Panggil {
            argumen_tipe,
            argumen,
            ..
        } => {
            assert_eq!(argumen_tipe.len(), 1);
            assert_eq!(argumen.len(), 1);
        }
        _ => panic!(),
    }
}

#[test]
fn tes_urai_saluran_pembuatan() {
    let e = urai_ekspresi_sumber("saluran<bulat>(10)").unwrap();
    match e.jenis {
        JenisEkspresi::Panggil {
            argumen_tipe,
            argumen,
            ..
        } => {
            assert_eq!(argumen_tipe.len(), 1);
            assert_eq!(argumen.len(), 1);
        }
        _ => panic!(),
    }
}

#[test]
fn tes_urai_pernyataan_tugas_gabungan() {
    let sumber = r#"
paket u
fungsi f() {
    var x = 1
    x += 10
    x -= 2
    x *= 3
    x /= 4
    x %= 5
}
"#;
    let berkas = urai_valid(sumber);
    match &berkas.deklarasi[0] {
        Deklarasi::Fungsi(f) => {
            assert_eq!(f.badan.pernyataan.len(), 6);
        }
        _ => panic!(),
    }
}

#[test]
fn tes_urai_komentar_di_sela_kode() {
    let sumber = r#"
paket u // komentar paket

// komentar sebelum fungsi
fungsi f(
    /* param a */ a: bulat,
    /* param b */ b: bulat,
) /* kembalian */ bulat {
    // kembali hasil
    kembali a + /* tengah ekspresi */ b
}
"#;
    let berkas = urai_valid(sumber);
    assert_eq!(berkas.deklarasi.len(), 1);
}

#[test]
fn tes_urai_ekspresi_rentang_inklusif() {
    let e = urai_ekspresi_sumber("0..=100").unwrap();
    match e.jenis {
        JenisEkspresi::Rentang {
            awal,
            akhir,
            inklusif,
        } => {
            assert!(inklusif);
            assert!(matches!(awal.jenis, JenisEkspresi::Bulat(0)));
            assert!(matches!(akhir.jenis, JenisEkspresi::Bulat(100)));
        }
        _ => panic!(),
    }
}

#[test]
fn tes_urai_pemanggilan_fungsi_tanpa_argumen() {
    let e = urai_ekspresi_sumber("cetak()").unwrap();
    match e.jenis {
        JenisEkspresi::Panggil { argumen, .. } => assert!(argumen.is_empty()),
        _ => panic!(),
    }
}

#[test]
fn tes_urai_pemanggilan_fungsi_banyak_argumen() {
    let e = urai_ekspresi_sumber("f(1, 2, \"halo\", benar)").unwrap();
    match e.jenis {
        JenisEkspresi::Panggil { argumen, .. } => assert_eq!(argumen.len(), 4),
        _ => panic!(),
    }
}

#[test]
fn tes_urai_larik_bersarang() {
    let e = urai_ekspresi_sumber("[[1, 2], [3, 4]]").unwrap();
    match e.jenis {
        JenisEkspresi::Larik(baris) => {
            assert_eq!(baris.len(), 2);
            assert!(matches!(baris[0].jenis, JenisEkspresi::Larik(_)));
        }
        _ => panic!(),
    }
}

#[test]
fn tes_urai_bidang_pada_panggilan() {
    let e = urai_ekspresi_sumber("dapatkan_pengguna().nama").unwrap();
    match e.jenis {
        JenisEkspresi::Bidang { objek, nama } => {
            assert_eq!(nama.nama, "nama");
            assert!(matches!(objek.jenis, JenisEkspresi::Panggil { .. }));
        }
        _ => panic!(),
    }
}

#[test]
fn tes_urai_indeks_pada_larik_langsung() {
    let e = urai_ekspresi_sumber("[10, 20, 30][1]").unwrap();
    match e.jenis {
        JenisEkspresi::Indeks { objek, indeks } => {
            assert!(matches!(objek.jenis, JenisEkspresi::Larik(_)));
            assert!(matches!(indeks.jenis, JenisEkspresi::Bulat(1)));
        }
        _ => panic!(),
    }
}

#[test]
fn tes_urai_pola_literal_angka() {
    let sumber = r#"
paket u
fungsi f(n: bulat) {
    cocok n {
        0: cetak("nol")
        1: cetak("satu")
        _: cetak("lain")
    }
}
"#;
    let berkas = urai_valid(sumber);
    assert_eq!(berkas.deklarasi.len(), 1);
}

#[test]
fn tes_urai_pola_literal_teks() {
    let sumber = r#"
paket u
fungsi f(s: teks) {
    cocok s {
        "halo": cetak("dunia")
        "komodo": cetak("bahasa")
        _: cetak("lain")
    }
}
"#;
    let berkas = urai_valid(sumber);
    assert_eq!(berkas.deklarasi.len(), 1);
}

#[test]
fn tes_urai_pola_literal_bool() {
    let sumber = r#"
paket u
fungsi f(b: bool) bulat {
    cocok b {
        benar: kembali 1
        salah: kembali 0
    }
}
"#;
    let berkas = urai_valid(sumber);
    assert_eq!(berkas.deklarasi.len(), 1);
}

#[test]
fn tes_urai_pola_pengenal() {
    let sumber = r#"
paket u
fungsi f(x: bulat) {
    cocok x {
        nilai: cetak(nilai)
    }
}
"#;
    let berkas = urai_valid(sumber);
    assert_eq!(berkas.deklarasi.len(), 1);
}

#[test]
fn tes_urai_blok_dalam_blok() {
    let sumber = r#"
paket u
fungsi f() {
    {
        biar a = 1
        {
            biar b = 2
        }
    }
}
"#;
    let berkas = urai_valid(sumber);
    assert_eq!(berkas.deklarasi.len(), 1);
}

#[test]
fn tes_urai_pernyataan_ekspresi_murni() {
    let sumber = r#"
paket u
fungsi f() {
    1 + 2
    "teks"
    benar
}
"#;
    let berkas = urai_valid(sumber);
    match &berkas.deklarasi[0] {
        Deklarasi::Fungsi(f) => assert_eq!(f.badan.pernyataan.len(), 3),
        _ => panic!(),
    }
}

#[test]
fn tes_urai_pernyataan_kembali_tanpa_nilai() {
    let sumber = r#"
paket u
fungsi f() {
    kembali
}
"#;
    let berkas = urai_valid(sumber);
    match &berkas.deklarasi[0] {
        Deklarasi::Fungsi(f) => {
            assert!(matches!(
                f.badan.pernyataan[0].jenis,
                JenisPernyataan::Kembali(None)
            ));
        }
        _ => panic!(),
    }
}

#[test]
fn tes_urai_tipe_antarmuka_banyak_parameter() {
    let sumber = r#"
paket u
tipe Hitung antarmuka {
    tiga_angka(a: bulat, b: bulat, c: bulat) bulat
}
"#;
    let berkas = urai_valid(sumber);
    assert_eq!(berkas.deklarasi.len(), 1);
}

#[test]
fn tes_urai_interpolasi_di_dalam_fungsi() {
    let sumber = r#"
paket u
fungsi sapa(nama: teks, umur: bulat) teks {
    kembali "Halo {nama}, umurmu {umur} tahun!"
}
"#;
    let berkas = urai_valid(sumber);
    assert_eq!(berkas.deklarasi.len(), 1);
}
