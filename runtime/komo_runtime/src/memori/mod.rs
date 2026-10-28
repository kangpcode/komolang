// Copyright (c) kangpcode (Dhafa Nazula Permadi). Licensed under MIT or Apache-2.0.

//! Modul manajemen memori runtime Komolang (Arena, Region, dan Tunda/Defer).

pub mod arena;
pub mod tunda;

pub use arena::AlokatorArena;
pub use tunda::{PenjagaTunda, TumpukanTunda};
