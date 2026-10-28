// Copyright (c) kangpcode (Dhafa Nazula Permadi). Licensed under MIT or Apache-2.0.

//! Test suite menyeluruh untuk Pustaka Standar (`std`) Komolang (Tahap 6).
//! Menguji parsing sintaksis, verifikasi semantik, penurunan KIR,
//! serta eksekusi fungsionalitas modul-modul pustaka standar.

use komo_codegen::EksekutorJit;
use komo_diag::render_semua;
use komo_ir::{turunkan_ke_kir, verifikasi_modul};
use komo_parser::urai;
use komo_sema::periksa;
use std::fs;
use std::path::{Path, PathBuf};

fn dapatkan_direktori_std() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("Workspace root harus ada")
        .join("std")
}

#[test]
fn tes_seluruh_modul_std_lolos_penguraian_sintaks() {
    let dir = dapatkan_direktori_std();
    let entri = fs::read_dir(&dir).expect("Direktori std/ harus dapat dibaca");

    let mut jumlah_berkas = 0;
    for e in entri {
        let e = e.unwrap();
        let jalur = e.path();
        if jalur.extension().and_then(|s| s.to_str()) == Some("komo") {
            let nama = jalur.file_name().unwrap().to_str().unwrap();
            let kode = fs::read_to_string(&jalur)
                .unwrap_or_else(|_| panic!("Gagal membaca berkas {:?}", jalur));
            let hasil = urai(&kode);
            assert!(
                hasil.berhasil(),
                "Penguraian berkas std/{:?} gagal:\n{}",
                nama,
                render_semua(&hasil.diagnostik, nama, &kode)
            );
            jumlah_berkas += 1;
        }
    }
    assert!(
        jumlah_berkas >= 20,
        "Jumlah berkas std yang diuji harus minimal 20, ditemukan: {}",
        jumlah_berkas
    );
}

#[test]
fn tes_seluruh_modul_std_lolos_pemeriksaan_semantik() {
    let dir = dapatkan_direktori_std();
    let entri = fs::read_dir(&dir).expect("Direktori std/ harus dapat dibaca");

    for e in entri {
        let e = e.unwrap();
        let jalur = e.path();
        if jalur.extension().and_then(|s| s.to_str()) == Some("komo") {
            let nama = jalur.file_name().unwrap().to_str().unwrap();
            let kode = fs::read_to_string(&jalur)
                .unwrap_or_else(|_| panic!("Gagal membaca berkas {:?}", jalur));
            let hasil_urai = urai(&kode);
            assert!(
                hasil_urai.berhasil(),
                "Sintaks berkas std/{:?} harus valid sebelum sema",
                nama
            );

            let hasil_sema = periksa(&hasil_urai.berkas);
            assert!(
                hasil_sema.berhasil(),
                "Pemeriksaan semantik std/{:?} gagal:\n{}",
                nama,
                render_semua(&hasil_sema.diagnostik, nama, &kode)
            );
        }
    }
}

#[test]
fn tes_seluruh_modul_std_lolos_lowering_dan_verifikasi_kir() {
    let dir = dapatkan_direktori_std();
    let entri = fs::read_dir(&dir).expect("Direktori std/ harus dapat dibaca");

    for e in entri {
        let e = e.unwrap();
        let jalur = e.path();
        if jalur.extension().and_then(|s| s.to_str()) == Some("komo") {
            let nama = jalur.file_name().unwrap().to_str().unwrap();
            let kode = fs::read_to_string(&jalur)
                .unwrap_or_else(|_| panic!("Gagal membaca berkas {:?}", jalur));
            let hasil_urai = urai(&kode);
            let hasil_sema = periksa(&hasil_urai.berkas);
            assert!(hasil_sema.berhasil(), "Sema std/{:?} harus berhasil", nama);

            let modul_ir = turunkan_ke_kir(&hasil_urai.berkas, &hasil_sema);
            let hasil_verif = verifikasi_modul(&modul_ir);
            assert!(
                hasil_verif.is_ok(),
                "Verifikasi IR modul std/{:?} gagal:\n{:?}",
                nama,
                hasil_verif.err().unwrap()
            );
        }
    }
}

#[test]
fn tes_integrasi_impor_dan_pemanggilan_std() {
    let program = r#"
paket utama

impor "std/matematika"
impor "std/larik"
impor "std/cetak"
impor "std/waktu"
impor "std/acak"

publik fungsi uji_std() bulat {
    cetak.baris("Memulai pengujian modul std...")

    // Uji matematika
    var m: bulat = matematika.maks(10, 25)
    var p: bulat = matematika.pangkat(2, 8)
    var f: bulat = matematika.faktorial(5)
    var prima: bool = matematika.apakah_prima(17)

    // Uji waktu
    var d: waktu.Durasi = waktu.dari_detik(5)
    var detik: bulat = waktu.ke_detik(d)

    // Uji acak
    var rng: acak.PembangkitAcak = acak.baru(12345)
    var angka: bulat = acak.rentang_bulat(rng, 1, 100)

    cetak.tulis_bulat(m + p + f + detik + angka)
    kembali m + p
}
"#;

    let hasil_urai = urai(program);
    assert!(
        hasil_urai.berhasil(),
        "Penguraian program impor std gagal:\n{}",
        render_semua(&hasil_urai.diagnostik, "tes_impor.komo", program)
    );

    let hasil_sema = periksa(&hasil_urai.berkas);
    assert!(
        hasil_sema.berhasil(),
        "Analisis semantik program impor std gagal:\n{}",
        render_semua(&hasil_sema.diagnostik, "tes_impor.komo", program)
    );

    let modul_ir = turunkan_ke_kir(&hasil_urai.berkas, &hasil_sema);
    let verif = verifikasi_modul(&modul_ir);
    assert!(
        verif.is_ok(),
        "Verifikasi IR program impor std gagal:\n{:?}",
        verif.err().unwrap()
    );
}

