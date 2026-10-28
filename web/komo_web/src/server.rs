// Copyright (c) kangpcode (Dhafa Nazula Permadi). Licensed under MIT or Apache-2.0.

//! Engine HTTP Server KomoWeb multi-thread dan runner pengembangan dengan hot-reload.

use std::collections::HashMap;
use std::fs;
use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::thread;
use std::time::{Duration, SystemTime};

use crate::http::{PermintaanHttp, ResponsHttp};
use crate::middleware::Middleware;
use crate::router::Router;
use crate::statis::PelayanBerkasStatis;
use crate::websocket::{buat_respons_handshake_ws, FrameWs};

/// Server aplikasi web KomoWeb.
pub struct ServerKomo {
    pub router: Arc<Router>,
    pub middleware: Vec<Arc<dyn Middleware>>,
    pub pelayan_statis: Option<(String, PelayanBerkasStatis)>,
}

impl ServerKomo {
    pub fn baru(router: Router) -> Self {
        Self {
            router: Arc::new(router),
            middleware: Vec::new(),
            pelayan_statis: None,
        }
    }

    /// Menambahkan middleware ke pipeline pemrosesan server.
    pub fn pasang_middleware(&mut self, m: impl Middleware + 'static) {
        self.middleware.push(Arc::new(m));
    }

    /// Mengaktifkan penyajian berkas statis pada prefiks jalur tertentu.
    pub fn layani_statis(&mut self, prefiks: &str, direktori: impl Into<PathBuf>) {
        let prefiks_bersih = prefiks.trim_end_matches('/').to_string();
        self.pelayan_statis = Some((prefiks_bersih, PelayanBerkasStatis::baru(direktori)));
    }

    /// Menjalankan server HTTP di thread saat ini (blocking).
    pub fn layani(&self, alamat: &str) -> Result<(), String> {
        let listener = TcpListener::bind(alamat)
            .map_err(|e| format!("Gagal mengikat soket TCP pada '{}': {}", alamat, e))?;

        println!("🚀 Server KomoWeb aktif dan melayani pada http://{}", alamat);

        for stream in listener.incoming() {
            match stream {
                Ok(s) => {
                    let router = self.router.clone();
                    let middleware = self.middleware.clone();
                    let statis = self.pelayan_statis.as_ref().map(|(p, s)| (p.clone(), s.direktori_dasar.clone()));

                    thread::spawn(move || {
                        tangani_koneksi(s, router, middleware, statis);
                    });
                }
                Err(_) => continue,
            }
        }

        Ok(())
    }

    /// Menjalankan server di thread latar belakang untuk pengujian terisolasi.
    pub fn layani_di_latar(
        &self,
        alamat_preferensi: &str,
    ) -> Result<(String, Arc<AtomicBool>), String> {
        let listener = TcpListener::bind(alamat_preferensi)
            .map_err(|e| format!("Gagal mengikat '{}': {}", alamat_preferensi, e))?;
        listener
            .set_nonblocking(true)
            .map_err(|e| format!("Gagal menyetel non-blocking: {}", e))?;

        let bound_addr = listener
            .local_addr()
            .map_err(|e| format!("Gagal mengambil local_addr: {}", e))?
            .to_string();

        let stop_flag = Arc::new(AtomicBool::new(false));
        let stop_clone = stop_flag.clone();

        let router = self.router.clone();
        let middleware = self.middleware.clone();
        let statis = self.pelayan_statis.as_ref().map(|(p, s)| (p.clone(), s.direktori_dasar.clone()));

        thread::spawn(move || {
            while !stop_clone.load(Ordering::SeqCst) {
                match listener.accept() {
                    Ok((s, _)) => {
                        let r = router.clone();
                        let m = middleware.clone();
                        let st = statis.clone();
                        thread::spawn(move || {
                            tangani_koneksi(s, r, m, st);
                        });
                    }
                    Err(ref e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                        thread::sleep(Duration::from_millis(10));
                    }
                    Err(_) => break,
                }
            }
        });

        Ok((bound_addr, stop_flag))
    }
}

