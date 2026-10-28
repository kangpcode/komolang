# Contoh Integrasi Komolang + Vue 3 (WebAssembly Interop)

Direktori ini mendemonstrasikan integrasi modul komputasi berkinerja tinggi yang ditulis dalam bahasa **Komolang** ke dalam aplikasi **Vue 3** menggunakan Composition API (`useKomolang`).

## Struktur Proyek

```
examples/integrasi_vue/
├── package.json               # Konfigurasi dependensi npm (Vue 3, Vite)
├── README.md                  # Dokumentasi petunjuk penggunaan
└── src/
    ├── logika_berat.komo      # Logika komputasi Komolang (Fibonacci, cek prima)
    ├── komolang_bridge.js     # Composable Vue 3 'useKomolang' yang menghubungkan WASM
    └── App.vue                # Antarmuka modern reaktif Vue 3
```

## Cara Menjalankan

1. **Instal dependensi frontend:**
   ```bash
   cd examples/integrasi_vue
   npm install
   ```

2. **Jalankan server pengembangan:**
   ```bash
   npm run dev
   ```

3. Buka peramban di `http://localhost:5173/` untuk mencoba komputasi interaktif langsung!
