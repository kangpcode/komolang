# Bab 8: Antarmuka Grafis & Pembuatan Game (`komoui` & `komogame`)

---

## 1. Antarmuka Deklaratif KomoUI

KomoUI mengadopsi paradigma deklaratif modern berbasis komponen pohon, tema elegan, dan sinyal reaktif:

```komo
impor komo_ui::komponen::{wadah_kolom, judul, tombol, masukan}
impor komo_ui::tema::TemaUi

fungsi buat_tampilan_login() {
    biar ui = wadah_kolom(vec![
        judul("Masuk ke Akun Anda"),
        masukan("Alamat Email", "input-email"),
        masukan("Kata Sandi", "input-password"),
        tombol("Masuk Sekarang", "btn-submit"),
    ])
}
```

---

## 2. Pembuatan Game dengan KomoGame (ECS & 60 FPS)

KomoGame menyediakan Entity Component System (ECS), loop deterministik terkunci pada 60 FPS, synthesizer audio retro, dan fisika 2D/3D:

```komo
impor komo_game::{DuniaEcs, MesinGame, Transformasi2D, Kecepatan2D, BadanKaku2D}

fungsi mulai_game() {
    var mesin = MesinGame::baru(320, 240)
    var pemain = mesin.dunia.buat_entitas()

    mesin.dunia.pasang_komponen(pemain, Transformasi2D::baru(50.0, 100.0))
    mesin.dunia.pasang_komponen(pemain, Kecepatan2D::baru(100.0, 0.0))
    mesin.dunia.pasang_komponen(pemain, BadanKaku2D::dinamis(1.0))

    // Jalankan game engine
}
```
