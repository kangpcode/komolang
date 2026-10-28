# Panduan Migrasi: Dari C / Rust ke Komolang

Komolang dirancang untuk memberikan kinerja bare-metal setara C dan Rust, tetapi dengan kurva pembelajaran yang jauh lebih ramah dan sintaksis yang lebih ekspresif.

---

## 1. Perbandingan Fitur

| Kebutuhan | C | Rust | Komolang |
| :--- | :--- | :--- | :--- |
| **Kecepatan Komputasi** | 100% (Acuan) | 98% - 100% | **95% - 98%** |
| **Keamanan Memori** | Manual (`malloc`/`free`, rawan leak/segfault) | Ketat (Borrow checker + seumur hidup `'a`) | **Aman bawaan (`biar`/`var` + `pinjam`)** |
| **Kompleksitas Sintaks** | Rendah (tapi minim fitur modern) | Sangat Tinggi (banyak simbol & tanda baca) | **Intuitif & Bersahabat** |
| **Tooling Bawaan** | Eksternal (Make, CMake) | Lengkap (Cargo) | **Lengkap (`komo` all-in-one)** |

---

## 2. Kemudahan Komolang dibanding Rust

Di Rust, pemula sering mengalami kesulitan akibat penolakan kompilator terkait anotasi masa hidup (*lifetime annotations* `'a, 'b`) dan aturan peminjaman yang rumit. Komolang menyederhanakan mekanisme peminjaman melalui token semantik `pinjam` dan `pinjam_var` tanpa memerlukan deklarasi lifetime eksplisit pada struktur data tingkat tinggi.
