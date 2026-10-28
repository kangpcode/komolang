// Copyright (c) kangpcode (Dhafa Nazula Permadi). Licensed under MIT or Apache-2.0.

//! Test suite integrasi dan konformansi engine game untuk Tahap 11: KomoGame.

use komo_game::audio::{AmplopAdsr, BentukGelombang, MixerAudio, Synthesizer};
use komo_game::demo_game::{DemoGame2D, DemoGame3D};
use komo_game::ecs::DuniaEcs;
use komo_game::fisika2d::{
    perbarui_fisika_2d, uji_aabb, BadanKaku2D, Kecepatan2D, KotakTabrakan2D, Transformasi2D,
};
use komo_game::input::{KodeTombol, ManajerInput, TombolMouse};
use komo_game::loop_game::MesinGame;
use komo_game::renderer2d::{EmitterPartikel, Kamera2D, Renderer2D, Sprite2D, Tilemap2D};
use komo_game::renderer3d::{Matriks4, Mesh3D, Renderer3D, Vektor3};
use komo_ui::renderer::FramebufferRgba;
use komo_ui::tema::WarnaRgba;

#[test]
fn tes_game_ecs_archetype_dan_kueri() {
    let mut dunia = DuniaEcs::baru();

    let e1 = dunia.buat_entitas();
    let e2 = dunia.buat_entitas();
    let e3 = dunia.buat_entitas();

    dunia.pasang_komponen(e1, Transformasi2D::baru(10.0, 20.0));
    dunia.pasang_komponen(e1, Kecepatan2D::baru(2.0, 0.0));
    dunia.pasang_komponen(e1, Sprite2D::baru(16.0, 16.0, WarnaRgba::PUTIH));

    dunia.pasang_komponen(e2, Transformasi2D::baru(50.0, 60.0));
    dunia.pasang_komponen(e2, Sprite2D::baru(32.0, 32.0, WarnaRgba::rgb(255, 0, 0)));

    dunia.pasang_komponen(e3, Transformasi2D::baru(100.0, 200.0));
    dunia.pasang_komponen(e3, Kecepatan2D::baru(-1.0, 5.0));

    assert_eq!(dunia.jumlah_entitas(), 3);

    // Kueri 1 komponen
    let semua_trans = dunia.kueri_1::<Transformasi2D>();
    assert_eq!(semua_trans.len(), 3);

    // Kueri 2 komponen
    let trans_dan_kecepatan = dunia.kueri_2::<Transformasi2D, Kecepatan2D>();
    assert_eq!(trans_dan_kecepatan.len(), 2);

    let trans_dan_sprite = dunia.kueri_2::<Transformasi2D, Sprite2D>();
    assert_eq!(trans_dan_sprite.len(), 2);

    // Daur ulang ID entitas
    assert!(dunia.hapus_entitas(e2));
    assert_eq!(dunia.jumlah_entitas(), 2);
    assert!(!dunia.apakah_hidup(e2));

    let e4 = dunia.buat_entitas();
    assert_eq!(e4.id, e2.id); // ID didaur ulang
    assert_eq!(e4.generasi, e2.generasi + 1); // Generasi dinaikkan
}

#[test]
fn tes_game_input_dan_aksi_mapping() {
    let mut input = ManajerInput::baru();

    input.daftarkan_aksi("Tembak", vec![KodeTombol::Spasi, KodeTombol::HurufJ]);
    input.daftarkan_aksi("Lompat", vec![KodeTombol::HurufW, KodeTombol::PanahAtas]);

    // Uji penekanan keyboard
    input.tekan_tombol(KodeTombol::HurufJ);
    assert!(input.apakah_ditekan(KodeTombol::HurufJ));
    assert!(input.apakah_baru_ditekan(KodeTombol::HurufJ));
    assert!(input.apakah_aksi_aktif("Tembak"));
    assert!(input.apakah_aksi_baru_ditekan("Tembak"));

    input.akhir_frame();
    assert!(input.apakah_ditekan(KodeTombol::HurufJ));
    assert!(!input.apakah_baru_ditekan(KodeTombol::HurufJ)); // Sudah stabil

    // Uji mouse
    input.perbarui_mouse(150.0, 220.0);
    assert_eq!(input.posisi_mouse(), (150.0, 220.0));

    input.perbarui_mouse(165.0, 210.0);
    assert_eq!(input.delta_mouse(), (15.0, -10.0));

    input.tekan_mouse(TombolMouse::Kiri);
    assert!(input.apakah_mouse_ditekan(TombolMouse::Kiri));

    input.lepas_tombol(KodeTombol::HurufJ);
    assert!(input.apakah_baru_dilepas(KodeTombol::HurufJ));
}

