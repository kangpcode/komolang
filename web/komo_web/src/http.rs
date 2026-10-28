// Copyright (c) kangpcode (Dhafa Nazula Permadi). Licensed under MIT or Apache-2.0.

//! Abstraksi protokol HTTP/1.1: Permintaan (Request), Respons (Response), Header, dan Status.

use std::collections::HashMap;
use std::fmt;
use std::io::{BufRead, BufReader, Read};

/// Metode permintaan HTTP yang didukung KomoWeb.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum MetodeHttp {
    Dapatkan, // GET
    Pos,      // POST
    Taruh,    // PUT
    Hapus,    // DELETE
    Tambal,   // PATCH
    Kepala,   // HEAD
    Opsi,     // OPTIONS
}

impl MetodeHttp {
    pub fn dari_str(s: &str) -> Option<Self> {
        match s.to_uppercase().as_str() {
            "GET" | "DAPATKAN" => Some(MetodeHttp::Dapatkan),
            "POST" | "POS" => Some(MetodeHttp::Pos),
            "PUT" | "TARUH" => Some(MetodeHttp::Taruh),
            "DELETE" | "HAPUS" => Some(MetodeHttp::Hapus),
            "PATCH" | "TAMBAL" => Some(MetodeHttp::Tambal),
            "HEAD" | "KEPALA" => Some(MetodeHttp::Kepala),
            "OPTIONS" | "OPSI" => Some(MetodeHttp::Opsi),
            _ => None,
        }
    }

    pub fn sebagai_str(&self) -> &'static str {
        match self {
            MetodeHttp::Dapatkan => "GET",
            MetodeHttp::Pos => "POST",
            MetodeHttp::Taruh => "PUT",
            MetodeHttp::Hapus => "DELETE",
            MetodeHttp::Tambal => "PATCH",
            MetodeHttp::Kepala => "HEAD",
            MetodeHttp::Opsi => "OPTIONS",
        }
    }
}

impl fmt::Display for MetodeHttp {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.sebagai_str())
    }
}

/// Kode status respons HTTP baku.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct KodeStatus(pub u16);

impl KodeStatus {
    pub const BERALIH_PROTOKOL: KodeStatus = KodeStatus(101);
    pub const OK: KodeStatus = KodeStatus(200);
    pub const DIBUAT: KodeStatus = KodeStatus(201);
    pub const DITERIMA: KodeStatus = KodeStatus(202);
    pub const TANPA_KONTEN: KodeStatus = KodeStatus(204);

    pub const PINDAH_PERMANEN: KodeStatus = KodeStatus(301);
    pub const DITEMUKAN: KodeStatus = KodeStatus(302);
    pub const TIDAK_BERUBAH: KodeStatus = KodeStatus(304);

    pub const PERMINTAAN_SALAH: KodeStatus = KodeStatus(400);
    pub const TIDAK_TEROTENTIKASI: KodeStatus = KodeStatus(401);
    pub const TERLARANG: KodeStatus = KodeStatus(403);
    pub const TIDAK_DITEMUKAN: KodeStatus = KodeStatus(404);
    pub const METODE_DITOLAK: KodeStatus = KodeStatus(405);
    pub const TERLALU_BANYAK_PERMINTAAN: KodeStatus = KodeStatus(429);

    pub const KESALAHAN_SERVER_INTERNAL: KodeStatus = KodeStatus(500);
    pub const TIDAK_DIIMPLEMENTASIKAN: KodeStatus = KodeStatus(501);
    pub const GERBANG_RUSAK: KodeStatus = KodeStatus(502);
    pub const LAYANAN_TAK_TERSEDIA: KodeStatus = KodeStatus(503);

    pub fn teks_penjelas(&self) -> &'static str {
        match self.0 {
            101 => "Switching Protocols",
            200 => "OK",
            201 => "Created",
            202 => "Accepted",
            204 => "No Content",
            301 => "Moved Permanently",
            302 => "Found",
            304 => "Not Modified",
            400 => "Bad Request",
            401 => "Unauthorized",
            403 => "Forbidden",
            404 => "Not Found",
            405 => "Method Not Allowed",
            429 => "Too Many Requests",
            500 => "Internal Server Error",
            501 => "Not Implemented",
            502 => "Bad Gateway",
            503 => "Service Unavailable",
            _ => "Status",
        }
    }
}

/// Struktur data permintaan HTTP masuk.
#[derive(Debug, Clone)]
pub struct PermintaanHttp {
    pub metode: MetodeHttp,
    pub jalur: String,
    pub parameter_jalur: HashMap<String, String>,
    pub kueri: HashMap<String, String>,
    pub header: HashMap<String, String>,
    pub muatan: Vec<u8>,
    pub alamat_klien: String,
}

