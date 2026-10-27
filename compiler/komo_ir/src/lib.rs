// Copyright (c) kangpcode (Dhafa Nazula Permadi). Licensed under MIT or Apache-2.0.

//! Crate `komo_ir`: Representasi Perantara (Intermediate Representation - KIR) Komolang.
//! Menyediakan representasi SSA murni, lowering dari AST, verifier keabsahan IR,
//! text dumper yang mudah dibaca, serta mesin pengoptimal (constant folding, DCE, mem2reg, inlining).

pub mod blok;
pub mod cetak;
pub mod fungsi;
pub mod instruksi;
pub mod lowering;
pub mod modul;
pub mod nilai;
pub mod optimizer;
pub mod tipe;
pub mod verifier;

pub use blok::BlokDasar;
pub use cetak::{cetak_fungsi, cetak_modul};
pub use fungsi::{FungsiIr, ParameterIr};
pub use instruksi::{Instruksi, OpBiner, OpUnari, Terminator};
pub use lowering::turunkan_ke_kir;
pub use modul::{DataGlobal, ModulIr};
pub use nilai::{IdBlok, IdNilai, Konstanta};
pub use optimizer::Pengoptimal;
pub use tipe::IrTipe;
pub use verifier::{verifikasi_fungsi, verifikasi_modul, GalatVerifikasi};
