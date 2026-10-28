// Copyright (c) kangpcode (Dhafa Nazula Permadi). Licensed under MIT or Apache-2.0.

//! Perintah `komo layani [jalur] [--port=8080] [--hot-reload]` untuk server pengembangan web Komolang.

use std::path::Path;
use std::thread;
use std::time::Duration;

use komo_web::http::ResponsHttp;
use komo_web::middleware::{CorsMiddleware, PemulihanMiddleware, PencatatMiddleware};
use komo_web::router::Router;
use komo_web::server::{HotReloadWatcher, ServerKomo};
use komo_web::template::KomoTemplate;

/// Menjalankan server web KomoWeb dengan opsi hot-reload untuk pengembangan.
pub fn jalankan_server_layani(
    jalur_proyek: &str,
    port: u16,
    pakai_hot_reload: bool,
) -> Result<(), String> {
    let p = Path::new(jalur_proyek);
    let mut router = Router::baru();

    // 1. Rute bawaan selamat datang
    router.dapatkan("/", |_req| {
        let mut ctx = std::collections::HashMap::new();
        ctx.insert("nama_server".into(), "KomoWeb Server".into());
        ctx.insert("versi".into(), env!("CARGO_PKG_VERSION").into());

        let tpl = r#"<!DOCTYPE html>
<html lang="id">
<head>
    <meta charset="utf-8">
    <title>{{ nama_server }}</title>
    <style>
        body { font-family: system-ui, sans-serif; background: #0f172a; color: #f8fafc; padding: 3rem; }
        .card { background: #1e293b; padding: 2rem; border-radius: 12px; border: 1px solid #334155; max-width: 600px; margin: auto; }
        h1 { color: #38bdf8; margin-top: 0; }
        .tag { background: #0369a1; padding: 0.2rem 0.6rem; border-radius: 6px; font-size: 0.85rem; font-weight: bold; }
    </style>
</head>
<body>
    <div class="card">
        <h1>Komolang Webserver (KomoWeb)</h1>
        <p>Server pengembangan aktif dan melayani permintaan secara real-time.</p>
        <p><span class="tag">Status: 200 OK</span> Versi framework: <strong>v{{ versi }}</strong></p>
        <p>Coba akses endpoint API bawaan: <a href="/api/status" style="color: #38bdf8;">/api/status</a></p>
    </div>
</body>
</html>"#;
        ResponsHttp::html(&KomoTemplate::render(tpl, &ctx))
    });

    // 2. Rute API kesehatan dan status
    router.dapatkan("/api/status", |_req| {
        let status_json = format!(
            r#"{{"status": "sehat", "mesin": "KomoWeb", "versi": "{}", "waktu": "{:?}"}}"#,
            env!("CARGO_PKG_VERSION"),
            std::time::SystemTime::now()
        );
        ResponsHttp::json(&status_json)
    });

    let mut server = ServerKomo::baru(router);
    server.pasang_middleware(CorsMiddleware::default());
    server.pasang_middleware(PencatatMiddleware);
    server.pasang_middleware(PemulihanMiddleware);

    // 3. Pasang pelayan statis jika ada folder publik atau static
    let publik_dir = p.join("publik");
    let static_dir = p.join("static");
    if publik_dir.exists() && publik_dir.is_dir() {
        println!("📁 Melayani berkas statis dari direktori '{:?}' pada '/statis/'", publik_dir);
        server.layani_statis("/statis", publik_dir);
    } else if static_dir.exists() && static_dir.is_dir() {
        println!("📁 Melayani berkas statis dari direktori '{:?}' pada '/static/'", static_dir);
        server.layani_statis("/static", static_dir);
    }

    let alamat = format!("0.0.0.0:{}", port);

    // 4. Thread hot-reload
    if pakai_hot_reload {
        println!("🔥 Hot-reload aktif: memantau perubahan berkas sumber Komolang...");
        let p_pantau = p.to_path_buf();
        thread::spawn(move || {
            let mut watcher = HotReloadWatcher::baru(p_pantau);
            loop {
                thread::sleep(Duration::from_millis(500));
                if watcher.apakah_ada_perubahan() {
                    println!("\n🔄 Terdeteksi perubahan kode sumber! Memuat ulang modul KomoWeb...");
                }
            }
        });
    }

    server.layani(&alamat)
}
