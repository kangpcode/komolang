// Copyright (c) kangpcode (Dhafa Nazula Permadi). Licensed under MIT or Apache-2.0.

//! Pelayan berkas statis dengan deteksi MIME, ETag, dan proteksi directory traversal.

use std::fs;
use std::path::{Path, PathBuf};

use crate::http::{KodeStatus, PermintaanHttp, ResponsHttp};

/// Pelayan berkas statis untuk melayani aset web (CSS, JS, gambar, font, dll).
pub struct PelayanBerkasStatis {
    pub direktori_dasar: PathBuf,
}

impl PelayanBerkasStatis {
    pub fn baru(direktori_dasar: impl Into<PathBuf>) -> Self {
        Self {
            direktori_dasar: direktori_dasar.into(),
        }
    }

    /// Melayani permintaan berkas statis.
    pub fn layani(&self, permintaan: &PermintaanHttp, jalur_relatif: &str) -> ResponsHttp {
        // Cegah Directory Traversal Attack
        if jalur_relatif.contains("..") || jalur_relatif.starts_with('/') {
            return ResponsHttp::baru(KodeStatus::TERLARANG)
                .dengan_header("Content-Type", "text/plain")
                .dengan_muatan_teks("Akses ditolak: jalur berkas tidak aman");
        }

        let jalur_berkas = self.direktori_dasar.join(jalur_relatif);
        if !jalur_berkas.exists() || !jalur_berkas.is_file() {
            return ResponsHttp::tidak_ditemukan();
        }

        let konten = match fs::read(&jalur_berkas) {
            Ok(k) => k,
            Err(_) => return ResponsHttp::kesalahan_internal("Gagal membaca berkas statis"),
        };

        let mime = deteksi_mime(&jalur_berkas);
        let etag = format!("\"{:x}-{:x}\"", konten.len(), jalur_berkas.metadata().map(|m| m.len()).unwrap_or(0));

        // Pemeriksaan If-None-Match untuk HTTP 304 Not Modified
        if let Some(if_none_match) = permintaan.ambil_header("if-none-match") {
            if if_none_match.trim() == etag {
                return ResponsHttp::baru(KodeStatus::TIDAK_BERUBAH)
                    .dengan_header("ETag", &etag);
            }
        }

        let mut res = ResponsHttp::baru(KodeStatus::OK)
            .dengan_header("Content-Type", mime)
            .dengan_header("ETag", &etag)
            .dengan_header("Cache-Control", "public, max-age=3600");
        res.muatan = konten;
        res
    }
}

impl ResponsHttp {
    fn dengan_muatan_teks(mut self, teks: &str) -> Self {
        self.muatan = teks.as_bytes().to_vec();
        self
    }
}

fn deteksi_mime(jalur: &Path) -> &'static str {
    match jalur.extension().and_then(|s| s.to_str()).unwrap_or("") {
        "html" | "htm" => "text/html; charset=utf-8",
        "css" => "text/css; charset=utf-8",
        "js" | "mjs" => "application/javascript; charset=utf-8",
        "json" => "application/json; charset=utf-8",
        "svg" => "image/svg+xml",
        "png" => "image/png",
        "jpg" | "jpeg" => "image/jpeg",
        "ico" => "image/x-icon",
        "woff2" => "font/woff2",
        "wasm" => "application/wasm",
        "txt" => "text/plain; charset=utf-8",
        _ => "application/octet-stream",
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::http::MetodeHttp;

    #[test]
    fn tes_pelayan_berkas_statis() {
        let temp_dir = std::env::temp_dir().join("komo_tes_statis");
        let _ = fs::remove_dir_all(&temp_dir);
        fs::create_dir_all(&temp_dir).unwrap();

        fs::write(temp_dir.join("gaya.css"), "body { margin: 0; }").unwrap();

        let pelayan = PelayanBerkasStatis::baru(&temp_dir);
        let req = PermintaanHttp::baru(MetodeHttp::Dapatkan, "/gaya.css");

        let res = pelayan.layani(&req, "gaya.css");
        assert_eq!(res.status, KodeStatus::OK);
        assert_eq!(res.header.get("Content-Type").unwrap(), "text/css; charset=utf-8");
        assert!(res.header.contains_key("ETag"));

        // Tes 304 Not Modified
        let etag = res.header.get("ETag").unwrap().clone();
        let mut req_cached = PermintaanHttp::baru(MetodeHttp::Dapatkan, "/gaya.css");
        req_cached.header.insert("If-None-Match".into(), etag);

        let res_cached = pelayan.layani(&req_cached, "gaya.css");
        assert_eq!(res_cached.status, KodeStatus::TIDAK_BERUBAH);

        // Tes traversal attack
        let res_traversal = pelayan.layani(&req, "../rahasia.txt");
        assert_eq!(res_traversal.status, KodeStatus::TERLARANG);

        let _ = fs::remove_dir_all(&temp_dir);
    }
}
