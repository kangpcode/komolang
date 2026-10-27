// Copyright (c) kangpcode (Dhafa Nazula Permadi). Licensed under MIT or Apache-2.0.

//! Crate `komo_asm`: Backend Assembly & Toolchain Mandiri untuk Komolang (Tahap 4B).
//! Menyediakan emitter assembly x86_64 dan ARM64 langsung dari KIR SSA,
//! linear scan register allocation, penulis berkas objek ELF64/Mach-O/COFF mandiri,
//! serta perakitan ke biner executable mandiri.

pub mod arm64;
pub mod assembler;
pub mod format;
pub mod regalloc;
pub mod x86_64;

use std::path::{Path, PathBuf};

use komo_ir::ModulIr;

pub use arm64::emit_arm64;
pub use assembler::{
    rakit_dan_tautkan_asm, rakit_ke_objek_mandiri, FormatBiner, TargetArsitekturAsm,
};
pub use format::{
    buat_objek_elf64, buat_objek_macho64, buat_objek_pe_coff, ArsitekturCoff, ArsitekturElf,
    ArsitekturMachO, SimbolElf,
};
pub use regalloc::{
    alokasikan_register_linear_scan, HasilAlokasiRegister, LokasiNilai, RentangHidup,
};
pub use x86_64::emit_x86_64;

/// Menghasilkan teks kode sumber assembly native untuk modul KIR sesuai target arsitektur.
pub fn kompilasi_ke_assembly(modul: &ModulIr, arsitektur: TargetArsitekturAsm) -> String {
    match arsitektur {
        TargetArsitekturAsm::X86_64 => emit_x86_64(modul),
        TargetArsitekturAsm::AArch64 => emit_arm64(modul),
    }
}

/// Mengompilasi modul KIR ke executable biner native menggunakan backend assembly mandiri (`--backend=asm`).
pub fn bangun_executable_asm(modul: &ModulIr, jalur_keluaran: &Path) -> Result<PathBuf, String> {
    let target_ars = if cfg!(target_arch = "aarch64") {
        TargetArsitekturAsm::AArch64
    } else {
        TargetArsitekturAsm::X86_64
    };

    let kode_asm = kompilasi_ke_assembly(modul, target_ars);
    rakit_dan_tautkan_asm(&kode_asm, &modul.nama, jalur_keluaran)
}
