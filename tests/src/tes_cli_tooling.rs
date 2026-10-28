// Copyright (c) kangpcode (Dhafa Nazula Permadi). Licensed under MIT or Apache-2.0.

use std::collections::HashMap;
use std::env;
use std::fs;
use std::path::PathBuf;

use komo::lsp::proses_permintaan_lsp;
use komo::perintah::{
    bersihkan_target, buat_dokumentasi, buat_proyek_baru, debug_berkas, format_kode,
    jalankan_benchmark, jalankan_pengujian, tambah_dependensi,
};

/// Direktori kerja sementara untuk pengujian tooling CLI
fn siapkan_direktori_uji(nama_sub: &str) -> PathBuf {
    let base = env::temp_dir().join(format!("komo_uji_tooling_{}", nama_sub));
    let _ = fs::remove_dir_all(&base);
    fs::create_dir_all(&base).expect("Gagal membuat direktori uji");
    base
}

#[test]
fn tes_cli_scaffold_baru() {
    let dir = siapkan_direktori_uji("baru");
    let target = dir.join("proyek_contoh");

    let target_str = target.to_str().unwrap();
    let res = buat_proyek_baru(target_str);
    assert!(res.is_ok(), "Pembangunan proyek baru harus sukses: {:?}", res);

    assert!(target.join("komo.toml").exists(), "komo.toml harus dibuat");
    assert!(target.join("src/utama.komo").exists(), "src/utama.komo harus dibuat");
    assert!(target.join("tests/tes_utama.komo").exists(), "tests/tes_utama.komo harus dibuat");
    assert!(target.join(".gitignore").exists(), ".gitignore harus dibuat");

    let toml_konten = fs::read_to_string(target.join("komo.toml")).unwrap();
    assert!(toml_konten.contains("proyek_contoh"));
    assert!(toml_konten.contains("0.1.0"));

    let utama_konten = fs::read_to_string(target.join("src/utama.komo")).unwrap();
    assert!(utama_konten.contains("paket utama"));
    assert!(utama_konten.contains("fungsi utama"));

    let _ = fs::remove_dir_all(&dir);
}

#[test]
fn tes_cli_formatter() {
    let dir = siapkan_direktori_uji("format");
    let berkas_komo = dir.join("tes_format.komo");

    let kode_awal = "paket   utama\n\nfungsi tambah(  a :  bulat, b : bulat ) bulat {\nkembali   a + b\n}\n";
    fs::write(&berkas_komo, kode_awal).unwrap();

    let berkas_str = berkas_komo.to_str().unwrap();

    // Mode cek: harus mengindikasikan bahwa berkas belum terformat (mengembalikan Err)
    let hasil_cek_awal = format_kode(berkas_str, true);
    assert!(hasil_cek_awal.is_err(), "Harus mendeteksi bahwa berkas belum terformat");

    // Mode format aktual
    let hasil_format = format_kode(berkas_str, false);
    assert!(hasil_format.is_ok(), "Format aktual harus sukses: {:?}", hasil_format);

    let kode_terformat = fs::read_to_string(&berkas_komo).unwrap();
    assert!(kode_terformat.contains("paket utama"));
    assert!(kode_terformat.contains("fungsi tambah(a: bulat, b: bulat) bulat"));
    assert!(kode_terformat.contains("kembali a + b"));

    // Mode cek kedua: sekarang sudah terformat, harus mengembalikan Ok
    let hasil_cek_kedua = format_kode(berkas_str, true);
    assert!(hasil_cek_kedua.is_ok(), "Setelah diformat, mode cek harus lulus");

    let _ = fs::remove_dir_all(&dir);
}

