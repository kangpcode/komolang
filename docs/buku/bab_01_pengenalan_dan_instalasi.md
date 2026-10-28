# Bab 1: Pengenalan & Instalasi Komolang

Komolang (Komodo Language) adalah bahasa pemrograman kompilasi native asli Indonesia yang memadukan performa puncak setara C dan Rust dengan keterbacaan sintaks alami berbahasa Indonesia.

---

## 1. Instalasi Kompilator `komo`

### Linux & macOS (Terminal Bash/Zsh)
Cukup jalankan satu baris perintah instalasi resmi:
```bash
curl -fsSL https://komolang.org/install.sh | bash
```

### Windows (PowerShell)
Buka PowerShell dan jalankan:
```powershell
irm https://komolang.org/install.ps1 | iex
```

Setelah instalasi selesai, verifikasi keberadaan kompilator:
```bash
komo versi
# Output: Komolang (Komodo Language) v0.1.0 / v1.0.0
```

---

## 2. Membuat Proyek Pertama

Gunakan subperintah `komo baru` untuk membuat kerangka kerja proyek terstruktur:

```bash
komo baru halo_komo
cd halo_komo
```

Struktur folder yang dihasilkan secara otomatis:
```
halo_komo/
├── komo.toml          # Manifes konfigurasi proyek & dependensi
└── src/
    └── utama.komo     # Titik masuk utama program
```

---

## 3. Anatomi Berkas `.komo`

Buka berkas `src/utama.komo`:

```komo
// Copyright (c) 2026. Program Pertama Komolang.

paket utama

impor std::cetak

publik fungsi utama() {
    cetak::cetak_baris("Halo, Dunia dari bahasa Komolang! 🦎")
}
```

Jalankan program Anda secara instan:
```bash
komo jalankan src/utama.komo
```

Atau bangun executable native mandiri:
```bash
komo bangun src/utama.komo -o bin/halo
./bin/halo
```
