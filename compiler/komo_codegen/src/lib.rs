// Copyright (c) kangpcode (Dhafa Nazula Permadi). Licensed under MIT or Apache-2.0.

//! Crate `komo_codegen`: Backend Native untuk Komolang via Cranelift.
//! Menghasilkan berkas objek dan kode mesin native untuk arsitektur x86_64 dan AArch64 (ARM64),
//! mendukung linking ke executable di Linux, Windows, dan macOS, serta eksekusi JIT in-memory.

pub mod fungsi;
pub mod jit;
pub mod linker;
pub mod modul;
pub mod runtime;
pub mod target;
pub mod tipe;

use std::path::{Path, PathBuf};

use komo_ir::ModulIr;

pub use jit::EksekutorJit;
pub use linker::{tautkan_executable, temukan_perintah_linker};
pub use modul::{kompilasi_ke_objek, HasilKompilasiObjek};
pub use target::{KonfigurasiTarget, TargetArsitektur, TargetSistemOperasi};

/// Membangun program Komolang menjadi berkas biner executable native mandiri.
pub fn bangun_executable_native(
    modul: &ModulIr,
    target: &KonfigurasiTarget,
    jalur_keluaran: &Path,
) -> Result<PathBuf, String> {
    let hasil_objek = kompilasi_ke_objek(modul, target)?;
    tautkan_executable(&hasil_objek, jalur_keluaran)
}

/// Menjalankan program Komolang secara langsung in-memory menggunakan JIT.
pub fn jalankan_jit(modul: &ModulIr) -> Result<i64, String> {
    let eksekutor = EksekutorJit::baru(modul)?;
    eksekutor.jalankan_utama()
}
