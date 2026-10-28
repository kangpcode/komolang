// Copyright (c) kangpcode (Dhafa Nazula Permadi). Licensed under MIT or Apache-2.0.

//! Scheduler task ringan berbasis Work-Stealing untuk mengeksekusi green task Komolang
//! secara asynchronous di atas thread pool berkinerja tinggi.

use std::collections::VecDeque;
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::sync::{Arc, Condvar, Mutex};
use std::thread::{self, JoinHandle};

type TaskFn = Box<dyn FnOnce() + Send + 'static>;

/// Antrean task lokal per worker thread yang mendukung operasi push/pop lokal dan steal oleh thread lain.
struct AntreanLokal {
    antrean: Mutex<VecDeque<TaskFn>>,
}

impl AntreanLokal {
    fn baru() -> Self {
        Self {
            antrean: Mutex::new(VecDeque::new()),
        }
    }

    fn dorong(&self, task: TaskFn) {
        self.antrean.lock().unwrap().push_back(task);
    }

    fn ambil_sendiri(&self) -> Option<TaskFn> {
        self.antrean.lock().unwrap().pop_back()
    }

    fn curi(&self) -> Option<TaskFn> {
        self.antrean.lock().unwrap().pop_front()
    }

    fn panjang(&self) -> usize {
        self.antrean.lock().unwrap().len()
    }
}

struct KondisiScheduler {
    berjalan: AtomicBool,
    cacah_aktif: AtomicUsize,
    antrean_global: Mutex<VecDeque<TaskFn>>,
    antrean_pekerja: Vec<Arc<AntreanLokal>>,
    sinyal_kerja: Condvar,
    kunci_sinyal: Mutex<()>,
    sinyal_selesai: Condvar,
    kunci_selesai: Mutex<()>,
}

/// Scheduler multi-core berbasis work-stealing untuk penataan task ringan Komolang.
pub struct Scheduler {
    kondisi: Arc<KondisiScheduler>,
    pekerja: Mutex<Vec<JoinHandle<()>>>,
    jumlah_pekerja: usize,
}

impl Scheduler {
    /// Membuat scheduler dengan jumlah pekerja sesuai inti CPU yang tersedia.
    pub fn bawaan() -> Self {
        let n = thread::available_parallelism()
            .map(|p| p.get())
            .unwrap_or(4);
        Self::baru(n)
    }

    /// Membuat scheduler baru dengan jumlah pekerja tertentu.
    pub fn baru(jumlah_pekerja: usize) -> Self {
        let cacah = jumlah_pekerja.max(1);
        let mut antrean_pekerja = Vec::with_capacity(cacah);
        for _ in 0..cacah {
            antrean_pekerja.push(Arc::new(AntreanLokal::baru()));
        }

        let kondisi = Arc::new(KondisiScheduler {
            berjalan: AtomicBool::new(true),
            cacah_aktif: AtomicUsize::new(0),
            antrean_global: Mutex::new(VecDeque::new()),
            antrean_pekerja,
            sinyal_kerja: Condvar::new(),
            kunci_sinyal: Mutex::new(()),
            sinyal_selesai: Condvar::new(),
            kunci_selesai: Mutex::new(()),
        });

        let mut pekerja = Vec::with_capacity(cacah);
        for id in 0..cacah {
            let k = kondisi.clone();
            let handle = thread::spawn(move || {
                worker_loop(id, k);
            });
            pekerja.push(handle);
        }

        Self {
            kondisi,
            pekerja: Mutex::new(pekerja),
            jumlah_pekerja: cacah,
        }
    }

    /// Mengirimkan fungsi komputasi ringan untuk dijalankan secara asynchronous oleh scheduler.
    pub fn jalankan<F>(&self, f: F)
    where
        F: FnOnce() + Send + 'static,
    {
        self.kondisi.cacah_aktif.fetch_add(1, Ordering::SeqCst);

        // Masukkan ke antrean global atau antrean pekerja dengan beban paling sedikit
        let mut target_idx = 0;
        let mut beban_min = usize::MAX;
        for (i, antrean) in self.kondisi.antrean_pekerja.iter().enumerate() {
            let p = antrean.panjang();
            if p < beban_min {
                beban_min = p;
                target_idx = i;
            }
        }

        self.kondisi.antrean_pekerja[target_idx].dorong(Box::new(f));
        self.kondisi.sinyal_kerja.notify_one();
    }

    /// Memblokir thread pemanggil sampai seluruh task yang terdaftar selesai diproses.
    pub fn tunggu_selesai(&self) {
        let mut lock = self.kondisi.kunci_selesai.lock().unwrap();
        while self.kondisi.cacah_aktif.load(Ordering::SeqCst) > 0 {
            lock = self.kondisi.sinyal_selesai.wait(lock).unwrap();
        }
    }

    /// Memberhentikan semua worker thread dalam scheduler secara tertib.
    pub fn hentikan(&self) {
        self.kondisi.berjalan.store(false, Ordering::SeqCst);
        self.kondisi.sinyal_kerja.notify_all();

        let mut handles = self.pekerja.lock().unwrap();
        for h in handles.drain(..) {
            let _ = h.join();
        }
    }

    /// Mengembalikan jumlah thread pekerja yang aktif dalam scheduler.
    pub fn jumlah_pekerja(&self) -> usize {
        self.jumlah_pekerja
    }
}

impl Drop for Scheduler {
    fn drop(&mut self) {
        self.hentikan();
    }
}

fn worker_loop(id: usize, kondisi: Arc<KondisiScheduler>) {
    let jumlah_worker = kondisi.antrean_pekerja.len();

    while kondisi.berjalan.load(Ordering::SeqCst) {
        // 1. Coba ambil dari antrean lokal sendiri
        let task_opsi = kondisi.antrean_pekerja[id].ambil_sendiri()
            // 2. Jika kosong, coba ambil dari antrean global
            .or_else(|| kondisi.antrean_global.lock().unwrap().pop_front())
            // 3. Jika masih kosong, coba curi dari worker lain (Work-Stealing)
            .or_else(|| {
                for offset in 1..jumlah_worker {
                    let korban_id = (id + offset) % jumlah_worker;
                    if let Some(stolen) = kondisi.antrean_pekerja[korban_id].curi() {
                        return Some(stolen);
                    }
                }
                None
            });

        if let Some(task) = task_opsi {
            // Jalankan task
            task();

            // Kurangi cacah task aktif
            let sisa = kondisi.cacah_aktif.fetch_sub(1, Ordering::SeqCst) - 1;
            if sisa == 0 {
                let _lock = kondisi.kunci_selesai.lock().unwrap();
                kondisi.sinyal_selesai.notify_all();
            }
        } else {
            // Jika tidak ada task sama sekali, tunggu notifikasi
            let lock = kondisi.kunci_sinyal.lock().unwrap();
            if !kondisi.berjalan.load(Ordering::SeqCst) {
                break;
            }
            let _ = kondisi
                .sinyal_kerja
                .wait_timeout(lock, std::time::Duration::from_millis(10));
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tes_scheduler_eksekusi_banyak_task() {
        let scheduler = Scheduler::baru(4);
        let counter = Arc::new(AtomicUsize::new(0));

        for _ in 0..1000 {
            let c = counter.clone();
            scheduler.jalankan(move || {
                c.fetch_add(1, Ordering::SeqCst);
            });
        }

        scheduler.tunggu_selesai();
        assert_eq!(counter.load(Ordering::SeqCst), 1000);
    }
}
