// Copyright (c) kangpcode (Dhafa Nazula Permadi). Licensed under MIT or Apache-2.0.

//! Implementasi contoh game 2D lengkap ("Petualangan Komo") dan demo 3D berputar pada 60 FPS stabil.

use crate::audio::Synthesizer;
use crate::ecs::{DuniaEcs, Entitas};
use crate::fisika2d::{
    perbarui_fisika_2d, uji_aabb, BadanKaku2D, Kecepatan2D, KotakTabrakan2D, Transformasi2D,
};
use crate::input::{KodeTombol, ManajerInput};
use crate::loop_game::SiklusGame;
use crate::renderer2d::{EmitterPartikel, Kamera2D, Renderer2D, Sprite2D, Tilemap2D};
use crate::renderer3d::{Matriks4, Mesh3D, Renderer3D, Vektor3};
use komo_ui::renderer::FramebufferRgba;
use komo_ui::tema::WarnaRgba;

// =============================================================================
// CONTOH GAME 2D LENGKAP: "Petualangan Komo"
// =============================================================================

/// Game platformer 2D lengkap dengan gravitasi, koin, rintangan musuh, dan efek partikel.
pub struct DemoGame2D {
    pub entitas_pemain: Option<Entitas>,
    pub kamera: Kamera2D,
    pub tilemap: Tilemap2D,
    pub partikel: EmitterPartikel,
    pub synth: Synthesizer,
    pub skor: u32,
    pub sisa_nyawa: u32,
    pub daftar_koin: Vec<Entitas>,
    pub daftar_musuh: Vec<Entitas>,
    pub apakah_selesai: bool,
}

impl DemoGame2D {
    pub fn baru() -> Self {
        let mut map = Tilemap2D::baru(30, 20, 16.0);
        // Buat lantai ubin (baris 18 & 19)
        for x in 0..30 {
            map.set_ubin(x, 18, 2); // Rumput
            map.set_ubin(x, 19, 1); // Tanah
        }
        // Buat beberapa platform melayang
        for x in 8..14 {
            map.set_ubin(x, 14, 2);
        }
        for x in 18..24 {
            map.set_ubin(x, 10, 2);
        }

        Self {
            entitas_pemain: None,
            kamera: Kamera2D::baru(240.0, 160.0, 1.0),
            tilemap: map,
            partikel: EmitterPartikel::baru(),
            synth: Synthesizer::baru(22050),
            skor: 0,
            sisa_nyawa: 3,
            daftar_koin: Vec::new(),
            daftar_musuh: Vec::new(),
            apakah_selesai: false,
        }
    }
}

impl SiklusGame for DemoGame2D {
    fn inisialisasi(&mut self, dunia: &mut DuniaEcs) {
        // 1. Buat Entitas Pemain
        let p = dunia.buat_entitas();
        dunia.pasang_komponen(p, Transformasi2D::baru(40.0, 100.0));
        dunia.pasang_komponen(p, Kecepatan2D::baru(0.0, 0.0));
        dunia.pasang_komponen(p, BadanKaku2D::dinamis(1.0));
        dunia.pasang_komponen(p, KotakTabrakan2D::baru(14.0, 18.0));
        dunia.pasang_komponen(
            p,
            Sprite2D::baru(14.0, 18.0, WarnaRgba::rgb(56, 189, 248)), // Sky blue
        );
        self.entitas_pemain = Some(p);

        // 2. Buat Entitas Koin untuk dikoleksi
        let koordinat_koin = [(100.0, 200.0), (160.0, 150.0), (200.0, 150.0), (320.0, 100.0)];
        for (kx, ky) in koordinat_koin {
            let koin = dunia.buat_entitas();
            dunia.pasang_komponen(koin, Transformasi2D::baru(kx, ky));
            dunia.pasang_komponen(koin, KotakTabrakan2D::sensor(10.0, 10.0));
            dunia.pasang_komponen(
                koin,
                Sprite2D::baru(10.0, 10.0, WarnaRgba::rgb(250, 204, 21)), // Emas
            );
            self.daftar_koin.push(koin);
        }

        // 3. Buat Entitas Rintangan / Musuh patroli
        let m = dunia.buat_entitas();
        dunia.pasang_komponen(m, Transformasi2D::baru(180.0, 260.0));
        dunia.pasang_komponen(m, Kecepatan2D::baru(40.0, 0.0));
        dunia.pasang_komponen(m, BadanKaku2D::statis());
        dunia.pasang_komponen(m, KotakTabrakan2D::baru(16.0, 14.0));
        dunia.pasang_komponen(
            m,
            Sprite2D::baru(16.0, 14.0, WarnaRgba::rgb(248, 113, 113)), // Merah
        );
        self.daftar_musuh.push(m);

        // 4. Buat Collider Solid untuk lantai dan platform
        let lantai = dunia.buat_entitas();
        dunia.pasang_komponen(lantai, Transformasi2D::baru(240.0, 296.0));
        dunia.pasang_komponen(lantai, BadanKaku2D::statis());
        dunia.pasang_komponen(lantai, KotakTabrakan2D::baru(480.0, 32.0));

        let plat1 = dunia.buat_entitas();
        dunia.pasang_komponen(plat1, Transformasi2D::baru(176.0, 232.0));
        dunia.pasang_komponen(plat1, BadanKaku2D::statis());
        dunia.pasang_komponen(plat1, KotakTabrakan2D::baru(96.0, 16.0));
    }

