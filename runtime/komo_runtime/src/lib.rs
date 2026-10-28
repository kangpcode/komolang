// Copyright (c) kangpcode (Dhafa Nazula Permadi). Licensed under MIT or Apache-2.0.

//! `komo_runtime` menyediakan sistem runtime esensial untuk Komolang (Tahap 5):
//! - **Manajemen Memori:** Alokator Region/Arena berantai tanpa jeda GC (`AlokatorArena`), penataan fungsi `tunda` LIFO (`TumpukanTunda`, `PenjagaTunda`).
//! - **Konkurensi & Green Tasks:** Multi-threaded Work-Stealing `Scheduler`, komunikasi antartask via `saluran` (channels MPSC), dan primitif sinkronisasi (`Kunci`, `RwKunci`, `Pemberitahu`).
//! - **Penanganan Kepanikan (Panic):** Pelacakan jejak tumpukan (stack trace) berbahasa Indonesia, deteksi overflow matematika & batas larik, serta pemulihan terisolasi (`tangkap_kepanikan`).
//! - **Dukungan FFI & Embedded:** Simbol ekspor C ABI dan dukungan feature mode `tanpa_std`.

pub mod ffi;
pub mod konkurensi;
pub mod memori;
pub mod panik;

// Ekspor publik memori
pub use memori::{AlokatorArena, PenjagaTunda, TumpukanTunda};

// Ekspor publik konkurensi
pub use konkurensi::{
    buat_saluran_tak_terbatas, buat_saluran_terbatas, GalatPenerimaan, GalatSaluran, KanalPenerima,
    KanalPengirim, Kunci, Pemberitahu, PenjagaBaca, PenjagaKunci, PenjagaTulis, RwKunci, Scheduler,
};

// Ekspor publik panik & deteksi batas
pub use panik::{
    laporkan_kepanikan, periksa_indeks_larik, periksa_overflow_kali, periksa_overflow_kurang,
    periksa_overflow_tambah, periksa_pembagian_nol_bulat, tangkap_kepanikan, BingkaiJejak,
    JejakTumpukan, KesalahanRuntime,
};

/// Status inisialisasi runtime Komolang.
pub struct KonteksRuntime {
    pub versi: &'static str,
    pub aktif: bool,
    pub scheduler: Option<Scheduler>,
}

impl KonteksRuntime {
    pub fn baru() -> Self {
        Self {
            versi: env!("CARGO_PKG_VERSION"),
            aktif: true,
            scheduler: None,
        }
    }

    /// Mengaktifkan scheduler green tasks pada konteks runtime.
    pub fn aktifkan_scheduler(&mut self) -> &Scheduler {
        if self.scheduler.is_none() {
            self.scheduler = Some(Scheduler::bawaan());
        }
        self.scheduler.as_ref().unwrap()
    }

    pub fn hentikan(&mut self) {
        if let Some(s) = self.scheduler.take() {
            s.hentikan();
        }
        self.aktif = false;
    }
}

impl Default for KonteksRuntime {
    fn default() -> Self {
        Self::baru()
    }
}

/// Fungsi pembantu kompatibilitas untuk menghasilkan string kepanikan.
pub fn tangani_kepanikan(alasan: &str, berkas: &str, baris: usize, kolom: usize) -> String {
    laporkan_kepanikan(alasan, "komo_main", berkas, baris, kolom)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tes_inisialisasi_dan_scheduler_konteks_runtime() {
        let mut rt = KonteksRuntime::baru();
        assert!(rt.aktif);
        assert!(!rt.versi.is_empty());

        let sched = rt.aktifkan_scheduler();
        assert!(sched.jumlah_pekerja() >= 1);

        rt.hentikan();
        assert!(!rt.aktif);
    }

    #[test]
    fn tes_kompatibilitas_tangani_kepanikan() {
        let pesan = tangani_kepanikan("Akses memori ilegal", "src/utama.komo", 15, 4);
        assert!(pesan.contains("KEPANIKAN RUNTIME KOMOLANG"));
        assert!(pesan.contains("Akses memori ilegal"));
        assert!(pesan.contains("src/utama.komo:15:4"));
    }
}
