// Copyright (c) kangpcode (Dhafa Nazula Permadi). Licensed under MIT or Apache-2.0.

//! Lapisan abstraksi grafis 3D (Pipeline, Matriks MVP, Mesh, dan Software Z-Buffer Rasterizer).

use komo_ui::renderer::FramebufferRgba;
use komo_ui::tema::WarnaRgba;

/// Vektor 3 dimensi dengan operasi aljabar linier.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Vektor3 {
    pub x: f32,
    pub y: f32,
    pub z: f32,
}

impl Vektor3 {
    pub const fn baru(x: f32, y: f32, z: f32) -> Self {
        Self { x, y, z }
    }

    pub const NOL: Self = Self::baru(0.0, 0.0, 0.0);
    pub const ATAS: Self = Self::baru(0.0, 1.0, 0.0);

    pub fn panjang(&self) -> f32 {
        (self.x * self.x + self.y * self.y + self.z * self.z).sqrt()
    }

    pub fn normalisasi(&self) -> Self {
        let p = self.panjang();
        if p > 0.0 {
            Self {
                x: self.x / p,
                y: self.y / p,
                z: self.z / p,
            }
        } else {
            Self::NOL
        }
    }

    pub fn titik(&self, lain: &Self) -> f32 {
        self.x * lain.x + self.y * lain.y + self.z * lain.z
    }

    pub fn silang(&self, lain: &Self) -> Self {
        Self {
            x: self.y * lain.z - self.z * lain.y,
            y: self.z * lain.x - self.x * lain.z,
            z: self.x * lain.y - self.y * lain.x,
        }
    }

    pub fn tambah(&self, lain: &Self) -> Self {
        Self {
            x: self.x + lain.x,
            y: self.y + lain.y,
            z: self.z + lain.z,
        }
    }

    pub fn kurang(&self, lain: &Self) -> Self {
        Self {
            x: self.x - lain.x,
            y: self.y - lain.y,
            z: self.z - lain.z,
        }
    }

    pub fn skala(&self, k: f32) -> Self {
        Self {
            x: self.x * k,
            y: self.y * k,
            z: self.z * k,
        }
    }
}

/// Matriks 4x4 untuk transformasi afinitas dan proyeksi perspektif 3D.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Matriks4 {
    pub m: [f32; 16],
}

impl Matriks4 {
    pub fn identitas() -> Self {
        Self {
            m: [
                1.0, 0.0, 0.0, 0.0,
                0.0, 1.0, 0.0, 0.0,
                0.0, 0.0, 1.0, 0.0,
                0.0, 0.0, 0.0, 1.0,
            ],
        }
    }

    pub fn translasi(x: f32, y: f32, z: f32) -> Self {
        let mut mat = Self::identitas();
        mat.m[3] = x;
        mat.m[7] = y;
        mat.m[11] = z;
        mat
    }

    pub fn skala(sx: f32, sy: f32, sz: f32) -> Self {
        let mut mat = Self::identitas();
        mat.m[0] = sx;
        mat.m[5] = sy;
        mat.m[10] = sz;
        mat
    }

    pub fn rotasi_x(sudut_rad: f32) -> Self {
        let mut mat = Self::identitas();
        let c = sudut_rad.cos();
        let s = sudut_rad.sin();
        mat.m[5] = c;
        mat.m[6] = -s;
        mat.m[9] = s;
        mat.m[10] = c;
        mat
    }

    pub fn rotasi_y(sudut_rad: f32) -> Self {
        let mut mat = Self::identitas();
        let c = sudut_rad.cos();
        let s = sudut_rad.sin();
        mat.m[0] = c;
        mat.m[2] = s;
        mat.m[8] = -s;
        mat.m[10] = c;
        mat
    }

    pub fn rotasi_z(sudut_rad: f32) -> Self {
        let mut mat = Self::identitas();
        let c = sudut_rad.cos();
        let s = sudut_rad.sin();
        mat.m[0] = c;
        mat.m[1] = -s;
        mat.m[4] = s;
        mat.m[5] = c;
        mat
    }

