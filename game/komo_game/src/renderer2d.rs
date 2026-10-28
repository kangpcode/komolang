// Copyright (c) kangpcode (Dhafa Nazula Permadi). Licensed under MIT or Apache-2.0.

//! Sistem rendering 2D, kamera 2D dengan transformasi ruang, Tilemap grid, dan partikel visual.

use komo_ui::renderer::FramebufferRgba;
use komo_ui::tema::WarnaRgba;

/// Komponen kamera 2D dengan zoom dan panning halus.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Kamera2D {
    pub x: f32,
    pub y: f32,
    pub zoom: f32,
}

impl Kamera2D {
    pub const fn baru(x: f32, y: f32, zoom: f32) -> Self {
        Self { x, y, zoom }
    }

    /// Mengonversi posisi di koordinat dunia 2D ke koordinat piksel layar framebuffer.
    pub fn dunia_ke_layar(
        &self,
        wx: f32,
        wy: f32,
        lebar_layar: usize,
        tinggi_layar: usize,
    ) -> (f32, f32) {
        let tengah_x = lebar_layar as f32 * 0.5;
        let tengah_y = tinggi_layar as f32 * 0.5;

        let sx = tengah_x + (wx - self.x) * self.zoom;
        let sy = tengah_y + (wy - self.y) * self.zoom;

        (sx, sy)
    }

    /// Mengonversi koordinat piksel layar ke posisi ruang dunia game.
    pub fn layar_ke_dunia(
        &self,
        sx: f32,
        sy: f32,
        lebar_layar: usize,
        tinggi_layar: usize,
    ) -> (f32, f32) {
        let tengah_x = lebar_layar as f32 * 0.5;
        let tengah_y = tinggi_layar as f32 * 0.5;

        let wx = self.x + (sx - tengah_x) / self.zoom;
        let wy = self.y + (sy - tengah_y) / self.zoom;

        (wx, wy)
    }
}

/// Komponen visual sprite 2D.
#[derive(Debug, Clone, PartialEq)]
pub struct Sprite2D {
    pub lebar: f32,
    pub tinggi: f32,
    pub warna: WarnaRgba,
    pub z_index: i32,
}

impl Sprite2D {
    pub const fn baru(lebar: f32, tinggi: f32, warna: WarnaRgba) -> Self {
        Self {
            lebar,
            tinggi,
            warna,
            z_index: 0,
        }
    }

    pub const fn dengan_z(mut self, z: i32) -> Self {
        self.z_index = z;
        self
    }
}

/// Komponen peta ubin (Tilemap) berbasis grid 2D yang efisien dengan culling.
#[derive(Debug, Clone)]
pub struct Tilemap2D {
    pub kolom: usize,
    pub baris: usize,
    pub ukuran_ubin: f32,
    pub petak: Vec<u16>, // 0 = kosong, >0 = id jenis ubin
    pub palet_warna: Vec<WarnaRgba>,
}

impl Tilemap2D {
    pub fn baru(kolom: usize, baris: usize, ukuran_ubin: f32) -> Self {
        Self {
            kolom,
            baris,
            ukuran_ubin,
            petak: vec![0; kolom * baris],
            palet_warna: vec![
                WarnaRgba::transparan(),      // 0: kosong
                WarnaRgba::rgb(71, 85, 105),   // 1: tanah / batu (slate-600)
                WarnaRgba::rgb(34, 197, 94),   // 2: rumput (green-500)
                WarnaRgba::rgb(234, 179, 8),   // 3: koin / emas (yellow-500)
                WarnaRgba::rgb(239, 68, 68),   // 4: duri / bahaya (red-500)
            ],
        }
    }

    pub fn set_ubin(&mut self, x: usize, y: usize, id_ubin: u16) {
        if x < self.kolom && y < self.baris {
            self.petak[y * self.kolom + x] = id_ubin;
        }
    }

    pub fn ambil_ubin(&self, x: usize, y: usize) -> u16 {
        if x < self.kolom && y < self.baris {
            self.petak[y * self.kolom + x]
        } else {
            0
        }
    }
}

/// Struktur data satu butir partikel dinamis.
#[derive(Debug, Clone, Copy)]
pub struct Partikel {
    pub x: f32,
    pub y: f32,
    pub vx: f32,
    pub vy: f32,
    pub warna: WarnaRgba,
    pub sisa_umur: f32,
    pub total_umur: f32,
    pub ukuran: f32,
}

/// Sistem emitter partikel untuk efek ledakan, api, atau percikan.
#[derive(Debug, Clone)]
pub struct EmitterPartikel {
    pub partikel: Vec<Partikel>,
}

impl EmitterPartikel {
    pub fn baru() -> Self {
        Self {
            partikel: Vec::new(),
        }
    }

