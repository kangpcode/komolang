# Panduan Migrasi: Dari Go ke Komolang

Bagi pengembang Go, Komolang terasa sangat akrab karena keduanya sama-sama mengutamakan kesederhanaan, kompilasi cepat, biner mandiri, dan model komunikasi saluran (*channels*).

---

## 1. Perbandingan Konsep Utama

| Fitur | Go | Komolang |
| :--- | :--- | :--- |
| **Goroutine / Task** | `go fungsi()` | `luncurkan fungsi()` |
| **Channel / Saluran** | `ch := make(chan string)` | `biar (tx, rx) = saluran::buat<teks>()` |
| **Kirim Pesan** | `ch <- "pesan"` | `tx.kirim("pesan")` |
| **Terima Pesan** | `msg := <-ch` | `biar msg = rx.terima()` |
| **Penanganan Galat** | `if err != nil { return err }` | `Hasil<T, G>` + pencocokan pola `cocok` |
| **Memori & GC** | Tracing Garbage Collector (jeda mikro) | Model Kepemilikan Komolang (**Zero GC Pause**) |

---

## 2. Mengapa Memilih Komolang?

- **Tanpa Overhead Garbage Collection**: Komolang tidak memiliki GC pause, menjadikannya ideal untuk game 60 FPS, sistem waktu nyata (*hard real-time*), dan embedded systems.
- **Penanganan Galat Lebih Kuat**: Tipe `Hasil<T, G>` memaksa setiap kemungkinan galat diperiksa pada saat kompilasi tanpa ada error yang terabaikan secara tidak sengaja.
