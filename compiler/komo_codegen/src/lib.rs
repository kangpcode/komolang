// Copyright (c) kangpcode (Dhafa Nazula Permadi). Licensed under MIT or Apache-2.0.

//! Crate `komo_codegen`: Backend Native untuk Komolang via Cranelift.
//! Menghasilkan berkas objek dan kode mesin native untuk arsitektur x86_64 dan AArch64 (ARM64),
//! mendukung linking ke executable di Linux, Windows, dan macOS, serta eksekusi JIT in-memory.

pub mod fungsi;
pub mod modul;
pub mod runtime;
pub mod target;
pub mod tipe;

pub use modul::{kompilasi_ke_objek, HasilKompilasiObjek};
pub use target::{KonfigurasiTarget, TargetArsitektur, TargetSistemOperasi};
