// Copyright (c) kangpcode (Dhafa Nazula Permadi). Licensed under MIT or Apache-2.0.

//! Loop game deterministik 60 FPS dan Profiler Frame-Time performa tinggi.

use std::time::{Duration, Instant};

use crate::ecs::DuniaEcs;
use crate::input::ManajerInput;
use komo_ui::renderer::FramebufferRgba;

/// Target durasi satu frame pada refresh rate 60 FPS (16.666 milidetik).
pub const DURASI_FRAME_60_FPS: Duration = Duration::from_nanos(16_666_667);
pub const DELTA_DETIK_60_FPS: f32 = 1.0 / 60.0;

/// Statistik kinerja frame-time yang dicatat oleh profiler engine.
#[derive(Debug, Clone)]
pub struct StatistikFrame {
    pub total_frame: u64,
    pub fps_saat_ini: f32,
    pub fps_rata_rata: f32,
    pub waktu_frame_ms: f32,
    pub waktu_terpanjang_ms: f32,
    pub waktu_terpendek_ms: f32,
    pub persentil_99_ms: f32, // 1% lowest performance metric
    pub jumlah_frame_drop: u64,
}

/// Profiler presisi tinggi untuk memantau stabilitas 60 FPS dan latensi frame.
#[derive(Debug, Clone)]
pub struct ProfilerFrame {
    riwayat_durasi_ms: Vec<f32>,
    kapasitas_riwayat: usize,
    waktu_mulai_frame: Option<Instant>,
    total_frame: u64,
    frame_drop_count: u64,
}

impl ProfilerFrame {
    pub fn baru(kapasitas_sampel: usize) -> Self {
        Self {
            riwayat_durasi_ms: Vec::with_capacity(kapasitas_sampel),
            kapasitas_riwayat: kapasitas_sampel,
            waktu_mulai_frame: None,
            total_frame: 0,
            frame_drop_count: 0,
        }
    }

    /// Menandai awal eksekusi frame baru.
    pub fn mulai_frame(&mut self) {
        self.waktu_mulai_frame = Some(Instant::now());
    }

    /// Menandai akhir eksekusi frame dan mencatat durasi elapsed time.
    pub fn catat_frame_selesai(&mut self) -> f32 {
        let mulai = self.waktu_mulai_frame.take().unwrap_or_else(Instant::now);
        let elapsed = mulai.elapsed();
        let durasi_ms = elapsed.as_secs_f32() * 1000.0;

        self.catat_durasi_ms(durasi_ms);
        durasi_ms
    }

    /// Mencatat durasi frame tertentu secara eksplisit (berguna untuk simulasi).
    pub fn catat_durasi_ms(&mut self, durasi_ms: f32) {
        self.total_frame += 1;
        if durasi_ms > 17.5 {
            self.frame_drop_count += 1;
        }

        if self.riwayat_durasi_ms.len() >= self.kapasitas_riwayat {
            self.riwayat_durasi_ms.remove(0);
        }
        self.riwayat_durasi_ms.push(durasi_ms);
    }

    /// Mengambil metrik statistik lengkap kinerja game.
    pub fn hitung_statistik(&self) -> StatistikFrame {
        if self.riwayat_durasi_ms.is_empty() {
            return StatistikFrame {
                total_frame: self.total_frame,
                fps_saat_ini: 60.0,
                fps_rata_rata: 60.0,
                waktu_frame_ms: 16.66,
                waktu_terpanjang_ms: 16.66,
                waktu_terpendek_ms: 16.66,
                persentil_99_ms: 16.66,
                jumlah_frame_drop: self.frame_drop_count,
            };
        }

        let mut urut = self.riwayat_durasi_ms.clone();
        urut.sort_by(|a, b| a.partial_cmp(b).unwrap());

        let terakhir = *self.riwayat_durasi_ms.last().unwrap();
        let terpendek = *urut.first().unwrap();
        let terpanjang = *urut.last().unwrap();

        let total: f32 = urut.iter().sum();
        let rata_rata = total / urut.len() as f32;

        let p99_idx = ((urut.len() as f32 * 0.99) as usize).min(urut.len() - 1);
        let p99 = urut[p99_idx];

        let fps_saat_ini = if terakhir > 0.0 { 1000.0 / terakhir } else { 60.0 };
        let fps_rata = if rata_rata > 0.0 { 1000.0 / rata_rata } else { 60.0 };

        StatistikFrame {
            total_frame: self.total_frame,
            fps_saat_ini,
            fps_rata_rata: fps_rata,
            waktu_frame_ms: terakhir,
            waktu_terpanjang_ms: terpanjang,
            waktu_terpendek_ms: terpendek,
            persentil_99_ms: p99,
            jumlah_frame_drop: self.frame_drop_count,
        }
    }

