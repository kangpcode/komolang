# Panduan Kontribusi Komolang

Terima kasih atas ketertarikan Anda untuk berkontribusi pada pengembangan bahasa pemrograman **Komolang**!

---

## 1. Kode Etik (*Code of Conduct*)

Komolang menjunjung tinggi komunitas yang ramah, inklusif, dan saling menghormati. Setiap peserta diharapkan bersikap sopan, konstruktif, dan terbuka terhadap ide-ide baru.

---

## 2. Alur Kerja Pengembangan

1. **Fork Repositori**: Buat fork dari repositori resmi di GitHub.
2. **Buat Cabang Fitur**: Gunakan penamaan deskriptif:
   ```bash
   git checkout -b fitur/optimasi-kir-inlining
   ```
3. **Standar Kode & Hak Cipta**:
   - Seluruh berkas kode sumber baru wajib menyertakan lisensi hak cipta:
     `// Copyright (c) kangpcode (Dhafa Nazula Permadi). Licensed under MIT or Apache-2.0.`
   - Jangan tinggalkan `TODO` atau kode semu (*pseudo-code*).
4. **Jalankan Pengujian Lengkap**:
   Sebelum mengajukan Pull Request, pastikan seluruh suite pengujian lolos 100%:
   ```bash
   cargo test --workspace
   cargo run --bin komo -- audit .
   ```
5. **Kirimkan Pull Request (PR)**: Jelaskan secara terperinci apa yang diubah, mengapa perubahan diperlukan, dan bagaimana Anda mengujinya.
