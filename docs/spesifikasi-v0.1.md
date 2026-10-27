# Spesifikasi Bahasa Komolang (Komodo Language)
## Versi 0.1 — Draf Fondasi

<!-- Copyright (c) kangpcode (Dhafa Nazula Permadi). Licensed under MIT or Apache-2.0. -->

**Penulis:** kangpcode (Dhafa Nazula Permadi)  
**Status:** Draf Standar Arsitektur  
**Tanggal:** 27 Oktober 2026  

---

## 1. Pendahuluan

Komolang (*Komodo Language*) adalah bahasa pemrograman berorientasi sistem, berkinerja tinggi, dan aman memori dengan sintaksis ekspresif berbasis Bahasa Indonesia. Komolang dikompilasi secara *ahead-of-time* (AOT) ke kode mesin native tanpa ketergantungan pada runtime virtual machine atau garbage collector berat.

---

## 2. Struktur Leksikal

### 2.1 Pengkodean Sumber
Kode sumber Komolang dikodekan dalam format **UTF-8**. Kompiler menerima simbol karakter Unicode pada literal string dan komentar.

### 2.2 Spasi dan Karakter Pemisah
Karakter pemisah mencakup spasi (`U+0020`), tabulasi horizontal (`U+0009`), *carriage return* (`U+000D`), dan baris baru (`U+000A`). Komolang tidak mewajibkan titik koma (`;`) di akhir pernyataan; pembatas pernyataan ditentukan secara otomatis oleh pemisah baris baru (*newline*) dan penutup kurung kurawal (`}`).

### 2.3 Komentar
Komolang mendukung dua gaya komentar:
1. **Komentar Sebaris:** Dimulai dengan tanda `//` hingga akhir baris.
2. **Komentar Blok:** Dimulai dengan `/*` dan diakhiri dengan `*/`. Komentar blok dapat bersarang (*nested*).

```komolang
// Ini komentar sebaris
/* Ini komentar blok
   /* Komentar bersarang */
*/
```

### 2.4 Pengenal (Identifier)
Pengenal digunakan untuk menamai variabel, fungsi, tipe, dan paket. Pengenal dimulai dengan huruf alfabet Latin (`a-z`, `A-Z`) atau garis bawah (`_`), diikuti oleh huruf, angka (`0-9`), atau garis bawah.

### 2.5 Literal

#### Literal Bilangan Bulat (Integer)
- Desimal: `12345`, `1_000_000` (garis bawah diperbolehkan untuk keterbacaan).
- Heksadesimal: Berawalan `0x` atau `0X`, misalnya `0xFF`, `0x1A_3F`.
- Biner: Berawalan `0b` atau `0B`, misalnya `0b1010_0110`.
- Oktal: Berawalan `0o` atau `0O`, misalnya `0o755`.

#### Literal Desimal (Floating Point)
Format standar: `3.14159`, `0.5`, `1.0e-5`, `6.022e23`.

#### Literal Teks (String) & Interpolasi
Teks diapit tanda petik ganda (`"`). String mendukung pemformatan interpolasi ekspresi langsung menggunakan kurung kurawal `{}`:
```komolang
biar nama = "Budi"
biar umur = 20
biar pesan = "Halo, nama saya {nama} dan umur saya {umur + 1} tahun."
```
Untuk menampilkan karakter kurung kurawal harfiah dalam string berinterpolasi, gunakan escape ganda: `{{` atau `}}`.

Escape sequence standar yang didukung:
- `\n`: Baris baru
- `\t`: Tab
- `\r`: Carriage return
- `\\`: Garis miring terbalik
- `\"`: Tanda petik ganda
- `\0`: Karakter null
- `\u{HHHH}`: Unicode code point

#### Literal Karakter (Rune)
Karakter Unicode tunggal diapit tanda petik tunggal (`'`), misalnya `'A'`, `'k'`, `'🔥'`, `'\n'`.

---

