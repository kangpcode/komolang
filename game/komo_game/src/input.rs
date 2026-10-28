// Copyright (c) kangpcode (Dhafa Nazula Permadi). Licensed under MIT or Apache-2.0.

//! Sistem penanganan input keyboard, mouse, dan pemetaan aksi (Action Mapping) untuk game.

use std::collections::{HashMap, HashSet};

/// Kode tombol keyboard yang didukung oleh engine KomoGame.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum KodeTombol {
    Spasi,
    Enter,
    Escape,
    Tab,
    PanahAtas,
    PanahBawah,
    PanahKiri,
    PanahKanan,
    HurufW,
    HurufA,
    HurufS,
    HurufD,
    HurufJ,
    HurufK,
    HurufL,
    HurufQ,
    HurufE,
    Angka1,
    Angka2,
    Angka3,
    Angka4,
    ShiftKiri,
    CtrlKiri,
    AltKiri,
}

/// Kode tombol mouse.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TombolMouse {
    Kiri,
    Kanan,
    Tengah,
}

/// Pengelola input game yang melacak penekanan tombol dan pergerakan pointer.
#[derive(Debug, Clone)]
pub struct ManajerInput {
    tombol_aktif: HashSet<KodeTombol>,
    tombol_baru_ditekan: HashSet<KodeTombol>,
    tombol_baru_dilepas: HashSet<KodeTombol>,

    mouse_aktif: HashSet<TombolMouse>,
    mouse_baru_ditekan: HashSet<TombolMouse>,
    mouse_baru_dilepas: HashSet<TombolMouse>,

    posisi_mouse: (f32, f32),
    delta_mouse: (f32, f32),
    scroll_delta: f32,

    peta_aksi: HashMap<String, Vec<KodeTombol>>,
}

impl ManajerInput {
    pub fn baru() -> Self {
        Self {
            tombol_aktif: HashSet::new(),
            tombol_baru_ditekan: HashSet::new(),
            tombol_baru_dilepas: HashSet::new(),
            mouse_aktif: HashSet::new(),
            mouse_baru_ditekan: HashSet::new(),
            mouse_baru_dilepas: HashSet::new(),
            posisi_mouse: (0.0, 0.0),
            delta_mouse: (0.0, 0.0),
            scroll_delta: 0.0,
            peta_aksi: HashMap::new(),
        }
    }

    /// Mendaftarkan pemetaan aksi semantik ke tombol-tombol fisik.
    pub fn daftarkan_aksi(&mut self, aksi: &str, tombol: Vec<KodeTombol>) {
        self.peta_aksi.insert(aksi.to_string(), tombol);
    }

    /// Mencatat kejadian penekanan tombol keyboard.
    pub fn tekan_tombol(&mut self, tombol: KodeTombol) {
        if !self.tombol_aktif.contains(&tombol) {
            self.tombol_baru_ditekan.insert(tombol);
        }
        self.tombol_aktif.insert(tombol);
    }

    /// Mencatat kejadian pelepasan tombol keyboard.
    pub fn lepas_tombol(&mut self, tombol: KodeTombol) {
        if self.tombol_aktif.contains(&tombol) {
            self.tombol_baru_dilepas.insert(tombol);
        }
        self.tombol_aktif.remove(&tombol);
    }

    /// Memeriksa apakah tombol sedang ditahan saat ini.
    pub fn apakah_ditekan(&self, tombol: KodeTombol) -> bool {
        self.tombol_aktif.contains(&tombol)
    }

    /// Memeriksa apakah tombol baru ditekan pada frame ini (edge-triggered).
    pub fn apakah_baru_ditekan(&self, tombol: KodeTombol) -> bool {
        self.tombol_baru_ditekan.contains(&tombol)
    }

    /// Memeriksa apakah tombol baru saja dilepas pada frame ini.
    pub fn apakah_baru_dilepas(&self, tombol: KodeTombol) -> bool {
        self.tombol_baru_dilepas.contains(&tombol)
    }

