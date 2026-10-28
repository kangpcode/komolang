// Copyright (c) kangpcode (Dhafa Nazula Permadi). Licensed under MIT or Apache-2.0.

//! Implementasi `saluran` (channels) MPSC (Multiple-Producer, Single-Consumer)
//! untuk komunikasi antar-task ringan / thread di runtime Komolang.
//! Mendukung saluran dengan kapasitas terbatas (bounded) maupun tak terbatas (unbounded).

use std::collections::VecDeque;
use std::sync::{Arc, Condvar, Mutex};
use std::time::Duration;

/// Galat ketika mencoba mengirim data ke saluran yang telah ditutup seluruh penerimanya.
#[derive(Debug, PartialEq, Eq)]
pub struct GalatSaluran<T>(pub T);

/// Galat ketika mencoba menerima dari saluran yang kosong dan tertutup.
#[derive(Debug, PartialEq, Eq)]
pub enum GalatPenerimaan {
    SaluranTertutup,
    SaluranKosong,
    BatasWaktuHabis,
}

struct KondisiSaluran<T> {
    antrean: VecDeque<T>,
    kapasitas: Option<usize>,
    tertutup: bool,
    cacah_pengirim: usize,
}

struct BagikanSaluran<T> {
    kondisi: Mutex<KondisiSaluran<T>>,
    tersedia_untuk_baca: Condvar,
    tersedia_untuk_tulis: Condvar,
}

/// Sisi pengirim saluran. Dapat dikloning untuk mendukung multi-producer (MPSC).
pub struct KanalPengirim<T> {
    bersama: Arc<BagikanSaluran<T>>,
}

/// Sisi penerima saluran.
pub struct KanalPenerima<T> {
    bersama: Arc<BagikanSaluran<T>>,
}

/// Membuat pasangan saluran dengan kapasitas tak terbatas.
pub fn buat_saluran_tak_terbatas<T>() -> (KanalPengirim<T>, KanalPenerima<T>) {
    buat_saluran_internal(None)
}

/// Membuat pasangan saluran dengan kapasitas terbatas tertentu (bounded channel).
pub fn buat_saluran_terbatas<T>(kapasitas: usize) -> (KanalPengirim<T>, KanalPenerima<T>) {
    buat_saluran_internal(Some(kapasitas.max(1)))
}

fn buat_saluran_internal<T>(kapasitas: Option<usize>) -> (KanalPengirim<T>, KanalPenerima<T>) {
    let bersama = Arc::new(BagikanSaluran {
        kondisi: Mutex::new(KondisiSaluran {
            antrean: VecDeque::new(),
            kapasitas,
            tertutup: false,
            cacah_pengirim: 1,
        }),
        tersedia_untuk_baca: Condvar::new(),
        tersedia_untuk_tulis: Condvar::new(),
    });

    (
        KanalPengirim {
            bersama: bersama.clone(),
        },
        KanalPenerima { bersama },
    )
}

impl<T> KanalPengirim<T> {
    /// Mengirim nilai ke dalam saluran. Jika saluran terbatas dan penuh, thread akan diblokir hingga ada ruang.
    pub fn kirim(&self, nilai: T) -> Result<(), GalatSaluran<T>> {
        let mut guard = self.bersama.kondisi.lock().unwrap();

        loop {
            if guard.tertutup {
                return Err(GalatSaluran(nilai));
            }

            if let Some(kap) = guard.kapasitas {
                if guard.antrean.len() >= kap {
                    guard = self.bersama.tersedia_untuk_tulis.wait(guard).unwrap();
                    continue;
                }
            }

            guard.antrean.push_back(nilai);
            self.bersama.tersedia_untuk_baca.notify_one();
            return Ok(());
        }
    }

    /// Menutup saluran secara manual dari sisi pengirim.
    pub fn tutup(&self) {
        let mut guard = self.bersama.kondisi.lock().unwrap();
        guard.tertutup = true;
        self.bersama.tersedia_untuk_baca.notify_all();
        self.bersama.tersedia_untuk_tulis.notify_all();
    }
}

impl<T> Clone for KanalPengirim<T> {
    fn clone(&self) -> Self {
        let mut guard = self.bersama.kondisi.lock().unwrap();
        guard.cacah_pengirim += 1;
        Self {
            bersama: self.bersama.clone(),
        }
    }
}

