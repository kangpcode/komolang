// Copyright (c) kangpcode (Dhafa Nazula Permadi). Licensed under MIT or Apache-2.0.

//! Sistem fisika 2D, integrasi gerak, dan deteksi/resolusi tabrakan (Collision Detection).

use crate::ecs::{DuniaEcs, Entitas};

/// Komponen posisi, rotasi, dan skala di ruang 2D.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Transformasi2D {
    pub x: f32,
    pub y: f32,
    pub rotasi: f32, // radian
    pub skala_x: f32,
    pub skala_y: f32,
}

impl Transformasi2D {
    pub const fn baru(x: f32, y: f32) -> Self {
        Self {
            x,
            y,
            rotasi: 0.0,
            skala_x: 1.0,
            skala_y: 1.0,
        }
    }
}

/// Komponen kecepatan linier dan sudut.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Kecepatan2D {
    pub vx: f32,
    pub vy: f32,
    pub v_rotasi: f32,
}

impl Kecepatan2D {
    pub const fn baru(vx: f32, vy: f32) -> Self {
        Self {
            vx,
            vy,
            v_rotasi: 0.0,
        }
    }

    pub const fn diam() -> Self {
        Self {
            vx: 0.0,
            vy: 0.0,
            v_rotasi: 0.0,
        }
    }
}

/// Karakteristik fisik dinamika badan kaku (Rigid Body).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct BadanKaku2D {
    pub massa: f32,
    pub skala_gravitasi: f32,
    pub gesekan_udara: f32, // linear damping (0.0 s/d 1.0)
    pub restitusi: f32,     // kelentingan pantulan (0.0 s/d 1.0)
    pub apakah_statis: bool,
}

impl BadanKaku2D {
    pub fn dinamis(massa: f32) -> Self {
        Self {
            massa,
            skala_gravitasi: 1.0,
            gesekan_udara: 0.05,
            restitusi: 0.0,
            apakah_statis: false,
        }
    }

    pub fn statis() -> Self {
        Self {
            massa: 0.0,
            skala_gravitasi: 0.0,
            gesekan_udara: 0.0,
            restitusi: 0.0,
            apakah_statis: true,
        }
    }
}

/// Komponen pembatas tabrakan kotak persegi panjang sejajar sumbu (AABB).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct KotakTabrakan2D {
    pub lebar: f32,
    pub tinggi: f32,
    pub offset_x: f32,
    pub offset_y: f32,
    pub apakah_sensor: bool, // Jika true, hanya mencatat event overlap tanpa tolakan fisik
}

impl KotakTabrakan2D {
    pub fn baru(lebar: f32, tinggi: f32) -> Self {
        Self {
            lebar,
            tinggi,
            offset_x: 0.0,
            offset_y: 0.0,
            apakah_sensor: false,
        }
    }

    pub fn sensor(lebar: f32, tinggi: f32) -> Self {
        Self {
            lebar,
            tinggi,
            offset_x: 0.0,
            offset_y: 0.0,
            apakah_sensor: true,
        }
    }

    /// Menghitung batas AABB (min_x, min_y, max_x, max_y) di koordinat dunia.
    pub fn batas_dunia(&self, t: &Transformasi2D) -> (f32, f32, f32, f32) {
        let cx = t.x + self.offset_x;
        let cy = t.y + self.offset_y;
        let setengah_w = (self.lebar * t.skala_x.abs()) * 0.5;
        let setengah_h = (self.tinggi * t.skala_y.abs()) * 0.5;

        (cx - setengah_w, cy - setengah_h, cx + setengah_w, cy + setengah_h)
    }
}

/// Informasi kontak benturan antara dua objek.
#[derive(Debug, Clone, PartialEq)]
pub struct KontakTabrakan {
    pub entitas_a: Entitas,
    pub entitas_b: Entitas,
    pub normal: (f32, f32), // Arah normal dari A ke B
    pub kedalaman: f32,     // Penetrasi tumpang tindih
}

/// Memeriksa tumpang tindih antara dua kotak AABB di koordinat dunia.
pub fn uji_aabb(
    t1: &Transformasi2D,
    k1: &KotakTabrakan2D,
    t2: &Transformasi2D,
    k2: &KotakTabrakan2D,
) -> Option<KontakTabrakan> {
    let (min_x1, min_y1, max_x1, max_y1) = k1.batas_dunia(t1);
    let (min_x2, min_y2, max_x2, max_y2) = k2.batas_dunia(t2);

    let tumpang_x = (max_x1.min(max_x2)) - (min_x1.max(min_x2));
    let tumpang_y = (max_y1.min(max_y2)) - (min_y1.max(min_y2));

    if tumpang_x > 0.0 && tumpang_y > 0.0 {
        let (normal, kedalaman) = if tumpang_x < tumpang_y {
            let n_x = if t1.x < t2.x { 1.0 } else { -1.0 };
            ((n_x, 0.0), tumpang_x)
        } else {
            let n_y = if t1.y < t2.y { 1.0 } else { -1.0 };
            ((0.0, n_y), tumpang_y)
        };

        Some(KontakTabrakan {
            entitas_a: Entitas::baru(0, 0),
            entitas_b: Entitas::baru(0, 0),
            normal,
            kedalaman,
        })
    } else {
        None
    }
}