    /// Memeriksa apakah game berjalan stabil di standar 60 FPS (rata-rata FPS >= 58.0).
    pub fn apakah_stabil_60_fps(&self) -> bool {
        let stats = self.hitung_statistik();
        stats.fps_rata_rata >= 58.0 && stats.persentil_99_ms <= 20.0
    }
}

/// Siklus hidup (Lifecycle) logika game.
pub trait SiklusGame {
    fn inisialisasi(&mut self, dunia: &mut DuniaEcs);
    fn perbarui(&mut self, dunia: &mut DuniaEcs, input: &ManajerInput, delta_detik: f32);
    fn render(&mut self, dunia: &DuniaEcs, fb: &mut FramebufferRgba);
}

/// Mesin pelaksana loop game utama dengan fixed update rate.
pub struct MesinGame {
    pub dunia: DuniaEcs,
    pub input: ManajerInput,
    pub fb: FramebufferRgba,
    pub profiler: ProfilerFrame,
    pub delta_tetap: f32,
    pub akumulator_waktu: f32,
}

impl MesinGame {
    pub fn baru(lebar: usize, tinggi: usize) -> Self {
        Self {
            dunia: DuniaEcs::baru(),
            input: ManajerInput::baru(),
            fb: FramebufferRgba::baru(lebar, tinggi),
            profiler: ProfilerFrame::baru(120),
            delta_tetap: DELTA_DETIK_60_FPS,
            akumulator_waktu: 0.0,
        }
    }

    /// Menjalankan inisialisasi game.
    pub fn siapkan(&mut self, game: &mut dyn SiklusGame) {
        game.inisialisasi(&mut self.dunia);
    }

    /// Mengeksekusi satu langkah frame penuh: input update -> game update -> render -> profiler.
    pub fn langkah_frame(&mut self, game: &mut dyn SiklusGame, waktu_nyata_ms: f32) {
        self.profiler.mulai_frame();

        // Fixed-timestep simulation step
        game.perbarui(&mut self.dunia, &self.input, self.delta_tetap);
        self.input.akhir_frame();

        // Rendering phase
        game.render(&self.dunia, &mut self.fb);

        // Catat durasi frame ke profiler
        self.profiler.catat_durasi_ms(waktu_nyata_ms);
    }

    /// Menjalankan serangkaian frame simulasi untuk keperluan validasi benchmark dan headless test.
    pub fn jalankan_simulasi_frame(
        &mut self,
        game: &mut dyn SiklusGame,
        jumlah_frame: usize,
        rata_rata_waktu_frame_ms: f32,
    ) {
        for _ in 0..jumlah_frame {
            self.langkah_frame(game, rata_rata_waktu_frame_ms);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct GameTes {
        skor: u32,
    }

    impl SiklusGame for GameTes {
        fn inisialisasi(&mut self, _dunia: &mut DuniaEcs) {
            self.skor = 100;
        }

        fn perbarui(&mut self, _dunia: &mut DuniaEcs, _input: &ManajerInput, _dt: f32) {
            self.skor += 1;
        }

        fn render(&mut self, _dunia: &DuniaEcs, _fb: &mut FramebufferRgba) {}
    }

    #[test]
    fn tes_loop_game_dan_profiler_60fps() {
        let mut mesin = MesinGame::baru(160, 120);
        let mut game = GameTes { skor: 0 };

        mesin.siapkan(&mut game);
        assert_eq!(game.skor, 100);

        // Simulasikan 60 frame tepat pada 16.66 ms (60 FPS ideal)
        mesin.jalankan_simulasi_frame(&mut game, 60, 16.66);

        assert_eq!(game.skor, 160);
        let stats = mesin.profiler.hitung_statistik();
        assert_eq!(stats.total_frame, 60);
        assert!(stats.fps_rata_rata >= 59.9 && stats.fps_rata_rata <= 60.1);
        assert!(mesin.profiler.apakah_stabil_60_fps());
    }
}
