// Copyright (c) kangpcode (Dhafa Nazula Permadi). Licensed under MIT or Apache-2.0.

use std::collections::BTreeMap;
use std::env;
use std::fs;
use std::path::PathBuf;

use komo_pkg::cache::{kemas_arsip_komopaket, ManajerCache};
use komo_pkg::kunci::BerkasKunci;
use komo_pkg::manifes::{ManifesPaket, SumberDependensi};
use komo_pkg::registry::{terbitkan_paket_ke_registry, RegistryMock};
use komo_pkg::resolusi::{IndeksPaket, InfoRilis, PenyelesaiDependensi};
use komo_pkg::ruang_kerja::RuangKerja;
use komo_pkg::semver::{KebutuhanVersi, Versi};
use komo_pkg::sha256::sha256_hex;

fn siapkan_direktori_uji(nama_sub: &str) -> PathBuf {
    let base = env::temp_dir().join(format!("komo_uji_pkg_{}", nama_sub));
    let _ = fs::remove_dir_all(&base);
    fs::create_dir_all(&base).expect("Gagal membuat direktori uji paket");
    base
}

#[test]
fn tes_siklus_penuh_manifes_dan_kunci() {
    let dir = siapkan_direktori_uji("manifes_kunci");
    let jalur_toml = dir.join("komo.toml");
    let jalur_kunci = dir.join("komo.kunci");

    let toml_teks = r#"[paket]
nama = "aplikasi_super"
versi = "1.0.0"
penulis = ["Dhafa Nazula Permadi <kangpcode@gmail.com>"]
lisensi = "MIT ATAU Apache-2.0"
deskripsi = "Aplikasi super Komolang"

[dependensi]
"pustaka_a" = "^1.0.0"
"pustaka_b" = "~2.1.0"
"#;
    fs::write(&jalur_toml, toml_teks).unwrap();

    let manifes = ManifesPaket::muat_dari_berkas(&jalur_toml).unwrap();
    assert_eq!(manifes.paket.as_ref().unwrap().nama, "aplikasi_super");
    assert_eq!(manifes.dependensi.len(), 2);

    // Siapkan indeks registry
    let mut indeks = IndeksPaket::baru();
    indeks.daftarkan_rilis(
        "pustaka_a",
        InfoRilis {
            versi: Versi::baru(1, 2, 3),
            dependensi: BTreeMap::new(),
            checksum: "a1a2a3a4a5".into(),
        },
    );
    indeks.daftarkan_rilis(
        "pustaka_b",
        InfoRilis {
            versi: Versi::baru(2, 1, 4),
            dependensi: BTreeMap::new(),
            checksum: "b1b2b3b4b5".into(),
        },
    );

    let solver = PenyelesaiDependensi::baru(&indeks);
    let hasil = solver.selesaikan(&manifes.dependensi).unwrap();
    let kunci = hasil.ke_berkas_kunci("https://komopaket.dev");

    kunci.simpan_ke_berkas(&jalur_kunci).unwrap();
    assert!(jalur_kunci.exists());

    let kunci_muat = BerkasKunci::muat_dari_berkas(&jalur_kunci).unwrap();
    assert_eq!(kunci, kunci_muat);

    // Verifikasi determinisme: penulisan ulang menghasilkan string identik
    let kunci_teks_1 = kunci.ke_string();
    let kunci_teks_2 = kunci_muat.ke_string();
    assert_eq!(kunci_teks_1, kunci_teks_2);

    let _ = fs::remove_dir_all(&dir);
}