#[test]
fn tes_cli_generator_dokumentasi() {
    let dir = siapkan_direktori_uji("dok");
    let berkas_komo = dir.join("modul_layanan.komo");
    let output_dok = dir.join("keluaran_html");

    let kode = r#"paket layanan

/// Struktur untuk mengelola data akun pengguna.
publik tipe Pengguna = struktur {
    id: bulat,
    nama: teks
}

/// Menghitung kuadrat dari suatu bilangan bulat.
publik fungsi hitung_kuadrat(n: bulat) bulat {
    kembali n * n
}
"#;
    fs::write(&berkas_komo, kode).unwrap();

    let res = buat_dokumentasi(berkas_komo.to_str().unwrap(), output_dok.to_str().unwrap());
    assert!(res.is_ok(), "Pembuatan dokumentasi harus berhasil: {:?}", res);

    let berkas_md = output_dok.join("DOKUMENTASI.md");
    let berkas_html = output_dok.join("index.html");

    assert!(berkas_md.exists(), "Berkas DOKUMENTASI.md harus ada");
    assert!(berkas_html.exists(), "Berkas index.html harus ada");

    let konten_md = fs::read_to_string(berkas_md).unwrap();
    assert!(konten_md.contains("Dokumentasi Resmi Komolang"));
    assert!(konten_md.contains("hitung_kuadrat"));
    assert!(konten_md.contains("Pengguna"));

    let konten_html = fs::read_to_string(berkas_html).unwrap();
    assert!(konten_html.contains("<!DOCTYPE html>"));
    assert!(konten_html.contains("Dokumentasi Komolang"));
    assert!(konten_html.contains("hitung_kuadrat"));

    let _ = fs::remove_dir_all(&dir);
}

#[test]
fn tes_cli_runner_pengujian() {
    let dir = siapkan_direktori_uji("uji");
    let berkas_tes = dir.join("tes_logika.komo");

    let kode_tes = r#"paket uji_logika

fungsi tes_penjumlahan_dasar() bool {
    biar a = 15
    biar b = 27
    kembali (a + b) == 42
}

fungsi tes_kondisi_logika() bool {
    biar p = benar
    biar q = salah
    kembali p dan (!q)
}
"#;
    fs::write(&berkas_tes, kode_tes).unwrap();

    let res = jalankan_pengujian(Some(berkas_tes.to_str().unwrap()));
    assert!(res.is_ok(), "Suite pengujian harus sukses: {:?}", res);

    let _ = fs::remove_dir_all(&dir);
}

#[test]
fn tes_cli_runner_benchmark() {
    let dir = siapkan_direktori_uji("bench");
    let berkas_bench = dir.join("tolok_ukur.komo");

    let kode_bench = r#"paket tolok_ukur

fungsi bench_perhitungan_deret() bulat {
    var akumulator = 0
    untuk i dalam 0..100 {
        akumulator += i
    }
    kembali akumulator
}
"#;
    fs::write(&berkas_bench, kode_bench).unwrap();

    let res = jalankan_benchmark(berkas_bench.to_str().unwrap(), 100);
    assert!(res.is_ok(), "Benchmark harus berhasil dieksekusi: {:?}", res);

    let _ = fs::remove_dir_all(&dir);
}

#[test]
fn tes_cli_manajemen_paket() {
    let dir = siapkan_direktori_uji("paket");
    let toml_jalur = dir.join("komo.toml");

    let toml_awal = r#"[paket]
nama = "aplikasi_saya"
versi = "0.1.0"

[dependensi]
"#;
    fs::write(&toml_jalur, toml_awal).unwrap();

    // Ubah direktori kerja sesaat untuk menguji penambahan dependensi
    let cwd_asli = env::current_dir().unwrap();
    env::set_current_dir(&dir).unwrap();

    let res_tambah = tambah_dependensi("std/jaringan");
    assert!(res_tambah.is_ok(), "tambah_dependensi harus berhasil: {:?}", res_tambah);

    let toml_terbaru = fs::read_to_string(&toml_jalur).unwrap();
    assert!(toml_terbaru.contains("std/jaringan"));

    // Uji pembersihan target
    let target_dir = dir.join("target");
    fs::create_dir_all(&target_dir).unwrap();
    fs::write(target_dir.join("biner_lama"), "123").unwrap();

    let res_bersih = bersihkan_target();
    assert!(res_bersih.is_ok(), "bersihkan_target harus sukses: {:?}", res_bersih);
    assert!(!target_dir.exists(), "Direktori target harus terhapus");

    env::set_current_dir(cwd_asli).unwrap();
    let _ = fs::remove_dir_all(&dir);
}

#[test]
fn tes_cli_debug_inspeksi() {
    let dir = siapkan_direktori_uji("debug");
    let berkas = dir.join("kode_debug.komo");

    let kode = r#"paket debug_sample

fungsi kuadrat(x: bulat) bulat {
    kembali x * x
}
"#;
    fs::write(&berkas, kode).unwrap();

    let res = debug_berkas(berkas.to_str().unwrap());
    assert!(res.is_ok(), "debug_berkas harus berhasil: {:?}", res);

    let _ = fs::remove_dir_all(&dir);
}