## 3. Tata Bahasa Baku (EBNF)

Berikut adalah notasi *Extended Backus-Naur Form* (EBNF) resmi Komolang v0.1:

```ebnf
Program             ::= DeklarasiPaket { DeklarasiImpor } { DeklarasiTingkatAtas } ;

DeklarasiPaket      ::= "paket" Pengenal ;
DeklarasiImpor      ::= "impor" ( LiteralTeks [ "sebagai" Pengenal ] | "(" { LiteralTeks [ "sebagai" Pengenal ] } ")" ) ;

DeklarasiTingkatAtas::= [ "publik" ] ( DeklarasiFungsi | DeklarasiTipe | DeklarasiKonstanta | DeklarasiVariabelGlobal ) ;

DeklarasiFungsi     ::= "fungsi" [ ReceiverMetode ] Pengenal [ ParameterGenerik ] "(" [ DaftarParameter ] ")" [ TipeData ] BlokKode ;
ReceiverMetode      ::= "(" Pengenal ":" TipeData ")" ;
DaftarParameter     ::= Parameter { "," Parameter } ;
Parameter           ::= Pengenal ":" [ "pinjam" [ "var" ] ] TipeData ;

DeklarasiKonstanta  ::= "konst" Pengenal [ ":" TipeData ] "=" Ekspresi ;
DeklarasiVariabel   ::= ( "biar" | "var" ) Pengenal [ ":" TipeData ] "=" Ekspresi ;

DeklarasiTipe       ::= "tipe" Pengenal [ ParameterGenerik ] ( DefinisiStruktur | DefinisiAntarmuka | DefinisiEnum | TipeData ) ;
DefinisiStruktur    ::= "struktur" "{" { BidangStruktur } "}" ;
BidangStruktur      ::= [ "publik" ] Pengenal ":" TipeData ;

DefinisiAntarmuka   ::= "antarmuka" "{" { TandaTanganMetode } "}" ;
TandaTanganMetode   ::= Pengenal "(" [ DaftarParameter ] ")" [ TipeData ] ;

DefinisiEnum        ::= "enum" "{" { VarianEnum } "}" ;
VarianEnum          ::= Pengenal [ "(" { TipeData } ")" ] ;

BlokKode            ::= "{" { Pernyataan } "}" ;

Pernyataan          ::= DeklarasiVariabel
                      | PernyataanTugas
                      | PernyataanJika
                      | PernyataanSelagi
                      | PernyataanUntuk
                      | PernyataanPilih
                      | PernyataanCocok
                      | PernyataanTunda
                      | PernyataanJalankan
                      | PernyataanKembali
                      | Ekspresi ;

PernyataanTugas     ::= Ekspresi ( "=" | "+=" | "-=" | "*=" | "/=" | "%=" ) Ekspresi ;
PernyataanJika      ::= "jika" Ekspresi BlokKode [ "lainnya" ( PernyataanJika | BlokKode ) ] ;
PernyataanSelagi    ::= "selagi" Ekspresi BlokKode ;
PernyataanUntuk     ::= "untuk" Pengenal "dalam" Ekspresi BlokKode ;

PernyataanPilih     ::= "pilih" Ekspresi "{" { CabangKasus } [ CabangLainnya ] "}" ;
CabangKasus         ::= "kasus" Ekspresi ":" { Pernyataan } ;
CabangLainnya       ::= "lainnya" ":" { Pernyataan } ;

PernyataanCocok     ::= "cocok" Ekspresi "{" { CabangPola } "}" ;
CabangPola          ::= Pola ":" ( Pernyataan | BlokKode ) ;

Pola                ::= Pengenal [ "(" { Pengenal } ")" ] | "_" | Literal ;

PernyataanTunda     ::= "tunda" ( PemanggilanFungsi | BlokKode ) ;
PernyataanJalankan  ::= "jalankan" PemanggilanFungsi ;
PernyataanKembali   ::= "kembali" [ Ekspresi ] ;

TipeData            ::= TipeDasar
                      | TipeGenerik
                      | TipeFungsi
                      | TipeArray
                      | "pinjam" [ "var" ] TipeData ;

TipeDasar           ::= "bulat" | "bulat8" | "bulat16" | "bulat32" | "bulat64"
                      | "tbulat" | "tbulat8" | "tbulat16" | "tbulat32" | "tbulat64"
                      | "desimal" | "desimal32" | "desimal64"
                      | "teks" | "bool" | "byte" | "rune" | Pengenal ;

TipeGenerik         ::= Pengenal "<" TipeData { "," TipeData } ">" ;

Ekspresi            ::= EkspresiLogika ;
EkspresiLogika      ::= EkspresiRelasi { ( "dan" | "atau" ) EkspresiRelasi } ;
EkspresiRelasi      ::= EkspresiBiner { ( "==" | "!=" | "<" | "<=" | ">" | ">=" ) EkspresiBiner } ;
EkspresiBiner       ::= EkspresiRentang ;
EkspresiRentang     ::= EkspresiAditif [ ".." [ "=" ] EkspresiAditif ] ;
EkspresiAditif      ::= EkspresiMultiplikatif { ( "+" | "-" ) EkspresiMultiplikatif } ;
EkspresiMultiplikatif::= EkspresiUnari { ( "*" | "/" | "%" ) EkspresiUnari } ;
EkspresiUnari       ::= ( "-" | "!" | "pinjam" [ "var" ] | "pindah" ) EkspresiUnari | EkspresiPostfiks ;
EkspresiPostfiks    ::= EkspresiPrimer { PemanggilanFungsi | AksesBidang | Pengindeksan } ;

PemanggilanFungsi   ::= "(" [ DaftarArgumen ] ")" ;
AksesBidang         ::= "." Pengenal ;
Pengindeksan        ::= "[" Ekspresi "]" ;
DaftarArgumen       ::= Ekspresi { "," Ekspresi } ;

EkspresiPrimer      ::= Pengenal
                      | Literal
                      | "(" Ekspresi ")"
                      | InisialisasiStruktur
                      | LiteralLarik ;

InisialisasiStruktur::= Pengenal "{" [ DaftarBidangInisialisasi ] "}" ;
DaftarBidangInisialisasi ::= BidangInisialisasi { "," BidangInisialisasi } ;
BidangInisialisasi  ::= Pengenal ":" Ekspresi ;
LiteralLarik        ::= "[" [ DaftarArgumen ] "]" ;
```

