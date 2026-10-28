# Bab 5: Konkurensi Modern & Saluran (Channels)

Komolang dirancang sejak awal untuk memanfaatkan arsitektur CPU multi-core modern secara maksimal melalui sistem thread ringan (*green threads*) berbasis *work-stealing scheduler*.

---

## 1. Thread Ringan

Anda dapat meluncurkan eksekusi fungsi di latar belakang secara asinkron tanpa beban memori stack thread OS:

```komo
paket utama

impor std::waktu

fungsi tugas_latar(id: bulat) {
    cetak("Tugas latar #", id, " mulai bekerja...")
    waktu::tidur_ms(500)
    cetak("Tugas latar #", id, " selesai!")
}

publik fungsi utama() {
    luncurkan tugas_latar(1)
    luncurkan tugas_latar(2)
}
```

---

## 2. Komunikasi Aman Antar-Task: `Saluran` (Channel)

Prinsip konkurensi Komolang:
> *"Jangan berkomunikasi dengan berbagi memori; melainkan bagilah memori dengan saling berkirim pesan."*

```komo
impor std::saluran

publik fungsi utama() {
    biar (pengirim, penerima) = saluran::buat<teks>()

    luncurkan fungsi() {
        pengirim.kirim("Laporan penjualan harian telah siap!")
    }()

    biar pesan = penerima.terima()
    cetak("Menerima: ", pesan)
}
```

---

## 3. Primitif Sinkronisasi: `Kunci` (Mutex) & `KunciBacaTulis` (RwLock)

Jika beberapa task membutuhkan akses aman ke sumber daya bersama:

```komo
impor std::sinkronisasi::{Kunci, KunciBacaTulis}

publik fungsi utama() {
    biar counter = Kunci::baru(0)

    // Akses thread-safe
    {
        var penjaga = counter.kunci()
        *penjaga += 1
    } // Kunci otomatis dilepaskan saat penjaga keluar lingkup (RAII)
}
```