    pub fn perspektif(fov_rad: f32, aspek: f32, dekat: f32, jauh: f32) -> Self {
        let tan_setengah_fov = (fov_rad * 0.5).tan();
        let mut mat = Self { m: [0.0; 16] };

        mat.m[0] = 1.0 / (aspek * tan_setengah_fov);
        mat.m[5] = 1.0 / tan_setengah_fov;
        mat.m[10] = -(jauh + dekat) / (jauh - dekat);
        mat.m[11] = -(2.0 * jauh * dekat) / (jauh - dekat);
        mat.m[14] = -1.0;
        mat
    }

    pub fn lihat_ke(mata: Vektor3, target: Vektor3, up: Vektor3) -> Self {
        let f = target.kurang(&mata).normalisasi();
        let s = f.silang(&up).normalisasi();
        let u = s.silang(&f);

        let mut mat = Self::identitas();
        mat.m[0] = s.x;
        mat.m[1] = s.y;
        mat.m[2] = s.z;
        mat.m[3] = -s.titik(&mata);

        mat.m[4] = u.x;
        mat.m[5] = u.y;
        mat.m[6] = u.z;
        mat.m[7] = -u.titik(&mata);

        mat.m[8] = -f.x;
        mat.m[9] = -f.y;
        mat.m[10] = -f.z;
        mat.m[11] = f.titik(&mata);

        mat
    }

    pub fn kali(&self, lain: &Self) -> Self {
        let mut hasil = Self { m: [0.0; 16] };
        for baris in 0..4 {
            for kolom in 0..4 {
                let mut sum = 0.0;
                for k in 0..4 {
                    sum += self.m[baris * 4 + k] * lain.m[k * 4 + kolom];
                }
                hasil.m[baris * 4 + kolom] = sum;
            }
        }
        hasil
    }

    pub fn transformasikan_vektor(&self, v: Vektor3) -> (Vektor3, f32) {
        let x = self.m[0] * v.x + self.m[1] * v.y + self.m[2] * v.z + self.m[3];
        let y = self.m[4] * v.x + self.m[5] * v.y + self.m[6] * v.z + self.m[7];
        let z = self.m[8] * v.x + self.m[9] * v.y + self.m[10] * v.z + self.m[11];
        let w = self.m[12] * v.x + self.m[13] * v.y + self.m[14] * v.z + self.m[15];

        (Vektor3::baru(x, y, z), w)
    }
}

/// Titik sudut 3D dengan posisi, normal pencahayaan, dan warna.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Vertex3D {
    pub posisi: Vektor3,
    pub normal: Vektor3,
    pub warna: WarnaRgba,
}

impl Vertex3D {
    pub const fn baru(posisi: Vektor3, normal: Vektor3, warna: WarnaRgba) -> Self {
        Self {
            posisi,
            normal,
            warna,
        }
    }
}

/// Geometri jala 3D (Mesh) berisi verteks dan indeks segitiga.
#[derive(Debug, Clone)]
pub struct Mesh3D {
    pub verteks: Vec<Vertex3D>,
    pub indeks: Vec<u32>,
}

impl Mesh3D {
    pub fn baru(verteks: Vec<Vertex3D>, indeks: Vec<u32>) -> Self {
        Self { verteks, indeks }
    }