#[test]
fn tes_algoritma_matematika_eksekusi_native() {
    let kode_sumber = r#"
paket utama

publik fungsi hitung_faktorial(n: bulat) bulat {
    jika n <= 1 {
        kembali 1
    }
    var hasil: bulat = 1
    var i: bulat = 2
    selagi i <= n {
        hasil = hasil * i
        i = i + 1
    }
    kembali hasil
}

publik fungsi hitung_akar_bulat(x: bulat) bulat {
    jika x <= 0 {
        kembali 0
    }
    jika x == 1 {
        kembali 1
    }
    var r: bulat = x / 2
    selagi r * r > x {
        r = (r + x / r) / 2
    }
    kembali r
}

publik fungsi hitung_fpb(a: bulat, b: bulat) bulat {
    var x: bulat = a
    var y: bulat = b
    selagi y != 0 {
        var sisa: bulat = x % y
        x = y
        y = sisa
    }
    kembali x
}
"#;

    let hasil_urai = urai(kode_sumber);
    assert!(hasil_urai.berhasil());
    let hasil_sema = periksa(&hasil_urai.berkas);
    assert!(hasil_sema.berhasil());
    let modul_ir = turunkan_ke_kir(&hasil_urai.berkas, &hasil_sema);

    let eksekutor = EksekutorJit::baru(&modul_ir).expect("Inisialisasi JIT berhasil");

    // 5! = 120
    let res_faktorial = eksekutor.jalankan_fungsi_i64_ke_i64("hitung_faktorial", 5);
    assert_eq!(res_faktorial, Ok(120));

    // 7! = 5040
    let res_faktorial7 = eksekutor.jalankan_fungsi_i64_ke_i64("hitung_faktorial", 7);
    assert_eq!(res_faktorial7, Ok(5040));

    // sqrt(49) = 7
    let res_akar = eksekutor.jalankan_fungsi_i64_ke_i64("hitung_akar_bulat", 49);
    assert_eq!(res_akar, Ok(7));

    // sqrt(100) = 10
    let res_akar100 = eksekutor.jalankan_fungsi_i64_ke_i64("hitung_akar_bulat", 100);
    assert_eq!(res_akar100, Ok(10));

    // fpb(48, 18) = 6
    let res_fpb = eksekutor.jalankan_fungsi_i64_2_ke_i64("hitung_fpb", 48, 18);
    assert_eq!(res_fpb, Ok(6));
}

#[test]
fn tes_algoritma_larik_dan_pencarian_eksekusi_native() {
    let kode_sumber = r#"
paket utama

publik fungsi hitung_maksimum(a: bulat, b: bulat) bulat {
    jika a > b {
        kembali a
    }
    kembali b
}

publik fungsi hitung_minimum(a: bulat, b: bulat) bulat {
    jika a < b {
        kembali a
    }
    kembali b
}
"#;

    let hasil_urai = urai(kode_sumber);
    assert!(hasil_urai.berhasil());
    let hasil_sema = periksa(&hasil_urai.berkas);
    assert!(hasil_sema.berhasil());
    let modul_ir = turunkan_ke_kir(&hasil_urai.berkas, &hasil_sema);

    let eksekutor = EksekutorJit::baru(&modul_ir).expect("Inisialisasi JIT berhasil");

    let res_maks = eksekutor.jalankan_fungsi_i64_2_ke_i64("hitung_maksimum", 42, 17);
    assert_eq!(res_maks, Ok(42));

    let res_min = eksekutor.jalankan_fungsi_i64_2_ke_i64("hitung_minimum", 42, 17);
    assert_eq!(res_min, Ok(17));
}

#[test]
fn tes_modul_format_data_json_dan_csv_eksekusi() {
    let program = r#"
paket utama

impor "std/json"
impor "std/csv"
impor "std/uji"
impor "std/bench"

publik fungsi uji_format() bulat {
    var str_json: json.NilaiJson = json.baru_teks("Halo Komolang")
    var num_json: json.NilaiJson = json.baru_angka(42)

    var teks_json: teks = json.seri(str_json)

    var uji_hasil: uji.HasilUji = uji.tegaskan_sama_bulat(42, 42, "tes_angka")

    var b: bench.HasilBenchmark = bench.baru("operasi_cepat", 15, 1000)

    jika uji_hasil.lulus {
        kembali 1
    } lainnya {
        kembali 0
    }
}
"#;

    let hasil_urai = urai(program);
    assert!(
        hasil_urai.berhasil(),
        "Penguraian program format data gagal:\n{}",
        render_semua(&hasil_urai.diagnostik, "tes_format.komo", program)
    );

    let hasil_sema = periksa(&hasil_urai.berkas);
    assert!(
        hasil_sema.berhasil(),
        "Analisis semantik program format data gagal:\n{}",
        render_semua(&hasil_sema.diagnostik, "tes_format.komo", program)
    );

    let modul_ir = turunkan_ke_kir(&hasil_urai.berkas, &hasil_sema);
    let verif = verifikasi_modul(&modul_ir);
    assert!(
        verif.is_ok(),
        "Verifikasi IR gagal:\n{:?}",
        verif.err().unwrap()
    );
}
