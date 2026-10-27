// Copyright (c) kangpcode (Dhafa Nazula Permadi). Licensed under MIT or Apache-2.0.

//! Layanan runtime C ABI dan registrasi pustaka standar untuk Komolang native.
//! Menyediakan antarmuka FFI untuk mencetak teks, bilangan, alokasi memori, dan format teks.

use std::ffi::{c_char, CStr};

/// Mencetak string null-terminated ke stdout dengan baris baru (implementasi resmi `cetak.baris`).
#[no_mangle]
pub unsafe extern "C" fn komo_std_cetak_baris(ptr: *const c_char) {
    if ptr.is_null() {
        println!();
        return;
    }
    let c_str = CStr::from_ptr(ptr);
    if let Ok(s) = c_str.to_str() {
        println!("{}", s);
    } else {
        println!("{:?}", c_str);
    }
}

/// Mencetak string null-terminated ke stdout tanpa baris baru (`cetak.teks`).
#[no_mangle]
pub unsafe extern "C" fn komo_std_cetak_teks(ptr: *const c_char) {
    if ptr.is_null() {
        return;
    }
    let c_str = CStr::from_ptr(ptr);
    if let Ok(s) = c_str.to_str() {
        print!("{}", s);
    }
}

/// Mencetak bilangan bulat (i64) ke stdout dengan baris baru.
#[no_mangle]
pub extern "C" fn komo_std_cetak_bulat(n: i64) {
    println!("{}", n);
}

/// Mencetak bilangan desimal (f64) ke stdout dengan baris baru.
#[no_mangle]
pub extern "C" fn komo_std_cetak_desimal(d: f64) {
    if d.fract() == 0.0 {
        println!("{:.1}", d);
    } else {
        println!("{}", d);
    }
}

/// Runtime helper untuk pemformatan string dasar jika dipanggil dari IR.
#[no_mangle]
pub unsafe extern "C" fn komo_rt_format_teks(ptr: *const c_char) -> *const c_char {
    ptr
}

/// Kode sumber C untuk runtime native standalone yang dapat dikompilasi bersama biner Komolang.
pub fn kode_sumber_c_runtime() -> &'static str {
    r#"/* Copyright (c) kangpcode (Dhafa Nazula Permadi). Licensed under MIT or Apache-2.0. */
#include <stdio.h>
#include <stdlib.h>
#include <stdint.h>

void komo_std_cetak_baris(const char* pesan) {
    if (pesan) {
        puts(pesan);
    } else {
        puts("");
    }
}

void komo_std_cetak_teks(const char* pesan) {
    if (pesan) {
        fputs(pesan, stdout);
        fflush(stdout);
    }
}

void komo_std_cetak_bulat(int64_t n) {
    printf("%lld\n", (long long)n);
}

void komo_std_cetak_desimal(double d) {
    printf("%g\n", d);
}

const char* komo_rt_format_teks(const char* ptr) {
    return ptr;
}
"#
}
