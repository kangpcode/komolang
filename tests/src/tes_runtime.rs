// Copyright (c) kangpcode (Dhafa Nazula Permadi). Licensed under MIT or Apache-2.0.

//! Pengujian integrasi dan stress test untuk Komolang Runtime (Tahap 5):
//! - Alokator memori berbasis Arena / Region berantai tanpa jeda GC
//! - Pengelola pernyataan `tunda` (defer) LIFO & RAII scope guard
//! - Scheduler green task multi-thread dengan algoritma Work-Stealing
//! - Saluran komunikasi MPSC (bounded & unbounded channel)
//! - Primitif sinkronisasi `Kunci`, `RwKunci`, dan `Pemberitahu`
//! - Penanganan kepanikan (panic) dengan jejak tumpukan (stack trace) berbahasa Indonesia
//! - Deteksi dini overflow aritmatika dan batas larik
//! - FFI C ABI runtime compatibility

use std::sync::atomic::{AtomicI64, AtomicUsize, Ordering};
use std::sync::Arc;
use std::thread;
use std::time::Duration;

use komo_runtime::{
    buat_saluran_tak_terbatas, buat_saluran_terbatas, periksa_indeks_larik,
    periksa_overflow_kali, periksa_overflow_kurang, periksa_overflow_tambah,
    periksa_pembagian_nol_bulat, tangkap_kepanikan, AlokatorArena, BingkaiJejak, GalatPenerimaan,
    JejakTumpukan, KesalahanRuntime, KonteksRuntime, Kunci, PenjagaTunda, RwKunci, Scheduler,
    TumpukanTunda,
};

#[test]
fn tes_stress_alokator_arena_tanpa_gc_pause() {
    let mut arena = AlokatorArena::dengan_kapasitas(8192);

    // Lakukan siklus alokasi masif dan daur ulang (reset) berulang kali
    for putaran in 0..10 {
        let mut simpanan = Vec::with_capacity(1000);
        for i in 0..1000 {
            let ptr = arena.alokasi_tipe((putaran * 10000 + i) as i64);
            simpanan.push(ptr);
        }

        // Verifikasi integritas setiap nilai yang dialokasikan
        for (i, val) in simpanan.iter().enumerate() {
            assert_eq!(**val, (putaran * 10000 + i) as i64);
        }

        assert!(arena.total_terpakai() >= 1000 * 8);

        // Reset instan nol-biaya
        arena.reset();
        assert_eq!(arena.total_terpakai(), 0);
    }
}

#[test]
fn tes_alokasi_larik_dan_string_arena() {
    let arena = AlokatorArena::baru();
    let teks = arena.alokasi_str("Komolang adalah bahasa sistem modern nusantara");
    assert_eq!(teks, "Komolang adalah bahasa sistem modern nusantara");

    let larik_awal = vec![1.5f64, 2.5, 3.5, 4.5, 5.5];
    let irisan = arena.alokasi_larik(&larik_awal);
    assert_eq!(irisan, &[1.5, 2.5, 3.5, 4.5, 5.5]);
}

#[test]
fn tes_stress_scheduler_work_stealing() {
    let scheduler = Scheduler::baru(4);
    let total_task = 5000;
    let akumulator = Arc::new(AtomicI64::new(0));

    for i in 1..=total_task {
        let ak = akumulator.clone();
        scheduler.jalankan(move || {
            ak.fetch_add(i as i64, Ordering::SeqCst);
        });
    }

    scheduler.tunggu_selesai();

    let expected: i64 = (1..=total_task as i64).sum();
    assert_eq!(
        akumulator.load(Ordering::SeqCst),
        expected,
        "Seluruh {} task harus selesai dieksekusi via work-stealing scheduler",
        total_task
    );
}

#[test]
fn tes_stress_saluran_mpsc_ribuan_pesan() {
    let (tx, rx) = buat_saluran_terbatas::<usize>(128);
    let jumlah_produsen = 8;
    let pesan_per_produsen = 1000;

    let mut handles = Vec::new();
    for p in 0..jumlah_produsen {
        let tx_p = tx.clone();
        handles.push(thread::spawn(move || {
            for m in 0..pesan_per_produsen {
                tx_p.kirim(p * 100000 + m).unwrap();
            }
        }));
    }
    drop(tx); // Tutup pengirim utama

    let mut total_diterima = 0;
    let mut jumlah_per_produsen = vec![0; jumlah_produsen];

    while let Ok(pesan) = rx.terima() {
        let p_id = pesan / 100000;
        jumlah_per_produsen[p_id] += 1;
        total_diterima += 1;
    }

    for h in handles {
        h.join().unwrap();
    }

    assert_eq!(total_diterima, jumlah_produsen * pesan_per_produsen);
    for count in jumlah_per_produsen {
        assert_eq!(count, pesan_per_produsen);
    }
}

#[test]
fn tes_tunda_bersarang_dan_pemulihan() {
    let jejak_tunda = Arc::new(Kunci::baru(Vec::new()));

    let j_clone = jejak_tunda.clone();
    let hasil = tangkap_kepanikan(move || {
        let mut tumpukan = TumpukanTunda::baru();

        let j1 = j_clone.clone();
        tumpukan.tunda(move || {
            j1.kunci().push("tunda_pertama");
        });

        let j2 = j_clone.clone();
        tumpukan.tunda(move || {
            j2.kunci().push("tunda_kedua");
        });

        // Picu kepanikan di dalam lingkup
        panic!("Pemicu kepanikan untuk menguji pelepasan tunda");
    });

    assert!(hasil.is_err());
    let isi = jejak_tunda.kunci().clone();
    assert_eq!(
        isi,
        vec!["tunda_kedua", "tunda_pertama"],
        "Tunda harus dieksekusi secara LIFO meski terjadi kepanikan"
    );
}

