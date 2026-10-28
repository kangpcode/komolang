// Copyright (c) kangpcode (Dhafa Nazula Permadi). Licensed under MIT or Apache-2.0.

//! Implementasi Language Server Protocol (LSP) resmi untuk perkakas Komolang.
//! Menyediakan layanan diagnostik langsung, pelengkapan kode (autocomplete),
//! dokumentasi hover, dan pemformatan dokumen untuk editor seperti VS Code.

pub mod server;

pub use server::{jalankan_server_lsp, proses_permintaan_lsp};
