// Copyright (c) kangpcode (Dhafa Nazula Permadi). Licensed under MIT or Apache-2.0.

//! Suite pengujian galat sintaks, diagnostik Indonesia, dan pemulihan galat untuk `komo_parser`.
//! Memastikan parser tidak pernah panik pada input salah, menghasilkan kode diagnostik tepat,
//! dan mampu memulihkan diri agar satu galat tidak menutupi galat lainnya.

use komo_diag::kode;
use komo_parser::urai;

fn cek_galat(sumber: &str, kode_diharapkan: &'static str) {
    let hasil = urai(sumber);
    assert!(
        !hasil.berhasil(),
        "Seharusnya gagal tetapi berhasil! Sumber:\n{}",
        sumber
    );
    assert!(
        hasil.diagnostik.iter().any(|d| d.kode == kode_diharapkan),
        "Tidak menemukan kode galat '{}'. Ditemukan:\n{:?}",
        kode_diharapkan,
        hasil.diagnostik
    );
}

#[test]
fn tes_galat_tanpa_paket() {
    cek_galat("fungsi f() {}\n", kode::PAKET_HILANG);
}

#[test]
fn tes_galat_nama_paket_bukan_pengenal() {
    cek_galat("paket 123\n", kode::DIHARAPKAN_NAMA);
}

#[test]
fn tes_galat_impor_setelah_deklarasi() {
    let sumber = r#"
paket u
konst X = 1
impor "std/cetak"
"#;
    cek_galat(sumber, kode::IMPOR_TERLAMBAT);
}

#[test]
fn tes_galat_jalur_impor_bukan_teks() {
    cek_galat("paket u\nimpor 123\n", kode::JALUR_IMPOR_TIDAK_VALID);
}

#[test]
fn tes_galat_nama_fungsi_kata_kunci() {
    let sumber = "paket u\nfungsi jika() {}\n";
    let hasil = urai(sumber);
    assert!(!hasil.berhasil());
    assert!(hasil
        .diagnostik
        .iter()
        .any(|d| d.kode == kode::DIHARAPKAN_NAMA));
    let d = hasil
        .diagnostik
        .iter()
        .find(|d| d.kode == kode::DIHARAPKAN_NAMA)
        .unwrap();
    assert!(d.saran.as_ref().unwrap().contains("kata kunci"));
}

#[test]
fn tes_galat_nama_fungsi_angka() {
    cek_galat("paket u\nfungsi 123() {}\n", kode::DIHARAPKAN_NAMA);
}

#[test]
fn tes_galat_fungsi_tanpa_kurung_tutup_parameter() {
    cek_galat("paket u\nfungsi f(a: bulat {}\n", kode::DIHARAPKAN_TOKEN);
}

#[test]
fn tes_galat_fungsi_tanpa_tipe_parameter() {
    cek_galat("paket u\nfungsi f(a: ) {}\n", kode::DIHARAPKAN_TIPE);
}

#[test]
fn tes_galat_blok_fungsi_tidak_ditutup() {
    let sumber = "paket u\nfungsi f() {\n    biar x = 1\n";
    cek_galat(sumber, kode::BLOK_TIDAK_DITUTUP);
}

#[test]
fn tes_galat_titik_koma_tidak_diperlukan() {
    let sumber = "paket u;\n";
    cek_galat(sumber, kode::TITIK_KOMA);
}

#[test]
fn tes_galat_titik_koma_pada_pernyataan() {
    let sumber = "paket u\nfungsi f() {\n    biar x = 1;\n}\n";
    cek_galat(sumber, kode::TITIK_KOMA);
}

#[test]
fn tes_galat_deklarasi_tidak_dikenal_di_tingkat_atas() {
    cek_galat("paket u\n12345\n", kode::DEKLARASI_TIDAK_DIKENAL);
}

