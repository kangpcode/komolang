// Copyright (c) kangpcode (Dhafa Nazula Permadi). Licensed under MIT or Apache-2.0.

//! Subperintah CLI `komo main` untuk menjalankan engine KomoGame (2D / 3D).

use komo_game::demo_game::{DemoGame2D, DemoGame3D};
use komo_game::loop_game::MesinGame;

/// Menjalankan sesi game KomoGame dalam mode interaktif atau simulasi performa.
pub fn jalankan_game(apakah_3d: bool, jumlah_frame: usize) -> Result<(), String> {
    if apakah_3d {
        println!("🎮 Menginisialisasi KomoGame Engine: Mode 3D (Kubus MVP + Z-Buffer)...");
        let mut mesin = MesinGame::baru(320, 240);
        let mut game = DemoGame3D::baru(320, 240);
        mesin.siapkan(&mut game);

        println!("⚡ Menjalankan simulasi {} frame pada target 60 FPS...", jumlah_frame);
        for _ in 0..jumlah_frame {
            mesin.langkah_frame(&mut game, 16.66);
        }

        let stats = mesin.profiler.hitung_statistik();
        println!("📊 Hasil Profiler Frame-Time 3D:");
        println!("   - Rata-rata FPS      : {:.1} FPS", stats.fps_rata_rata);
        println!("   - Frame Time Terakhir: {:.2} ms", stats.waktu_frame_ms);
        println!("   - 1% Lowest Frame    : {:.2} ms", stats.persentil_99_ms);
        println!("   - Frame Drop         : {} frame", stats.jumlah_frame_drop);
        if mesin.profiler.apakah_stabil_60_fps() {
            println!("✅ Performa 3D STABIL pada 60 FPS!");
        } else {
            println!("⚠️  Performa 3D mengalami penurunan latensi.");
        }
    } else {
        println!("🎮 Menginisialisasi KomoGame Engine: Mode 2D (Petualangan Komo)...");
        let mut mesin = MesinGame::baru(320, 240);
        let mut game = DemoGame2D::baru();
        mesin.siapkan(&mut game);

        println!("⚡ Menjalankan simulasi {} frame gameplay pada target 60 FPS...", jumlah_frame);
        for _ in 0..jumlah_frame {
            mesin.langkah_frame(&mut game, 16.66);
        }

        let stats = mesin.profiler.hitung_statistik();
        println!("📊 Hasil Profiler Frame-Time 2D:");
        println!("   - Skor Akhir         : {}", game.skor);
        println!("   - Sisa Koin          : {}", game.daftar_koin.len());
        println!("   - Rata-rata FPS      : {:.1} FPS", stats.fps_rata_rata);
        println!("   - 1% Lowest Frame    : {:.2} ms", stats.persentil_99_ms);
        println!("   - Frame Drop         : {} frame", stats.jumlah_frame_drop);
        if mesin.profiler.apakah_stabil_60_fps() {
            println!("✅ Performa 2D STABIL pada 60 FPS!");
        } else {
            println!("⚠️  Performa 2D mengalami penurunan latensi.");
        }
    }

    Ok(())
}
