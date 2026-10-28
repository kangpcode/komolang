# Buku Panduan Resmi: "Belajar Komolang"

Selamat datang di buku resmi **"Belajar Komolang: Dari Pemula Hingga Mahir"**.
Buku ini dirancang untuk membimbing Anda memahami konsep bahasa pemrograman Komolang dari dasar hingga membangun aplikasi web berkinerja tinggi, sistem multimedia, dan komputasi native.

---

## Daftar Isi

1. [Bab 1: Pengenalan & Instalasi](bab_01_pengenalan_dan_instalasi.md)
   - Filosofi bahasa Komolang: Secepat C/Rust, Semudah Python.
   - Instalasi satu baris (`install.sh` / `install.ps1`).
   - Struktur proyek, `komo baru`, dan anatomi berkas `.komo`.
2. [Bab 2: Variabel, Tipe Data, dan Kontrol Alur](bab_02_variabel_tipe_dan_kontrol.md)
   - Deklarasi imutabel `biar` vs mutabel `var`.
   - Tipe data primitif (`bulat`, `desimal`, `bool`, `teks`).
   - Percabangan `jika-lainnya`, `pilih-kasus`, dan perulangan `untuk`, `selagi`.
3. [Bab 3: Fungsi, Peminjaman, dan Manajemen Memori](bab_03_fungsi_dan_kepemilikan.md)
   - Deklarasi fungsi dan inferensi tipe kembalian.
   - Sistem kepemilikan (*ownership*) dan peminjaman (`pinjam`, `pinjam_var`).
   - Tanpa garbage collector pause: deterministik dan hemat memori.
4. [Bab 4: Struktur Data, Metode, dan Penanganan Galat](bab_04_struktur_dan_metode.md)
   - Definisi `struktur` dan implementasi metode `impl`.
   - Enum dan pencocokan pola `cocok`.
   - Pola aman penanganan galat `Hasil<T, G>` dan `Mungkin<T>`.
5. [Bab 5: Konkurensi Modern & Saluran (Channels)](bab_05_konkurensi_dan_saluran.md)
   - Thread ringan (*green threads*) berbasis work-stealing scheduler.
   - Pengiriman pesan aman dengan `saluran`.
   - Primitif sinkronisasi `Kunci` (Mutex) dan `KunciBacaTulis` (RwLock).
6. [Bab 6: Menguasai Pustaka Standar (`std`)](bab_06_pustaka_standar_lengkap.md)
   - 23 modul bawaan: I/O berkas, jaringan, JSON/CSV/TOML, regex, kripto.
7. [Bab 7: Pengembangan Web & Layanan RESTful (`komoweb`)](bab_07_pembuatan_web_dan_api.md)
   - Arsitektur web, routing Radix Tree, middleware pipeline, dan WebSocket.
8. [Bab 8: Antarmuka Grafis & Pembuatan Game (`komoui` & `komogame`)](bab_08_antarmuka_dan_game.md)
   - Komponen reaktif UI modern, interop WebAssembly Vue 3, dan game engine 60 FPS.
