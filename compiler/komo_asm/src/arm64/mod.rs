// Copyright (c) kangpcode (Dhafa Nazula Permadi). Licensed under MIT or Apache-2.0.

//! Backend emisi assembly untuk arsitektur ARM64 (AArch64).

pub mod emitter;
pub mod register;

pub use emitter::emit_arm64;
pub use register::{RegArm64, POOL_REGISTER_ALOKASI_ARM64, REG_PARAMETER_ARM64};
