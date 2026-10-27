# Komolang (Komodo Language) 🦎

<!-- Copyright (c) kangpcode (Dhafa Nazula Permadi). Licensed under MIT or Apache-2.0. -->

**Komolang** adalah bahasa pemrograman *open source* berorientasi sistem dan berkinerja tinggi dengan **sintaksis murni Bahasa Indonesia**. Komolang dirancang untuk menghasilkan kode mesin native (*ahead-of-time*) secepat C/Rust, aman memori tanpa *garbage collection pause*, serta semudah Go dan Python untuk dipelajari pemula.

---

## 🚀 Fitur Utama & Filosofi Desain

- **Sintaksis Bahasa Indonesia Asli**: Kata kunci alami, konsisten, dan mudah dipahami tanpa percampuran bahasa asing.
- **Kinerja Native Dekat Perangkat Keras**: Dikompilasi ke kode mesin (AOT) via IR (KIR), Cranelift/LLVM, dan emitter native sendiri.
- **Keamanan Memori Tanpa GC**: Model kepemilikan (*ownership*), peminjaman (*borrowing*), dan alokasi region/arena.
- **Tanpa Null Implisit**: Menghilangkan *billion-dollar mistake* dengan tipe aljabar aman `mungkin<T>` dan `hasil<T, E>`.
- **Konkurensi Ringan Bawaan**: Dukungan task ringan (`jalankan`) dan komunikasi antartask (`saluran<T>`) berstandar CSP.
- **Diagnostik Ramah Developer**: Pesan kesalahan compiler modern dalam Bahasa Indonesia yang menunjukkan nomor baris, kolom, dan saran perbaikan.

---

## 📂 Struktur Repositori Monorepo

```text
komolang/
├── Cargo.toml                  # Konfigurasi workspace Cargo
├── COPYRIGHT                   # Hak cipta resmi kangpcode (Dhafa Nazula Permadi)
├── LICENSE-APACHE              # Lisensi Apache-2.0
├── LICENSE-MIT                 # Lisensi MIT (Dual license)
├── CONTRIBUTING.md             # Panduan kontribusi komunitas
├── CODE_OF_CONDUCT.md         # Kode etik komunitas
├── .github/workflows/ci.yml    # CI multiplatform (Linux, Windows, macOS)
├── cli/                        # Crate komo: Command-line interface toolchain
├── compiler/
│   ├── komo_ast/               # Pohon sintaks abstrak (AST)
│   ├── komo_lexer/             # Pemindai leksikal UTF-8 & token
│   └── komo_parser/            # Pengurai tata bahasa EBNF
├── runtime/
│   └── komo_runtime/           # Runtime minimal: arena allocator, panic handler
├── std/                        # Pustaka standar Komolang
├── docs/
│   ├── spesifikasi-v0.1.md     # Dokumen spesifikasi resmi bahasa v0.1
│   └── rfc/
│       └── 0001-kata-kunci...   # RFC 0001: Standarisasi kata kunci & konvensi
├── examples/                   # Contoh kode resmi Komolang
├── tests/                      # Suite pengujian integrasi & konformansi
├── web/                        # Fondasi framework komoweb (Tahap 9)
├── ui/                         # Fondasi toolkit UI KomoUI (Tahap 10)
└── game/                       # Fondasi library mesin KomoGame (Tahap 11)
```

---

## 🛠️ Membangun dan Menguji Proyek

Pastikan Anda telah memasang **Rust 1.80+**:

```bash
# Memeriksa seluruh workspace
cargo check --workspace --all-targets

# Menjalankan seluruh tes unit dan integrasi
cargo test --workspace

# Menjalankan CLI Komo
cargo run --bin komo -- versi
cargo run --bin komo -- bantuan
cargo run --bin komo -- periksa examples/halo_dunia.komo
```

---

## 📜 Lisensi & Hak Cipta

Komolang didistribusikan di bawah lisensi ganda:
- **MIT License** ([LICENSE-MIT](LICENSE-MIT))
- **Apache License, Version 2.0** ([LICENSE-APACHE](LICENSE-APACHE))

Hak Cipta © 2026 kangpcode (Dhafa Nazula Permadi). Seluruh hak cipta dilindungi undang-undang.
