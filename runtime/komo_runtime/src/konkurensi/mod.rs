// Copyright (c) kangpcode (Dhafa Nazula Permadi). Licensed under MIT or Apache-2.0.

//! Modul konkurensi runtime Komolang: Scheduler green task, work-stealing, saluran (channels), dan sinkronisasi.

pub mod saluran;
pub mod scheduler;
pub mod sinkronisasi;

pub use saluran::{
    buat_saluran_tak_terbatas, buat_saluran_terbatas, GalatPenerimaan, GalatSaluran, KanalPenerima,
    KanalPengirim,
};
pub use scheduler::Scheduler;
pub use sinkronisasi::{Kunci, Pemberitahu, PenjagaBaca, PenjagaKunci, PenjagaTulis, RwKunci};