/// Menjalankan satu langkah simulasi fisika: gaya berat (gravitasi), pembaruan posisi, dan resolusi benturan.
pub fn perbarui_fisika_2d(dunia: &mut DuniaEcs, dt: f32, gravitasi_y: f32) {
    let daftar_entitas = dunia.daftar_entitas().to_vec();

    // 1. Integrasi gaya & kecepatan
    for &e in &daftar_entitas {
        let badan = dunia.ambil_komponen::<BadanKaku2D>(e).copied();
        if let Some(b) = badan {
            if b.apakah_statis {
                continue;
            }

            if let Some(kecepatan) = dunia.ambil_komponen_mut::<Kecepatan2D>(e) {
                // Terapkan percepatan gravitasi
                kecepatan.vy += gravitasi_y * b.skala_gravitasi * dt;

                // Terapkan redaman gesekan udara
                let faktor_redam = (1.0 - b.gesekan_udara * dt).clamp(0.0, 1.0);
                kecepatan.vx *= faktor_redam;
                kecepatan.vy *= faktor_redam;
            }
        }
    }

    // 2. Pembaruan posisi (Semi-implicit Euler)
    for &e in &daftar_entitas {
        let (vx, vy) = if let Some(v) = dunia.ambil_komponen::<Kecepatan2D>(e) {
            (v.vx, v.vy)
        } else {
            continue;
        };

        if let Some(trans) = dunia.ambil_komponen_mut::<Transformasi2D>(e) {
            trans.x += vx * dt;
            trans.y += vy * dt;
        }
    }

    // 3. Resolusi tabrakan sederhana terhadap objek statis
    for i in 0..daftar_entitas.len() {
        let ea = daftar_entitas[i];
        let badan_a = match dunia.ambil_komponen::<BadanKaku2D>(ea) {
            Some(&b) if !b.apakah_statis => b,
            _ => continue,
        };

        let trans_a = match dunia.ambil_komponen::<Transformasi2D>(ea) {
            Some(&t) => t,
            _ => continue,
        };
        let kotak_a = match dunia.ambil_komponen::<KotakTabrakan2D>(ea) {
            Some(&k) if !k.apakah_sensor => k,
            _ => continue,
        };

        for j in 0..daftar_entitas.len() {
            if i == j {
                continue;
            }
            let eb = daftar_entitas[j];
            let badan_b = match dunia.ambil_komponen::<BadanKaku2D>(eb) {
                Some(&b) => b,
                _ => continue,
            };

            let trans_b = match dunia.ambil_komponen::<Transformasi2D>(eb) {
                Some(&t) => t,
                _ => continue,
            };
            let kotak_b = match dunia.ambil_komponen::<KotakTabrakan2D>(eb) {
                Some(&k) if !k.apakah_sensor => k,
                _ => continue,
            };

            // Uji benturan
            if let Some(kontak) = uji_aabb(&trans_a, &kotak_a, &trans_b, &kotak_b) {
                if badan_b.apakah_statis {
                    // Pindahkan A keluar dari B sesuai normal dan kedalaman
                    if let Some(t_a) = dunia.ambil_komponen_mut::<Transformasi2D>(ea) {
                        t_a.x -= kontak.normal.0 * kontak.kedalaman;
                        t_a.y -= kontak.normal.1 * kontak.kedalaman;
                    }

                    // Hentikan kecepatan di sumbu benturan
                    if let Some(v_a) = dunia.ambil_komponen_mut::<Kecepatan2D>(ea) {
                        if kontak.normal.0.abs() > 0.5 {
                            v_a.vx = -v_a.vx * badan_a.restitusi;
                        }
                        if kontak.normal.1.abs() > 0.5 {
                            v_a.vy = -v_a.vy * badan_a.restitusi;
                        }
                    }
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tes_deteksi_dan_resolusi_tabrakan() {
        let t1 = Transformasi2D::baru(10.0, 10.0);
        let k1 = KotakTabrakan2D::baru(20.0, 20.0);

        let t2 = Transformasi2D::baru(25.0, 10.0);
        let k2 = KotakTabrakan2D::baru(20.0, 20.0);

        let hasil = uji_aabb(&t1, &k1, &t2, &k2);
        assert!(hasil.is_some());
        let kontak = hasil.unwrap();
        assert_eq!(kontak.normal.0, 1.0);
        assert_eq!(kontak.kedalaman, 5.0); // (20 + 20)/2 - 15 = 5.0

        let mut dunia = DuniaEcs::baru();
        let bola = dunia.buat_entitas();
        dunia.pasang_komponen(bola, Transformasi2D::baru(0.0, 0.0));
        dunia.pasang_komponen(bola, Kecepatan2D::baru(0.0, 0.0));
        dunia.pasang_komponen(bola, BadanKaku2D::dinamis(1.0));
        dunia.pasang_komponen(bola, KotakTabrakan2D::baru(10.0, 10.0));

        let lantai = dunia.buat_entitas();
        dunia.pasang_komponen(lantai, Transformasi2D::baru(0.0, 20.0));
        dunia.pasang_komponen(lantai, BadanKaku2D::statis());
        dunia.pasang_komponen(lantai, KotakTabrakan2D::baru(100.0, 10.0));

        // Simulasi 10 langkah fisika dengan gravitasi ke bawah (+98.0 px/s^2)
        for _ in 0..20 {
            perbarui_fisika_2d(&mut dunia, 0.05, 98.0);
        }

        let pos_bola = dunia.ambil_komponen::<Transformasi2D>(bola).unwrap();
        // Bola tidak boleh tembus melewati lantai (lantai top y = 20 - 5 = 15; bola bottom y = pos_bola.y + 5 <= 15 -> pos_bola.y <= 10)
        assert!(pos_bola.y <= 10.1, "Bola tidak boleh tembus lantai");
    }
}
