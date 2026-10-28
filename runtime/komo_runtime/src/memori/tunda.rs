// Copyright (c) kangpcode (Dhafa Nazula Permadi). Licensed under MIT or Apache-2.0.

//! Pengelola pernyataan `tunda` (defer) LIFO (Last-In, First-Out).
//! Menjamin eksekusi fungsi pembersihan/penutup saat keluar dari lingkup fungsi
//! baik dalam alur normal maupun saat kepanikan (panic unwinding).

/// Tumpukan fungsi penutup yang tertunda.
pub struct TumpukanTunda {
    daftar_aksi: Vec<Box<dyn FnOnce()>>,
}

impl TumpukanTunda {
    /// Membuat tumpukan tunda baru yang kosong.
    pub fn baru() -> Self {
        Self {
            daftar_aksi: Vec::new(),
        }
    }

    /// Mendaftarkan sebuah aksi fungsi penutup ke dalam tumpukan tunda.
    pub fn tunda<F>(&mut self, aksi: F)
    where
        F: FnOnce() + 'static,
    {
        self.daftar_aksi.push(Box::new(aksi));
    }

    /// Menjalankan seluruh fungsi penutup yang terdaftar dalam urutan terbalik (LIFO).
    pub fn jalankan_semua(&mut self) {
        while let Some(aksi) = self.daftar_aksi.pop() {
            aksi();
        }
    }

    /// Mengembalikan jumlah aksi yang sedang tertunda.
    pub fn cacah(&self) -> usize {
        self.daftar_aksi.len()
    }

    /// Memeriksa apakah tidak ada aksi tertunda.
    pub fn apakah_kosong(&self) -> bool {
        self.daftar_aksi.is_empty()
    }
}

impl Default for TumpukanTunda {
    fn default() -> Self {
        Self::baru()
    }
}

impl Drop for TumpukanTunda {
    fn drop(&mut self) {
        self.jalankan_semua();
    }
}

/// Penjaga RAII lingkup untuk mengeksekusi closure tunggal saat objek keluar dari scope.
pub struct PenjagaTunda<F: FnOnce()> {
    aksi: Option<F>,
}

impl<F: FnOnce()> PenjagaTunda<F> {
    /// Membuat penjaga tunda baru.
    pub fn baru(aksi: F) -> Self {
        Self { aksi: Some(aksi) }
    }

    /// Membatalkan eksekusi aksi tunda.
    pub fn batalkan(mut self) {
        self.aksi = None;
    }
}

impl<F: FnOnce()> Drop for PenjagaTunda<F> {
    fn drop(&mut self) {
        if let Some(aksi) = self.aksi.take() {
            aksi();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicI32, Ordering};
    use std::sync::Arc;

    #[test]
    fn tes_urutan_eksekusi_lifo_tunda() {
        let urutan = Arc::new(std::sync::Mutex::new(Vec::new()));

        {
            let mut tumpukan = TumpukanTunda::baru();

            let u1 = urutan.clone();
            tumpukan.tunda(move || {
                u1.lock().unwrap().push(1);
            });

            let u2 = urutan.clone();
            tumpukan.tunda(move || {
                u2.lock().unwrap().push(2);
            });

            let u3 = urutan.clone();
            tumpukan.tunda(move || {
                u3.lock().unwrap().push(3);
            });
        } // Drop TumpukanTunda memicu jalankan_semua

        let hasil = urutan.lock().unwrap().clone();
        assert_eq!(hasil, vec![3, 2, 1], "Tunda harus dieksekusi secara LIFO");
    }

    #[test]
    fn tes_penjaga_tunda_raii() {
        let counter = Arc::new(AtomicI32::new(10));

        {
            let c = counter.clone();
            let _penjaga = PenjagaTunda::baru(move || {
                c.fetch_add(5, Ordering::SeqCst);
            });
            assert_eq!(counter.load(Ordering::SeqCst), 10);
        }

        assert_eq!(counter.load(Ordering::SeqCst), 15);
    }
}