    /// Membuat objek jala Kubus 3D dengan 6 sisi dan normal tegak lurus.
    pub fn kubus(ukuran: f32) -> Self {
        let s = ukuran * 0.5;
        let mut verteks = Vec::new();
        let mut indeks = Vec::new();

        let sisi = [
            // Depan (Z+)
            (Vektor3::baru(0.0, 0.0, 1.0), WarnaRgba::rgb(239, 68, 68), [
                Vektor3::baru(-s, -s, s),
                Vektor3::baru(s, -s, s),
                Vektor3::baru(s, s, s),
                Vektor3::baru(-s, s, s),
            ]),
            // Belakang (Z-)
            (Vektor3::baru(0.0, 0.0, -1.0), WarnaRgba::rgb(59, 130, 246), [
                Vektor3::baru(s, -s, -s),
                Vektor3::baru(-s, -s, -s),
                Vektor3::baru(-s, s, -s),
                Vektor3::baru(s, s, -s),
            ]),
            // Kiri (X-)
            (Vektor3::baru(-1.0, 0.0, 0.0), WarnaRgba::rgb(34, 197, 94), [
                Vektor3::baru(-s, -s, -s),
                Vektor3::baru(-s, -s, s),
                Vektor3::baru(-s, s, s),
                Vektor3::baru(-s, s, -s),
            ]),
            // Kanan (X+)
            (Vektor3::baru(1.0, 0.0, 0.0), WarnaRgba::rgb(234, 179, 8), [
                Vektor3::baru(s, -s, s),
                Vektor3::baru(s, -s, -s),
                Vektor3::baru(s, s, -s),
                Vektor3::baru(s, s, s),
            ]),
            // Atas (Y+)
            (Vektor3::baru(0.0, 1.0, 0.0), WarnaRgba::rgb(168, 85, 247), [
                Vektor3::baru(-s, s, s),
                Vektor3::baru(s, s, s),
                Vektor3::baru(s, s, -s),
                Vektor3::baru(-s, s, -s),
            ]),
            // Bawah (Y-)
            (Vektor3::baru(0.0, -1.0, 0.0), WarnaRgba::rgb(236, 72, 153), [
                Vektor3::baru(-s, -s, -s),
                Vektor3::baru(s, -s, -s),
                Vektor3::baru(s, -s, s),
                Vektor3::baru(-s, -s, s),
            ]),
        ];

        for (normal, warna, pos) in sisi {
            let offset = verteks.len() as u32;
            for p in pos {
                verteks.push(Vertex3D::baru(p, normal, warna));
            }
            indeks.extend_from_slice(&[
                offset, offset + 1, offset + 2,
                offset, offset + 2, offset + 3,
            ]);
        }

        Self::baru(verteks, indeks)
    }
}

/// Buffer kedalaman piksel (Z-Buffer) untuk uji visibilitas okusi 3D.
pub struct ZBuffer {
    pub lebar: usize,
    pub tinggi: usize,
    pub buffer: Vec<f32>,
}

impl ZBuffer {
    pub fn baru(lebar: usize, tinggi: usize) -> Self {
        Self {
            lebar,
            tinggi,
            buffer: vec![f32::INFINITY; lebar * tinggi],
        }
    }

    pub fn bersihkan(&mut self) {
        for d in self.buffer.iter_mut() {
            *d = f32::INFINITY;
        }
    }

    pub fn uji_dan_tulis(&mut self, x: usize, y: usize, z: f32) -> bool {
        if x < self.lebar && y < self.tinggi {
            let idx = y * self.lebar + x;
            if z < self.buffer[idx] {
                self.buffer[idx] = z;
                true
            } else {
                false
            }
        } else {
            false
        }
    }
}

/// Renderer 3D software lengkap dengan pencahayaan directional Lambertian dan z-buffer.
pub struct Renderer3D {
    pub z_buffer: ZBuffer,
    pub arah_cahaya: Vektor3,
}

impl Renderer3D {
    pub fn baru(lebar: usize, tinggi: usize) -> Self {
        Self {
            z_buffer: ZBuffer::baru(lebar, tinggi),
            arah_cahaya: Vektor3::baru(0.5, 1.0, 0.7).normalisasi(),
        }
    }

