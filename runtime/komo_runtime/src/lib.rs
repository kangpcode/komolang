// Copyright (c) kangpcode (Dhafa Nazula Permadi). Licensed under MIT or Apache-2.0.

//! `komo_runtime` menyediakan sistem runtime esensial untuk Komolang:
//! manajemen memori berbasis arena, penanganan kepanikan (panic), dan penataan task ringan.

/// Status inisialisasi runtime Komolang.
pub struct KonteksRuntime {
    pub versi: &'static str,
    pub aktif: bool,
}

impl KonteksRuntime {
    pub fn baru() -> Self {
        Self {
            versi: env!("CARGO_PKG_VERSION"),
            aktif: true,
        }
    }

    pub fn hentikan(&mut self) {
        self.aktif = false;
    }
}

impl Default for KonteksRuntime {
    fn default() -> Self {
        Self::baru()
    }
}

/// Alokator berbasis arena / region sederhana untuk alokasi terkelompok tanpa GC.
pub struct AlokatorArena {
    buffer: Vec<u8>,
    kapasitas: usize,
}

impl AlokatorArena {
    pub fn dengan_kapasitas(kapasitas: usize) -> Self {
        Self {
            buffer: Vec::with_capacity(kapasitas),
            kapasitas,
        }
    }

    pub fn alokasikan(&mut self, data: &[u8]) -> Result<usize, &'static str> {
        if self.buffer.len() + data.len() > self.kapasitas {
            return Err("Kapasitas alokator arena terlampaui");
        }
        let offset = self.buffer.len();
        self.buffer.extend_from_slice(data);
        Ok(offset)
    }

    pub fn reset(&mut self) {
        self.buffer.clear();
    }

    pub fn jumlah_terpakai(&self) -> usize {
        self.buffer.len()
    }
}

/// Penanganan kepanikan (panic handler) resmi Komolang dengan pesan berbahasa Indonesia.
pub fn tangani_kepanikan(alasan: &str, berkas: &str, baris: usize, kolom: usize) -> String {
    format!(
        "🚨 KEPANIKAN RUNTIME KOMOLANG:\n  Alasan: {}\n  Lokasi: {}:{}:{}\n  Catatan: Silakan laporkan perilaku ini jika merupakan bug internal.",
        alasan, berkas, baris, kolom
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tes_inisialisasi_runtime() {
        let mut rt = KonteksRuntime::baru();
        assert!(rt.aktif);
        rt.hentikan();
        assert!(!rt.aktif);
    }

    #[test]
    fn tes_alokator_arena() {
        let mut arena = AlokatorArena::dengan_kapasitas(1024);
        let offset1 = arena.alokasikan(b"Komolang").expect("Alokasi berhasil");
        assert_eq!(offset1, 0);
        assert_eq!(arena.jumlah_terpakai(), 8);

        arena.reset();
        assert_eq!(arena.jumlah_terpakai(), 0);
    }

    #[test]
    fn tes_pesan_kepanikan() {
        let pesan = tangani_kepanikan("Pembagian dengan nol", "utama.komo", 12, 5);
        assert!(pesan.contains("KEPANIKAN RUNTIME KOMOLANG"));
        assert!(pesan.contains("Pembagian dengan nol"));
        assert!(pesan.contains("utama.komo:12:5"));
    }
}
