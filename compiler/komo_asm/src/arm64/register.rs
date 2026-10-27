// Copyright (c) kangpcode (Dhafa Nazula Permadi). Licensed under MIT or Apache-2.0.

//! Konfigurasi register ARM64 (AArch64) dan konvensi pemanggilan AAPCS64 ABI.

use std::fmt;

/// Register fisik 64-bit pada arsitektur ARM64 / AArch64.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum RegArm64 {
    X0,
    X1,
    X2,
    X3,
    X4,
    X5,
    X6,
    X7,
    X9,
    X10,
    X11,
    X12,
    X13,
    X14,
    X15,
    X19,
    X20,
    X29, // Frame Pointer (FP)
    X30, // Link Register (LR)
    SP,  // Stack Pointer
    XZR, // Zero Register
}

impl RegArm64 {
    /// Format nama register dalam teks assembly GNU ARM64.
    pub fn sebagai_str(&self) -> &'static str {
        match self {
            RegArm64::X0 => "x0",
            RegArm64::X1 => "x1",
            RegArm64::X2 => "x2",
            RegArm64::X3 => "x3",
            RegArm64::X4 => "x4",
            RegArm64::X5 => "x5",
            RegArm64::X6 => "x6",
            RegArm64::X7 => "x7",
            RegArm64::X9 => "x9",
            RegArm64::X10 => "x10",
            RegArm64::X11 => "x11",
            RegArm64::X12 => "x12",
            RegArm64::X13 => "x13",
            RegArm64::X14 => "x14",
            RegArm64::X15 => "x15",
            RegArm64::X19 => "x19",
            RegArm64::X20 => "x20",
            RegArm64::X29 => "x29",
            RegArm64::X30 => "x30",
            RegArm64::SP => "sp",
            RegArm64::XZR => "xzr",
        }
    }
}

impl fmt::Display for RegArm64 {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.sebagai_str())
    }
}

/// Register parameter AAPCS64 ABI: X0 sampai X7.
pub const REG_PARAMETER_ARM64: [RegArm64; 8] = [
    RegArm64::X0,
    RegArm64::X1,
    RegArm64::X2,
    RegArm64::X3,
    RegArm64::X4,
    RegArm64::X5,
    RegArm64::X6,
    RegArm64::X7,
];

/// Pool register yang dialokasikan oleh linear scan pada ARM64 (caller-saved X9-X15).
pub const POOL_REGISTER_ALOKASI_ARM64: [RegArm64; 7] = [
    RegArm64::X9,
    RegArm64::X10,
    RegArm64::X11,
    RegArm64::X12,
    RegArm64::X13,
    RegArm64::X14,
    RegArm64::X15,
];
