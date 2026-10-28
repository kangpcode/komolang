// Copyright (c) kangpcode (Dhafa Nazula Permadi). Licensed under MIT or Apache-2.0.

//! Simbol C-ABI / FFI ekspor dari Komolang Runtime untuk tautan biner native dan JIT.

use std::ffi::CStr;
use std::os::raw::c_char;
use std::sync::Mutex;

use crate::memori::AlokatorArena;
use crate::panik::tangani::laporkan_kepanikan;

static ARENA_GLOBAL: Mutex<Option<AlokatorArena>> = Mutex::new(None);

/// Inisialisasi runtime Komolang dari native entry point.
#[no_mangle]
pub extern "C" fn komo_rt_inisialisasi() {
    let mut guard = ARENA_GLOBAL.lock().unwrap();
    if guard.is_none() {
        *guard = Some(AlokatorArena::baru());
    }
}

/// Alokasi memori cepat dari arena thread/global.
#[no_mangle]
pub extern "C" fn komo_rt_alokasi_arena(ukuran: usize, alignment: usize) -> *mut u8 {
    let mut guard = ARENA_GLOBAL.lock().unwrap();
    let arena = guard.get_or_insert_with(AlokatorArena::baru);
    arena.alokasikan_mentah(ukuran, alignment)
}

/// Reset arena memori global.
#[no_mangle]
pub extern "C" fn komo_rt_reset_arena() {
    let mut guard = ARENA_GLOBAL.lock().unwrap();
    if let Some(ref mut arena) = *guard {
        arena.reset();
    }
}

/// Penanganan kepanikan resmi Komolang dengan pencatatan stderr.
#[no_mangle]
pub extern "C" fn komo_rt_panik(pesan: *const c_char, berkas: *const c_char, baris: usize) -> ! {
    let pesan_str = if !pesan.is_null() {
        unsafe { CStr::from_ptr(pesan).to_string_lossy() }
    } else {
        "Kepanikan runtime fatal".into()
    };

    let berkas_str = if !berkas.is_null() {
        unsafe { CStr::from_ptr(berkas).to_string_lossy() }
    } else {
        "tidak_diketahui.komo".into()
    };

    let laporan = laporkan_kepanikan(&pesan_str, "komo_main", &berkas_str, baris, 1);
    eprintln!("{}", laporan);
    std::process::exit(101);
}

/// Pemeriksaan pembagian nol dari kode mesin.
#[no_mangle]
pub extern "C" fn komo_rt_periksa_bagi_nol(pembagi: i64, berkas: *const c_char, baris: usize) {
    if pembagi == 0 {
        komo_rt_panik(
            b"Pembagian dengan angka nol\0".as_ptr() as *const c_char,
            berkas,
            baris,
        );
    }
}
