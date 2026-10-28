// Copyright (c) kangpcode (Dhafa Nazula Permadi). Licensed under MIT or Apache-2.0.

//! Software 2D Rasterizer dan Framebuffer RGBA32 untuk rendering UI lintas platform.

use std::fs;
use std::path::Path;

use crate::komponen::{ArahTataLetak, ElemenUi};
use crate::tema::{TemaUi, WarnaRgba};

/// Framebuffer citra raster 32-bit RGBA di memori.
#[derive(Debug, Clone)]
pub struct FramebufferRgba {
    pub lebar: usize,
    pub tinggi: usize,
    pub piksel: Vec<u8>, // [R, G, B, A, ...]
}

impl FramebufferRgba {
    pub fn baru(lebar: usize, tinggi: usize) -> Self {
        Self {
            lebar,
            tinggi,
            piksel: vec![0u8; lebar * tinggi * 4],
        }
    }

    /// Mengisi seluruh layar dengan warna latar tertentu.
    pub fn bersihkan(&mut self, warna: WarnaRgba) {
        for chunk in self.piksel.chunks_exact_mut(4) {
            chunk[0] = warna.r;
            chunk[1] = warna.g;
            chunk[2] = warna.b;
            chunk[3] = warna.a;
        }
    }

    /// Menggambar satu piksel dengan pencampuran transparansi (Alpha Blending).
    pub fn set_piksel(&mut self, x: usize, y: usize, warna: WarnaRgba) {
        if x >= self.lebar || y >= self.tinggi || warna.a == 0 {
            return;
        }

        let idx = (y * self.lebar + x) * 4;
        if warna.a == 255 {
            self.piksel[idx] = warna.r;
            self.piksel[idx + 1] = warna.g;
            self.piksel[idx + 2] = warna.b;
            self.piksel[idx + 3] = 255;
        } else {
            let alpha = warna.a as f32 / 255.0;
            let inv_alpha = 1.0 - alpha;

            let r_lama = self.piksel[idx] as f32;
            let g_lama = self.piksel[idx + 1] as f32;
            let b_lama = self.piksel[idx + 2] as f32;

            self.piksel[idx] = (warna.r as f32 * alpha + r_lama * inv_alpha) as u8;
            self.piksel[idx + 1] = (warna.g as f32 * alpha + g_lama * inv_alpha) as u8;
            self.piksel[idx + 2] = (warna.b as f32 * alpha + b_lama * inv_alpha) as u8;
            self.piksel[idx + 3] = 255;
        }
    }

    /// Menggambar persegi panjang berisi warna.
    pub fn gambar_persegi(
        &mut self,
        x: usize,
        y: usize,
        lebar: usize,
        tinggi: usize,
        warna: WarnaRgba,
    ) {
        let x_akhir = (x + lebar).min(self.lebar);
        let y_akhir = (y + tinggi).min(self.tinggi);

        for py in y..y_akhir {
            for px in x..x_akhir {
                self.set_piksel(px, py, warna);
            }
        }
    }

    /// Menggambar garis pembatas persegi panjang.
    pub fn gambar_garis_batas(
        &mut self,
        x: usize,
        y: usize,
        lebar: usize,
        tinggi: usize,
        tebal: usize,
        warna: WarnaRgba,
    ) {
        if tebal == 0 || lebar == 0 || tinggi == 0 {
            return;
        }
        // Garis atas dan bawah
        self.gambar_persegi(x, y, lebar, tebal, warna);
        if tinggi > tebal {
            self.gambar_persegi(x, y + tinggi - tebal, lebar, tebal, warna);
        }
        // Garis kiri dan kanan
        self.gambar_persegi(x, y, tebal, tinggi, warna);
        if lebar > tebal {
            self.gambar_persegi(x + lebar - tebal, y, tebal, tinggi, warna);
        }
    }

