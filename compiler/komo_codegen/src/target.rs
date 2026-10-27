// Copyright (c) kangpcode (Dhafa Nazula Permadi). Licensed under MIT or Apache-2.0.

//! Konfigurasi target arsitektur dan pembuatan Cranelift ISA (Instruction Set Architecture).
//! Mendukung arsitektur x86_64 dan AArch64 (ARM64) untuk Linux, Windows, dan macOS.

use cranelift_codegen::isa::{self, TargetIsa};
use cranelift_codegen::settings::{self, Configurable, Flags};
use std::str::FromStr;
use std::sync::Arc;
use target_lexicon::{Architecture, OperatingSystem, Triple};

/// Pilihan arsitektur mesin target yang didukung oleh Komolang.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TargetArsitektur {
    X86_64,
    AArch64,
}

/// Pilihan sistem operasi target.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TargetSistemOperasi {
    Linux,
    Windows,
    MacOS,
}

/// Konfigurasi lengkap target kompilasi native.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct KonfigurasiTarget {
    pub arsitektur: TargetArsitektur,
    pub sistem_operasi: TargetSistemOperasi,
    pub debug_info: bool,
    pub optimasi: bool,
}

impl KonfigurasiTarget {
    /// Membuat konfigurasi target sesuai mesin lokal saat ini (host).
    pub fn untuk_host() -> Self {
        let triple = Triple::host();
        let arsitektur = match triple.architecture {
            Architecture::Aarch64(_) => TargetArsitektur::AArch64,
            _ => TargetArsitektur::X86_64,
        };
        let sistem_operasi = match triple.operating_system {
            OperatingSystem::Windows => TargetSistemOperasi::Windows,
            OperatingSystem::Darwin(_) | OperatingSystem::MacOSX { .. } => TargetSistemOperasi::MacOS,
            _ => TargetSistemOperasi::Linux,
        };
        Self {
            arsitektur,
            sistem_operasi,
            debug_info: true,
            optimasi: false,
        }
    }

    /// Menghasilkan string triple LLVM/TargetLexicon standar (misal `x86_64-unknown-linux-gnu`).
    pub fn sebagai_triple_str(&self) -> &'static str {
        match (self.arsitektur, self.sistem_operasi) {
            (TargetArsitektur::X86_64, TargetSistemOperasi::Linux) => "x86_64-unknown-linux-gnu",
            (TargetArsitektur::X86_64, TargetSistemOperasi::Windows) => "x86_64-pc-windows-msvc",
            (TargetArsitektur::X86_64, TargetSistemOperasi::MacOS) => "x86_64-apple-darwin",
            (TargetArsitektur::AArch64, TargetSistemOperasi::Linux) => "aarch64-unknown-linux-gnu",
            (TargetArsitektur::AArch64, TargetSistemOperasi::Windows) => "aarch64-pc-windows-msvc",
            (TargetArsitektur::AArch64, TargetSistemOperasi::MacOS) => "aarch64-apple-darwin",
        }
    }

    /// Membangun Cranelift `TargetIsa` sesuai konfigurasi target ini.
    pub fn bangun_isa(&self) -> Result<Arc<dyn TargetIsa>, String> {
        let triple_str = self.sebagai_triple_str();
        let triple = Triple::from_str(triple_str)
            .map_err(|e| format!("Gagal mem-parsing target triple '{}': {}", triple_str, e))?;

        let mut flags_builder = settings::builder();
        flags_builder
            .set("is_pic", "true")
            .map_err(|e| format!("Gagal mengatur is_pic: {}", e))?;

        if self.optimasi {
            flags_builder
                .set("opt_level", "speed")
                .map_err(|e| format!("Gagal mengatur opt_level: {}", e))?;
        } else {
            flags_builder
                .set("opt_level", "none")
                .map_err(|e| format!("Gagal mengatur opt_level: {}", e))?;
        }

        let flags = Flags::new(flags_builder);
        let isa_builder = isa::lookup(triple)
            .map_err(|e| format!("Target arsitektur '{}' tidak didukung: {}", triple_str, e))?;

        Ok(isa_builder.finish(flags).map_err(|e| format!("Gagal membuat ISA Cranelift: {}", e))?)
    }
}
