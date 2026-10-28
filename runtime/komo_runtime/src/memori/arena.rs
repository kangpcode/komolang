// Copyright (c) kangpcode (Dhafa Nazula Permadi). Licensed under MIT or Apache-2.0.

//! Alokator berbasis Arena / Region untuk alokasi memori berkecepatan tinggi tanpa GC pause.
//! Mendukung chunking berantai dinamis, alignment alami tiap tipe data,
//! serta daur ulang buffer memori instan (`reset`).

use std::alloc::{alloc, dealloc, Layout};
use std::cell::UnsafeCell;
use std::ptr::NonNull;

/// Ukuran chunk default untuk alokator arena (64 KB).
pub const UKURAN_CHUNK_DEFAULT: usize = 64 * 1024;

/// Satu blok buffer memori konkret di dalam arena.
struct ChunkMemori {
    penunjuk: NonNull<u8>,
    kapasitas: usize,
    terpakai: usize,
    chunk_berikutnya: Option<Box<ChunkMemori>>,
}

impl ChunkMemori {
    fn baru(kapasitas: usize) -> Self {
        let layout = Layout::from_size_align(kapasitas, 16)
            .unwrap_or_else(|_| Layout::from_size_align(kapasitas, 8).unwrap());
        let ptr = unsafe { alloc(layout) };
        let penunjuk = NonNull::new(ptr).expect("Alokasi memori chunk arena gagal");

        Self {
            penunjuk,
            kapasitas,
            terpakai: 0,
            chunk_berikutnya: None,
        }
    }

    fn alokasikan(&mut self, ukuran: usize, alignment: usize) -> Option<*mut u8> {
        let alamat_dasar = self.penunjuk.as_ptr() as usize;
        let alamat_sekarang = alamat_dasar + self.terpakai;
        let sisa = alamat_sekarang % alignment;
        let padding = if sisa == 0 { 0 } else { alignment - sisa };

        let total_diperlukan = padding + ukuran;
        if self.terpakai + total_diperlukan > self.kapasitas {
            return None;
        }

        let alamat_hasil = alamat_sekarang + padding;
        self.terpakai += total_diperlukan;
        Some(alamat_hasil as *mut u8)
    }

    fn reset(&mut self) {
        self.terpakai = 0;
        if let Some(ref mut berikutnya) = self.chunk_berikutnya {
            berikutnya.reset();
        }
    }
}

impl Drop for ChunkMemori {
    fn drop(&mut self) {
        let layout = Layout::from_size_align(self.kapasitas, 16)
            .unwrap_or_else(|_| Layout::from_size_align(self.kapasitas, 8).unwrap());
        unsafe {
            dealloc(self.penunjuk.as_ptr(), layout);
        }
    }
}

/// Alokator Region / Arena Komolang dengan nol GC overhead.
pub struct AlokatorArena {
    chunk_pertama: UnsafeCell<ChunkMemori>,
    ukuran_chunk: usize,
}

unsafe impl Send for ChunkMemori {}
unsafe impl Send for AlokatorArena {}
unsafe impl Sync for AlokatorArena {}

impl AlokatorArena {
    /// Membuat arena baru dengan ukuran chunk default (64 KB).
    pub fn baru() -> Self {
        Self::dengan_kapasitas(UKURAN_CHUNK_DEFAULT)
    }

    /// Membuat arena baru dengan ukuran chunk awal tertentu.
    pub fn dengan_kapasitas(ukuran_chunk: usize) -> Self {
        let kapasitas_aman = ukuran_chunk.max(1024);
        Self {
            chunk_pertama: UnsafeCell::new(ChunkMemori::baru(kapasitas_aman)),
            ukuran_chunk: kapasitas_aman,
        }
    }

    /// Mengalokasikan blok memori mentah dengan ukuran dan alignment tertentu.
    pub fn alokasikan_mentah(&self, ukuran: usize, alignment: usize) -> *mut u8 {
        if ukuran == 0 {
            return alignment as *mut u8;
        }

        let chunk = unsafe { &mut *self.chunk_pertama.get() };

        // Cari chunk yang masih muat
        let mut saat_ini = chunk;
        loop {
            if let Some(ptr) = saat_ini.alokasikan(ukuran, alignment) {
                return ptr;
            }
            if saat_ini.chunk_berikutnya.is_none() {
                break;
            }
            saat_ini = saat_ini.chunk_berikutnya.as_mut().unwrap();
        }

        // Jika semua chunk penuh, tambahkan chunk baru
        let ukuran_baru = self.ukuran_chunk.max(ukuran * 2);
        let mut chunk_baru = Box::new(ChunkMemori::baru(ukuran_baru));
        let ptr = chunk_baru
            .alokasikan(ukuran, alignment)
            .expect("Alokasi pada chunk baru harus berhasil");

        saat_ini.chunk_berikutnya = Some(chunk_baru);
        ptr
    }