#[test]
fn tes_galat_deklarasi_fungsi_dalam_fungsi() {
    let sumber = r#"
paket u
fungsi luar() {
    fungsi dalam() {
    }
}
"#;
    cek_galat(sumber, kode::DEKLARASI_DALAM_FUNGSI);
}

#[test]
fn tes_galat_tipe_dalam_fungsi() {
    let sumber = r#"
paket u
fungsi f() {
    tipe Titik struktur { x: bulat }
}
"#;
    cek_galat(sumber, kode::DEKLARASI_DALAM_FUNGSI);
}

#[test]
fn tes_galat_jika_tanpa_kondisi() {
    cek_galat(
        "paket u\nfungsi f() {\n    jika {\n    }\n}\n",
        kode::DIHARAPKAN_EKSPRESI,
    );
}

#[test]
fn tes_galat_selagi_tanpa_kondisi() {
    cek_galat(
        "paket u\nfungsi f() {\n    selagi {\n    }\n}\n",
        kode::DIHARAPKAN_EKSPRESI,
    );
}

#[test]
fn tes_galat_untuk_tanpa_dalam() {
    cek_galat(
        "paket u\nfungsi f() {\n    untuk i 0..10 {\n    }\n}\n",
        kode::DIHARAPKAN_TOKEN,
    );
}

#[test]
fn tes_galat_target_tugas_tidak_valid() {
    let sumber = "paket u\nfungsi f() {\n    1 + 2 = 3\n}\n";
    cek_galat(sumber, kode::TARGET_TUGAS_TIDAK_VALID);
}

#[test]
fn tes_galat_rentang_berantai() {
    let sumber = "paket u\nfungsi f() {\n    biar r = 1..2..3\n}\n";
    cek_galat(sumber, kode::RENTANG_BERANTAI);
}

#[test]
fn tes_galat_jalankan_bukan_panggilan() {
    let sumber = "paket u\nfungsi f() {\n    jalankan 42\n}\n";
    cek_galat(sumber, kode::PANGGILAN_DIPERLUKAN);
}

#[test]
fn tes_galat_tunda_bukan_panggilan() {
    let sumber = "paket u\nfungsi f() {\n    tunda 42\n}\n";
    cek_galat(sumber, kode::PANGGILAN_DIPERLUKAN);
}

#[test]
fn tes_galat_pilih_cabang_lainnya_ganda() {
    let sumber = r#"
paket u
fungsi f(x: bulat) {
    pilih x {
        kasus 1: cetak(1)
        lainnya: cetak(2)
        lainnya: cetak(3)
    }
}
"#;
    cek_galat(sumber, kode::CABANG_GANDA);
}

#[test]
fn tes_galat_cocok_pola_tidak_valid() {
    let sumber = r#"
paket u
fungsi f(x: bulat) {
    cocok x {
        +: cetak(1)
    }
}
"#;
    cek_galat(sumber, kode::POLA_TIDAK_VALID);
}

