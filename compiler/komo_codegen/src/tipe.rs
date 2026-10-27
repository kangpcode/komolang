// Copyright (c) kangpcode (Dhafa Nazula Permadi). Licensed under MIT or Apache-2.0.

//! Konversi tipe data Komo IR (`IrTipe`) ke tipe Cranelift IR (`types::Type`).

use cranelift_codegen::ir::types;
use cranelift_codegen::ir::Type as ClType;
use komo_ir::IrTipe;

/// Mengonversi `IrTipe` ke tipe Cranelift konkret.
/// Mengembalikan `None` jika tipe adalah `IrTipe::Void`.
pub fn konversi_tipe_ke_cranelift(tipe: &IrTipe, tipe_pointer: ClType) -> Option<ClType> {
    match tipe {
        IrTipe::I1 | IrTipe::I8 => Some(types::I8),
        IrTipe::I16 => Some(types::I16),
        IrTipe::I32 | IrTipe::Char => Some(types::I32),
        IrTipe::I64 => Some(types::I64),
        IrTipe::F32 => Some(types::F32),
        IrTipe::F64 => Some(types::F64),
        IrTipe::Ptr | IrTipe::Struktur(_) | IrTipe::Larik(_, _) | IrTipe::Fungsi { .. } => {
            Some(tipe_pointer)
        }
        IrTipe::Void => None,
    }
}
