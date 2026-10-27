// Copyright (c) kangpcode (Dhafa Nazula Permadi). Licensed under MIT or Apache-2.0.

//! Suite fuzzing parser dan uji ketahanan (stress test) untuk `komo_lexer` dan `komo_parser`.
//! Memastikan parser dan lexer tidak pernah panik (crash) saat menerima masukan acak,
//! penyarangan sangat dalam, maupun mutasi kode sumber yang rusak.

use komo_lexer::pindai;
use komo_parser::urai;

/// Generator bilangan pseudo-acak deterministik (Linear Congruential Generator sederhana)
/// agar tes fuzzing dapat diulang persis sama (reproducible) tanpa dependensi eksternal.
struct Lcg(u64);

impl Lcg {
    fn baru(benih: u64) -> Self {
        Self(benih)
    }

    fn berikutnya(&mut self) -> u32 {
        self.0 = self.0.wrapping_mul(6364136223846793005).wrapping_add(1);
        (self.0 >> 32) as u32
    }

    fn rentang(&mut self, min: usize, maks: usize) -> usize {
        if min >= maks {
            return min;
        }
        min + (self.berikutnya() as usize % (maks - min + 1))
    }

    fn karakter_utf8_acak(&mut self) -> char {
        const CHARS: &[char] = &[
            'a', 'b', 'z', 'A', 'Z', '_', '0', '9', ' ', '\t', '\n', '\r', '+', '-', '*', '/', '%',
            '=', '!', '<', '>', '.', ',', ':', ';', '(', ')', '[', ']', '{', '}', '"', '\'', '\\',
            '#', '$', '@', '`', 'α', 'β', 'γ', '🦎', '🌟', '中', '文',
        ];
        let idx = self.rentang(0, CHARS.len() - 1);
        CHARS[idx]
    }
}

#[test]
fn tes_fuzz_string_acak_tidak_pernah_panik() {
    let mut rng = Lcg::baru(123456789);

    for _ in 0..100 {
        let panjang = rng.rentang(1, 200);
        let mut sumber = String::with_capacity(panjang);
        for _ in 0..panjang {
            sumber.push(rng.karakter_utf8_acak());
        }

        // Invarian: urai tidak boleh panik
        let hasil = urai(&sumber);
        let _ = hasil.diagnostik.len();
    }
}

#[test]
fn tes_fuzz_token_acak_gabungan_kata_kunci() {
    let mut rng = Lcg::baru(987654321);
    const KATA: &[&str] = &[
        "paket",
        "impor",
        "fungsi",
        "kembali",
        "biar",
        "var",
        "konst",
        "jika",
        "lainnya",
        "selagi",
        "untuk",
        "dalam",
        "pilih",
        "kasus",
        "cocok",
        "tipe",
        "struktur",
        "antarmuka",
        "enum",
        "jalankan",
        "saluran",
        "tunda",
        "benar",
        "salah",
        "kosong",
        "pinjam",
        "pindah",
        "publik",
        "pribadi",
        "tidak_aman",
        "{",
        "}",
        "(",
        ")",
        "[",
        "]",
        "+",
        "-",
        "*",
        "/",
        "==",
        "!=",
        "<",
        ">",
        "=",
        ":",
        ",",
        "\n",
        "foo",
        "123",
        "\"teks\"",
        "'c'",
    ];

    for _ in 0..50 {
        let jumlah_kata = rng.rentang(5, 50);
        let mut sumber = String::new();
        for _ in 0..jumlah_kata {
            let k = KATA[rng.rentang(0, KATA.len() - 1)];
            sumber.push_str(k);
            sumber.push(' ');
        }

        let hasil = urai(&sumber);
        let _ = hasil.diagnostik.len();
    }
}

#[test]
fn tes_fuzz_mutasi_kode_valid() {
    let mut rng = Lcg::baru(42424242);
    let kode_dasar = include_str!("../../examples/sintaks_lengkap.komo");

    for _ in 0..50 {
        let mut chars: Vec<char> = kode_dasar.chars().collect();
        let jumlah_mutasi = rng.rentang(1, 10);

        for _ in 0..jumlah_mutasi {
            let pos = rng.rentang(0, chars.len() - 1);
            match rng.rentang(0, 3) {
                0 => {
                    // Hapus karakter
                    chars.remove(pos);
                }
                1 => {
                    // Sisipkan karakter acak
                    chars.insert(pos, rng.karakter_utf8_acak());
                }
                2 => {
                    // Timpa karakter
                    chars[pos] = rng.karakter_utf8_acak();
                }
                _ => {
                    // Sisipkan kurung acak
                    chars.insert(pos, if rng.rentang(0, 1) == 0 { '{' } else { '}' });
                }
            }
        }

        let sumber_mutasi: String = chars.into_iter().collect();
        let hasil = urai(&sumber_mutasi);
        let _ = hasil.diagnostik.len();
    }
}

#[test]
fn tes_stress_penyarangan_kurung_bulat_sangat_dalam() {
    for n in [10, 50, 100, 200, 500] {
        let sumber = format!("paket u\nbiar x = {}1{}\n", "(".repeat(n), ")".repeat(n));
        let hasil = urai(&sumber);
        let _ = hasil.berhasil();
    }
}

#[test]
fn tes_stress_penyarangan_kurung_kurawal_sangat_dalam() {
    for n in [10, 50, 100, 200] {
        let sumber = format!(
            "paket u\nfungsi f() {}{}{}\n",
            "{".repeat(n),
            "biar x = 1",
            "}".repeat(n)
        );
        let hasil = urai(&sumber);
        let _ = hasil.berhasil();
    }
}

#[test]
fn tes_stress_penyarangan_kurung_siku_larik_dalam() {
    for n in [10, 50, 100, 200] {
        let sumber = format!("paket u\nbiar x = {}1{}\n", "[".repeat(n), "]".repeat(n));
        let hasil = urai(&sumber);
        let _ = hasil.berhasil();
    }
}

#[test]
fn tes_stress_penyarangan_komentar_blok_dalam() {
    for n in [10, 50, 100, 300] {
        let sumber = format!("{} 42 {}", "/* ".repeat(n), " */".repeat(n));
        let hasil = pindai(&sumber);
        assert!(hasil.diagnostik.is_empty());
    }
}

#[test]
fn tes_stress_unari_berantai() {
    let sumber = format!("paket u\nbiar x = {}1\n", "-".repeat(100));
    let hasil = urai(&sumber);
    let _ = hasil.diagnostik.len();
}

#[test]
fn tes_stress_bukan_logika_berantai() {
    let sumber = format!("paket u\nbiar x = {}benar\n", "!".repeat(100));
    let hasil = urai(&sumber);
    let _ = hasil.diagnostik.len();
}

#[test]
fn tes_stress_ribuan_pernyataan_sebaris() {
    let mut sumber = String::from("paket u\nfungsi f() {\n");
    for i in 0..1000 {
        sumber.push_str(&format!("    biar v{} = {}\n", i, i));
    }
    sumber.push_str("}\n");
    let hasil = urai(&sumber);
    assert!(hasil.berhasil());
    match &hasil.berkas.deklarasi[0] {
        komo_ast::Deklarasi::Fungsi(f) => {
            assert_eq!(f.badan.pernyataan.len(), 1000);
        }
        _ => panic!(),
    }
}
