# Bab 4: Struktur Data, Metode, dan Penanganan Galat

---

## 1. Struktur Data (`struktur`) dan Metode (`impl`)

```komo
paket utama

publik struktur Titik {
    x: bulat,
    y: bulat,
}

impl Titik {
    publik fungsi baru(x: bulat, y: bulat) Titik {
        kembali Titik { x: x, y: y }
    }

    publik fungsi jarak_ke_pusat(diri: pinjam Titik) desimal {
        kembali matematika::akar((diri.x * diri.x + diri.y * diri.y) sebagai desimal)
    }
}
```

---

## 2. Enum dan Pencocokan Pola (`cocok`)

```komo
publik enum StatusPesanan {
    Tertunda,
    Diproses,
    Selesai(bulat), // Membawa ID resi
    Batal(teks),    // Membawa alasan pembatalan
}

fungsi cetak_status(status: StatusPesanan) {
    cocok status {
        StatusPesanan::Tertunda => cetak("Menunggu pembayaran..."),
        StatusPesanan::Diproses => cetak("Sedang disiapkan di gudang..."),
        StatusPesanan::Selesai(resi) => cetak("Selesai dikirim dengan resi #", resi),
        StatusPesanan::Batal(alasan) => cetak("Pesanan dibatalkan: ", alasan),
    }
}
```

---

## 3. Penanganan Galat: `Hasil<T, G>` dan `Mungkin<T>`

Komolang tidak menggunakan exception tersembunyi yang rentan menyebabkan panic tiba-tiba:
- `Hasil::Sukses(nilai)` atau `Hasil::Galat(pesan)`
- `Mungkin::Ada(nilai)` atau `Mungkin::Kosong`

```komo
fungsi bagi(a: bulat, b: bulat) Hasil<bulat, teks> {
    jika b == 0 {
        kembali Hasil::Galat("Pembagian dengan angka nol tidak diizinkan")
    }
    kembali Hasil::Sukses(a / b)
}
```