#[test]
fn tes_game_audio_synthesizer_dan_sfx_retro() {
    let synth = Synthesizer::baru(44100);

    // Hasilkan nada instrumen dengan ADSR
    let klip_nada = synth.hasilkan_nada(
        440.0, // A4
        0.2,
        BentukGelombang::Sinus,
        AmplopAdsr::baru(0.02, 0.04, 0.7, 0.04),
    );
    assert_eq!(klip_nada.frekuensi_sampel, 44100);
    assert!((klip_nada.durasi_detik() - 0.2).abs() < 0.001);

    // Hasilkan SFX game retro
    let sfx_lompat = synth.sfx_lompat();
    let sfx_laser = synth.sfx_laser();
    let sfx_ledakan = synth.sfx_ledakan();
    let sfx_koin = synth.sfx_koin();

    assert!(!sfx_lompat.sampel.is_empty());
    assert!(!sfx_laser.sampel.is_empty());
    assert!(!sfx_ledakan.sampel.is_empty());
    assert!(!sfx_koin.sampel.is_empty());

    // Uji serialisasi ke RIFF WAVE biner
    let wav_bytes = sfx_koin.ke_wav_bytes();
    assert!(wav_bytes.starts_with(b"RIFF"));
    assert_eq!(&wav_bytes[8..12], b"WAVE");
    assert_eq!(&wav_bytes[12..16], b"fmt ");
    assert_eq!(&wav_bytes[36..40], b"data");

    // Uji Mixer
    let mut mixer = MixerAudio::baru();
    mixer.putar(sfx_koin);
    mixer.putar(sfx_laser);

    let mix_blok = mixer.sintesis_blok(512);
    assert_eq!(mix_blok.len(), 512);
    assert!(mix_blok.iter().any(|&s| s != 0.0));
}

#[test]
fn tes_game_fisika_dan_deteksi_aabb() {
    let t_a = Transformasi2D::baru(50.0, 50.0);
    let k_a = KotakTabrakan2D::baru(20.0, 20.0);

    let t_b = Transformasi2D::baru(62.0, 50.0);
    let k_b = KotakTabrakan2D::baru(20.0, 20.0);

    let hasil_kontak = uji_aabb(&t_a, &k_a, &t_b, &k_b);
    assert!(hasil_kontak.is_some());
    let kontak = hasil_kontak.unwrap();
    assert_eq!(kontak.normal.0, 1.0);
    assert_eq!(kontak.kedalaman, 8.0); // 20 - 12 = 8

    // Simulasi integrasi fisika di dunia ECS
    let mut dunia = DuniaEcs::baru();
    let kotak = dunia.buat_entitas();
    dunia.pasang_komponen(kotak, Transformasi2D::baru(100.0, 0.0));
    dunia.pasang_komponen(kotak, Kecepatan2D::baru(0.0, 0.0));
    dunia.pasang_komponen(kotak, BadanKaku2D::dinamis(1.0));
    dunia.pasang_komponen(kotak, KotakTabrakan2D::baru(16.0, 16.0));

    let lantai = dunia.buat_entitas();
    dunia.pasang_komponen(lantai, Transformasi2D::baru(100.0, 60.0));
    dunia.pasang_komponen(lantai, BadanKaku2D::statis());
    dunia.pasang_komponen(lantai, KotakTabrakan2D::baru(200.0, 20.0));

    // Jalankan integrasi fisika selama 30 frame
    for _ in 0..30 {
        perbarui_fisika_2d(&mut dunia, 0.033, 200.0);
    }

    let pos = dunia.ambil_komponen::<Transformasi2D>(kotak).unwrap();
    // Lantai y=60, tinggi 20 -> atas lantai y=50. Kotak tinggi 16 -> dasar kotak pos.y + 8 <= 50 -> pos.y <= 42.1
    assert!(pos.y <= 42.5, "Kotak harus bertumpu di lantai dan tidak tembus");
}