#[test]
fn tes_resolusi_semver_kompleks_dan_transitif() {
    let mut indeks = IndeksPaket::baru();

    // A -> B (^1.0), C (^2.0)
    // B -> D (^1.2)
    // C -> D (>=1.0, <2.0)
    // D rilis: 1.0.0, 1.2.0, 1.5.0, 2.0.0

    indeks.daftarkan_rilis(
        "dep_b",
        InfoRilis {
            versi: Versi::baru(1, 0, 0),
            dependensi: BTreeMap::from([("dep_d".into(), KebutuhanVersi::urai("^1.2").unwrap())]),
            checksum: "hash_b".into(),
        },
    );

    indeks.daftarkan_rilis(
        "dep_c",
        InfoRilis {
            versi: Versi::baru(2, 0, 0),
            dependensi: BTreeMap::from([(
                "dep_d".into(),
                KebutuhanVersi::urai(">=1.0, <2.0").unwrap(),
            )]),
            checksum: "hash_c".into(),
        },
    );

    indeks.daftarkan_rilis(
        "dep_d",
        InfoRilis {
            versi: Versi::baru(1, 0, 0),
            dependensi: BTreeMap::new(),
            checksum: "d100".into(),
        },
    );
    indeks.daftarkan_rilis(
        "dep_d",
        InfoRilis {
            versi: Versi::baru(1, 2, 0),
            dependensi: BTreeMap::new(),
            checksum: "d120".into(),
        },
    );
    indeks.daftarkan_rilis(
        "dep_d",
        InfoRilis {
            versi: Versi::baru(1, 5, 0),
            dependensi: BTreeMap::new(),
            checksum: "d150".into(),
        },
    );
    indeks.daftarkan_rilis(
        "dep_d",
        InfoRilis {
            versi: Versi::baru(2, 0, 0),
            dependensi: BTreeMap::new(),
            checksum: "d200".into(),
        },
    );

    let solver = PenyelesaiDependensi::baru(&indeks);
    let akar = BTreeMap::from([
        ("dep_b".into(), SumberDependensi::Registry(KebutuhanVersi::urai("^1.0").unwrap())),
        ("dep_c".into(), SumberDependensi::Registry(KebutuhanVersi::urai("^2.0").unwrap())),
    ]);

    let hasil = solver.selesaikan(&akar).unwrap();
    let (v_d, chk_d, _) = hasil.paket_terpilih.get("dep_d").unwrap();

    // Versi tertinggi yang memenuhi ^1.2 DAN >=1.0, <2.0 adalah 1.5.0
    assert_eq!(*v_d, Versi::baru(1, 5, 0));
    assert_eq!(chk_d, "d150");
}

#[test]
fn tes_verifikasi_integritas_checksum_sha256() {
    let dir = siapkan_direktori_uji("checksum");
    let cache = ManajerCache::di_jalur(dir.clone());

    let paket_dir = dir.join("proyek_sumber");
    fs::create_dir_all(paket_dir.join("src")).unwrap();
    fs::write(
        paket_dir.join("komo.toml"),
        "[paket]\nnama = \"kripto_aman\"\nversi = \"1.0.0\"\n",
    )
    .unwrap();
    fs::write(paket_dir.join("src/lib.komo"), "paket kripto_aman\n").unwrap();

    let arsip_sah = kemas_arsip_komopaket(&paket_dir).unwrap();
    let hash_sah = sha256_hex(&arsip_sah);
    let versi = Versi::baru(1, 0, 0);

    // 1. Ekstraksi dengan hash sah: harus sukses
    let res_sukses = cache.pasang_dan_verifikasi("kripto_aman", &versi, &arsip_sah, &hash_sah);
    assert!(res_sukses.is_ok(), "Pemasangan dengan checksum sah harus berhasil");
    assert!(cache.apakah_tersedia("kripto_aman", &versi));

    // 2. Manipulasi data payload arsip (tampering)
    let mut arsip_rusak = arsip_sah.clone();
    let len = arsip_rusak.len();
    arsip_rusak[len - 1] ^= 0xFF; // ubah bit terakhir

    let res_gagal = cache.pasang_dan_verifikasi("kripto_rusak", &versi, &arsip_rusak, &hash_sah);
    assert!(
        res_gagal.is_err(),
        "Pemasangan dengan data yang diubah harus ditolak oleh verifikasi checksum"
    );
    let err_msg = res_gagal.err().unwrap();
    assert!(err_msg.contains("Integritas paket"));
    assert!(err_msg.contains("Checksum tidak cocok"));

    let _ = fs::remove_dir_all(&dir);
}

