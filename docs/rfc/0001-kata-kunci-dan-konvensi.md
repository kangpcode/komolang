# RFC 0001: Standarisasi Kata Kunci dan Konvensi Penamaan Komolang

<!-- Copyright (c) kangpcode (Dhafa Nazula Permadi). Licensed under MIT or Apache-2.0. -->

- **Status:** Diterima (Accepted)
- **Penulis:** kangpcode (Dhafa Nazula Permadi) & Tim Arsitektur Komolang
- **Tanggal:** 27 Oktober 2026
- **Kategori:** Sintaks & Leksikal

---

## 1. Motivasi & Latar Belakang
Komolang dirancang sebagai bahasa pemrograman sistem bersintaks Bahasa Indonesia yang konsisten, ekspresif, dan tidak bercampur dengan kata kunci Bahasa Inggris. Untuk memastikan keseragaman pembacaan kode, parser deterministik, dan pengalaman developer (DevEx) yang mulus, RFC ini meresmikan:
1. Daftar kata kunci baku (reserved keywords).
2. Tipe data bawaan (built-in primitive & composite types).
3. Nilai literal khusus.
4. Konvensi penamaan kode (casing and naming conventions).

---

## 2. Kata Kunci Baku (30 Kata Kunci Inti)

Kata kunci Komolang tidak boleh digunakan sebagai pengenal (identifier) variabel, fungsi, tipe, atau paket:

| Kata Kunci | Kategori | Penjelasan / Makna |
| :--- | :--- | :--- |
| `paket` | Deklarasi Modul | Mendefinisikan paket sumber (`package`) |
| `impor` | Manajemen Modul | Mengimpor paket eksternal atau standar (`import`) |
| `fungsi` | Deklarasi Subrutin | Mendeklarasikan fungsi atau metode (`fn` / `func`) |
| `kembali` | Alur Kontrol | Mengembalikan nilai dari fungsi (`return`) |
| `biar` | Variabel / Bindings | Deklarasi variabel imutabel (*immutable binding*) (`let`) |
| `var` | Variabel | Deklarasi variabel mutabel (*mutable variable*) (`var`) |
| `konst` | Konstanta | Nilai konstan pada masa kompilasi (`const`) |
| `jika` | Percabangan | Evaluasi kondisi bersyarat (`if`) |
| `lainnya` | Percabangan | Cabang alternatif dari `jika` (`else`) |
| `selagi` | Perulangan | Loop kondisional (`while`) |
| `untuk` | Perulangan | Loop iterasi koleksi atau rentang (`for`) |
| `dalam` | Operator Keanggotaan | Menentukan elemen dalam rentang/koleksi (`in`) |
| `pilih` | Percabangan Nilai | Percabangan multi-kondisi diskrit (`switch`) |
| `kasus` | Percabangan Nilai | Cabang pada blok `pilih` (`case`) |
| `cocok` | Pencocokan Pola | Pencocokan pola struktural aljabar (`match`) |
| `tipe` | Definisi Tipe | Mendeklarasikan definisi tipe data baru (`type`) |
| `struktur` | Tipe Komposit | Struktur data bidang bertipe (`struct`) |
| `antarmuka` | Polimorfisme | Kontrak perilaku dinamis/statis (`interface` / `trait`) |
| `enum` | Tipe Aljabar | Enumerasi dan varian aljabar (`enum`) |
| `jalankan` | Konkurensi | Menjalankan fungsi sebagai task ringan / green thread (`go` / `spawn`) |
| `saluran` | Konkurensi | Deklarasi tipe channel komunikasi antar-task (`channel`) |
| `tunda` | Manajemen Sumber Daya | Eksekusi ekspresi saat meninggalkan cakupan fungsi saat ini (`defer`) |
| `benar` | Literal Boolean | Nilai logika benar (`true`) |
| `salah` | Literal Boolean | Nilai logika salah (`false`) |
| `kosong` | Representasi Ketiadaan | Nilai unit / tipe kosong (`()`, `void`, `nil`) |
| `pinjam` | Model Memori | Peminjaman referensi tanpa memindah kepemilikan (`borrow` / `&`) |
| `pindah` | Model Memori | Pemindahan kepemilikan eksplisit (`move`) |
| `publik` | Visibilitas | Simbol dapat diakses dari luar paket (`pub`) |
| `pribadi` | Visibilitas | Simbol hanya dapat diakses dalam paket saat ini (`priv`) |
| `tidak_aman` | Batas Keamanan | Blok kode dengan dereferensi pointer mentah / FFI C (`unsafe`) |