#[test]
fn tes_game_renderer2d_kamera_dan_tilemap() {
    let kamera = Kamera2D::baru(200.0, 150.0, 1.5);
    let (sx, sy) = kamera.dunia_ke_layar(200.0, 150.0, 400, 300);
    assert_eq!((sx, sy), (200.0, 150.0));

    let (wx, wy) = kamera.layar_ke_dunia(200.0, 150.0, 400, 300);
    assert_eq!((wx, wy), (200.0, 150.0));

    let mut map = Tilemap2D::baru(20, 15, 16.0);
    map.set_ubin(2, 3, 1);
    map.set_ubin(5, 5, 2);
    assert_eq!(map.ambil_ubin(2, 3), 1);
    assert_eq!(map.ambil_ubin(5, 5), 2);

    // Emitter partikel
    let mut emitter = EmitterPartikel::baru();
    emitter.ledakkan(100.0, 100.0, 20, WarnaRgba::rgb(250, 204, 21), 120.0, 0.5);
    assert_eq!(emitter.partikel.len(), 20);

    emitter.perbarui(0.1);
    assert_eq!(emitter.partikel.len(), 20);

    let mut fb = FramebufferRgba::baru(200, 150);
    let kamera_fokus = Kamera2D::baru(60.0, 60.0, 1.0);
    Renderer2D::gambar_tilemap(&mut fb, &kamera_fokus, &map);
    Renderer2D::gambar_partikel(&mut fb, &kamera_fokus, &emitter);

    let ada_warna = fb.piksel.iter().any(|&b| b > 0);
    assert!(ada_warna, "Piksel harus digambar oleh 2D renderer");
}

#[test]
fn tes_game_renderer3d_dan_zbuffer() {
    let v_a = Vektor3::baru(1.0, 2.0, 3.0);
    let v_b = Vektor3::baru(4.0, 5.0, 6.0);

    assert_eq!(v_a.tambah(&v_b), Vektor3::baru(5.0, 7.0, 9.0));
    assert_eq!(v_a.titik(&v_b), 32.0); // 1*4 + 2*5 + 3*6 = 32

    let kubus = Mesh3D::kubus(2.0);
    assert_eq!(kubus.verteks.len(), 24); // 6 sisi x 4 verteks
    assert_eq!(kubus.indeks.len(), 36);  // 6 sisi x 2 segitiga x 3 indeks

    let mut fb = FramebufferRgba::baru(160, 120);
    let mut renderer3d = Renderer3D::baru(160, 120);

    let model = Matriks4::rotasi_y(0.5);
    let view = Matriks4::lihat_ke(
        Vektor3::baru(0.0, 1.0, 4.0),
        Vektor3::NOL,
        Vektor3::ATAS,
    );
    let proj = Matriks4::perspektif(1.0, 160.0 / 120.0, 0.1, 50.0);
    let mvp = proj.kali(&view).kali(&model);

    renderer3d.render_mesh(&mut fb, &kubus, &mvp, &model);
    let render_berhasil = fb.piksel.iter().any(|&b| b > 30);
    assert!(render_berhasil, "Kubus 3D harus terender dengan z-buffering");
}

#[test]
fn tes_game_stabilitas_60fps_dan_profiler() {
    let mut mesin = MesinGame::baru(320, 240);

    // Uji Demo Game 2D
    let mut game2d = DemoGame2D::baru();
    mesin.siapkan(&mut game2d);

    // Simulasikan 60 frame (1 detik) pada 16.66 ms
    for _ in 0..60 {
        mesin.langkah_frame(&mut game2d, 16.66);
    }

    let stats2d = mesin.profiler.hitung_statistik();
    assert_eq!(stats2d.total_frame, 60);
    assert!(stats2d.fps_rata_rata >= 59.0 && stats2d.fps_rata_rata <= 61.0);
    assert!(mesin.profiler.apakah_stabil_60_fps());

    // Uji Demo Game 3D
    let mut mesin3d = MesinGame::baru(160, 120);
    let mut game3d = DemoGame3D::baru(160, 120);
    mesin3d.siapkan(&mut game3d);

    for _ in 0..60 {
        mesin3d.langkah_frame(&mut game3d, 16.66);
    }

    let stats3d = mesin3d.profiler.hitung_statistik();
    assert_eq!(stats3d.total_frame, 60);
    assert!(stats3d.fps_rata_rata >= 59.0);
    assert!(mesin3d.profiler.apakah_stabil_60_fps());
}
