# Ekstensi VS Code Komolang (Bahasa Komodo)

Ekstensi resmi Visual Studio Code untuk bahasa pemrograman **Komolang**.

## Fitur Unggulan

- **Pewarnaan Sintaks Komprehensif (Syntax Highlighting)**: Mendukung seluruh 30 kata kunci Komolang, tipe data primitif (`bulat`, `desimal`, `teks`, `bool`, `byte`, `larik`), literal biner/heksadesimal, string interpolasi `{ekspresi}`, komentar baris/blok, serta operator.
- **Integrasi Server LSP Mandiri**: Terhubung langsung dengan `komo lsp` untuk:
  - Pemeriksaan galat sintaks & semantik langsung (*real-time error squiggles*).
  - Pelengkapan kode cerdas (*auto-completion*).
  - Dokumentasi bantuan saat kursor melayang (*hover information*).
  - Pemformatan kode kanonik otomatis saat menyimpan berkas (*format on save* via `komo format`).
- **Cuplikan Kode Cepat (Snippets)**: Pintasan cepat untuk deklarasi fungsi (`fungsi`), titik masuk (`utama`), percabangan (`jika`, `jikalain`), perulangan (`selagi`, `untuk`), struktur data (`struktur`, `antarmuka`, `enum`), pencocokan pola (`cocok`), dan unit testing (`tes`, `bench`).
- **Dukungan Tanda Kurung & Indentasi Otomatis**: Auto-closing brackets dan aturan indentasi berbasis konvensi Komolang.

## Cara Pemasangan

### 1. Salin ke Direktori Ekstensi Pengguna
Salin folder ini ke direktori ekstensi VS Code Anda:

- **Linux / macOS**: `~/.vscode/extensions/komolang`
- **Windows**: `%USERPROFILE%\.vscode\extensions\komolang`

Contoh perintah bash:
```bash
mkdir -p ~/.vscode/extensions/komolang
cp -r editors/vscode/* ~/.vscode/extensions/komolang/
```

### 2. Memastikan Kakas `komo` Tersedia di PATH
Pastikan biner `komo` telah dikompilasi dan berada dalam variabel lingkungan `PATH`:
```bash
cargo build --release --bin komo
sudo cp target/release/komo /usr/local/bin/komo
```

Verifikasi dengan menjalankan:
```bash
komo versi
```

## Konfigurasi Klien LSP (VS Code)

Jika menggunakan ekstensi klien LSP umum seperti [Generic LSP Client](https://marketplace.visualstudio.com/items?itemName=redhat.vscode-xml) atau konfigurasi bawaan VS Code, tambahkan pengaturan berikut ke `.vscode/settings.json`:

```json
{
  "[komo]": {
    "editor.defaultFormatter": "kangpcode.komolang-vscode",
    "editor.formatOnSave": true,
    "editor.tabSize": 4,
    "editor.insertSpaces": true
  }
}
```

## Lisensi
Hak Cipta (c) 2026 kangpcode (Dhafa Nazula Permadi). Dilindungi lisensi MIT atau Apache-2.0.
