# Bab 6: Menguasai Pustaka Standar (`std`)

Komolang mengusung filosofi *"Batteries Included"*. Terdapat 23 modul pustaka standar bawaan yang siap pakai tanpa perlu mengunduh dependensi pihak ketiga.

---

## Modul-Modul Utama Pustaka Standar

1. **`std::cetak`**: Pencetakan teks konsol, formatting warna ANSI, dan inspeksi nilai.
2. **`std::teks`**: Manipulasi string UTF-8, pemotongan, huruf besar/kecil, penggantian substring.
3. **`std::larik` & `std::peta`**: Struktur data array dinamis dan hash map berkecepatan tinggi.
4. **`std::matematika`**: Fungsi trigonometri, logaritma, akar kuadrat, perpangkatan, konstanta pi dan e.
5. **`std::waktu`**: Pengukuran durasi stopwatch milidetik, tanggal ISO-8601, timer tidur.
6. **`std::acak`**: Generator bilangan acak semu (PRNG) dan UUID v4.
7. **`std::berkas` & `std::jalur`**: Pembacaan/penulisan berkas streaming, manipulasi path lintas OS.
8. **`std::os` & `std::lingkungan`**: Variabel lingkungan (`ENV`), deteksi CPU/RAM sistem, eksekusi proses.
9. **`std::jaringan` & `std::http`**: Klien TCP/UDP socket dan klien HTTP/1.1 REST dengan header kustom.
10. **`std::json`, `std::toml`, `std::yaml`, `std::csv`**: Pengurai dan serialisasi format data terstruktur.
11. **`std::regex`**: Mesin pencocokan ekspresi reguler DFA berkinerja linier O(n).
12. **`std::kripto`**: Algoritma hashing SHA-256, SHA-512, MD5, HMAC, dan enkripsi simetris.
13. **`std::kompresi`**: Kompresi data Deflate dan Gzip.
14. **`std::uji` & `std::bench`**: Runner unit testing dan mikro-benchmark frame time bawaan.