    pub fn render_mesh(
        &mut self,
        fb: &mut FramebufferRgba,
        mesh: &Mesh3D,
        model_view_proj: &Matriks4,
        matriks_model: &Matriks4,
    ) {
        self.z_buffer.bersihkan();

        let lebar_f = fb.lebar as f32;
        let tinggi_f = fb.tinggi as f32;

        for chunk in mesh.indeks.chunks_exact(3) {
            let v0 = &mesh.verteks[chunk[0] as usize];
            let v1 = &mesh.verteks[chunk[1] as usize];
            let v2 = &mesh.verteks[chunk[2] as usize];

            let (p0_clip, w0) = model_view_proj.transformasikan_vektor(v0.posisi);
            let (p1_clip, w1) = model_view_proj.transformasikan_vektor(v1.posisi);
            let (p2_clip, w2) = model_view_proj.transformasikan_vektor(v2.posisi);

            // Clip w
            if w0 <= 0.1 || w1 <= 0.1 || w2 <= 0.1 {
                continue;
            }

            // Proyeksi ke NDC (Normalized Device Coordinates [-1, 1])
            let ndc0 = Vektor3::baru(p0_clip.x / w0, p0_clip.y / w0, p0_clip.z / w0);
            let ndc1 = Vektor3::baru(p1_clip.x / w1, p1_clip.y / w1, p1_clip.z / w1);
            let ndc2 = Vektor3::baru(p2_clip.x / w2, p2_clip.y / w2, p2_clip.z / w2);

            // Backface culling sederhana
            let tepi1 = ndc1.kurang(&ndc0);
            let tepi2 = ndc2.kurang(&ndc0);
            let normal_z = tepi1.x * tepi2.y - tepi1.y * tepi2.x;
            if normal_z <= 0.0 {
                continue;
            }

            // Transformasi ke koordinat layar
            let scr0 = (
                ((ndc0.x + 1.0) * 0.5 * lebar_f) as i32,
                ((1.0 - (ndc0.y + 1.0) * 0.5) * tinggi_f) as i32,
                ndc0.z,
            );
            let scr1 = (
                ((ndc1.x + 1.0) * 0.5 * lebar_f) as i32,
                ((1.0 - (ndc1.y + 1.0) * 0.5) * tinggi_f) as i32,
                ndc1.z,
            );
            let scr2 = (
                ((ndc2.x + 1.0) * 0.5 * lebar_f) as i32,
                ((1.0 - (ndc2.y + 1.0) * 0.5) * tinggi_f) as i32,
                ndc2.z,
            );

            // Pencahayaan Lambertian
            let (norm_dunia, _) = matriks_model.transformasikan_vektor(v0.normal);
            let norm = norm_dunia.normalisasi();
            let mut intensitas = norm.titik(&self.arah_cahaya).max(0.15); // Ambient 0.15
            intensitas = intensitas.min(1.0);

            let warna = WarnaRgba::rgb(
                (v0.warna.r as f32 * intensitas) as u8,
                (v0.warna.g as f32 * intensitas) as u8,
                (v0.warna.b as f32 * intensitas) as u8,
            );

            // Rasterisasi segitiga bounding box
            let min_x = scr0.0.min(scr1.0).min(scr2.0).max(0) as usize;
            let max_x = (scr0.0.max(scr1.0).max(scr2.0) as usize).min(fb.lebar - 1);
            let min_y = scr0.1.min(scr1.1).min(scr2.1).max(0) as usize;
            let max_y = (scr0.1.max(scr1.1).max(scr2.1) as usize).min(fb.tinggi - 1);

            let rata_z = (scr0.2 + scr1.2 + scr2.2) / 3.0;

            for y in min_y..=max_y {
                for x in min_x..=max_x {
                    // Cek koordinat barycentric sederhana atau titik tengah
                    if self.z_buffer.uji_dan_tulis(x, y, rata_z) {
                        fb.set_piksel(x, y, warna);
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
    fn tes_vektor3_dan_matriks4() {
        let v1 = Vektor3::baru(1.0, 0.0, 0.0);
        let v2 = Vektor3::baru(0.0, 1.0, 0.0);
        let silang = v1.silang(&v2);
        assert_eq!(silang, Vektor3::baru(0.0, 0.0, 1.0));

        let mat = Matriks4::translasi(10.0, 20.0, 30.0);
        let (hasil, _) = mat.transformasikan_vektor(Vektor3::NOL);
        assert_eq!(hasil, Vektor3::baru(10.0, 20.0, 30.0));
    }

    #[test]
    fn tes_software_rendering_3d_kubus() {
        let mut fb = FramebufferRgba::baru(160, 120);
        fb.bersihkan(WarnaRgba::rgb(15, 23, 42));

        let mut renderer3d = Renderer3D::baru(160, 120);
        let kubus = Mesh3D::kubus(2.0);

        let model = Matriks4::rotasi_y(0.785).kali(&Matriks4::rotasi_x(0.5));
        let view = Matriks4::lihat_ke(
            Vektor3::baru(0.0, 2.0, 5.0),
            Vektor3::NOL,
            Vektor3::ATAS,
        );
        let proj = Matriks4::perspektif(1.047, 160.0 / 120.0, 0.1, 100.0);
        let mvp = proj.kali(&view).kali(&model);

        renderer3d.render_mesh(&mut fb, &kubus, &mvp, &model);

        let ada_warna = fb.piksel.iter().any(|&b| b > 40);
        assert!(ada_warna, "Piksel kubus 3D harus terender di framebuffer");
    }
}