    /// Memicu ledakan partikel memancar dari satu titik pusat.
    pub fn ledakkan(
        &mut self,
        x: f32,
        y: f32,
        jumlah: usize,
        warna: WarnaRgba,
        kecepatan: f32,
        umur: f32,
    ) {
        let mut rng: u64 = 424242;
        for i in 0..jumlah {
            rng = rng.wrapping_mul(6364136223846793005).wrapping_add(i as u64 + 1);
            let sudut = ((rng >> 32) as f32 / 4294967296.0) * 2.0 * std::f32::consts::PI;

            rng = rng.wrapping_mul(6364136223846793005).wrapping_add(1);
            let spd = kecepatan * (0.3 + 0.7 * ((rng >> 32) as f32 / 4294967296.0));

            self.partikel.push(Partikel {
                x,
                y,
                vx: sudut.cos() * spd,
                vy: sudut.sin() * spd,
                warna,
                sisa_umur: umur,
                total_umur: umur,
                ukuran: 3.0,
            });
        }
    }

    /// Memperbarui posisi dan umur partikel berdasarkan delta waktu.
    pub fn perbarui(&mut self, dt: f32) {
        self.partikel.retain_mut(|p| {
            p.x += p.vx * dt;
            p.y += p.vy * dt;
            p.sisa_umur -= dt;
            p.sisa_umur > 0.0
        });
    }
}

impl Default for EmitterPartikel {
    fn default() -> Self {
        Self::baru()
    }
}

/// Renderer grafis 2D yang merasterisasi objek-objek game ke dalam framebuffer.
pub struct Renderer2D;

impl Renderer2D {
    /// Menggambar sprite kotak terpusat di koordinat dunia dengan memperhitungkan kamera.
    pub fn gambar_sprite(
        fb: &mut FramebufferRgba,
        kamera: &Kamera2D,
        x: f32,
        y: f32,
        lebar: f32,
        tinggi: f32,
        warna: WarnaRgba,
    ) {
        let (sx, sy) = kamera.dunia_ke_layar(x, y, fb.lebar, fb.tinggi);
        let sw = (lebar * kamera.zoom).max(1.0);
        let sh = (tinggi * kamera.zoom).max(1.0);

        let kiri = (sx - sw * 0.5) as i32;
        let atas = (sy - sh * 0.5) as i32;

        if kiri + (sw as i32) < 0
            || kiri >= fb.lebar as i32
            || atas + (sh as i32) < 0
            || atas >= fb.tinggi as i32
        {
            return; // Culling luar layar
        }

        let px = kiri.max(0) as usize;
        let py = atas.max(0) as usize;
        let pw = ((kiri + sw as i32).min(fb.lebar as i32) - px as i32).max(0) as usize;
        let ph = ((atas + sh as i32).min(fb.tinggi as i32) - py as i32).max(0) as usize;

        fb.gambar_persegi(px, py, pw, ph, warna);
    }

    /// Merender petak-petak ubin Tilemap yang terlihat di dalam kamera (Viewport Culling).
    pub fn gambar_tilemap(fb: &mut FramebufferRgba, kamera: &Kamera2D, tilemap: &Tilemap2D) {
        let s_ubin = tilemap.ukuran_ubin;

        for ty in 0..tilemap.baris {
            let wy = ty as f32 * s_ubin + (s_ubin * 0.5);
            for tx in 0..tilemap.kolom {
                let id = tilemap.ambil_ubin(tx, ty);
                if id == 0 || (id as usize) >= tilemap.palet_warna.len() {
                    continue;
                }

                let wx = tx as f32 * s_ubin + (s_ubin * 0.5);
                let warna = tilemap.palet_warna[id as usize];
                Self::gambar_sprite(fb, kamera, wx, wy, s_ubin, s_ubin, warna);
            }
        }
    }

    /// Merender seluruh partikel yang aktif.
    pub fn gambar_partikel(fb: &mut FramebufferRgba, kamera: &Kamera2D, emitter: &EmitterPartikel) {
        for p in &emitter.partikel {
            let rasio = (p.sisa_umur / p.total_umur).clamp(0.0, 1.0);
            let mut w = p.warna;
            w.a = (w.a as f32 * rasio) as u8;

            Self::gambar_sprite(fb, kamera, p.x, p.y, p.ukuran, p.ukuran, w);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tes_kamera_dan_transformasi_koordinat() {
        let kamera = Kamera2D::baru(100.0, 100.0, 2.0);
        let (sx, sy) = kamera.dunia_ke_layar(100.0, 100.0, 800, 600);
        assert_eq!((sx, sy), (400.0, 300.0)); // Titik tepat di tengah layar

        let (wx, wy) = kamera.layar_ke_dunia(400.0, 300.0, 800, 600);
        assert_eq!((wx, wy), (100.0, 100.0));
    }

    #[test]
    fn tes_tilemap_dan_rendering_2d() {
        let mut map = Tilemap2D::baru(10, 10, 16.0);
        map.set_ubin(0, 0, 1);
        map.set_ubin(1, 0, 2);

        assert_eq!(map.ambil_ubin(0, 0), 1);
        assert_eq!(map.ambil_ubin(1, 0), 2);
        assert_eq!(map.ambil_ubin(5, 5), 0);

        let mut fb = FramebufferRgba::baru(160, 120);
        let kamera = Kamera2D::baru(80.0, 60.0, 1.0);

        Renderer2D::gambar_tilemap(&mut fb, &kamera, &map);
        let ada_warna = fb.piksel.iter().any(|&b| b > 0);
        assert!(ada_warna);
    }
}
