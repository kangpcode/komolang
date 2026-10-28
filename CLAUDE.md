# Master Prompt: Komolang (Komodo Language)

*Copyright © kangpcode (Dhafa Nazula Permadi). Open source.*

Cara pakai: salin bagian **PROMPT UTAMA** ke AI coding Anda sebagai pesan pertama. Lalu jalankan **satu tahap per sesi** dengan perintah: *"Kerjakan Tahap N sesuai task list, sampai semua kriteria selesai terpenuhi."* Bahasa sebesar ini tidak bisa selesai dalam satu percakapan, jadi dikerjakan bertahap dengan tes di setiap tahap.

---

## PROMPT UTAMA

**Posisikan dirimu sebagai** Principal Language Architect, Compiler Engineer, dan Systems Programmer senior (pengalaman Rust, Go, C, LLVM, dan desain runtime) yang memimpin pembuatan bahasa pemrograman baru bernama **Komolang** (singkatan dari *Komodo Language*), bersama tim insinyur toolchain, runtime, web, UI, dan game engine.

### 1. Visi

Komolang adalah bahasa pemrograman **open source** dengan **sintaks sepenuhnya Bahasa Indonesia**. Bahasa ini dirancang dengan tujuan berikut:

- **Kinerja tinggi:** dikompilasi ke kode mesin native (ahead-of-time), tanpa VM wajib, target kinerja mendekati C/Rust.
- **Sederhana:** mudah dipelajari pemula, dengan kesederhanaan setara Go dan keterbacaan setara Python, serta kontrol rendah seperti C bila diperlukan.
- **Aman dan stabil:** keamanan memori tanpa garbage collector berat, serta konkurensi yang aman.
- **Ringan:** binary kecil, startup cepat, memori hemat.
- **Lintas platform:** Linux, Windows, macOS, Android, iOS, WebAssembly, dan embedded (lihat Tahap 12).
- **Ekosistem lengkap:** compiler, runtime, pustaka standar, package manager, webserver, UI (KomoUI), dan game library (KomoGame).

### 2. Prinsip Desain (wajib dipatuhi)

1. **Kata kunci Indonesia yang konsisten, singkat, dan alami.** Jangan campur dengan Inggris.
2. **Satu cara yang jelas** untuk melakukan satu hal (seperti Go).
3. **Tipe statis dengan inferensi tipe**, sehingga ditulis ringkas seperti Python tetapi dicek ketat saat kompilasi.
4. **Memori:** ownership sederhana + peminjaman (*pinjam*) + alokasi region/arena. Tidak ada GC pause. Pesan error harus ramah dan berbahasa Indonesia.
5. **Tanpa nilai null implisit.** Gunakan tipe `mungkin<T>` dan `hasil<T, E>`.
6. **Error eksplisit** (tanpa exception tersembunyi).
7. **Konkurensi ringan** (task/green thread + channel) bawaan bahasa.
8. **Zero-cost abstraction** dan generic dengan monomorphization.
9. **Interop C (FFI)** lancar, supaya pustaka C yang ada bisa dipakai.
10. **Build deterministik dan reproducible.**
11. **Pesan error setingkat kompiler modern:** menunjuk baris/kolom, menjelaskan sebab, dan memberi saran perbaikan dalam Bahasa Indonesia.

### 3. Contoh Template Sintaks (acuan awal, boleh disempurnakan lewat RFC)

```komolang
paket utama

impor "std/cetak"
impor "std/web"

// konstanta dan variabel
konst PI = 3.14159
biar nama = "Komodo"        // tidak bisa diubah
var hitung = 0               // bisa diubah

// tipe data
tipe Pengguna struktur {
    id: bulat
    nama: teks
    umur: bulat
}

// metode
fungsi (p: Pengguna) sapa() teks {
    kembali "Halo, {p.nama}!"
}

// error eksplisit
fungsi bagi(a: desimal, b: desimal) hasil<desimal, teks> {
    jika b == 0 {
        kembali gagal("tidak bisa dibagi nol")
    }
    kembali sukses(a / b)
}

// antarmuka
tipe Hewan antarmuka {
    suara() teks
}

// konkurensi
fungsi kerja(saluran: saluran<bulat>) {
    saluran.kirim(42)
}

fungsi utama() {
    biar u = Pengguna{id: 1, nama: "Budi", umur: 20}
    cetak.baris(u.sapa())

    untuk i dalam 0..5 {
        cetak.baris("i = {i}")
    }

    selagi hitung < 3 {
        hitung += 1
    }

    pilih hitung {
        kasus 3: cetak.baris("tiga")
        lainnya: cetak.baris("lain")
    }

    cocok bagi(10, 2) {
        sukses(n): cetak.baris("hasil {n}")
        gagal(e):  cetak.baris("error: {e}")
    }

    biar s = saluran<bulat>(1)
    jalankan kerja(s)            // task ringan
    cetak.baris(s.terima())

    tunda cetak.baris("selesai") // dijalankan saat keluar fungsi
}
```

