// Copyright (c) kangpcode (Dhafa Nazula Permadi). Licensed under MIT or Apache-2.0.

//! Pengujian integrasi untuk penurunan KIR valid dan verifikasi keabsahan IR SSA.

use komo_ir::{cetak_modul, turunkan_ke_kir, verifikasi_modul};
use komo_parser::urai;
use komo_sema::periksa;

fn turunkan_dan_verifikasi(kode: &str) -> (komo_ir::ModulIr, String) {
    let hasil_urai = urai(kode);
    assert!(
        hasil_urai.berhasil(),
        "Gagal parsing kode:\n{:?}",
        hasil_urai.diagnostik
    );

    let hasil_sema = periksa(&hasil_urai.berkas);
    assert!(
        hasil_sema.berhasil(),
        "Gagal semantik:\n{:?}",
        hasil_sema.diagnostik
    );

    let modul_ir = turunkan_ke_kir(&hasil_urai.berkas, &hasil_sema);
    let hasil_verif = verifikasi_modul(&modul_ir);
    assert!(
        hasil_verif.is_ok(),
        "IR gagal diverifikasi:\n{:?}",
        hasil_verif.err().unwrap()
    );

    let teks_ir = cetak_modul(&modul_ir);
    (modul_ir, teks_ir)
}

#[test]
fn tes_ir_fungsi_sederhana_dan_aritmatika() {
    let kode = r#"
        paket matematika

        fungsi hitung(a: bulat, b: bulat) bulat {
            biar hasil = a * 2 + b
            kembali hasil
        }
    "#;

    let (_, dump) = turunkan_dan_verifikasi(kode);
    assert!(dump.contains("fungsi @hitung") && dump.contains("-> i64"));
    assert!(dump.contains("biner.kali"));
    assert!(dump.contains("biner.tambah"));
    assert!(dump.contains("kembali"));
}

#[test]
fn tes_ir_percabangan_jika_lainnya() {
    let kode = r#"
        paket logika

        fungsi cek_nilai(x: bulat) bulat {
            jika x > 0 {
                kembali 1
            } lainnya {
                kembali 0
            }
        }
    "#;

    let (_, dump) = turunkan_dan_verifikasi(kode);
    assert!(dump.contains("cabang"));
    assert!(dump.contains("jika_maka"));
    assert!(dump.contains("jika_salah"));
}

#[test]
fn tes_ir_pengulangan_selagi() {
    let kode = r#"
        paket loop_test

        fungsi hitung_mundur(n: bulat) {
            var x = n
            selagi x > 0 {
                x = x - 1
            }
        }
    "#;

    let (_, dump) = turunkan_dan_verifikasi(kode);
    assert!(dump.contains("selagi_kondisi"));
    assert!(dump.contains("selagi_badan"));
    assert!(dump.contains("selagi_keluar"));
}

#[test]
fn tes_ir_pengulangan_untuk_rentang() {
    let kode = r#"
        paket loop_range

        fungsi jumlahkan(batas: bulat) bulat {
            var total = 0
            untuk i dalam 0..batas {
                total += i
            }
            kembali total
        }
    "#;

    let (_, dump) = turunkan_dan_verifikasi(kode);
    assert!(dump.contains("untuk_kondisi"));
    assert!(dump.contains("untuk_badan"));
    assert!(dump.contains("untuk_keluar"));
    assert!(dump.contains("biner.kurang_dari"));
}

#[test]
fn tes_ir_struktur_dan_akses_bidang() {
    let kode = r#"
        paket entitas

        tipe Titik struktur {
            x: bulat,
            y: bulat
        }

        fungsi geser(t: Titik) bulat {
            kembali t.x + t.y
        }
    "#;

    let (_, dump) = turunkan_dan_verifikasi(kode);
    assert!(dump.contains("akses_bidang"));
    assert!(dump.contains("fungsi @geser"));
}

#[test]
fn tes_ir_pencocokan_cocok() {
    let kode = r#"
        paket pola_test

        fungsi nilai_pola(x: bulat) bulat {
            cocok x {
                1: kembali 10
                2: kembali 20
                _: kembali 0
            }
        }
    "#;

    let (_, dump) = turunkan_dan_verifikasi(kode);
    assert!(dump.contains("cocok_badan_0"));
    assert!(dump.contains("cocok_badan_1"));
    assert!(dump.contains("cocok_gabung"));
}

#[test]
fn tes_ir_seluruh_contoh_resmi() {
    let kode_contoh = vec![
        include_str!("../../examples/halo_dunia.komo"),
        include_str!("../../examples/perhitungan.komo"),
        include_str!("../../examples/sintaks_lengkap.komo"),
    ];

    for kode in kode_contoh {
        let (modul, dump) = turunkan_dan_verifikasi(kode);
        assert!(!modul.nama.is_empty());
        assert!(!dump.is_empty());
    }
}
