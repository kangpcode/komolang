// Copyright (c) kangpcode (Dhafa Nazula Permadi). Licensed under MIT or Apache-2.0.

//! Penulis format berkas biner objek mandiri untuk Linux (ELF), macOS (Mach-O), dan Windows (PE/COFF).

pub mod elf;
pub mod macho;
pub mod pe_coff;

pub use elf::{buat_objek_elf64, ArsitekturElf, SimbolElf};
pub use macho::{buat_objek_macho64, ArsitekturMachO};
pub use pe_coff::{buat_objek_pe_coff, ArsitekturCoff};
