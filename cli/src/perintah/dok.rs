// Copyright (c) kangpcode (Dhafa Nazula Permadi). Licensed under MIT or Apache-2.0.

//! Perintah `komo dok [jalur] [-o direktori_keluar]` untuk generator dokumentasi resmi Komolang.

use std::fs;
use std::path::{Path, PathBuf};

use komo_ast::*;
use komo_parser::urai;

/// Menghasilkan dokumentasi dari berkas atau direktori sumber Komolang.
pub fn buat_dokumentasi(sumber: &str, output_dir: &str) -> Result<(), String> {
    let p_sumber = Path::new(sumber);
    let p_out = PathBuf::from(output_dir);

    fs::create_dir_all(&p_out)
        .map_err(|e| format!("Gagal membuat direktori keluaran dokumentasi: {}", e))?;

    let mut berkas_list = Vec::new();
    if p_sumber.is_file() {
        berkas_list.push(p_sumber.to_path_buf());
    } else {
        kumpulkan_berkas_komo(p_sumber, &mut berkas_list);
    }

    let mut markdown_semua = String::from("# Dokumentasi Resmi Komolang\n\n");
    markdown_semua.push_str("Dokumentasi API dihasilkan otomatis oleh `komo dok`.\n\n");
    markdown_semua.push_str("## Indeks Modul\n\n");

    for b_path in &berkas_list {
        let kode = fs::read_to_string(b_path)
            .map_err(|e| format!("Gagal membaca '{:?}': {}", b_path, e))?;
        let hasil = urai(&kode);
        if !hasil.berhasil() {
            continue;
        }

        let berkas = hasil.berkas;
        let nama_paket = &berkas.paket.nama.nama;
        let rel_nama = b_path.file_name().and_then(|n| n.to_str()).unwrap_or("");

        markdown_semua.push_str(&format!("### Paket `{}` (`{}`)\n\n", nama_paket, rel_nama));

        for dek in &berkas.deklarasi {
            match dek {
                Deklarasi::Fungsi(f) => {
                    let vis = if f.visibilitas == Visibilitas::Publik {
                        "publik "
                    } else {
                        ""
                    };
                    let params = f
                        .parameter
                        .iter()
                        .map(|p| format!("{}: {:?}", p.nama.nama, p.tipe))
                        .collect::<Vec<_>>()
                        .join(", ");
                    markdown_semua.push_str(&format!(
                        "- **Fungsi:** `{}fungsi {}({})`\n",
                        vis, f.nama.nama, params
                    ));
                }
                Deklarasi::Tipe(t) => {
                    let vis = if t.visibilitas == Visibilitas::Publik {
                        "publik "
                    } else {
                        ""
                    };
                    markdown_semua.push_str(&format!("- **Tipe:** `{}tipe {}`\n", vis, t.nama.nama));
                }
                Deklarasi::Konstanta(k) => {
                    let vis = if k.visibilitas == Visibilitas::Publik {
                        "publik "
                    } else {
                        ""
                    };
                    markdown_semua.push_str(&format!("- **Konstanta:** `{}konst {}`\n", vis, k.nama.nama));
                }
                _ => {}
            }
        }
        markdown_semua.push('\n');
    }

    let out_file = p_out.join("DOKUMENTASI.md");
    fs::write(&out_file, &markdown_semua)
        .map_err(|e| format!("Gagal menulis berkas dokumentasi '{:?}': {}", out_file, e))?;

    // Buat juga versi HTML sederhana yang rapi
    let html_content = format!(
        r#"<!DOCTYPE html>
<html lang="id">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>Dokumentasi Komolang</title>
    <style>
        body {{ font-family: -apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, sans-serif; line-height: 1.6; max-width: 800px; margin: 40px auto; padding: 0 20px; color: #24292f; }}
        h1, h2, h3 {{ color: #0969da; border-bottom: 1px solid #d0d7de; padding-bottom: 0.3em; }}
        code {{ background-color: #f6f8fa; padding: 0.2em 0.4em; border-radius: 6px; font-family: monospace; font-size: 85%; }}
        ul {{ padding-left: 20px; }}
        li {{ margin-bottom: 6px; }}
        .header {{ margin-bottom: 30px; }}
    </style>
</head>
<body>
    <div class="header">
        <h1>Dokumentasi API Komolang</h1>
        <p>Dihasilkan otomatis oleh perkakas resmi <code>komo dok</code>.</p>
    </div>
    <pre><code>{}</code></pre>
</body>
</html>
"#,
        markdown_semua.replace('<', "&lt;").replace('>', "&gt;")
    );

    let html_out = p_out.join("index.html");
    fs::write(&html_out, html_content)
        .map_err(|e| format!("Gagal menulis berkas HTML '{:?}': {}", html_out, e))?;

    println!("✔ Dokumentasi berhasil dibuat di direktori '{:?}'", p_out);
    println!("  - Markdown: {:?}", out_file);
    println!("  - HTML:     {:?}", html_out);

    Ok(())
}

fn kumpulkan_berkas_komo(dir: &Path, list: &mut Vec<PathBuf>) {
    let entri = match fs::read_dir(dir) {
        Ok(e) => e,
        Err(_) => return,
    };
    for e in entri.flatten() {
        let p = e.path();
        if p.is_dir() {
            let nama = p.file_name().and_then(|n| n.to_str()).unwrap_or("");
            if nama != "target" && nama != ".git" {
                kumpulkan_berkas_komo(&p, list);
            }
        } else if p.extension().and_then(|s| s.to_str()) == Some("komo") {
            list.push(p);
        }
    }
}
