// Copyright (c) kangpcode (Dhafa Nazula Permadi). Licensed under MIT or Apache-2.0.

//! Pengujian integrasi untuk Tahap 10: KomoUI & Framework Interoperabilitas Vue/React/JS.

use komo_ui::komponen::{kotak_centang, masukan, teks, tombol, wadah_kolom};
use komo_ui::interop::{buat_binding_vue, buat_definisi_typescript, DeskripsiFungsiWasm, NilaiJs};
use komo_ui::reaktif::{bedakan_pohon, OperasiDiff, Sinyal};
use komo_ui::renderer::{FramebufferRgba, SoftwareRenderer};
use komo_ui::tema::{TemaUi, WarnaRgba};
use std::collections::HashMap;
use std::path::Path;

#[test]
fn tes_pohon_komponen_komoui() {
    let t = tombol("Kirim Data", "btn-kirim");
    let m = masukan("Masukkan nama Anda...", "input-nama");
    let j = teks("Dashboard KomoUI");
    let c = kotak_centang(true, "Setujui syarat", "chk-syarat");

    let wadah = wadah_kolom(vec![j, m, c, t]);

    assert_eq!(wadah.jumlah_simpul(), 5); // wadah + 4 anak
}

#[test]
fn tes_sistem_tema_dan_palet() {
    let tema_gelap = TemaUi::gelap();
    let tema_terang = TemaUi::terang();

    assert_eq!(tema_gelap.latar_belakang, WarnaRgba::rgb(15, 23, 42));
    assert_eq!(tema_terang.latar_belakang, WarnaRgba::rgb(248, 250, 252));

    // Uji parsing hex
    let warna_aksen = WarnaRgba::dari_hex("#4f46e5").expect("Harus dapat mem-parse hex");
    assert_eq!(warna_aksen.r, 79);
    assert_eq!(warna_aksen.g, 70);
    assert_eq!(warna_aksen.b, 229);
    assert_eq!(warna_aksen.a, 255);

    // Alpha blending factor
    let transparan = warna_aksen.dengan_alpha(128);
    assert_eq!(transparan.a, 128);
}

#[test]
fn tes_reaktivitas_sinyal_dan_diffing_ui() {
    let counter = Sinyal::baru(10);
    let terpanggil = std::sync::Arc::new(std::sync::atomic::AtomicBool::new(false));
    let terpanggil_clone = terpanggil.clone();

    counter.dengarkan(move |_nilai_baru| {
        terpanggil_clone.store(true, std::sync::atomic::Ordering::SeqCst);
    });

    counter.setel(25);
    assert_eq!(counter.ambil(), 25);
    assert!(terpanggil.load(std::sync::atomic::Ordering::SeqCst));

    // Diffing pohon elemen
    let pohon_lama = wadah_kolom(vec![teks("Awal"), tombol("Klik", "btn-1")]);
    let pohon_baru = wadah_kolom(vec![teks("Diperbarui"), tombol("Klik", "btn-1")]);

    let daftar_diff = bedakan_pohon(&pohon_lama, &pohon_baru);

    assert_eq!(daftar_diff.len(), 1);
    assert!(matches!(
        daftar_diff[0],
        OperasiDiff::PerbaruiTeks { ref jalur, ref teks_baru } if jalur == &vec![0] && teks_baru == "Diperbarui"
    ));
}

#[test]
fn tes_rasterizer_dan_ekspor_ppm() {
    let mut fb = FramebufferRgba::baru(200, 100);
    fb.bersihkan(WarnaRgba::rgb(10, 10, 20));

    // Gambar persegi dan border
    fb.gambar_persegi(10, 10, 80, 40, WarnaRgba::rgb(99, 102, 241));
    fb.gambar_garis_batas(10, 10, 80, 40, 2, WarnaRgba::PUTIH);

    // Gambar teks bitmap
    fb.gambar_teks_bitmap(15, 20, "KOMO UI", WarnaRgba::PUTIH);

    // Verifikasi ada piksel yang berubah
    let ada_warna = fb.piksel.iter().any(|&b| b > 20);
    assert!(ada_warna, "Framebuffer harus memuat piksel yang telah digambar");

    // Render pohon via SoftwareRenderer
    let mut renderer = SoftwareRenderer::baru(200, 100, TemaUi::gelap());
    let ui = wadah_kolom(vec![teks("KomoUI Demo"), tombol("AKSI", "btn")]);
    renderer.render(&ui);

    let temp_file = std::env::temp_dir().join("komo_ui_interop_test.ppm");
    let hasil_simpan = renderer.fb.simpan_ppm(&temp_file);
    assert!(hasil_simpan.is_ok());
    assert!(temp_file.exists());
    let _ = std::fs::remove_file(temp_file);
}

#[test]
fn tes_interop_binding_vue_dan_ts() {
    let funcs = vec![
        DeskripsiFungsiWasm {
            nama: "hitung_deret_fibonacci".into(),
            parameter: vec![("n".into(), "bulat".into())],
            tipe_kembali: "bulat".into(),
            deskripsi: "Menghitung bilangan Fibonacci".into(),
        },
        DeskripsiFungsiWasm {
            nama: "cek_apakah_prima".into(),
            parameter: vec![("n".into(), "bulat".into())],
            tipe_kembali: "bool".into(),
            deskripsi: "Memeriksa keprimaan bilangan".into(),
        },
    ];

    let binding_vue = buat_binding_vue("logika_berat.wasm", &funcs);
    assert!(binding_vue.contains("export function useKomolang()"));
    assert!(binding_vue.contains("const hitung_deret_fibonacci = (n) =>"));
    assert!(binding_vue.contains("const cek_apakah_prima = (n) =>"));

    let def_ts = buat_definisi_typescript(&funcs);
    assert!(def_ts.contains("hitung_deret_fibonacci(n: number): number;"));
    assert!(def_ts.contains("cek_apakah_prima(n: number): boolean;"));

    // Uji serialisasi NilaiJs
    let mut map = HashMap::new();
    map.insert("status".to_string(), NilaiJs::Teks("sukses".to_string()));
    map.insert("kode".to_string(), NilaiJs::Bulat(200));
    let nilai = NilaiJs::Objek(map);
    let json = nilai.ke_json();
    assert!(json.contains("\"status\": \"sukses\""));
    assert!(json.contains("\"kode\": 200"));
}

#[test]
fn tes_penguraian_kode_logika_berat_komo() {
    let contoh_dir = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .join("examples")
        .join("integrasi_vue")
        .join("src");

    let jalur = contoh_dir.join("logika_berat.komo");
    assert!(jalur.exists(), "Berkas {:?} harus ada", jalur);

    let kode = std::fs::read_to_string(&jalur).expect("Harus dapat membaca logika_berat.komo");
    let hasil_urai = komo_parser::urai(&kode);
    assert!(
        hasil_urai.berhasil(),
        "Penguraian logika_berat.komo gagal: {:?}",
        hasil_urai.diagnostik
    );

    let hasil_sema = komo_sema::periksa(&hasil_urai.berkas);
    assert!(
        hasil_sema.berhasil(),
        "Analisis semantik logika_berat.komo gagal: {:?}",
        hasil_sema.diagnostik
    );
}