impl<T> Drop for KanalPengirim<T> {
    fn drop(&mut self) {
        let mut guard = self.bersama.kondisi.lock().unwrap();
        guard.cacah_pengirim -= 1;
        if guard.cacah_pengirim == 0 {
            guard.tertutup = true;
            self.bersama.tersedia_untuk_baca.notify_all();
            self.bersama.tersedia_untuk_tulis.notify_all();
        }
    }
}

impl<T> KanalPenerima<T> {
    /// Menerima nilai dari saluran. Memblokir jika saluran kosong sampai data tersedia atau saluran ditutup.
    pub fn terima(&self) -> Result<T, GalatPenerimaan> {
        let mut guard = self.bersama.kondisi.lock().unwrap();

        loop {
            if let Some(item) = guard.antrean.pop_front() {
                self.bersama.tersedia_untuk_tulis.notify_one();
                return Ok(item);
            }

            if guard.tertutup {
                return Err(GalatPenerimaan::SaluranTertutup);
            }

            guard = self.bersama.tersedia_untuk_baca.wait(guard).unwrap();
        }
    }

    /// Mencoba menerima nilai tanpa memblokir thread.
    pub fn coba_terima(&self) -> Result<T, GalatPenerimaan> {
        let mut guard = self.bersama.kondisi.lock().unwrap();

        if let Some(item) = guard.antrean.pop_front() {
            self.bersama.tersedia_untuk_tulis.notify_one();
            Ok(item)
        } else if guard.tertutup {
            Err(GalatPenerimaan::SaluranTertutup)
        } else {
            Err(GalatPenerimaan::SaluranKosong)
        }
    }

    /// Menerima nilai dengan batas tenggat waktu maksimal tertentu.
    pub fn terima_dengan_tenggat(&self, batas_waktu: Duration) -> Result<T, GalatPenerimaan> {
        let mut guard = self.bersama.kondisi.lock().unwrap();

        loop {
            if let Some(item) = guard.antrean.pop_front() {
                self.bersama.tersedia_untuk_tulis.notify_one();
                return Ok(item);
            }

            if guard.tertutup {
                return Err(GalatPenerimaan::SaluranTertutup);
            }

            let (g, timeout_res) = self
                .bersama
                .tersedia_untuk_baca
                .wait_timeout(guard, batas_waktu)
                .unwrap();
            guard = g;

            if timeout_res.timed_out() {
                return if let Some(item) = guard.antrean.pop_front() {
                    self.bersama.tersedia_untuk_tulis.notify_one();
                    Ok(item)
                } else if guard.tertutup {
                    Err(GalatPenerimaan::SaluranTertutup)
                } else {
                    Err(GalatPenerimaan::BatasWaktuHabis)
                };
            }
        }
    }

    /// Mengembalikan jumlah pesan yang sedang menunggu di dalam antrean.
    pub fn jumlah_antrean(&self) -> usize {
        self.bersama.kondisi.lock().unwrap().antrean.len()
    }
}

impl<T> Drop for KanalPenerima<T> {
    fn drop(&mut self) {
        let mut guard = self.bersama.kondisi.lock().unwrap();
        guard.tertutup = true;
        self.bersama.tersedia_untuk_tulis.notify_all();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::thread;

    #[test]
    fn tes_saluran_tak_terbatas_mpsc() {
        let (tx, rx) = buat_saluran_tak_terbatas::<i32>();

        let mut handles = Vec::new();
        for i in 0..5 {
            let tx_clone = tx.clone();
            handles.push(thread::spawn(move || {
                for j in 0..10 {
                    tx_clone.kirim(i * 100 + j).unwrap();
                }
            }));
        }
        drop(tx); // Tutup pengirim utama

        for h in handles {
            h.join().unwrap();
        }

        let mut total_diterima = 0;
        while let Ok(_nilai) = rx.terima() {
            total_diterima += 1;
        }

        assert_eq!(total_diterima, 50);
        assert_eq!(rx.terima(), Err(GalatPenerimaan::SaluranTertutup));
    }

    #[test]
    fn tes_saluran_terbatas_backpressure() {
        let (tx, rx) = buat_saluran_terbatas::<i32>(2);

        tx.kirim(1).unwrap();
        tx.kirim(2).unwrap();
        assert_eq!(rx.jumlah_antrean(), 2);

        let tx_thread = thread::spawn(move || {
            tx.kirim(3).unwrap(); // Harus memblokir sampai ada ruang
        });

        thread::sleep(Duration::from_millis(50));
        assert_eq!(rx.terima().unwrap(), 1);

        tx_thread.join().unwrap();
        assert_eq!(rx.terima().unwrap(), 2);
        assert_eq!(rx.terima().unwrap(), 3);
    }
}
