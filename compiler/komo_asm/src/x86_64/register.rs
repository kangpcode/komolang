// Copyright (c) kangpcode (Dhafa Nazula Permadi). Licensed under MIT or Apache-2.0.

//! Konfigurasi register x86_64 dan konvensi pemanggilan System V AMD64 ABI.

use std::fmt;

/// Register fisik 64-bit pada arsitektur x86_64.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum RegX86 {
    Rax,
    Rbx,
    Rcx,
    Rdx,
    Rsi,
    Rdi,
    Rbp,
    Rsp,
    R8,
    R9,
    R10,
    R11,
    R12,
    R13,
    R14,
    R15,
}

impl RegX86 {
    /// Format nama register dalam sintaks AT&T / GNU Assembler (contoh `%rax`).
    pub fn sebagai_att(&self) -> &'static str {
        match self {
            RegX86::Rax => "%rax",
            RegX86::Rbx => "%rbx",
            RegX86::Rcx => "%rcx",
            RegX86::Rdx => "%rdx",
            RegX86::Rsi => "%rsi",
            RegX86::Rdi => "%rdi",
            RegX86::Rbp => "%rbp",
            RegX86::Rsp => "%rsp",
            RegX86::R8 => "%r8",
            RegX86::R9 => "%r9",
            RegX86::R10 => "%r10",
            RegX86::R11 => "%r11",
            RegX86::R12 => "%r12",
            RegX86::R13 => "%r13",
            RegX86::R14 => "%r14",
            RegX86::R15 => "%r15",
        }
    }

    /// Format nama sub-register 8-bit rendah (contoh `%al`, `%bl`, `%r8b`).
    pub fn sebagai_att_8bit(&self) -> &'static str {
        match self {
            RegX86::Rax => "%al",
            RegX86::Rbx => "%bl",
            RegX86::Rcx => "%cl",
            RegX86::Rdx => "%dl",
            RegX86::Rsi => "%sil",
            RegX86::Rdi => "%dil",
            RegX86::Rbp => "%bpl",
            RegX86::Rsp => "%spl",
            RegX86::R8 => "%r8b",
            RegX86::R9 => "%r9b",
            RegX86::R10 => "%r10b",
            RegX86::R11 => "%r11b",
            RegX86::R12 => "%r12b",
            RegX86::R13 => "%r13b",
            RegX86::R14 => "%r14b",
            RegX86::R15 => "%r15b",
        }
    }
}

impl fmt::Display for RegX86 {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.sebagai_att())
    }
}

/// Daftar register parameter System V AMD64 ABI berurutan: RDI, RSI, RDX, RCX, R8, R9.
pub const REG_PARAMETER_X86: [RegX86; 6] = [
    RegX86::Rdi,
    RegX86::Rsi,
    RegX86::Rdx,
    RegX86::Rcx,
    RegX86::R8,
    RegX86::R9,
];

/// Pool register yang dialokasikan oleh Linear Scan (caller-saved dan callee-saved aman).
pub const POOL_REGISTER_ALOKASI_X86: [RegX86; 8] = [
    RegX86::R10,
    RegX86::R11,
    RegX86::Rbx,
    RegX86::R12,
    RegX86::R13,
    RegX86::R14,
    RegX86::R15,
    RegX86::Rcx,
];