    /// Memeriksa apakah aksi semantik tertentu sedang aktif ditekan.
    pub fn apakah_aksi_aktif(&self, aksi: &str) -> bool {
        if let Some(daftar_tombol) = self.peta_aksi.get(aksi) {
            daftar_tombol.iter().any(|t| self.apakah_ditekan(*t))
        } else {
            false
        }
    }

    /// Memeriksa apakah aksi semantik baru saja ditekan pada frame ini.
    pub fn apakah_aksi_baru_ditekan(&self, aksi: &str) -> bool {
        if let Some(daftar_tombol) = self.peta_aksi.get(aksi) {
            daftar_tombol.iter().any(|t| self.apakah_baru_ditekan(*t))
        } else {
            false
        }
    }

    /// Memperbarui posisi mouse dan menghitung delta gerak.
    pub fn perbarui_mouse(&mut self, x: f32, y: f32) {
        self.delta_mouse = (x - self.posisi_mouse.0, y - self.posisi_mouse.1);
        self.posisi_mouse = (x, y);
    }

    pub fn posisi_mouse(&self) -> (f32, f32) {
        self.posisi_mouse
    }

    pub fn delta_mouse(&self) -> (f32, f32) {
        self.delta_mouse
    }

    /// Mencatat penekanan tombol mouse.
    pub fn tekan_mouse(&mut self, tombol: TombolMouse) {
        if !self.mouse_aktif.contains(&tombol) {
            self.mouse_baru_ditekan.insert(tombol);
        }
        self.mouse_aktif.insert(tombol);
    }

    /// Mencatat pelepasan tombol mouse.
    pub fn lepas_mouse(&mut self, tombol: TombolMouse) {
        if self.mouse_aktif.contains(&tombol) {
            self.mouse_baru_dilepas.insert(tombol);
        }
        self.mouse_aktif.remove(&tombol);
    }

    pub fn apakah_mouse_ditekan(&self, tombol: TombolMouse) -> bool {
        self.mouse_aktif.contains(&tombol)
    }

    /// Mengatur perputaran scroll wheel.
    pub fn set_scroll(&mut self, delta: f32) {
        self.scroll_delta = delta;
    }

    pub fn scroll_delta(&self) -> f32 {
        self.scroll_delta
    }

    /// Mengosongkan status transisi pada akhir setiap frame tick.
    pub fn akhir_frame(&mut self) {
        self.tombol_baru_ditekan.clear();
        self.tombol_baru_dilepas.clear();
        self.mouse_baru_ditekan.clear();
        self.mouse_baru_dilepas.clear();
        self.delta_mouse = (0.0, 0.0);
        self.scroll_delta = 0.0;
    }
}

impl Default for ManajerInput {
    fn default() -> Self {
        Self::baru()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tes_input_keyboard_dan_aksi() {
        let mut input = ManajerInput::baru();
        input.daftarkan_aksi("Lompat", vec![KodeTombol::Spasi, KodeTombol::HurufW]);

        assert!(!input.apakah_ditekan(KodeTombol::Spasi));
        assert!(!input.apakah_aksi_aktif("Lompat"));

        input.tekan_tombol(KodeTombol::Spasi);
        assert!(input.apakah_ditekan(KodeTombol::Spasi));
        assert!(input.apakah_baru_ditekan(KodeTombol::Spasi));
        assert!(input.apakah_aksi_aktif("Lompat"));
        assert!(input.apakah_aksi_baru_ditekan("Lompat"));

        input.akhir_frame();
        assert!(input.apakah_ditekan(KodeTombol::Spasi));
        assert!(!input.apakah_baru_ditekan(KodeTombol::Spasi)); // Sudah tidak "baru ditekan"

        input.lepas_tombol(KodeTombol::Spasi);
        assert!(input.apakah_baru_dilepas(KodeTombol::Spasi));
        assert!(!input.apakah_ditekan(KodeTombol::Spasi));
        assert!(!input.apakah_aksi_aktif("Lompat"));
    }
}
