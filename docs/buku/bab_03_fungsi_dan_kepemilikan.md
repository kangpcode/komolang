# Bab 3: Fungsi, Peminjaman, dan Manajemen Memori

Komolang mencapai performa setara C dan Rust tanpa jeda pembersihan memori (*zero garbage collection pause*) melalui model kepemilikan terpadu.

---

## 1. Deklarasi Fungsi

Fungsi dalam Komolang dideklarasikan dengan kata kunci `fungsi`. Jika fungsi mengembalikan nilai, tipe kembalian ditulis langsung setelah tanda kurung parameter:

```komo
paket utama

/// Menjumlahkan dua bilangan bulat
publik fungsi tambah(a: bulat, b: bulat) bulat {
    kembali a + b
}
```

---

## 2. Sistem Kepemilikan (*Ownership*)

Setiap nilai dalam Komolang memiliki pemilik (*owner*) berupa variabel tunggal. Saat variabel keluar dari lingkup kurung kurawal `{ }`, memorinya langsung dibebaskan seketika secara deterministik:

```komo
fungsi buat_data() {
    biar pesan = "Pesan Rahasia" // 'pesan' memiliki alokasi string
    // pesan valid di sini
} // 'pesan' keluar dari lingkup -> memori langsung dibebaskan tanpa GC
```

---

## 3. Peminjaman Referensi (`pinjam` dan `pinjam_var`)

Alih-alih memindahkan kepemilikan (*move*), Anda dapat meminjamkan data:
- `pinjam T`: Peminjaman baca-saja (*immutable borrow*).
- `pinjam_var T`: Peminjaman yang mengizinkan mutasi (*mutable borrow*).

```komo
fungsi hitung_panjang(teks_ref: pinjam teks) bulat {
    kembali teks_ref.panjang()
}

fungsi tambah_tanda_seru(teks_mut: pinjam_var teks) {
    teks_mut.tambah("!")
}
```