    fn perbarui(&mut self, dunia: &mut DuniaEcs, input: &ManajerInput, dt: f32) {
        if self.apakah_selesai {
            return;
        }

        // 1. Kontrol Pemain
        if let Some(ep) = self.entitas_pemain {
            let mut gerak_x = 0.0;
            if input.apakah_ditekan(KodeTombol::PanahKiri) || input.apakah_ditekan(KodeTombol::HurufA) {
                gerak_x -= 140.0;
            }
            if input.apakah_ditekan(KodeTombol::PanahKanan) || input.apakah_ditekan(KodeTombol::HurufD) {
                gerak_x += 140.0;
            }

            let lompat = input.apakah_baru_ditekan(KodeTombol::Spasi)
                || input.apakah_baru_ditekan(KodeTombol::HurufW);

            if let Some(v) = dunia.ambil_komponen_mut::<Kecepatan2D>(ep) {
                v.vx = gerak_x;
                if lompat && v.vy.abs() < 10.0 {
                    v.vy = -280.0; // Impuls lompat ke atas
                    let _sfx = self.synth.sfx_lompat();
                }
            }

            // Update posisi kamera agar mengikuti pemain
            if let Some(t) = dunia.ambil_komponen::<Transformasi2D>(ep) {
                self.kamera.x = t.x;
                self.kamera.y = t.y - 20.0;
            }
        }

        // 2. Simulasi Fisika 2D (Gravitasi 450 px/s^2)
        perbarui_fisika_2d(dunia, dt, 450.0);

        // 3. Deteksi Pengambilan Koin
        if let Some(ep) = self.entitas_pemain {
            let trans_p = dunia.ambil_komponen::<Transformasi2D>(ep).copied();
            let kotak_p = dunia.ambil_komponen::<KotakTabrakan2D>(ep).copied();

            if let (Some(tp), Some(kp)) = (trans_p, kotak_p) {
                let mut koin_terambil = Vec::new();
                for (idx, &ek) in self.daftar_koin.iter().enumerate() {
                    let trans_k = match dunia.ambil_komponen::<Transformasi2D>(ek) {
                        Some(&t) => t,
                        None => continue,
                    };
                    let kotak_k = match dunia.ambil_komponen::<KotakTabrakan2D>(ek) {
                        Some(&k) => k,
                        None => continue,
                    };

                    if uji_aabb(&tp, &kp, &trans_k, &kotak_k).is_some() {
                        koin_terambil.push((idx, ek, trans_k.x, trans_k.y));
                    }
                }

                for (idx, ek, kx, ky) in koin_terambil.into_iter().rev() {
                    self.daftar_koin.remove(idx);
                    dunia.hapus_entitas(ek);
                    self.skor += 100;

                    // Efek suara & partikel
                    let _sfx = self.synth.sfx_koin();
                    self.partikel.ledakkan(
                        kx,
                        ky,
                        12,
                        WarnaRgba::rgb(250, 204, 21),
                        100.0,
                        0.4,
                    );
                }
            }
        }

        // 4. Update Partikel
        self.partikel.perbarui(dt);
    }

    fn render(&mut self, dunia: &DuniaEcs, fb: &mut FramebufferRgba) {
        // Latar langit malam modern
        fb.bersihkan(WarnaRgba::rgb(15, 23, 42));

        // Gambar Tilemap
        Renderer2D::gambar_tilemap(fb, &self.kamera, &self.tilemap);

        // Gambar Sprites dari ECS
        let entitas_sprite = dunia.kueri_2::<Transformasi2D, Sprite2D>();
        for (_, trans, sprite) in entitas_sprite {
            Renderer2D::gambar_sprite(
                fb,
                &self.kamera,
                trans.x,
                trans.y,
                sprite.lebar,
                sprite.tinggi,
                sprite.warna,
            );
        }

        // Gambar Partikel
        Renderer2D::gambar_partikel(fb, &self.kamera, &self.partikel);

        // Gambar HUD UI (Skor & Nyawa)
        fb.gambar_teks_bitmap(
            12,
            12,
            &format!("SKOR: {:05}", self.skor),
            WarnaRgba::rgb(250, 204, 21),
        );
        fb.gambar_teks_bitmap(
            12,
            24,
            &format!("NYAWA: {}", self.sisa_nyawa),
            WarnaRgba::rgb(248, 113, 113),
        );
    }
}