---

## 4. Sistem Tipe

### 4.1 Tipe Primitif
1. **Integer Bertanda:** `bulat` (arsitektur target, 64-bit pada modern OS), `bulat8`, `bulat16`, `bulat32`, `bulat64`.
2. **Integer Tanpa Tanda:** `tbulat` (arsitektur target), `tbulat8`, `tbulat16`, `tbulat32`, `tbulat64`, `byte` (alias `tbulat8`), `rune` (UTF-32).
3. **Pecahan:** `desimal` (alias `desimal64`), `desimal32`, `desimal64`.
4. **Logika & Teks:** `bool` (`benar`, `salah`), `teks` (string UTF-8 aman memori).

### 4.2 Tipe Tanpa Null Implisit
Komolang **tidak memiliki pointer null implisit**. Ketiadaan nilai diwakili oleh tipe generik aljabar:

```komolang
tipe mungkin<T> enum {
    ada(T)
    tidak_ada
}

tipe hasil<T, E> enum {
    sukses(T)
    gagal(E)
}
```

Pengecekan nilai harus ditangani secara eksplisit melalui pencocokan pola `cocok`:

```komolang
cocok bagi(10, 2) {
    sukses(nilai): cetak.baris("Hasil: {nilai}")
    gagal(alasan): cetak.baris("Gagal membagi: {alasan}")
}
```

