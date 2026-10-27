# Panduan Kontribusi Komolang

<!-- Copyright (c) kangpcode (Dhafa Nazula Permadi). Licensed under MIT or Apache-2.0. -->

Terima kasih atas minat Anda untuk berkontribusi pada pengembangan **Komolang (Komodo Language)**! Komolang adalah bahasa pemrograman open source bersintaks Bahasa Indonesia yang berorientasi kinerja tinggi, keamanan memori, dan kemudahan bagi pemula.

## 1. Kode Etik
Setiap kontributor wajib membaca dan mematuhi [CODE_OF_CONDUCT.md](CODE_OF_CONDUCT.md). Kami berkomitmen menciptakan lingkungan yang ramah, inklusif, dan saling menghormati.

## 2. Cara Berkontribusi
Anda dapat berkontribusi melalui:
- **Pelaporan Bug:** Laporkan perilaku yang tidak terduga atau kesalahan compiler menggunakan template issue.
- **Usulan Fitur / RFC:** Ajukan ide perbaikan sintaks atau pustaka standar melalui proses RFC (*Request for Comments*) di folder `docs/rfc/`.
- **Implementasi Kode:** Perbaiki bug atau tambahkan implementasi pada compiler (`compiler/`), runtime (`runtime/`), CLI (`cli/`), atau pustaka standar (`std/`).
- **Dokumentasi & Contoh:** Bantu memperkaya contoh program (`examples/`) dan panduan belajar (`docs/`).

## 3. Alur Kerja Git & Pull Request
1. Fork repositori dan buat branch fitur dari branch `main`:
   ```bash
   git checkout -b fitur/nama-fitur-anda
   ```
2. Pastikan setiap file sumber baru memuat header hak cipta:
   ```rust
   // Copyright (c) kangpcode (Dhafa Nazula Permadi)
   ```
3. Jalankan suite pengujian dan pemeriksaan kode sebelum membuat PR:
   ```bash
   cargo check --workspace --all-targets
   cargo test --workspace
   cargo fmt --all -- --check
   cargo clippy --workspace --all-targets -- -D warnings
   ```
4. Buat Pull Request dengan penjelasan yang jelas mengenai tujuan perubahan serta referensi nomor Issue terkait.

## 4. Filosofi Desain Komolang
Saat merancang fitur atau sintaks baru, selalu ingat prinsip utama Komolang:
- **Satu cara yang jelas** untuk melakukan satu hal.
- **Konsistensi Bahasa Indonesia** (tidak mencampur kata kunci Inggris).
- **Kinerja dan Keamanan** tanpa kompromi.
- **Pesan error yang mendidik dan solutif**.

---
*Copyright (c) kangpcode (Dhafa Nazula Permadi)*
