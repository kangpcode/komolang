// Copyright (c) kangpcode (Dhafa Nazula Permadi). Licensed under MIT or Apache-2.0.

//! Modul penanganan kepanikan (panic), pelacakan jejak tumpukan bahasa Indonesia, dan deteksi overflow.

pub mod jejak;
pub mod tangani;

pub use jejak::{BingkaiJejak, JejakTumpukan};
pub use tangani::{
    laporkan_kepanikan, periksa_indeks_larik, periksa_overflow_kali, periksa_overflow_kurang,
    periksa_overflow_tambah, periksa_pembagian_nol_bulat, tangkap_kepanikan, KesalahanRuntime,
};