    /// Mengalokasikan nilai tipe konkret `T` ke dalam arena dan mengembalikan referensi terikat lifetime arena.
    pub fn alokasi_tipe<T>(&self, nilai: T) -> &mut T {
        let layout = Layout::new::<T>();
        let ptr = self.alokasikan_mentah(layout.size(), layout.align()) as *mut T;
        unsafe {
            ptr.write(nilai);
            &mut *ptr
        }
    }

    /// Mengalokasikan irisan (slice) nilai `&[T]` ke dalam arena.
    pub fn alokasi_larik<T: Clone>(&self, elemen: &[T]) -> &mut [T] {
        if elemen.is_empty() {
            return &mut [];
        }

        let layout = Layout::array::<T>(elemen.len()).expect("Ukuran larik meluap");
        let ptr = self.alokasikan_mentah(layout.size(), layout.align()) as *mut T;

        unsafe {
            for (i, item) in elemen.iter().enumerate() {
                ptr.add(i).write(item.clone());
            }
            std::slice::from_raw_parts_mut(ptr, elemen.len())
        }
    }

    /// Mengalokasikan string literal ke dalam arena memori.
    pub fn alokasi_str(&self, teks: &str) -> &str {
        let bita = self.alokasi_larik(teks.as_bytes());
        unsafe { std::str::from_utf8_unchecked(bita) }
    }

    /// Mengosongkan arena secara instan tanpa mendealokasi blok chunk ke sistem operasi.
    pub fn reset(&mut self) {
        unsafe {
            (*self.chunk_pertama.get()).reset();
        }
    }

    /// Mengembalikan total byte memori yang sedang terpakai di seluruh chunk.
    pub fn total_terpakai(&self) -> usize {
        let mut total = 0;
        let mut saat_ini = unsafe { &*self.chunk_pertama.get() };
        loop {
            total += saat_ini.terpakai;
            match saat_ini.chunk_berikutnya {
                Some(ref b) => saat_ini = b,
                None => break,
            }
        }
        total
    }

    /// Mengembalikan total kapasitas memori yang telah dialokasikan oleh arena.
    pub fn total_kapasitas(&self) -> usize {
        let mut total = 0;
        let mut saat_ini = unsafe { &*self.chunk_pertama.get() };
        loop {
            total += saat_ini.kapasitas;
            match saat_ini.chunk_berikutnya {
                Some(ref b) => saat_ini = b,
                None => break,
            }
        }
        total
    }

    /// Mengembalikan jumlah chunk yang aktif saat ini.
    pub fn jumlah_chunk(&self) -> usize {
        let mut count = 0;
        let mut saat_ini = unsafe { &*self.chunk_pertama.get() };
        loop {
            count += 1;
            match saat_ini.chunk_berikutnya {
                Some(ref b) => saat_ini = b,
                None => break,
            }
        }
        count
    }
}

impl Default for AlokatorArena {
    fn default() -> Self {
        Self::baru()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tes_alokasi_arena_tipe_primitif() {
        let arena = AlokatorArena::dengan_kapasitas(4096);
        let a = arena.alokasi_tipe(42i64);
        let b = arena.alokasi_tipe(100i64);
        let c = arena.alokasi_str("Halo Komolang Arena");

        assert_eq!(*a, 42);
        assert_eq!(*b, 100);
        assert_eq!(c, "Halo Komolang Arena");
        assert!(arena.total_terpakai() >= 16 + c.len());
    }

    #[test]
    fn tes_alokasi_arena_melebihi_satu_chunk() {
        let arena = AlokatorArena::dengan_kapasitas(1024);
        let mut referensi = Vec::new();

        // Alokasikan lebih dari 1024 byte untuk memicu chunk berikutnya
        for i in 0..200 {
            let r = arena.alokasi_tipe(i as i64);
            referensi.push(r);
        }

        assert!(arena.jumlah_chunk() > 1, "Harus mengembang menjadi beberapa chunk");
        for (i, r) in referensi.iter().enumerate() {
            assert_eq!(**r, i as i64);
        }
    }

    #[test]
    fn tes_reset_arena_daur_ulang() {
        let mut arena = AlokatorArena::dengan_kapasitas(1024);
        let _ = arena.alokasi_tipe(12345i64);
        assert!(arena.total_terpakai() > 0);

        arena.reset();
        assert_eq!(arena.total_terpakai(), 0);

        let baru = arena.alokasi_tipe(999i64);
        assert_eq!(*baru, 999);
    }
}
