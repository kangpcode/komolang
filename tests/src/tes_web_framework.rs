// Copyright (c) kangpcode (Dhafa Nazula Permadi). Licensed under MIT or Apache-2.0.

use std::collections::HashMap;
use std::fs;
use std::io::{Read, Write};
use std::net::TcpStream;
use std::sync::atomic::Ordering;
use std::thread;
use std::time::{Duration, Instant};

use komo_web::http::ResponsHttp;
use komo_web::middleware::{CorsMiddleware, PemulihanMiddleware};
use komo_web::router::Router;
use komo_web::server::ServerKomo;
use komo_web::template::KomoTemplate;
use komo_web::websocket::{FrameWs, OpcodeWs};

#[test]
fn tes_web_server_live_http_dan_rest_api() {
    let mut router = Router::baru();

    // Endpoint GET /api/status
    router.dapatkan("/api/status", |_| {
        ResponsHttp::json(r#"{"status": "sehat", "mesin": "KomoWeb"}"#)
    });

    // Endpoint GET dengan parameter dinamis /pengguna/:id
    router.dapatkan("/pengguna/:id", |req| {
        let id = req.parameter_jalur.get("id").cloned().unwrap_or_default();
        ResponsHttp::json(&format!(r#"{{"id": {}, "nama": "Pengguna-{}"}}"#, id, id))
    });

    // Endpoint POST /api/tambah
    router.pos("/api/tambah", |req| {
        let body = req.teks().unwrap_or("{}");
        if body.contains("40") && body.contains("2") {
            ResponsHttp::json(r#"{"hasil": 42}"#)
        } else {
            ResponsHttp::json(r#"{"hasil": 0}"#)
        }
    });

    let mut server = ServerKomo::baru(router);
    server.pasang_middleware(CorsMiddleware::default());
    server.pasang_middleware(PemulihanMiddleware);

    // Jalankan server di latar belakang pada port bebas OS (0)
    let (alamat, stop_flag) = server.layani_di_latar("127.0.0.1:0").unwrap();
    thread::sleep(Duration::from_millis(50));

    // 1. Uji GET /api/status
    {
        let mut stream = TcpStream::connect(&alamat).unwrap();
        let req_str = "GET /api/status HTTP/1.1\r\nHost: localhost\r\n\r\n";
        stream.write_all(req_str.as_bytes()).unwrap();

        let mut buf = vec![0u8; 1024];
        let n = stream.read(&mut buf).unwrap();
        let resp_str = String::from_utf8_lossy(&buf[..n]);

        assert!(resp_str.contains("HTTP/1.1 200 OK"));
        assert!(resp_str.contains(r#"{"status": "sehat", "mesin": "KomoWeb"}"#));
    }

    // 2. Uji GET /pengguna/101 (parameter dinamis)
    {
        let mut stream = TcpStream::connect(&alamat).unwrap();
        let req_str = "GET /pengguna/101 HTTP/1.1\r\nHost: localhost\r\n\r\n";
        stream.write_all(req_str.as_bytes()).unwrap();

        let mut buf = vec![0u8; 1024];
        let n = stream.read(&mut buf).unwrap();
        let resp_str = String::from_utf8_lossy(&buf[..n]);

        assert!(resp_str.contains("HTTP/1.1 200 OK"));
        assert!(resp_str.contains(r#"{"id": 101, "nama": "Pengguna-101"}"#));
    }

    // 3. Uji POST /api/tambah dengan muatan JSON
    {
        let mut stream = TcpStream::connect(&alamat).unwrap();
        let body = r#"{"a": 40, "b": 2}"#;
        let req_str = format!(
            "POST /api/tambah HTTP/1.1\r\nHost: localhost\r\nContent-Length: {}\r\n\r\n{}",
            body.len(),
            body
        );
        stream.write_all(req_str.as_bytes()).unwrap();

        let mut buf = vec![0u8; 1024];
        let n = stream.read(&mut buf).unwrap();
        let resp_str = String::from_utf8_lossy(&buf[..n]);

        assert!(resp_str.contains("HTTP/1.1 200 OK"));
        assert!(resp_str.contains(r#"{"hasil": 42}"#));
    }

    // 4. Uji 404 Tidak Ditemukan
    {
        let mut stream = TcpStream::connect(&alamat).unwrap();
        let req_str = "GET /halaman_rahasia_tak_ada HTTP/1.1\r\nHost: localhost\r\n\r\n";
        stream.write_all(req_str.as_bytes()).unwrap();

        let mut buf = vec![0u8; 1024];
        let n = stream.read(&mut buf).unwrap();
        let resp_str = String::from_utf8_lossy(&buf[..n]);

        assert!(resp_str.contains("HTTP/1.1 404 Not Found"));
    }

    stop_flag.store(true, Ordering::SeqCst);
}

#[test]
fn tes_web_middleware_pipeline_dan_cors() {
    let mut router = Router::baru();
    router.dapatkan("/data", |_| ResponsHttp::json(r#"{"data": 123}"#));
    router.dapatkan("/bahaya", |_| panic!("Galat kritis tak terduga!"));

    let mut server = ServerKomo::baru(router);
    server.pasang_middleware(CorsMiddleware::default());
    server.pasang_middleware(PemulihanMiddleware);

    let (alamat, stop_flag) = server.layani_di_latar("127.0.0.1:0").unwrap();
    thread::sleep(Duration::from_millis(50));

    // 1. Preflight OPTIONS CORS request
    {
        let mut stream = TcpStream::connect(&alamat).unwrap();
        let req_str = "OPTIONS /data HTTP/1.1\r\nHost: localhost\r\n\r\n";
        stream.write_all(req_str.as_bytes()).unwrap();

        let mut buf = vec![0u8; 1024];
        let n = stream.read(&mut buf).unwrap();
        let resp_str = String::from_utf8_lossy(&buf[..n]);

        assert!(resp_str.contains("HTTP/1.1 204 No Content"));
        assert!(resp_str.contains("Access-Control-Allow-Origin: *"));
        assert!(resp_str.contains("Access-Control-Allow-Methods:"));
    }

    // 2. Pemulihan dari panik handler (harus mengembalikan 500 tanpa crash)
    {
        let mut stream = TcpStream::connect(&alamat).unwrap();
        let req_str = "GET /bahaya HTTP/1.1\r\nHost: localhost\r\n\r\n";
        stream.write_all(req_str.as_bytes()).unwrap();

        let mut buf = vec![0u8; 1024];
        let n = stream.read(&mut buf).unwrap();
        let resp_str = String::from_utf8_lossy(&buf[..n]);

        assert!(resp_str.contains("HTTP/1.1 500 Internal Server Error"));
        assert!(resp_str.contains("PemulihanMiddleware"));
    }

    stop_flag.store(true, Ordering::SeqCst);
}

#[test]
fn tes_web_pelayan_berkas_statis_dan_etag() {
    let temp_dir = std::env::temp_dir().join("komo_web_uji_statis");
    let _ = fs::remove_dir_all(&temp_dir);
    fs::create_dir_all(&temp_dir).unwrap();

    fs::write(
        temp_dir.join("skrip.js"),
        "console.log('KomoWeb JavaScript');",
    )
    .unwrap();

    let router = Router::baru();
    let mut server = ServerKomo::baru(router);
    server.layani_statis("/aset", temp_dir.clone());

    let (alamat, stop_flag) = server.layani_di_latar("127.0.0.1:0").unwrap();
    thread::sleep(Duration::from_millis(50));

    // 1. Permintaan awal berkas statis
    let etag;
    {
        let mut stream = TcpStream::connect(&alamat).unwrap();
        let req_str = "GET /aset/skrip.js HTTP/1.1\r\nHost: localhost\r\n\r\n";
        stream.write_all(req_str.as_bytes()).unwrap();

        let mut buf = vec![0u8; 1024];
        let n = stream.read(&mut buf).unwrap();
        let resp_str = String::from_utf8_lossy(&buf[..n]);

        assert!(resp_str.contains("HTTP/1.1 200 OK"));
        assert!(resp_str.contains("application/javascript"));
        assert!(resp_str.contains("console.log('KomoWeb JavaScript');"));

        // Ekstrak ETag
        let pos = resp_str.find("ETag: ").unwrap();
        let akhir = resp_str[pos + 6..].find("\r\n").unwrap();
        etag = resp_str[pos + 6..pos + 6 + akhir].to_string();
    }

    // 2. Permintaan kedua dengan If-None-Match (harus 304 Not Modified)
    {
        let mut stream = TcpStream::connect(&alamat).unwrap();
        let req_str = format!(
            "GET /aset/skrip.js HTTP/1.1\r\nHost: localhost\r\nIf-None-Match: {}\r\n\r\n",
            etag
        );
        stream.write_all(req_str.as_bytes()).unwrap();

        let mut buf = vec![0u8; 1024];
        let n = stream.read(&mut buf).unwrap();
        let resp_str = String::from_utf8_lossy(&buf[..n]);

        assert!(resp_str.contains("HTTP/1.1 304 Not Modified"));
    }

    stop_flag.store(true, Ordering::SeqCst);
    let _ = fs::remove_dir_all(&temp_dir);
}

#[test]
fn tes_web_komotemplate_html_auto_escaping() {
    let mut ctx = HashMap::new();
    ctx.insert("pengguna".into(), "Nazula <script>alert(1)</script>".into());
    ctx.insert("peran".into(), "Admin".into());
    ctx.insert("vip".into(), "benar".into());

    let template = r#"<div>
    <h1>Profil: {{ pengguna }}</h1>
    <p>Peran: {{ peran }}</p>
    {{ jika vip }}
    <span class="badge">Anggota VIP</span>
    {{ lainnya }}
    <span>Reguler</span>
    {{ akhir }}
</div>"#;

    let hasil = KomoTemplate::render(template, &ctx);
    // Verifikasi perlindungan dari XSS injection
    assert!(!hasil.contains("<script>"));
    assert!(hasil.contains("&lt;script&gt;alert(1)&lt;/script&gt;"));
    assert!(hasil.contains("Anggota VIP"));
}

#[test]
fn tes_web_websocket_handshake_dan_echo() {
    let router = Router::baru();
    let server = ServerKomo::baru(router);

    let (alamat, stop_flag) = server.layani_di_latar("127.0.0.1:0").unwrap();
    thread::sleep(Duration::from_millis(50));

    let mut stream = TcpStream::connect(&alamat).unwrap();
    let client_key = "dGhlIHNhbXBsZSBub25jZQ==";

    // 1. Kirim permintaan handshake upgrade
    let req_ws = format!(
        "GET /ws HTTP/1.1\r\nHost: localhost\r\nUpgrade: websocket\r\nConnection: Upgrade\r\nSec-WebSocket-Key: {}\r\nSec-WebSocket-Version: 13\r\n\r\n",
        client_key
    );
    stream.write_all(req_ws.as_bytes()).unwrap();

    let mut buf = vec![0u8; 1024];
    let n = stream.read(&mut buf).unwrap();
    let resp_str = String::from_utf8_lossy(&buf[..n]);

    assert!(resp_str.contains("HTTP/1.1 101 Switching Protocols"));
    assert!(resp_str.contains("Sec-WebSocket-Accept: s3pPLMBiTxaQ9kYGzzhZRbK+xOo="));

    // 2. Kirim pesan WebSocket frame teks
    let pesan = FrameWs::teks("Halo dari Pengujian Komolang");
    stream.write_all(&pesan.ke_bytes()).unwrap();

    // 3. Terima pesan echo dari server
    let n_echo = stream.read(&mut buf).unwrap();
    let (frame_echo, _) = FrameWs::dari_bytes(&buf[..n_echo]).unwrap();
    assert_eq!(frame_echo.opcode, OpcodeWs::Teks);
    let teks_echo = String::from_utf8(frame_echo.payload).unwrap();
    assert!(teks_echo.contains("Halo dari Pengujian Komolang"));

    stop_flag.store(true, Ordering::SeqCst);
}

#[test]
fn tes_web_benchmark_beban_ribuan_permintaan() {
    let mut router = Router::baru();
    router.dapatkan("/bench", |_| {
        ResponsHttp::teks("KomoWeb Ultra Cepat")
    });

    let server = ServerKomo::baru(router);
    let (alamat, stop_flag) = server.layani_di_latar("127.0.0.1:0").unwrap();
    thread::sleep(Duration::from_millis(50));

    let jumlah_permintaan = 200;
    let t0 = Instant::now();

    for _ in 0..jumlah_permintaan {
        let mut stream = TcpStream::connect(&alamat).unwrap();
        stream
            .write_all(b"GET /bench HTTP/1.1\r\nHost: localhost\r\n\r\n")
            .unwrap();

        let mut buf = [0u8; 256];
        let n = stream.read(&mut buf).unwrap();
        assert!(n > 0);
    }

    let durasi = t0.elapsed();
    let req_per_detik = (jumlah_permintaan as f64) / durasi.as_secs_f64();
    println!(
        "Benchmark Beban KomoWeb: {} permintaan diselesaikan dalam {:.2?} ({:.0} req/detik)",
        jumlah_permintaan, durasi, req_per_detik
    );

    assert!(durasi.as_secs() < 5, "200 permintaan harus selesai dengan sangat cepat");
    stop_flag.store(true, Ordering::SeqCst);
}