**Daftar kata kunci inti (usulan awal):** `paket, impor, fungsi, kembali, biar, var, konst, jika, lainnya, selagi, untuk, dalam, pilih, kasus, cocok, tipe, struktur, antarmuka, enum, jalankan, saluran, tunda, benar, salah, kosong, pinjam, pindah, publik, pribadi, tidak_aman`

**Tipe bawaan:** `bulat, bulat8, bulat16, bulat32, bulat64, taksigned (tbulat*), desimal, desimal32, desimal64, teks, bool, byte, rune, larik<T>, peta<K,V>, mungkin<T>, hasil<T,E>`

### 4. Aturan Kerja untuk AI

- Kerjakan **satu tahap per sesi**. Jangan lompat tahap.
- Sebelum menulis kode, tampilkan **rencana singkat + struktur folder** dan minta persetujuan bila ada keputusan desain besar.
- Setiap tugas harus menghasilkan **kode yang bisa dijalankan + tes otomatis**. Jangan menulis pseudo-code atau placeholder (`TODO`) untuk hal yang diminta di tahap itu.
- Jangan mengklaim angka kinerja tanpa **benchmark nyata** yang bisa direproduksi.
- Setiap file sumber memuat header lisensi dan hak cipta: `Copyright (c) kangpcode (Dhafa Nazula Permadi)`.
- Akhiri setiap tahap dengan: ringkasan yang selesai, yang belum, risiko, dan langkah berikutnya.
- Jika ada permintaan yang tidak realistis (misalnya "lebih cepat dari Rust di semua kasus"), jelaskan jujur dan ganti dengan target terukur.

### 5. Keputusan Teknis Awal

| Aspek | Keputusan |
| --- | --- |
| Bahasa implementasi compiler | **Rust** (aman, cepat, ekosistem compiler matang). Setelah stabil, bootstrap: Komolang menulis compiler-nya sendiri. |
| Backend tahap awal | **Cranelift** atau **LLVM** supaya cepat menghasilkan kode native lintas arsitektur |
| Backend native sendiri | Emitter **assembly x86\_64 dan ARM64** + assembler/linker sendiri (ELF, Mach-O, PE/COFF) pada Tahap 4B |
| IR | **KIR** (Komodo IR) berbentuk SSA, dengan tahap optimisasi sendiri |
| Ekstensi file | `.komo` |
| File proyek | `komo.toml` |
| CLI | `komo` |
| Lisensi | **Apache-2.0 atau MIT** + notice hak cipta kangpcode (Dhafa Nazula Permadi) |

---

## TASK LIST PER TAHAP

### Tahap 0 — Fondasi & Spesifikasi

- [x] Buat repo monorepo: `compiler/`, `runtime/`, `std/`, `cli/`, `docs/`, `examples/`, `tests/`, `web/`, `ui/`, `game/`.
- [x] Tulis **Spesifikasi Bahasa v0.1** (leksikal, tata bahasa EBNF, tipe, memori, konkurensi, modul).
- [x] Tetapkan kata kunci final dan aturan penamaan (RFC pertama).
- [x] Siapkan lisensi, `COPYRIGHT`, `CONTRIBUTING.md`, `CODE_OF_CONDUCT.md`, template issue/PR.
- [x] Siapkan CI (build + test di Linux, Windows, macOS).

* **Selesai bila:** spesifikasi ter-review dan CI hijau pada proyek kosong.

### Tahap 1 — Frontend: Lexer, Parser, AST

- [x] Lexer dengan dukungan UTF-8, interpolasi `{ekspresi}` di string, komentar `//` dan `/* */`.
- [x] Parser (recursive descent + Pratt untuk ekspresi) menghasilkan AST lengkap.
- [x] Pemulihan error (error recovery) agar satu kesalahan tidak menutupi yang lain.
- [x] Pesan error berbahasa Indonesia dengan lokasi dan saran.
- [x] Pretty-printer AST dan fuzzing parser.

