# Bab 2: Variabel, Tipe Data, dan Kontrol Alur

Komolang menganut prinsip keamanan tipe statis (*statically typed*) dengan inferensi tipe cerdas.

---

## 1. Deklarasi Variabel

- **`biar` (Imutabel)**: Nilai konstan yang tidak dapat diubah setelah diinisialisasi (disarankan secara bawaan).
- **`var` (Mutabel)**: Variabel yang nilainya dapat diubah kembali.

```komo
paket utama

publik fungsi utama() {
    biar pi: desimal = 3.14159
    biar nama = "Komolang" // Inferensi tipe otomatis ke 'teks'

    var hitungan = 0
    hitungan += 1
}
```

---

## 2. Tipe Data Primitif

- **`bulat`**: Bilangan bulat bertanda 64-bit (`i64`).
- **`desimal`**: Bilangan pecahan presisi ganda 64-bit (`f64`).
- **`bool`**: Nilai kebenaran logika (`benar` atau `salah`).
- **`teks`**: Rangkaian karakter Unicode UTF-8 dinamis.

---

## 3. Kontrol Alur

### Percabangan `jika-lainnya`
```komo
jika skor >= 90 {
    cetak("Peringkat: A")
} lainnya jika skor >= 75 {
    cetak("Peringkat: B")
} lainnya {
    cetak("Peringkat: C")
}
```

### Percabangan `pilih-kasus`
```komo
pilih mode {
    kasus 1 { cetak("Mode Mudah") }
    kasus 2 { cetak("Mode Sedang") }
    kasus 3 { cetak("Mode Ahli") }
    lainnya { cetak("Pilihan tidak valid") }
}
```

### Perulangan `untuk` dan `selagi`
```komo
// Perulangan rentang nilai
untuk i dalam 1..=5 {
    cetak("Nilai ke-", i)
}

// Perulangan berbasis kondisi
var angka = 10
selagi angka > 0 {
    angka -= 1
}
```
