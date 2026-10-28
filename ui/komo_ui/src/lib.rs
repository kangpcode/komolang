// Copyright (c) kangpcode (Dhafa Nazula Permadi). Licensed under MIT or Apache-2.0.

//! `komo_ui` adalah toolkit UI deklaratif lintas platform dan framework interoperabilitas JS/WASM
//! untuk bahasa pemrograman Komolang.
//! Mendukung pohon komponen deklaratif (Container, Text, Button, Input, Checkbox, Canvas),
//! sistem tema visual modern (Dark/Light mode), state management reaktif (Sinyal) dengan Virtual UI diffing,
//! 2D Software Rasterizer ke framebuffer piksel RGBA32, serta jembatan integrasi WebAssembly dua arah
//! ke ekosistem web modern (Vue.js 3 Composition API & React).

pub mod interop;
pub mod komponen;
pub mod reaktif;
pub mod renderer;
pub mod tema;

pub use interop::{
    buat_binding_vue, buat_definisi_typescript, DeskripsiFungsiWasm, NilaiJs,
};
pub use komponen::{
    judul, kotak_centang, masukan, teks, tombol, wadah_baris, wadah_kolom, ArahTataLetak,
    ElemenUi, GayaLayout, Perataan,
};
pub use reaktif::{bedakan_pohon, OperasiDiff, Sinyal};
pub use renderer::{FramebufferRgba, SoftwareRenderer};
pub use tema::{TemaUi, WarnaRgba};