// =============================================================================
// CONTOH DEMO 3D: "Kubus 3D Komolang Berputar pada 60 FPS"
// =============================================================================

/// Demo rendering 3D kubus berputar dengan pencahayaan dan z-buffer.
pub struct DemoGame3D {
    pub renderer3d: Renderer3D,
    pub kubus: Mesh3D,
    pub sudut_rotasi_x: f32,
    pub sudut_rotasi_y: f32,
    pub sudut_rotasi_z: f32,
    pub frame_counter: u64,
}

impl DemoGame3D {
    pub fn baru(lebar: usize, tinggi: usize) -> Self {
        Self {
            renderer3d: Renderer3D::baru(lebar, tinggi),
            kubus: Mesh3D::kubus(2.2),
            sudut_rotasi_x: 0.0,
            sudut_rotasi_y: 0.0,
            sudut_rotasi_z: 0.0,
            frame_counter: 0,
        }
    }
}

impl SiklusGame for DemoGame3D {
    fn inisialisasi(&mut self, _dunia: &mut DuniaEcs) {
        self.sudut_rotasi_x = 0.2;
        self.sudut_rotasi_y = 0.4;
    }

    fn perbarui(&mut self, _dunia: &mut DuniaEcs, _input: &ManajerInput, dt: f32) {
        self.sudut_rotasi_x += 0.8 * dt;
        self.sudut_rotasi_y += 1.2 * dt;
        self.sudut_rotasi_z += 0.5 * dt;
        self.frame_counter += 1;
    }

    fn render(&mut self, _dunia: &DuniaEcs, fb: &mut FramebufferRgba) {
        fb.bersihkan(WarnaRgba::rgb(10, 15, 30));

        let model = Matriks4::rotasi_y(self.sudut_rotasi_y)
            .kali(&Matriks4::rotasi_x(self.sudut_rotasi_x))
            .kali(&Matriks4::rotasi_z(self.sudut_rotasi_z));

        let view = Matriks4::lihat_ke(
            Vektor3::baru(0.0, 1.5, 4.5),
            Vektor3::NOL,
            Vektor3::ATAS,
        );

        let aspek = fb.lebar as f32 / fb.tinggi as f32;
        let proj = Matriks4::perspektif(1.047, aspek, 0.1, 100.0);
        let mvp = proj.kali(&view).kali(&model);

        self.renderer3d.render_mesh(fb, &self.kubus, &mvp, &model);

        // HUD Status 3D
        fb.gambar_teks_bitmap(10, 10, "KOMO 3D ENGINE 60 FPS", WarnaRgba::PUTIH);
        fb.gambar_teks_bitmap(
            10,
            20,
            &format!("FRAME: {}", self.frame_counter),
            WarnaRgba::rgb(129, 140, 248),
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::loop_game::MesinGame;

    #[test]
    fn tes_demo_game_2d_lengkap_dan_koin() {
        let mut mesin = MesinGame::baru(320, 240);
        let mut game = DemoGame2D::baru();

        mesin.siapkan(&mut game);
        assert_eq!(game.skor, 0);
        assert_eq!(game.daftar_koin.len(), 4);

        // Jalankan beberapa frame dan uji pergerakan
        mesin.input.tekan_tombol(KodeTombol::PanahKanan);
        mesin.langkah_frame(&mut game, 16.66);
        mesin.langkah_frame(&mut game, 16.66);

        // Pastikan pemain bergerak
        let ep = game.entitas_pemain.unwrap();
        let trans = mesin.dunia.ambil_komponen::<Transformasi2D>(ep).unwrap();
        assert!(trans.x > 40.0, "Pemain harus bergerak ke kanan");
    }

    #[test]
    fn tes_demo_game_3d_rotasi_dan_profiler_60fps() {
        let mut mesin = MesinGame::baru(160, 120);
        let mut game3d = DemoGame3D::baru(160, 120);

        mesin.siapkan(&mut game3d);

        // Simulasikan 60 frame (1 detik game time pada 60 FPS)
        for _ in 0..60 {
            mesin.langkah_frame(&mut game3d, 16.66);
        }

        assert_eq!(game3d.frame_counter, 60);
        assert!(game3d.sudut_rotasi_x > 0.5);

        let stats = mesin.profiler.hitung_statistik();
        assert_eq!(stats.total_frame, 60);
        assert!(stats.fps_rata_rata >= 59.0);
        assert!(mesin.profiler.apakah_stabil_60_fps());
    }
}