#[test]
fn tes_pemulihan_fungsi_rusak_fungsi_berikutnya_tetap_terurai() {
    let sumber = r#"
paket u
fungsi rusak( {
}
fungsi sehat() {
    biar x = 10
}
"#;
    let hasil = urai(sumber);
    assert!(!hasil.berhasil());
    let nama_fungsi: Vec<_> = hasil
        .berkas
        .deklarasi
        .iter()
        .filter_map(|d| match d {
            komo_ast::Deklarasi::Fungsi(f) => Some(f.nama.nama.clone()),
            _ => None,
        })
        .collect();
    assert!(nama_fungsi.contains(&"sehat".to_string()));
}

#[test]
fn tes_pemulihan_banyak_pernyataan_rusak_dalam_satu_fungsi() {
    let sumber = r#"
paket u
fungsi f() {
    biar = 1
    biar a = 10
    var = 2
    var b = 20
    biar c = 30
}
"#;
    let hasil = urai(sumber);
    assert_eq!(hasil.diagnostik.len(), 2);
    match &hasil.berkas.deklarasi[0] {
        komo_ast::Deklarasi::Fungsi(f) => {
            assert_eq!(f.badan.pernyataan.len(), 3);
        }
        _ => panic!(),
    }
}

#[test]
fn tes_render_diagnostik_memuat_cuplikan_dan_saran() {
    let sumber = "paket u\nfungsi coba() {\n    biar jika = 1\n}\n";
    let hasil = urai(sumber);
    assert!(!hasil.berhasil());
    let render = komo_diag::render_semua(&hasil.diagnostik, "tes.komo", sumber);
    assert!(render.contains("tes.komo:3:"));
    assert!(render.contains("biar jika = 1"));
    assert!(render.contains("^"));
    assert!(render.contains("saran:"));
}

#[test]
fn tes_galat_struktur_tanpa_kurung_kurawal() {
    cek_galat(
        "paket u\ntipe P struktur x: bulat\n",
        kode::DIHARAPKAN_TOKEN,
    );
}

#[test]
fn tes_galat_antarmuka_tanpa_kurung_kurawal() {
    cek_galat(
        "paket u\ntipe A antarmuka suara() teks\n",
        kode::DIHARAPKAN_TOKEN,
    );
}

#[test]
fn tes_galat_enum_tanpa_kurung_kurawal() {
    cek_galat("paket u\ntipe E enum Merah, Biru\n", kode::DIHARAPKAN_TOKEN);
}

#[test]
fn tes_galat_ekspresi_tak_lengkap() {
    cek_galat(
        "paket u\nfungsi f() {\n    biar x = 1 +\n}\n",
        kode::DIHARAPKAN_EKSPRESI,
    );
}

#[test]
fn tes_galat_deklarasi_var_tanpa_nama() {
    cek_galat(
        "paket u\nfungsi f() {\n    var = 10\n}\n",
        kode::DIHARAPKAN_NAMA,
    );
}

#[test]
fn tes_galat_deklarasi_biar_tanpa_nama() {
    cek_galat(
        "paket u\nfungsi f() {\n    biar = 20\n}\n",
        kode::DIHARAPKAN_NAMA,
    );
}

#[test]
fn tes_galat_deklarasi_konst_tanpa_nama() {
    cek_galat("paket u\nkonst = 30\n", kode::DIHARAPKAN_NAMA);
}

#[test]
fn tes_galat_deklarasi_konst_tanpa_nilai() {
    cek_galat("paket u\nkonst X: bulat\n", kode::DIHARAPKAN_TOKEN);
}

#[test]
fn tes_galat_larik_tidak_ditutup() {
    cek_galat(
        "paket u\nfungsi f() {\n    biar arr = [1, 2, 3\n}\n",
        kode::DIHARAPKAN_TOKEN,
    );
}

#[test]
fn tes_galat_kurung_ekspresi_tidak_ditutup() {
    cek_galat(
        "paket u\nfungsi f() {\n    biar x = (1 + 2\n}\n",
        kode::DIHARAPKAN_TOKEN,
    );
}

#[test]
fn tes_galat_indeks_tidak_ditutup() {
    cek_galat(
        "paket u\nfungsi f() {\n    biar x = arr[0\n}\n",
        kode::DIHARAPKAN_TOKEN,
    );
}

#[test]
fn tes_galat_metode_tanpa_nama_penerima() {
    cek_galat(
        "paket u\nfungsi (: Orang) sapa() {}\n",
        kode::DIHARAPKAN_NAMA,
    );
}

#[test]
fn tes_galat_metode_tanpa_tipe_penerima() {
    cek_galat("paket u\nfungsi (p: ) sapa() {}\n", kode::DIHARAPKAN_TIPE);
}

#[test]
fn tes_galat_banyak_galat_pada_program_besar() {
    let sumber = r#"
paket 123
impor 456
fungsi () {
    biar = 1
    var = 2
    1 + 2 = 3
}
"#;
    let hasil = urai(sumber);
    assert!(hasil.diagnostik.len() >= 4);
}