impl PermintaanHttp {
    pub fn baru(metode: MetodeHttp, jalur_lengkap: &str) -> Self {
        let (jalur, kueri) = pisah_jalur_dan_kueri(jalur_lengkap);
        Self {
            metode,
            jalur,
            parameter_jalur: HashMap::new(),
            kueri,
            header: HashMap::new(),
            muatan: Vec::new(),
            alamat_klien: "127.0.0.1".to_string(),
        }
    }

    /// Mengambil nilai header (tidak peka huruf besar/kecil).
    pub fn ambil_header(&self, nama: &str) -> Option<&str> {
        let nama_lower = nama.to_lowercase();
        self.header
            .iter()
            .find(|(k, _)| k.to_lowercase() == nama_lower)
            .map(|(_, v)| v.as_str())
    }

    /// Mengambil muatan teks ber-pengkodean UTF-8.
    pub fn teks(&self) -> Result<&str, std::str::Utf8Error> {
        std::str::from_utf8(&self.muatan)
    }

    /// Membaca dan mengurai permintaan HTTP/1.1 dari aliran byte IO.
    pub fn urai_dari_aliran<R: Read>(aliran: &mut R) -> Result<Self, String> {
        let mut reader = BufReader::new(aliran);

        // 1. Baca baris permintaan awal (Request Line)
        let mut baris_pertama = String::new();
        if reader.read_line(&mut baris_pertama).unwrap_or(0) == 0 {
            return Err("Koneksi ditutup sebelum menerima permintaan".into());
        }

        let bagian_pertama: Vec<&str> = baris_pertama.trim().split_whitespace().collect();
        if bagian_pertama.len() < 2 {
            return Err(format!(
                "Baris permintaan HTTP tidak valid: '{}'",
                baris_pertama
            ));
        }

        let metode = MetodeHttp::dari_str(bagian_pertama[0])
            .ok_or_else(|| format!("Metode HTTP tidak dikenal: '{}'", bagian_pertama[0]))?;
        let jalur_raw = bagian_pertama[1];
        let (jalur, kueri) = pisah_jalur_dan_kueri(jalur_raw);

        // 2. Baca baris-baris Header
        let mut header = HashMap::new();
        let mut panjang_konten: usize = 0;

        loop {
            let mut baris_header = String::new();
            if reader.read_line(&mut baris_header).unwrap_or(0) == 0 {
                break;
            }
            let trim = baris_header.trim();
            if trim.is_empty() {
                break; // Akhir header
            }

            if let Some(pos) = trim.find(':') {
                let nama = trim[..pos].trim().to_string();
                let nilai = trim[pos + 1..].trim().to_string();
                if nama.eq_ignore_ascii_case("content-length") {
                    panjang_konten = nilai.parse::<usize>().unwrap_or(0);
                }
                header.insert(nama, nilai);
            }
        }

        // 3. Baca body muatan (jika ada)
        let mut muatan = vec![0u8; panjang_konten];
        if panjang_konten > 0 {
            reader
                .read_exact(&mut muatan)
                .map_err(|e| format!("Gagal membaca body permintaan: {}", e))?;
        }

        Ok(Self {
            metode,
            jalur,
            parameter_jalur: HashMap::new(),
            kueri,
            header,
            muatan,
            alamat_klien: "127.0.0.1".to_string(),
        })
    }
}

/// Struktur data respons HTTP keluar.
#[derive(Debug, Clone)]
pub struct ResponsHttp {
    pub status: KodeStatus,
    pub header: HashMap<String, String>,
    pub muatan: Vec<u8>,
}

impl ResponsHttp {
    pub fn baru(status: KodeStatus) -> Self {
        let mut res = Self {
            status,
            header: HashMap::new(),
            muatan: Vec::new(),
        };
        res.header.insert("Server".into(), "KomoWeb/0.1.0".into());
        res
    }

    /// Mengambil muatan teks ber-pengkodean UTF-8 dari respons.
    pub fn teks_muatan(&self) -> Result<&str, std::str::Utf8Error> {
        std::str::from_utf8(&self.muatan)
    }

    /// Membuat respons status 200 OK dengan muatan teks polos.
    pub fn teks(konten: &str) -> Self {
        let mut res = Self::baru(KodeStatus::OK);
        res.header
            .insert("Content-Type".into(), "text/plain; charset=utf-8".into());
        res.muatan = konten.as_bytes().to_vec();
        res
    }