#[test]
fn tes_alur_penerbitan_paket_komopaket_end_to_end() {
    let dir = siapkan_direktori_uji("terbit_e2e");
    let proyek = dir.join("pustaka_pesan");
    fs::create_dir_all(proyek.join("src")).unwrap();

    let toml = r#"[paket]
nama = "pustaka_pesan"
versi = "2.3.1"
penulis = ["Dhafa Nazula Permadi <kangpcode@gmail.com>"]
lisensi = "Apache-2.0"
deskripsi = "Pustaka pemformatan pesan bahasa Indonesia"
"#;
    fs::write(proyek.join("komo.toml"), toml).unwrap();
    fs::write(
        proyek.join("src/pesan.komo"),
        "paket pustaka_pesan\npublik fungsi sapa() teks { kembali \"Halo\" }\n",
    )
    .unwrap();

    let mut registry = RegistryMock::baru();
    let token = "kmp_token_pengembang_sah_2026";

    // 1. Penerbitan paket
    let hasil_terbit = terbitkan_paket_ke_registry(&proyek, token, &mut registry);
    assert!(hasil_terbit.is_ok(), "Penerbitan harus sukses: {:?}", hasil_terbit);

    let info = hasil_terbit.unwrap();
    assert_eq!(info.nama, "pustaka_pesan");
    assert_eq!(info.versi, Versi::baru(2, 3, 1));
    assert_eq!(info.checksum_sha256.len(), 64);
    assert!(info.ukuran_byte > 0);

    // 2. Pencarian paket di registry
    let hasil_cari = registry.cari("pesan");
    assert_eq!(hasil_cari.len(), 1);
    assert_eq!(hasil_cari[0].nama, "pustaka_pesan");
    assert_eq!(hasil_cari[0].versi_terbaru, Versi::baru(2, 3, 1));

    // 3. Pengunduhan dan pemasangan dari registry ke cache lokal
    let arsip_terunduh = registry
        .unduh_arsip("pustaka_pesan", &Versi::baru(2, 3, 1))
        .unwrap();

    let cache = ManajerCache::di_jalur(dir.join("cache"));
    let jalur_sumber = cache
        .pasang_dan_verifikasi(
            "pustaka_pesan",
            &Versi::baru(2, 3, 1),
            arsip_terunduh,
            &info.checksum_sha256,
        )
        .unwrap();

    assert!(jalur_sumber.join("komo.toml").exists());
    assert!(jalur_sumber.join("src/pesan.komo").exists());

    // 4. Verifikasi reproduktibilitas: kemas ulang harus menghasilkan checksum identik
    let arsip_ulang = kemas_arsip_komopaket(&jalur_sumber).unwrap();
    let hash_ulang = sha256_hex(&arsip_ulang);
    assert_eq!(info.checksum_sha256, hash_ulang);

    let _ = fs::remove_dir_all(&dir);
}

#[test]
fn tes_monorepo_ruang_kerja_multi_anggota() {
    let dir = siapkan_direktori_uji("workspace_dag");

    // Struktur Monorepo:
    // root/
    //   komo.toml ([ruang_kerja] anggota = ["inti", "layanan", "antarmuka"])
    //   inti/ (tanpa dependensi internal)
    //   layanan/ (bergantung pada inti)
    //   antarmuka/ (bergantung pada layanan dan inti)

    fs::create_dir_all(dir.join("inti/src")).unwrap();
    fs::create_dir_all(dir.join("layanan/src")).unwrap();
    fs::create_dir_all(dir.join("antarmuka/src")).unwrap();

    fs::write(
        dir.join("komo.toml"),
        "[ruang_kerja]\nanggota = [\"inti\", \"layanan\", \"antarmuka\"]\n",
    )
    .unwrap();

    fs::write(
        dir.join("inti/komo.toml"),
        "[paket]\nnama = \"inti\"\nversi = \"0.1.0\"\n",
    )
    .unwrap();

    fs::write(
        dir.join("layanan/komo.toml"),
        "[paket]\nnama = \"layanan\"\nversi = \"0.1.0\"\n[dependensi]\n\"inti\" = { path = \"../inti\" }\n",
    )
    .unwrap();

    fs::write(
        dir.join("antarmuka/komo.toml"),
        "[paket]\nnama = \"antarmuka\"\nversi = \"0.1.0\"\n[dependensi]\n\"layanan\" = { path = \"../layanan\" }\n\"inti\" = { path = \"../inti\" }\n",
    )
    .unwrap();

    let rk = RuangKerja::muat(&dir).unwrap();
    assert_eq!(rk.anggota.len(), 3);

    let urutan = rk.urutan_kompilasi_topologis().unwrap();

    let idx_inti = urutan.iter().position(|x| x == "inti").unwrap();
    let idx_layanan = urutan.iter().position(|x| x == "layanan").unwrap();
    let idx_antarmuka = urutan.iter().position(|x| x == "antarmuka").unwrap();

    assert!(idx_inti < idx_layanan, "inti harus dibangun sebelum layanan");
    assert!(idx_layanan < idx_antarmuka, "layanan harus dibangun sebelum antarmuka");

    let _ = fs::remove_dir_all(&dir);
}
