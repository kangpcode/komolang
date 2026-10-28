// Copyright (c) kangpcode (Dhafa Nazula Permadi). Licensed under MIT or Apache-2.0.

//! Sistem tema dan palet warna modern (Mode Gelap dan Mode Terang) untuk KomoUI.

/// Representasi warna RGBA 32-bit.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct WarnaRgba {
    pub r: u8,
    pub g: u8,
    pub b: u8,
    pub a: u8,
}

impl WarnaRgba {
    pub const fn baru(r: u8, g: u8, b: u8, a: u8) -> Self {
        Self { r, g, b, a }
    }

    pub const fn rgb(r: u8, g: u8, b: u8) -> Self {
        Self { r, g, b, a: 255 }
    }

    pub const fn transparan() -> Self {
        Self { r: 0, g: 0, b: 0, a: 0 }
    }

    pub const fn dengan_alpha(mut self, a: u8) -> Self {
        self.a = a;
        self
    }

    pub const PUTIH: Self = Self::rgb(255, 255, 255);
    pub const HITAM: Self = Self::rgb(0, 0, 0);

    /// Mengonversi nilai hex string seperti "#38bdf8" atau "38bdf8".
    pub fn dari_hex(hex: &str) -> Option<Self> {
        let hex = hex.trim_start_matches('#');
        if hex.len() == 6 {
            let r = u8::from_str_radix(&hex[0..2], 16).ok()?;
            let g = u8::from_str_radix(&hex[2..4], 16).ok()?;
            let b = u8::from_str_radix(&hex[4..6], 16).ok()?;
            Some(Self::rgb(r, g, b))
        } else if hex.len() == 8 {
            let r = u8::from_str_radix(&hex[0..2], 16).ok()?;
            let g = u8::from_str_radix(&hex[2..4], 16).ok()?;
            let b = u8::from_str_radix(&hex[4..6], 16).ok()?;
            let a = u8::from_str_radix(&hex[6..8], 16).ok()?;
            Some(Self::baru(r, g, b, a))
        } else {
            None
        }
    }
}

/// Definisi tema desain visual lengkap KomoUI.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TemaUi {
    pub apakah_gelap: bool,
    pub latar_belakang: WarnaRgba,
    pub permukaan: WarnaRgba,
    pub primer: WarnaRgba,
    pub primer_hover: WarnaRgba,
    pub sekunder: WarnaRgba,
    pub teks_primer: WarnaRgba,
    pub teks_sekunder: WarnaRgba,
    pub batas: WarnaRgba,
    pub aksen: WarnaRgba,
    pub sukses: WarnaRgba,
    pub bahaya: WarnaRgba,
    pub radius_sudut_bawaan: u32,
}

impl TemaUi {
    /// Tema gelap modern (Sleek Dark Mode).
    pub fn gelap() -> Self {
        Self {
            apakah_gelap: true,
            latar_belakang: WarnaRgba::rgb(15, 23, 42),   // slate-900
            permukaan: WarnaRgba::rgb(30, 41, 59),        // slate-800
            primer: WarnaRgba::rgb(14, 165, 233),         // sky-500
            primer_hover: WarnaRgba::rgb(56, 189, 248),   // sky-400
            sekunder: WarnaRgba::rgb(71, 85, 105),        // slate-600
            teks_primer: WarnaRgba::rgb(248, 250, 252),   // slate-50
            teks_sekunder: WarnaRgba::rgb(148, 163, 184), // slate-400
            batas: WarnaRgba::rgb(51, 65, 85),            // slate-700
            aksen: WarnaRgba::rgb(168, 85, 247),          // purple-500
            sukses: WarnaRgba::rgb(34, 197, 94),          // green-500
            bahaya: WarnaRgba::rgb(239, 68, 68),          // red-500
            radius_sudut_bawaan: 8,
        }
    }

    /// Tema terang modern (Clean Light Mode).
    pub fn terang() -> Self {
        Self {
            apakah_gelap: false,
            latar_belakang: WarnaRgba::rgb(248, 250, 252), // slate-50
            permukaan: WarnaRgba::rgb(255, 255, 255),      // white
            primer: WarnaRgba::rgb(2, 132, 199),           // sky-600
            primer_hover: WarnaRgba::rgb(14, 165, 233),    // sky-500
            sekunder: WarnaRgba::rgb(226, 232, 240),       // slate-200
            teks_primer: WarnaRgba::rgb(15, 23, 42),       // slate-900
            teks_sekunder: WarnaRgba::rgb(100, 116, 139),  // slate-500
            batas: WarnaRgba::rgb(203, 213, 225),          // slate-300
            aksen: WarnaRgba::rgb(147, 51, 234),           // purple-600
            sukses: WarnaRgba::rgb(22, 163, 74),           // green-600
            bahaya: WarnaRgba::rgb(220, 38, 38),           // red-600
            radius_sudut_bawaan: 8,
        }
    }
}

impl Default for TemaUi {
    fn default() -> Self {
        Self::gelap()
    }
}