fn tangani_koneksi(
    mut stream: TcpStream,
    router: Arc<Router>,
    middleware: Vec<Arc<dyn Middleware>>,
    statis_config: Option<(String, PathBuf)>,
) {
    let mut req = match PermintaanHttp::urai_dari_aliran(&mut stream) {
        Ok(r) => r,
        Err(_) => return,
    };

    if let Ok(peer) = stream.peer_addr() {
        req.alamat_klien = peer.ip().to_string();
    }

    // 1. Cek apakah permintaan upgrade ke WebSocket
    if let (Some(upgrade), Some(ws_key)) = (
        req.ambil_header("upgrade"),
        req.ambil_header("sec-websocket-key"),
    ) {
        if upgrade.eq_ignore_ascii_case("websocket") {
            let res = buat_respons_handshake_ws(ws_key);
            let _ = stream.write_all(&res.ke_bytes());
            let _ = stream.flush();

            // Sesi WebSocket loop echo sederhana
            let _ = stream.set_read_timeout(Some(Duration::from_millis(500)));
            let mut buf = [0u8; 1024];
            while let Ok(n) = stream.read(&mut buf) {
                if n == 0 {
                    break;
                }
                if let Ok((frame, _)) = FrameWs::dari_bytes(&buf[..n]) {
                    if frame.opcode == crate::websocket::OpcodeWs::Tutup {
                        let tutup = FrameWs::tutup().ke_bytes();
                        let _ = stream.write_all(&tutup);
                        break;
                    }
                    // Echo balik pesan teks
                    let echo = FrameWs::teks(&format!("Echo: {}", std::str::from_utf8(&frame.payload).unwrap_or("")));
                    let _ = stream.write_all(&echo.ke_bytes());
                }
            }
            return;
        }
    }

    // 2. Jalankan pipeline middleware
    let core_handler = move |r: &mut PermintaanHttp| -> ResponsHttp {
        // Cek berkas statis
        if let Some((ref prefiks, ref dir)) = statis_config {
            if r.jalur.starts_with(prefiks) {
                let sisa = r.jalur[prefiks.len()..].trim_start_matches('/');
                let pelayan = PelayanBerkasStatis::baru(dir);
                return pelayan.layani(r, sisa);
            }
        }

        // Cek perutean dinamis
        if let Some((penangan, params)) = router.cocokkan(r.metode, &r.jalur) {
            r.parameter_jalur = params;
            penangan(r)
        } else {
            ResponsHttp::tidak_ditemukan()
        }
    };

    let respons = jalankan_pipeline_middleware(&middleware, 0, &mut req, &core_handler);
    let _ = stream.write_all(&respons.ke_bytes());
    let _ = stream.flush();
}

fn jalankan_pipeline_middleware(
    middleware: &[Arc<dyn Middleware>],
    indeks: usize,
    req: &mut PermintaanHttp,
    handler_inti: &dyn Fn(&mut PermintaanHttp) -> ResponsHttp,
) -> ResponsHttp {
    if indeks < middleware.len() {
        let m = &middleware[indeks];
        m.tangani(req, &|r| {
            jalankan_pipeline_middleware(middleware, indeks + 1, r, handler_inti)
        })
    } else {
        handler_inti(req)
    }
}

/// Runner pengembangan dengan pendeteksi perubahan berkas hot-reload.
pub struct HotReloadWatcher {
    pub direktori_pantau: PathBuf,
    pub mtime_terakhir: HashMap<PathBuf, SystemTime>,
}

impl HotReloadWatcher {
    pub fn baru(dir: impl Into<PathBuf>) -> Self {
        let mut watcher = Self {
            direktori_pantau: dir.into(),
            mtime_terakhir: HashMap::new(),
        };
        watcher.pindai_semua();
        watcher
    }

    fn pindai_semua(&mut self) {
        let mut tumpukan = vec![self.direktori_pantau.clone()];
        while let Some(current) = tumpukan.pop() {
            if let Ok(entri) = fs::read_dir(current) {
                for item in entri.flatten() {
                    let path = item.path();
                    if path.is_dir() {
                        tumpukan.push(path);
                    } else if path.extension().and_then(|s| s.to_str()) == Some("komo") {
                        if let Ok(meta) = path.metadata() {
                            if let Ok(mtime) = meta.modified() {
                                self.mtime_terakhir.insert(path, mtime);
                            }
                        }
                    }
                }
            }
        }
    }

    /// Memeriksa apakah terdapat berkas kode Komolang yang baru diubah.
    pub fn apakah_ada_perubahan(&mut self) -> bool {
        let mut berubah = false;
        let mut tumpukan = vec![self.direktori_pantau.clone()];

        while let Some(current) = tumpukan.pop() {
            if let Ok(entri) = fs::read_dir(current) {
                for item in entri.flatten() {
                    let path = item.path();
                    if path.is_dir() {
                        tumpukan.push(path);
                    } else if path.extension().and_then(|s| s.to_str()) == Some("komo") {
                        if let Ok(meta) = path.metadata() {
                            if let Ok(mtime) = meta.modified() {
                                if let Some(mtime_lama) = self.mtime_terakhir.get(&path) {
                                    if mtime > *mtime_lama {
                                        berubah = true;
                                        self.mtime_terakhir.insert(path, mtime);
                                    }
                                } else {
                                    berubah = true;
                                    self.mtime_terakhir.insert(path, mtime);
                                }
                            }
                        }
                    }
                }
            }
        }

        berubah
    }
}
