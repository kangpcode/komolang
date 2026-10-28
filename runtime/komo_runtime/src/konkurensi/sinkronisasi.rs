// Copyright (c) kangpcode (Dhafa Nazula Permadi). Licensed under MIT or Apache-2.0.

//! Primitif sinkronisasi Komolang: `Kunci` (Mutex), `RwKunci` (RwLock), dan `Pemberitahu` (Condvar).

use std::ops::{Deref, DerefMut};
use std::sync::{Condvar, Mutex, MutexGuard, RwLock, RwLockReadGuard, RwLockWriteGuard};

/// Mutex sinkronisasi thread-safe dengan API Komolang.
pub struct Kunci<T> {
    inner: Mutex<T>,
}

impl<T> Kunci<T> {
    /// Membuat `Kunci` baru dengan nilai awal.
    pub fn baru(nilai: T) -> Self {
        Self {
            inner: Mutex::new(nilai),
        }
    }

    /// Mengunci objek dan mengembalikan penjaga akses eksklusif.
    pub fn kunci(&self) -> PenjagaKunci<'_, T> {
        PenjagaKunci {
            guard: self.inner.lock().unwrap(),
        }
    }

    /// Mencoba mengunci tanpa memblokir thread.
    pub fn coba_kunci(&self) -> Option<PenjagaKunci<'_, T>> {
        self.inner.try_lock().ok().map(|guard| PenjagaKunci { guard })
    }
}

pub struct PenjagaKunci<'a, T> {
    guard: MutexGuard<'a, T>,
}

impl<'a, T> Deref for PenjagaKunci<'a, T> {
    type Target = T;
    fn deref(&self) -> &Self::Target {
        &self.guard
    }
}

impl<'a, T> DerefMut for PenjagaKunci<'a, T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.guard
    }
}

/// Read-Write Lock untuk banyak pembaca bersamaan atau satu penulis eksklusif.
pub struct RwKunci<T> {
    inner: RwLock<T>,
}

impl<T> RwKunci<T> {
    pub fn baru(nilai: T) -> Self {
        Self {
            inner: RwLock::new(nilai),
        }
    }

    pub fn baca(&self) -> PenjagaBaca<'_, T> {
        PenjagaBaca {
            guard: self.inner.read().unwrap(),
        }
    }

    pub fn tulis(&self) -> PenjagaTulis<'_, T> {
        PenjagaTulis {
            guard: self.inner.write().unwrap(),
        }
    }
}

pub struct PenjagaBaca<'a, T> {
    guard: RwLockReadGuard<'a, T>,
}

impl<'a, T> Deref for PenjagaBaca<'a, T> {
    type Target = T;
    fn deref(&self) -> &Self::Target {
        &self.guard
    }
}

pub struct PenjagaTulis<'a, T> {
    guard: RwLockWriteGuard<'a, T>,
}

impl<'a, T> Deref for PenjagaTulis<'a, T> {
    type Target = T;
    fn deref(&self) -> &Self::Target {
        &self.guard
    }
}

impl<'a, T> DerefMut for PenjagaTulis<'a, T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.guard
    }
}

/// Pemberitahu kondisi antar thread (Condvar).
pub struct Pemberitahu {
    inner: Condvar,
}

impl Pemberitahu {
    pub fn baru() -> Self {
        Self {
            inner: Condvar::new(),
        }
    }

    pub fn beri_tahu_satu(&self) {
        self.inner.notify_one();
    }

    pub fn beri_tahu_semua(&self) {
        self.inner.notify_all();
    }
}

impl Default for Pemberitahu {
    fn default() -> Self {
        Self::baru()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tes_kunci_mutex() {
        let kunci = Kunci::baru(10);
        {
            let mut g = kunci.kunci();
            *g += 5;
        }
        assert_eq!(*kunci.kunci(), 15);
    }

    #[test]
    fn tes_rwkunci_baca_tulis() {
        let rw = RwKunci::baru("Komolang".to_string());
        {
            let b1 = rw.baca();
            let b2 = rw.baca();
            assert_eq!(*b1, "Komolang");
            assert_eq!(*b2, "Komolang");
        }
        {
            let mut t = rw.tulis();
            t.push_str(" 2026");
        }
        assert_eq!(*rw.baca(), "Komolang 2026");
    }
}