#[test]
fn tes_jejak_tumpukan_dan_laporan_panik_indonesia() {
    let mut jejak = JejakTumpukan::baru();
    jejak.tambah(BingkaiJejak::baru(
        "matematika.hitung_akar",
        "src/matematika.komo",
        55,
        12,
    ));
    jejak.tambah(BingkaiJejak::baru("utama.kalkulasi", "src/utama.komo", 20, 8));
    jejak.tambah(BingkaiJejak::baru("utama.main", "src/utama.komo", 5, 1));

    let laporan = jejak.format_laporan(
        "Nilai di bawah akar kuadrat tidak boleh negatif",
        Some("pekerja-2"),
    );

    assert!(laporan.contains("🚨 KEPANIKAN RUNTIME KOMOLANG (Panic)"));
    assert!(laporan.contains("Thread : 'pekerja-2'"));
    assert!(laporan.contains("Alasan : Nilai di bawah akar kuadrat tidak boleh negatif"));
    assert!(laporan.contains("Lokasi : src/matematika.komo:55:12"));
    assert!(laporan.contains("Jejak Tumpukan (Stack Trace):"));
    assert!(laporan.contains("[0] matematika.hitung_akar (src/matematika.komo:55:12)"));
    assert!(laporan.contains("[1] utama.kalkulasi (src/utama.komo:20:8)"));
    assert!(laporan.contains("[2] utama.main (src/utama.komo:5:1)"));
}

#[test]
fn tes_deteksi_overflow_dan_kesalahan_runtime() {
    // 1. Pembagian dengan nol
    assert_eq!(
        periksa_pembagian_nol_bulat(0),
        Err(KesalahanRuntime::PembagianDenganNol)
    );
    assert!(periksa_pembagian_nol_bulat(10).is_ok());

    // 2. Overflow tambah, kurang, kali
    assert!(periksa_overflow_tambah(i64::MAX, 1).is_err());
    assert!(periksa_overflow_kurang(i64::MIN, 1).is_err());
    assert!(periksa_overflow_kali(i64::MAX / 2 + 1, 2).is_err());
    assert_eq!(periksa_overflow_kali(7, 8).unwrap(), 56);

    // 3. Batas indeks larik
    assert_eq!(
        periksa_indeks_larik(10, 10),
        Err(KesalahanRuntime::IndeksDiLuarBatas {
            indeks: 10,
            panjang: 10
        })
    );
    assert!(periksa_indeks_larik(9, 10).is_ok());
}

#[test]
fn tes_sinkronisasi_rwlock_dan_mutex() {
    let data = Arc::new(RwKunci::baru(100));
    let mut pembaca = Vec::new();

    for _ in 0..10 {
        let d = data.clone();
        pembaca.push(thread::spawn(move || {
            let nilai = *d.baca();
            assert!(nilai >= 100);
        }));
    }

    {
        let mut t = data.tulis();
        *t += 50;
    }

    for p in pembaca {
        p.join().unwrap();
    }

    assert_eq!(*data.baca(), 150);
}

#[test]
fn tes_ffi_runtime_c_abi() {
    komo_runtime::ffi::komo_rt_inisialisasi();

    let ptr = komo_runtime::ffi::komo_rt_alokasi_arena(64, 8);
    assert!(!ptr.is_null());

    unsafe {
        std::ptr::write_bytes(ptr, 0xAB, 64);
        assert_eq!(*ptr, 0xAB);
    }

    komo_runtime::ffi::komo_rt_reset_arena();
}

#[test]
fn tes_konteks_runtime_dan_saluran_tak_terbatas_timeout() {
    let mut rt = KonteksRuntime::baru();
    let sched = rt.aktifkan_scheduler();

    let (tx, rx) = buat_saluran_tak_terbatas::<String>();
    let tx_keep = tx.clone();

    sched.jalankan(move || {
        thread::sleep(Duration::from_millis(10));
        let _ = tx.kirim("Pesan Async dari Green Task".to_string());
    });

    let hasil = rx.terima_dengan_tenggat(Duration::from_millis(500));
    assert_eq!(hasil, Ok("Pesan Async dari Green Task".to_string()));

    let hasil_timeout = rx.terima_dengan_tenggat(Duration::from_millis(20));
    assert_eq!(hasil_timeout, Err(GalatPenerimaan::BatasWaktuHabis));

    drop(tx_keep);
    assert_eq!(rx.terima(), Err(GalatPenerimaan::SaluranTertutup));

    rt.hentikan();
}

#[test]
fn tes_penjaga_tunda_langsung() {
    let flag = Arc::new(AtomicUsize::new(0));
    {
        let f = flag.clone();
        let _penjaga = PenjagaTunda::baru(move || {
            f.store(99, Ordering::SeqCst);
        });
        assert_eq!(flag.load(Ordering::SeqCst), 0);
    }
    assert_eq!(flag.load(Ordering::SeqCst), 99);
}
