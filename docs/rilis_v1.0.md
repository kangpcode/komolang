# Pengumuman Rilis Resmi Komolang v1.0.0 (General Availability)

Hari ini menandai tonggak sejarah penting dalam ekosistem bahasa pemrograman Indonesia. Kami dengan bangga merilis **Komolang v1.0.0 (Versi Produksi Stabil)**.

---

## Jaminan Stabilitas v1.0 (Stability Guarantee)

Dengan rilis v1.0.0:
1. **Jaminan Kompatibilitas Mundur**: Seluruh kode sumber Komolang yang valid di v1.0.0 dijamin dapat terus dikompilasi pada seluruh pembaruan minor v1.x di masa mendatang tanpa perubahan yang merusak (*breaking changes*).
2. **Kestabilan Pustaka Standar (`std`)**: Seluruh 23 modul pustaka standar resmi telah dibekukan spesifikasinya dan siap digunakan untuk beban kerja produksi kritis.
3. **Penyelesai Dependensi Deterministik**: File kunci `komo.kunci` menjamin hasil build identik dan *reproducible* kapan pun dan di mana pun.

---

## Ringkasan Fitur Unggulan v1.0.0

- **Kompilator Mandiri Berkinerja Tinggi**: Backend native ganda (Cranelift SSA & Assembly Emitter x86_64/ARM64 mandiri).
- **Zero GC Pause**: Model kepemilikan dan peminjaman referensi hemat memori tanpa jeda penghentian aplikasi.
- **Kakas Lengkap Bawaan**: Language Server Protocol (LSP), ekstensi VS Code, REPL interaktif, debugger semantik, package manager `komopaket`, server web `komoweb`, toolkit UI `komo_ui`, dan engine game `komo_game`.
- **Dukungan Lintas Platform Luas**: Linux, Windows, macOS (Intel & Apple Silicon), WebAssembly, dan embedded.
