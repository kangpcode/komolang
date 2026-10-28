# Panduan Migrasi: Dari Python ke Komolang

Bagi pemrogram Python, Komolang menawarkan keterbacaan kode yang ramah dan alami, tetapi dengan **kecepatan eksekusi 50 hingga 100 kali lebih cepat** serta keamanan tipe statis (*compile-time type safety*).

---

## 1. Perbandingan Sintaks Dasar

| Konsep | Python | Komolang |
| :--- | :--- | :--- |
| **Cetak Konsol** | `print("Halo")` | `cetak("Halo")` |
| **Variabel Imutabel** | `x = 10` (konvensi `X = 10`) | `biar x = 10` |
| **Variabel Mutabel** | `x = 10; x = 20` | `var x = 10; x = 20` |
| **Fungsi** | `def tambah(a: int, b: int) -> int:` | `fungsi tambah(a: bulat, b: bulat) bulat` |
| **Percabangan** | `if x > 5: ... elif: ... else:` | `jika x > 5 { ... } lainnya jika { ... } lainnya { ... }` |
| **Perulangan** | `for i in range(5):` | `untuk i dalam 0..5 {` |
| **Struktur Objek** | `class Titik: def __init__(self, x, y):` | `struktur Titik { x: bulat, y: bulat }` |

---

## 2. Keuntungan Beralih ke Komolang

1. **Kecepatan Native**: Tidak ada overhead interpretasi bytecode atau GIL (Global Interpreter Lock). Komolang dikompilasi langsung ke kode mesin x86_64/ARM64.
2. **Distribusi Biner Tunggal**: Menghasilkan satu file executable mandiri tanpa perlu pusing dengan virtualenv atau dependensi runtime Python di server target.
3. **Konkurensi Tanpa Batas**: Thread ringan Komolang dapat berjalan paralel pada seluruh core CPU secara simultan.