    /// Merasterisasi teks dengan font bitmap 5x7 bawaan.
    pub fn gambar_teks_bitmap(&mut self, mut x: usize, y: usize, teks: &str, warna: WarnaRgba) {
        for ch in teks.chars() {
            if ch == '\n' {
                continue;
            }
            gambar_karakter_bitmap(self, x, y, ch, warna);
            x += 6; // Lebar karakter 5 + 1 spasi
        }
    }

    /// Menyimpan framebuffer ke berkas format citra portabel Netpbm PPM.
    pub fn simpan_ppm(&self, jalur: &Path) -> std::io::Result<()> {
        let mut buffer = Vec::new();
        buffer.extend_from_slice(format!("P6\n{} {}\n255\n", self.lebar, self.tinggi).as_bytes());

        for chunk in self.piksel.chunks_exact(4) {
            buffer.push(chunk[0]); // R
            buffer.push(chunk[1]); // G
            buffer.push(chunk[2]); // B
        }

        fs::write(jalur, buffer)
    }
}

/// Software renderer yang memetakan pohon komponen KomoUI ke framebuffer piksel.
pub struct SoftwareRenderer {
    pub fb: FramebufferRgba,
    pub tema: TemaUi,
}

impl SoftwareRenderer {
    pub fn baru(lebar: usize, tinggi: usize, tema: TemaUi) -> Self {
        Self {
            fb: FramebufferRgba::baru(lebar, tinggi),
            tema,
        }
    }

    /// Merender seluruh pohon UI ke dalam framebuffer.
    pub fn render(&mut self, elemen: &ElemenUi) {
        self.fb.bersihkan(self.tema.latar_belakang);
        self.render_elemen(elemen, 16, 16, self.fb.lebar - 32, self.fb.tinggi - 32);
    }

    fn render_elemen(
        &mut self,
        elemen: &ElemenUi,
        x: usize,
        y: usize,
        lebar_tersedia: usize,
        _tinggi_tersedia: usize,
    ) {
        match elemen {
            ElemenUi::Wadah { gaya, anak } => {
                let p = gaya.padding as usize;
                let bg = gaya.warna_latar.unwrap_or(self.tema.permukaan);

                let tinggi_estimasi = 40 + anak.len() * 32;
                self.fb.gambar_persegi(x, y, lebar_tersedia, tinggi_estimasi, bg);

                if gaya.tebal_batas > 0.0 {
                    let bc = gaya.warna_batas.unwrap_or(self.tema.batas);
                    self.fb.gambar_garis_batas(
                        x,
                        y,
                        lebar_tersedia,
                        tinggi_estimasi,
                        gaya.tebal_batas as usize,
                        bc,
                    );
                }

                let mut cur_x = x + p;
                let mut cur_y = y + p;

                for a in anak {
                    match gaya.arah {
                        ArahTataLetak::Kolom => {
                            self.render_elemen(a, cur_x, cur_y, lebar_tersedia.saturating_sub(p * 2), 30);
                            cur_y += 32 + gaya.jarak_antar_anak as usize;
                        }
                        ArahTataLetak::Baris => {
                            self.render_elemen(a, cur_x, cur_y, 120, 30);
                            cur_x += 120 + gaya.jarak_antar_anak as usize;
                        }
                    }
                }
            }
            ElemenUi::Teks { isi, warna, .. } => {
                let c = warna.unwrap_or(self.tema.teks_primer);
                self.fb.gambar_teks_bitmap(x, y, isi, c);
            }
            ElemenUi::Tombol { label, .. } => {
                let w = (label.len() * 6 + 24).min(lebar_tersedia);
                let h = 26;
                self.fb.gambar_persegi(x, y, w, h, self.tema.primer);
                self.fb.gambar_teks_bitmap(x + 12, y + 8, label, WarnaRgba::PUTIH);
            }
            ElemenUi::Masukan { placeholder, nilai, lebar, .. } => {
                let w = (*lebar as usize).min(lebar_tersedia);
                let h = 26;
                self.fb.gambar_persegi(x, y, w, h, self.tema.latar_belakang);
                self.fb.gambar_garis_batas(x, y, w, h, 1, self.tema.batas);

                let teks_tampil = if nilai.is_empty() { placeholder } else { nilai };
                let warna = if nilai.is_empty() {
                    self.tema.teks_sekunder
                } else {
                    self.tema.teks_primer
                };
                self.fb.gambar_teks_bitmap(x + 6, y + 8, teks_tampil, warna);
            }
            ElemenUi::KotakCentang { tercentang, label, .. } => {
                let kotak_size = 14;
                self.fb.gambar_persegi(x, y, kotak_size, kotak_size, self.tema.latar_belakang);
                self.fb.gambar_garis_batas(x, y, kotak_size, kotak_size, 1, self.tema.batas);

                if *tercentang {
                    self.fb.gambar_persegi(x + 3, y + 3, kotak_size - 6, kotak_size - 6, self.tema.primer);
                }

                self.fb.gambar_teks_bitmap(x + kotak_size + 8, y + 2, label, self.tema.teks_primer);
            }
            ElemenUi::Gambar { lebar, tinggi, .. } => {
                self.fb.gambar_persegi(x, y, *lebar as usize, *tinggi as usize, self.tema.sekunder);
            }
        }
    }
}