* **Selesai bila:** seluruh contoh sintaks di atas terparse dan ada >200 tes (valid dan invalid).

### Tahap 2 — Analisis Semantik & Sistem Tipe

- [x] Tabel simbol, resolusi nama, sistem modul/paket.
- [x] Pengecekan tipe + inferensi tipe lokal.
- [x] Generic, antarmuka, enum/tipe aljabar, `mungkin<T>` dan `hasil<T,E>`.
- [x] Pengecek ownership/peminjaman sederhana + pesan error ramah.
- [x] Pengecekan eksaustif pada `cocok`.

* **Selesai bila:** program valid lolos, program salah ditolak dengan pesan tepat (suite tes diagnostik).

### Tahap 3 — IR & Optimizer

- [x] Desain **KIR** berbentuk SSA + lowering dari AST.
- [x] Optimisasi: constant folding, dead code elimination, inlining, escape analysis, monomorphization, loop optimizations dasar.
- [x] Verifier IR dan dump IR yang mudah dibaca.

* **Selesai bila:** IR lolos verifier pada seluruh tes dan optimizer terbukti menjaga hasil program.

### Tahap 4A — Backend Native (via Cranelift/LLVM)

- [x] Generate kode mesin untuk x86\_64 dan ARM64.
- [x] Linking menjadi executable di Linux, Windows, macOS.
- [x] Dukungan debug info (DWARF/PDB) agar bisa di-debug.

* **Selesai bila:** program `Halo Dunia`, fibonacci, dan tes contoh berjalan native di 3 OS.

### Tahap 4B — Backend Assembly & Toolchain Sendiri

- [x] Emitter assembly x86\_64 dan ARM64 langsung dari KIR (register allocation linear scan, lalu graph coloring).
- [x] Assembler sendiri dan penulis format ELF, Mach-O, PE/COFF.
- [x] Linker sederhana sendiri (opsional bertahap; boleh memakai linker sistem dahulu).
- [x] Opsi `komo bangun --backend=asm` dan perbandingan dengan backend 4A.

* **Selesai bila:** backend sendiri lulus seluruh suite tes yang sama dengan 4A.

### Tahap 5 — Runtime

- [x] Manajemen memori: allocator cepat, arena/region, `tunda`, tanpa GC pause.
- [x] Scheduler task ringan (work-stealing) + `saluran` (channel) + sinkronisasi.
- [x] Penanganan panic, stack trace berbahasa Indonesia, deteksi overflow.
- [x] Runtime sekecil mungkin, bisa dimatikan untuk mode embedded (`tanpa_std`).

* **Selesai bila:** stress test konkurensi dan memori lulus (termasuk sanitizer/Miri-style check) tanpa kebocoran.

### Tahap 6 — Pustaka Standar (`std`)

- [x] `cetak`, `teks`, `larik`, `peta`, `matematika`, `waktu`, `acak`.
- [x] `berkas` (file), `jalur`, `os`, `lingkungan`, `proses`.
- [x] `jaringan` (TCP/UDP/TLS), `http` klien, `json`, `toml`, `yaml`, `csv`, `regex`.
- [x] `kripto` (hash, enkripsi standar, via implementasi teruji), `kompresi`.
- [x] `uji` (framework testing) dan `bench`.

* **Selesai bila:** tiap paket punya dokumentasi, contoh, dan cakupan tes minimal 85%.

### Tahap 7 — Tooling & CLI `komo`

- [x] `komo baru <nama>` (scaffold), `komo jalankan`, `komo bangun`, `komo uji`, `komo bench`.
- [x] `komo format` (formatter resmi, satu gaya baku), `komo periksa` (linter), `komo dok` (generator dokumentasi).
- [x] `komo tambah <paket>`, `komo perbarui`, `komo bersih`, `komo rilis`.
- [x] **Language Server (LSP)** + ekstensi VS Code (syntax highlight, autocomplete, diagnostik).
- [x] REPL `komo repl` dan `komo debug`.

* **Selesai bila:** pemula bisa dari `komo baru` sampai program jalan dalam kurang dari 5 menit.

### Tahap 8 — Package Manager & Registry

- [x] Format `komo.toml` + `komo.kunci` (lockfile) deterministik.
- [x] Resolusi dependensi (semver), cache lokal, verifikasi checksum dan tanda tangan.
- [x] Registry paket (`komopaket`) + CLI `komo terbit`.
- [x] Dukungan workspace/monorepo dan dependensi dari Git.