---

## 3. Tipe Data Primitif & Bawaan

### 3.1 Bilangan Bulat (Integer)
- `bulat`: Bilangan bulat bertanda ukuran pointer arsitektur (`isize`, default 64-bit pada x86_64/ARM64).
- `bulat8`, `bulat16`, `bulat32`, `bulat64`: Integer bertanda dengan ukuran bit presisi.
- `tbulat`: Bilangan bulat tanpa tanda ukuran pointer (`usize`).
- `tbulat8`, `tbulat16`, `tbulat32`, `tbulat64`: Integer tanpa tanda (*unsigned*) dengan bit presisi.
- `byte`: Alias untuk `tbulat8` (8-bit unsigned integer).
- `rune`: Karakter Unicode (alias untuk `bulat32` / 4-byte UTF-32 code point).

### 3.2 Bilangan Desimal (Floating Point)
- `desimal`: Default desimal presisi ganda (`desimal64`).
- `desimal32`: Floating-point IEEE 754 presisi tunggal.
- `desimal64`: Floating-point IEEE 754 presisi ganda.

### 3.3 Teks dan Logika
- `teks`: String UTF-8 valid yang imutabel secara default, didukung pengindeksan berbasis slice.
- `bool`: Tipe boolean bernilai `benar` atau `salah`.

### 3.4 Tipe Generik Bawaan
- `larik<T>`: Array dinamis (vektor contiguous di memori).
- `peta<K, V>`: Tabel hash berkinerja tinggi.
- `mungkin<T>`: Tipe wadah pembungkus yang aman dari null (`ada(T)` atau `tidak_ada`).
- `hasil<T, E>`: Tipe wadah penanganan error eksplisit (`sukses(T)` atau `gagal(E)`).

---

## 4. Konvensi Penamaan (Naming Conventions)

Untuk menjaga kode Komolang tetap rapi, indah, dan konsisten di seluruh ekosistem:

| Elemen Sintaks | Gaya Penulisan | Contoh |
| :--- | :--- | :--- |
| **Nama Paket** | Huruf kecil (`lowercase`), tanpa garis bawah | `paket utama`, `paket matematika` |
| **Tipe / Struktur / Antarmuka / Enum** | Huruf kapital setiap kata (`PascalCase`) | `Pengguna`, `PohonBiner`, `PembacaBerkas` |
| **Varian Enum** | Huruf kapital setiap kata (`PascalCase`) | `sukses(T)`, `gagal(E)`, `ada(T)`, `tidak_ada` |
| **Fungsi / Metode** | Huruf kecil dengan garis bawah (`snake_case`) | `hitung_total()`, `baca_baris()` |
| **Variabel / Parameter** | Huruf kecil dengan garis bawah (`snake_case`) | `nama_pengguna`, `indeks_baris` |
| **Konstanta** | Huruf besar penuh dengan pemisah garis bawah (`SCREAMING_SNAKE_CASE`) | `MAX_KONEKSI`, `UKURAN_BUFFER_DEFAULT` |
| **Ekstensi File** | `.komo` | `utama.komo`, `layanan.komo` |
| **File Konfigurasi Proyek** | `komo.toml` | Konfigurasi dependensi dan metadata |
| **Lockfile Dependensi** | `komo.kunci` | Penguncian versi paket deterministik |

---

## 5. Ringkasan Evaluasi
Daftar kata kunci dan konvensi ini telah dievaluasi untuk menjamin:
- Sintaks yang intuitif bagi penutur Bahasa Indonesia.
- Ketahanan leksikal: tidak ada ambiguitas antartoken.
- Pemisahan yang tegas antara kode aman dan tidak aman (`tidak_aman`).
- Keselarasan dengan arsitektur compiler Rust modern.