fn gambar_karakter_bitmap(fb: &mut FramebufferRgba, x: usize, y: usize, ch: char, warna: WarnaRgba) {
    let bitmap = get_bitmap_5x7(ch);
    for row in 0..7 {
        let baris_bits = bitmap[row];
        for col in 0..5 {
            if (baris_bits & (1 << (4 - col))) != 0 {
                fb.set_piksel(x + col, y + row, warna);
            }
        }
    }
}

/// Peta font bitmap 5x7 ringkas untuk rendering teks dasar.
fn get_bitmap_5x7(ch: char) -> [u8; 7] {
    match ch {
        'A' | 'a' => [0b01110, 0b10001, 0b10001, 0b11111, 0b10001, 0b10001, 0b10001],
        'B' | 'b' => [0b11110, 0b10001, 0b10001, 0b11110, 0b10001, 0b10001, 0b11110],
        'C' | 'c' => [0b01111, 0b10000, 0b10000, 0b10000, 0b10000, 0b10000, 0b01111],
        'D' | 'd' => [0b11110, 0b10001, 0b10001, 0b10001, 0b10001, 0b10001, 0b11110],
        'E' | 'e' => [0b11111, 0b10000, 0b10000, 0b11110, 0b10000, 0b10000, 0b11111],
        'F' | 'f' => [0b11111, 0b10000, 0b10000, 0b11110, 0b10000, 0b10000, 0b10000],
        'G' | 'g' => [0b01111, 0b10000, 0b10000, 0b10111, 0b10001, 0b10001, 0b01111],
        'H' | 'h' => [0b10001, 0b10001, 0b10001, 0b11111, 0b10001, 0b10001, 0b10001],
        'I' | 'i' => [0b01110, 0b00100, 0b00100, 0b00100, 0b00100, 0b00100, 0b01110],
        'J' | 'j' => [0b00001, 0b00001, 0b00001, 0b00001, 0b10001, 0b10001, 0b01110],
        'K' | 'k' => [0b10001, 0b10010, 0b10100, 0b11000, 0b10100, 0b10010, 0b10001],
        'L' | 'l' => [0b10000, 0b10000, 0b10000, 0b10000, 0b10000, 0b10000, 0b11111],
        'M' | 'm' => [0b10001, 0b11011, 0b10101, 0b10101, 0b10001, 0b10001, 0b10001],
        'N' | 'n' => [0b10001, 0b11001, 0b10101, 0b10011, 0b10001, 0b10001, 0b10001],
        'O' | 'o' => [0b01110, 0b10001, 0b10001, 0b10001, 0b10001, 0b10001, 0b01110],
        'P' | 'p' => [0b11110, 0b10001, 0b10001, 0b11110, 0b10000, 0b10000, 0b10000],
        'Q' | 'q' => [0b01110, 0b10001, 0b10001, 0b10001, 0b10101, 0b10010, 0b01101],
        'R' | 'r' => [0b11110, 0b10001, 0b10001, 0b11110, 0b10100, 0b10010, 0b10001],
        'S' | 's' => [0b01111, 0b10000, 0b10000, 0b01110, 0b00001, 0b00001, 0b11110],
        'T' | 't' => [0b11111, 0b00100, 0b00100, 0b00100, 0b00100, 0b00100, 0b00100],
        'U' | 'u' => [0b10001, 0b10001, 0b10001, 0b10001, 0b10001, 0b10001, 0b01110],
        'V' | 'v' => [0b10001, 0b10001, 0b10001, 0b10001, 0b10001, 0b01010, 0b00100],
        'W' | 'w' => [0b10001, 0b10001, 0b10001, 0b10101, 0b10101, 0b11011, 0b10001],
        'X' | 'x' => [0b10001, 0b10001, 0b01010, 0b00100, 0b01010, 0b10001, 0b10001],
        'Y' | 'y' => [0b10001, 0b10001, 0b01010, 0b00100, 0b00100, 0b00100, 0b00100],
        'Z' | 'z' => [0b11111, 0b00001, 0b00010, 0b00100, 0b01000, 0b10000, 0b11111],
        '0' => [0b01110, 0b10011, 0b10101, 0b10101, 0b11001, 0b10001, 0b01110],
        '1' => [0b00100, 0b01100, 0b00100, 0b00100, 0b00100, 0b00100, 0b01110],
        '2' => [0b01110, 0b10001, 0b00001, 0b00110, 0b01000, 0b10000, 0b11111],
        '3' => [0b11110, 0b00001, 0b00001, 0b01110, 0b00001, 0b00001, 0b11110],
        '4' => [0b00010, 0b00110, 0b01010, 0b10010, 0b11111, 0b00010, 0b00010],
        '5' => [0b11111, 0b10000, 0b11110, 0b00001, 0b00001, 0b10001, 0b01110],
        '6' => [0b00110, 0b01000, 0b10000, 0b11110, 0b10001, 0b10001, 0b01110],
        '7' => [0b11111, 0b00001, 0b00010, 0b00100, 0b01000, 0b01000, 0b01000],
        '8' => [0b01110, 0b10001, 0b10001, 0b01110, 0b10001, 0b10001, 0b01110],
        '9' => [0b01110, 0b10001, 0b10001, 0b01111, 0b00001, 0b00010, 0b01100],
        ':' => [0b00000, 0b01100, 0b01100, 0b00000, 0b01100, 0b01100, 0b00000],
        '-' => [0b00000, 0b00000, 0b00000, 0b11111, 0b00000, 0b00000, 0b00000],
        '.' => [0b00000, 0b00000, 0b00000, 0b00000, 0b00000, 0b01100, 0b01100],
        _ => [0b00000, 0b00000, 0b00000, 0b00000, 0b00000, 0b00000, 0b00000],
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::komponen::{teks, tombol, wadah_kolom};

    #[test]
    fn tes_software_rendering_framebuffer() {
        let mut renderer = SoftwareRenderer::baru(320, 240, TemaUi::gelap());
        let ui = wadah_kolom(vec![
            teks("KOMOLANG UI"),
            tombol("KIRIM", "btn_kirim"),
        ]);

        renderer.render(&ui);

        // Pastikan bukan buffer kosong serba nol
        let non_nol = renderer.fb.piksel.iter().any(|&b| b > 0);
        assert!(non_nol, "Piksel harus terisi setelah rendering");

        let temp_dir = std::env::temp_dir().join("komo_ui_test.ppm");
        let res = renderer.fb.simpan_ppm(&temp_dir);
        assert!(res.is_ok());
        assert!(temp_dir.exists());

        let _ = fs::remove_file(temp_dir);
    }
}
