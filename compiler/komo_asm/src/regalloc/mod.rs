// Copyright (c) kangpcode (Dhafa Nazula Permadi). Licensed under MIT or Apache-2.0.

//! Modul alokasi register untuk backend assembly native Komolang.

pub mod linear_scan;

pub use linear_scan::{
    alokasikan_register_linear_scan, hitung_rentang_hidup, HasilAlokasiRegister, LokasiNilai,
    RentangHidup,
};
