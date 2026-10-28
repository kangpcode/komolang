// Copyright (c) kangpcode (Dhafa Nazula Permadi). Licensed under MIT or Apache-2.0.

//! Test suite integrasi dan konformansi untuk Tahap 14: Dokumentasi, Komunitas, Koleksi Contoh, dan Rilis 1.0.

use std::fs;
use std::path::Path;

use komo_diag::render_semua;
use komo_parser::urai;
use komo_sema::periksa;

#[test]
fn tes_seluruh_30_contoh_proyek_lolos_parser_dan_sema() {
    let contoh_dir = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .join("examples")
        .join("koleksi_30_contoh");

    assert!(contoh_dir.exists(), "Direktori koleksi 30 contoh harus ada");

    let mut jumlah_berkas_teruji = 0;
    let entries = fs::read_dir(&contoh_dir).expect("Harus dapat membaca folder contoh");

    for entry in entries.flatten() {
        let p = entry.path();
        if p.is_file() && p.extension().map(|e| e == "komo").unwrap_or(false) {
            let nama = p.file_name().unwrap().to_string_lossy().to_string();
            let kode = fs::read_to_string(&p).expect("Harus dapat membaca berkas contoh");

            // 1. Uji Parser
            let hasil_urai = urai(&kode);
            assert!(
                hasil_urai.berhasil(),
                "Penguraian berkas '{}' gagal:\n{}",
                nama,
                render_semua(&hasil_urai.diagnostik, &nama, &kode)
            );

            // 2. Uji Pemeriksaan Semantik
            let hasil_sema = periksa(&hasil_urai.berkas);
            assert!(
                hasil_sema.berhasil(),
                "Analisis semantik berkas '{}' gagal:\n{}",
                nama,
                render_semua(&hasil_sema.diagnostik, &nama, &kode)
            );

            jumlah_berkas_teruji += 1;
        }
    }

    assert_eq!(
        jumlah_berkas_teruji, 30,
        "Harus ada tepat 30 berkas contoh yang teruji dan lolos"
    );
}

#[test]
fn tes_kelengkapan_buku_panduan_dan_migrasi() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).parent().unwrap();

    // 1. Cek kelengkapan buku
    let buku_dir = root.join("docs").join("buku");
    let bab_bab = vec![
        "README.md",
        "bab_01_pengenalan_dan_instalasi.md",
        "bab_02_variabel_tipe_dan_kontrol.md",
        "bab_03_fungsi_dan_kepemilikan.md",
        "bab_04_struktur_dan_metode.md",
        "bab_05_konkurensi_dan_saluran.md",
        "bab_06_pustaka_standar_lengkap.md",
        "bab_07_pembuatan_web_dan_api.md",
        "bab_08_antarmuka_dan_game.md",
    ];

    for bab in bab_bab {
        let file_path = buku_dir.join(bab);
        assert!(file_path.exists(), "Berkas buku '{:?}' harus ada", file_path);
        let konten = fs::read_to_string(&file_path).unwrap();
        assert!(!konten.trim().is_empty(), "Berkas buku tidak boleh kosong");
    }

    // 2. Cek kelengkapan panduan migrasi
    let migrasi_dir = root.join("docs").join("migrasi");
    let panduan_migrasi = vec![
        "migrasi_dari_python.md",
        "migrasi_dari_go.md",
        "migrasi_dari_c_rust.md",
    ];

    for panduan in panduan_migrasi {
        let file_path = migrasi_dir.join(panduan);
        assert!(
            file_path.exists(),
            "Panduan migrasi '{:?}' harus ada",
            file_path
        );
    }

    // 3. Cek playground web
    let pg_html = root.join("web_playground").join("index.html");
    let pg_js = root.join("web_playground").join("playground.js");
    assert!(pg_html.exists(), "web_playground/index.html harus ada");
    assert!(pg_js.exists(), "web_playground/playground.js harus ada");

    // 4. Cek dokumen rilis v1.0 dan contributing
    let rilis_doc = root.join("docs").join("rilis_v1.0.md");
    let contributing = root.join("CONTRIBUTING.md");
    let komunitas = root.join("docs").join("komunitas.md");

    assert!(rilis_doc.exists(), "docs/rilis_v1.0.md harus ada");
    assert!(contributing.exists(), "CONTRIBUTING.md harus ada");
    assert!(komunitas.exists(), "docs/komunitas.md harus ada");
}