    /// Membuat respons status 200 OK dengan konten JSON.
    pub fn json(json_str: &str) -> Self {
        let mut res = Self::baru(KodeStatus::OK);
        res.header
            .insert("Content-Type".into(), "application/json; charset=utf-8".into());
        res.muatan = json_str.as_bytes().to_vec();
        res
    }

    /// Membuat respons status 200 OK dengan dokumen HTML.
    pub fn html(html_str: &str) -> Self {
        let mut res = Self::baru(KodeStatus::OK);
        res.header
            .insert("Content-Type".into(), "text/html; charset=utf-8".into());
        res.muatan = html_str.as_bytes().to_vec();
        res
    }

    /// Membuat respons 404 Tidak Ditemukan.
    pub fn tidak_ditemukan() -> Self {
        Self::teks("404 Halaman Tidak Ditemukan (KomoWeb)")
            .status_kode(KodeStatus::TIDAK_DITEMUKAN)
    }

    /// Membuat respons 500 Kesalahan Server Internal.
    pub fn kesalahan_internal(pesan: &str) -> Self {
        Self::teks(&format!("500 Kesalahan Server: {}", pesan))
            .status_kode(KodeStatus::KESALAHAN_SERVER_INTERNAL)
    }

    /// Mengatur kode status respons.
    pub fn status_kode(mut self, status: KodeStatus) -> Self {
        self.status = status;
        self
    }

    /// Menambahkan header HTTP ke dalam respons.
    pub fn dengan_header(mut self, nama: &str, nilai: &str) -> Self {
        self.header.insert(nama.to_string(), nilai.to_string());
        self
    }

    /// Mengonversi respons menjadi byte HTTP/1.1 siap kirim melalui soket TCP.
    pub fn ke_bytes(&self) -> Vec<u8> {
        let mut out = Vec::new();
        let baris_status = format!(
            "HTTP/1.1 {} {}\r\n",
            self.status.0,
            self.status.teks_penjelas()
        );
        out.extend_from_slice(baris_status.as_bytes());

        let mut header_salinan = self.header.clone();
        if self.status.0 != 101 {
            header_salinan.insert("Content-Length".into(), self.muatan.len().to_string());
            header_salinan.insert("Connection".into(), "close".into());
        }

        for (k, v) in &header_salinan {
            out.extend_from_slice(format!("{}: {}\r\n", k, v).as_bytes());
        }
        out.extend_from_slice(b"\r\n");
        out.extend_from_slice(&self.muatan);

        out
    }
}

fn pisah_jalur_dan_kueri(s: &str) -> (String, HashMap<String, String>) {
    let mut kueri = HashMap::new();
    if let Some(pos) = s.find('?') {
        let jalur = s[..pos].to_string();
        let query_str = &s[pos + 1..];
        for pair in query_str.split('&') {
            if let Some(eq_pos) = pair.find('=') {
                let k = pair[..eq_pos].to_string();
                let v = pair[eq_pos + 1..].to_string();
                kueri.insert(k, v);
            } else if !pair.is_empty() {
                kueri.insert(pair.to_string(), String::new());
            }
        }
        (jalur, kueri)
    } else {
        (s.to_string(), kueri)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tes_penguraian_permintaan_http() {
        let raw_http = b"POST /api/pengguna?halaman=2&aktif=true HTTP/1.1\r\nHost: localhost:8080\r\nContent-Type: application/json\r\nContent-Length: 18\r\n\r\n{\"nama\": \"Komodo\"}";
        let mut cursor = &raw_http[..];

        let req = PermintaanHttp::urai_dari_aliran(&mut cursor).unwrap();
        assert_eq!(req.metode, MetodeHttp::Pos);
        assert_eq!(req.jalur, "/api/pengguna");
        assert_eq!(req.kueri.get("halaman").unwrap(), "2");
        assert_eq!(req.kueri.get("aktif").unwrap(), "true");
        assert_eq!(req.teks().unwrap(), "{\"nama\": \"Komodo\"}");
    }

    #[test]
    fn tes_pembuatan_respons_http() {
        let resp = ResponsHttp::json("{\"status\":\"ok\"}").dengan_header("X-Komo", "1.0");
        let bytes = resp.ke_bytes();
        let teks = String::from_utf8(bytes).unwrap();

        assert!(teks.starts_with("HTTP/1.1 200 OK\r\n"));
        assert!(teks.contains("application/json"));
        assert!(teks.contains("X-Komo: 1.0"));
        assert!(teks.ends_with("{\"status\":\"ok\"}"));
    }
}
