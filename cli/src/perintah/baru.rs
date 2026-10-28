// Copyright (c) kangpcode (Dhafa Nazula Permadi). Licensed under MIT or Apache-2.0.

//! Perintah `komo baru <nama>` untuk inisialisasi scaffold proyek Komolang baru.

use std::fs;
use std::path::Path;

/// Menjalankan inisialisasi scaffold proyek Komolang baru.
pub fn buat_proyek_baru(nama: &str) -> Result<(), String> {
    let jalur_proyek = Path::new(nama);
    if jalur_proyek.exists() {
        return Err(format!("Direktori '{}' sudah ada. Pilih nama lain.", nama));
    }

    // 1. Buat hierarki direktori
    fs::create_dir_all(jalur_proyek.join("src"))
        .map_err(|e| format!("Gagal membuat direktori src: {}", e))?;
    fs::create_dir_all(jalur_proyek.join("tests"))
        .map_err(|e| format!("Gagal membuat direktori tests: {}", e))?;

    // 2. Berkas komo.toml
    let komo_toml = format!(
        r#"# Berkas konfigurasi proyek Komolang
[paket]
nama = "{nama}"
versi = "0.1.0"
penulis = ["Pengembang Komolang"]
lisensi = "MIT ATAU Apache-2.0"
deskripsi = "Aplikasi Komolang mandiri"

[dependensi]
"#
    );
    fs::write(jalur_proyek.join("komo.toml"), komo_toml)
        .map_err(|e| format!("Gagal membuat komo.toml: {}", e))?;

    // 3. Berkas src/utama.komo
    let sumber_utama = format!(
        r#"// Copyright (c) 2026 {nama}. Dilisensikan di bawah MIT atau Apache-2.0.

//! Titik masuk utama aplikasi {nama}.

paket utama

impor "std/cetak"

publik fungsi utama() {{
    cetak.baris("Halo dari proyek {nama} di Komolang!")
}}
"#
    );
    fs::write(jalur_proyek.join("src").join("utama.komo"), sumber_utama)
        .map_err(|e| format!("Gagal membuat src/utama.komo: {}", e))?;

    // 4. Berkas tests/tes_utama.komo
    let sumber_tes = format!(
        r#"// Copyright (c) 2026 {nama}. Dilisensikan di bawah MIT atau Apache-2.0.

//! Pengujian unit bawaan untuk proyek {nama}.

paket tes

impor "std/uji"

publik fungsi uji_halo() {{
    uji.tegaskan_benar(benar, "pengujian_inisial")
}}
"#
    );
    fs::write(jalur_proyek.join("tests").join("tes_utama.komo"), sumber_tes)
        .map_err(|e| format!("Gagal membuat tests/tes_utama.komo: {}", e))?;

    // 5. Berkas .gitignore
    let gitignore = r#"# Komolang build artifacts
target/
*.o
*.s
*.bin
*.log
"#;
    fs::write(jalur_proyek.join(".gitignore"), gitignore)
        .map_err(|e| format!("Gagal membuat .gitignore: {}", e))?;

    println!("✔ Proyek Komolang '{}' berhasil dibuat!", nama);
    println!();
    println!("Mulai berkarya dengan perintah berikut:");
    println!("  cd {}", nama);
    println!("  komo jalankan src/utama.komo");
    println!("  komo bangun src/utama.komo");
    println!("  komo uji");

    Ok(())
}