### 4.3 Generik & Monomorphization
Fungsi dan tipe dapat memiliki parameter generik:
```komolang
fungsi identitas<T>(nilai: T) T {
    kembali nilai
}
```
Kompiler menggunakan teknik **monomorphization** penuh: setiap penggunaan generik dengan tipe konkret akan menghasilkan kode mesin terspesialisasi tanpa overhead vtable runtime (*zero-cost abstraction*).

---

## 5. Model Memori & Kepemilikan

Komolang mencapai keamanan memori dan performa sekelas C/Rust **tanpa garbage collection pause** dengan menerapkan tiga prinsip dasar:

### 5.1 Kepemilikan Tunggal (*Affine Ownership*)
- Setiap nilai di memori memiliki tepat satu variabel pemilik (*owner*).
- Jika nilai dipindahkan (*move*), pemilik sebelumnya tidak lagi dapat diakses.

### 5.2 Peminjaman (*Borrowing*)
- **Peminjaman Imutabel (`pinjam T`):** Boleh ada banyak peminjaman imutabel secara simultan selama tidak ada peminjaman mutabel.
- **Peminjaman Mutabel (`pinjam var T`):** Hanya boleh ada tepat satu referensi mutabel aktif pada satu waktu, menjamin ketiadaan data race pada waktu kompilasi.

### 5.3 Alokasi Berbasis Wilayah (Region / Arena) & `tunda`
- Komolang menyediakan alokator arena bawaan untuk siklus hidup terkelompok (misalnya dalam request web atau per-frame render).
- Pernyataan `tunda` (*defer*) memastikan deallokasi atau pembersihan sumber daya (seperti menutup file atau soket jaringan) dijalankan secara pasti saat keluar dari cakupan blok.

---

## 6. Model Konkurensi

Komolang mengadopsi model konkurensi berbasis CSP (*Communicating Sequential Processes*):

1. **Task Ringan (`jalankan`):**
   - Menjalankan fungsi sebagai green-thread ringan yang dijadwalkan oleh scheduler work-stealing M:N di runtime Komolang.
   - Jejak memori awal task sangat kecil (2 KB) dan dapat bertumbuh secara dinamis.

2. **Saluran Komunikasi (`saluran<T>`):**
   - Komunikasi antar-task dilakukan melalui channel mpsc (multi-producer single-consumer) atau mpmc aman.
   - Operasi `saluran.kirim(nilai)` dan `saluran.terima()` bersifat non-blocking secara OS thread, tetapi menangguhkan task ringan sampai data tersedia.

```komolang
fungsi pekerja(s: saluran<bulat>) {
    s.kirim(100)
}

fungsi utama() {
    biar s = saluran<bulat>(1)
    jalankan pekerja(s)
    biar val = s.terima()
}
```

---

## 7. Sistem Modul dan Paket

### 7.1 Paket
Setiap file kode Komolang diawali dengan deklarasi paket:
```komolang
paket matematika
```

### 7.2 Visibilitas Simbol
- Simbol (fungsi, variabel, tipe, bidang struktur) berstatus **pribadi secara default**.
- Simbol yang diawali kata kunci `publik` dapat diakses oleh modul lain yang mengimpor paket tersebut.

### 7.3 Impor Modul
Pustaka standar diimpor menggunakan prefix `std/`:
```komolang
impor "std/cetak"
impor "std/waktu"
impor "vendor/basisdata" sebagai db
```

---

## 8. Diagnostik dan Pesan Kesalahan

Kompiler Komolang wajib menyajikan pesan kesalahan diagnostik dengan kualitas terbaik:
1. Menunjukkan nama berkas, nomor baris, dan nomor kolom yang presisi.
2. Menampilkan cuplikan baris kode yang salah dengan penanda garis bawah (`^^^`).
3. Penjelasan kesalahan dan saran tindakan perbaikan disajikan dalam **Bahasa Indonesia** yang jelas, ramah, dan solutif.
