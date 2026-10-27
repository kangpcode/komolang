// Copyright (c) kangpcode (Dhafa Nazula Permadi). Licensed under MIT or Apache-2.0.

//! Backend emisi assembly untuk arsitektur x86_64.

pub mod emitter;
pub mod register;

pub use emitter::emit_x86_64;
pub use register::{RegX86, POOL_REGISTER_ALOKASI_X86, REG_PARAMETER_X86};