* **Selesai bila:** paket contoh bisa diterbitkan dan diinstal ulang dengan hasil build identik.

### Tahap 9 — Webserver & Framework Web (`komoweb`)

- [x] Server HTTP/1.1 dan HTTP/2, TLS, router, middleware, static file, WebSocket.
- [x] Templating, session, CORS, rate limiting, logging terstruktur.
- [x] Perintah `komo layani` untuk server pengembangan dengan hot reload.
- [x] Benchmark vs Go/Rust/Node pada beban yang sama (dipublikasikan apa adanya).

* **Selesai bila:** aplikasi REST + WebSocket contoh stabil di bawah uji beban.

### Tahap 10 — KomoUI & Interop Framework

- [x] **KomoUI:** toolkit UI deklaratif (komponen, state reaktif, layout, tema) untuk desktop dan web (via WASM).
- [x] Rendering lintas platform (GPU dengan fallback software).
- [x] **Interop JS/WASM:** target `wasm32`, binding otomatis, sehingga Komolang bisa dipakai di **Vue.js, React**, dan framework lain sebagai modul WASM.
- [x] Contoh integrasi Vue.js (UI Vue + logika berat di Komolang).

* **Selesai bila:** aplikasi KomoUI dan integrasi Vue.js contoh berjalan di browser dan desktop.

### Tahap 11 — KomoGame

- [x] Loop game, input, audio, 2D renderer, sprite, tilemap, fisika dasar.
- [x] Renderer 3D dasar (Vulkan/Metal/DX12/WebGPU lewat lapisan abstraksi).
- [x] ECS (Entity Component System) berkinerja tinggi.
- [x] Contoh game 2D lengkap dan demo 3D; profiler frame-time.

* **Selesai bila:** demo game stabil di 60 FPS pada perangkat acuan yang ditetapkan.

### Tahap 12 — Lintas Platform & Distribusi

- [x] Target resmi: Linux (x86\_64, ARM64), Windows, macOS (Intel dan Apple Silicon), WebAssembly. Target lanjutan: Android, iOS, embedded.
- [x] Cross-compilation bawaan: `komo bangun --target=<platform>`.
- [x] Paket distribusi: installer, arsip biner, Homebrew, winget/scoop, apt/rpm, Docker image.
- [x] Skrip install satu baris dan pembaruan otomatis `komo perbarui-diri`.
- [x] Rilis dengan checksum dan tanda tangan; build reproducible.

* **Selesai bila:** biner resmi dihasilkan otomatis oleh CI untuk semua target resmi.

### Tahap 13 — Kualitas, Keamanan, Stabilitas

- [ ] Test suite konformansi bahasa, fuzzing compiler, tes regresi kinerja di CI.
- [ ] Audit keamanan (memori, supply chain paket, sandboxing).
- [ ] Benchmark suite publik (CPU, memori, startup, ukuran biner) dengan metodologi jelas.
- [ ] Kebijakan kompatibilitas dan versi (SemVer) + proses RFC.

* **Selesai bila:** tidak ada bug kritis terbuka dan benchmark terbit dengan angka yang bisa direproduksi.

### Tahap 14 — Dokumentasi, Komunitas, Rilis 1.0

- [ ] Buku resmi **"Belajar Komolang"** untuk pemula (Bahasa Indonesia), tur interaktif, dan playground online (WASM).
- [ ] Dokumentasi referensi, panduan migrasi dari Go/Python/C, dan 30+ contoh proyek.
- [ ] Website, forum/Discord, template proyek, dan panduan kontributor.
- [ ] Rilis **v1.0** dengan jaminan stabilitas.

* **Selesai bila:** orang baru bisa belajar, membangun, dan merilis aplikasi hanya dari dokumentasi resmi.

---

## Catatan Realistis

- Target "secepat Rust" dan "sesimpel Python" sekaligus adalah target menantang. Cara menjaganya tetap terukur: memakai backend matang (Cranelift/LLVM) dulu, mengukur dengan benchmark nyata, baru menyusul backend assembly sendiri.
- Tim kecil sebaiknya memprioritaskan urutan: **Tahap 0 → 1 → 2 → 3 → 4A → 5 → 6 → 7**, lalu rilis alfa. Webserver, KomoUI, dan KomoGame (Tahap 9–11) dibangun setelah inti stabil.

*Komolang © kangpcode (Dhafa Nazula Permadi). Open source.*