#[test]
fn tes_lsp_siklus_hidup_dan_kapabilitas() {
    let mut dokumen = HashMap::new();

    // 1. Initialize
    let init_pesan = r#"{"jsonrpc": "2.0", "id": 1, "method": "initialize", "params": {}}"#;
    let (resp, notif) = proses_permintaan_lsp(init_pesan, &mut dokumen);
    assert!(resp.is_some());
    let resp_str = resp.unwrap();
    assert!(resp_str.contains("\"name\": \"komo-lsp\""));
    assert!(resp_str.contains("\"completionProvider\""));
    assert!(resp_str.contains("\"hoverProvider\": true"));
    assert!(notif.is_empty());

    // 2. Shutdown
    let shutdown_pesan = r#"{"jsonrpc": "2.0", "id": 2, "method": "shutdown", "params": {}}"#;
    let (resp_shut, _) = proses_permintaan_lsp(shutdown_pesan, &mut dokumen);
    assert!(resp_shut.is_some());
    assert!(resp_shut.unwrap().contains("\"result\": null"));
}

#[test]
fn tes_lsp_pelengkapan_kode_dan_hover() {
    let mut dokumen = HashMap::new();

    // Completion request
    let comp_pesan = r#"{"jsonrpc": "2.0", "id": 10, "method": "textDocument/completion", "params": {}}"#;
    let (resp_comp, _) = proses_permintaan_lsp(comp_pesan, &mut dokumen);
    assert!(resp_comp.is_some());
    let comp_str = resp_comp.unwrap();
    assert!(comp_str.contains("\"label\": \"fungsi\""));
    assert!(comp_str.contains("\"label\": \"struktur\""));
    assert!(comp_str.contains("\"label\": \"bulat\""));
    assert!(comp_str.contains("\"label\": \"larik\""));

    // Hover request
    let hover_pesan = r#"{"jsonrpc": "2.0", "id": 11, "method": "textDocument/hover", "params": {}}"#;
    let (resp_hover, _) = proses_permintaan_lsp(hover_pesan, &mut dokumen);
    assert!(resp_hover.is_some());
    let hover_str = resp_hover.unwrap();
    assert!(hover_str.contains("Komolang Official Language Server"));
}

#[test]
fn tes_lsp_diagnostik_langsung_dan_formatting() {
    let mut dokumen = HashMap::new();

    // 1. didOpen dengan kode ber-galat sintaks
    let open_rusak = r#"{"jsonrpc": "2.0", "method": "textDocument/didOpen", "params": {"textDocument": {"uri": "file:///workspace/aplikasi.komo", "text": "fungsi salah_sintaks( { kembali 1 }"}}}"#;
    let (_, notif_rusak) = proses_permintaan_lsp(open_rusak, &mut dokumen);
    assert_eq!(notif_rusak.len(), 1);
    let diag_rusak = &notif_rusak[0];
    assert!(diag_rusak.contains("textDocument/publishDiagnostics"));
    assert!(diag_rusak.contains("\"severity\": 1"));

    // 2. didChange dengan kode valid
    let change_valid = r#"{"jsonrpc": "2.0", "method": "textDocument/didChange", "params": {"textDocument": {"uri": "file:///workspace/aplikasi.komo", "text": "paket utama\n\nfungsi tes() bulat {\n    kembali 42\n}\n"}}}"#;
    let (_, notif_valid) = proses_permintaan_lsp(change_valid, &mut dokumen);
    assert_eq!(notif_valid.len(), 1);
    let diag_valid = &notif_valid[0];
    assert!(diag_valid.contains("\"diagnostics\": []"));

    // 3. formatting dokumen yang sudah diubah
    let format_req = r#"{"jsonrpc": "2.0", "id": 20, "method": "textDocument/formatting", "params": {"textDocument": {"uri": "file:///workspace/aplikasi.komo"}}}"#;
    let (resp_format, _) = proses_permintaan_lsp(format_req, &mut dokumen);
    assert!(resp_format.is_some());
    let format_str = resp_format.unwrap();
    assert!(format_str.contains("newText"));
    assert!(format_str.contains("paket utama"));
}
