// Copyright (c) kangpcode (Dhafa Nazula Permadi). Licensed under MIT or Apache-2.0.

//! Sistem audit keamanan kode, pemindai supply-chain CVE, dan runtime sandboxing policy.

use std::fs;
use std::path::{Path, PathBuf};

/// Tingkat keparahan temuan keamanan.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum TingkatKeparahan {
    Info,
    Peringatan,
    Tinggi,
    Kritis,
}

/// Satu temuan hasil audit keamanan.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TemuanKeamanan {
    pub tingkat: TingkatKeparahan,
    pub kategori: String,
    pub pesan: String,
    pub berkas: String,
    pub baris: usize,
}

/// Laporan lengkap hasil pemindaian audit keamanan proyek.
#[derive(Debug, Clone)]
pub struct LaporanAudit {
    pub total_berkas: usize,
    pub temuan: Vec<TemuanKeamanan>,
    pub jumlah_kritis: usize,
    pub jumlah_tinggi: usize,
    pub jumlah_peringatan: usize,
}

impl LaporanAudit {
    pub fn baru() -> Self {
        Self {
            total_berkas: 0,
            temuan: Vec::new(),
            jumlah_kritis: 0,
            jumlah_tinggi: 0,
            jumlah_peringatan: 0,
        }
    }

    pub fn tambah_temuan(&mut self, t: TemuanKeamanan) {
        match t.tingkat {
            TingkatKeparahan::Kritis => self.jumlah_kritis += 1,
            TingkatKeparahan::Tinggi => self.jumlah_tinggi += 1,
            TingkatKeparahan::Peringatan => self.jumlah_peringatan += 1,
            TingkatKeparahan::Info => {}
        }
        self.temuan.push(t);
    }

    pub fn apakah_aman(&self) -> bool {
        self.jumlah_kritis == 0 && self.jumlah_tinggi == 0
    }
}

impl Default for LaporanAudit {
    fn default() -> Self {
        Self::baru()
    }
}

/// Aturan sandboxing runtime untuk membatasi kapabilitas program Komolang.
#[derive(Debug, Clone)]
pub struct KebijakanSandboxing {
    pub direktori_diizinkan: Vec<PathBuf>,
    pub izin_jaringan: bool,
    pub izin_subprocess: bool,
    pub batasan_memori_mb: Option<usize>,
}

impl KebijakanSandboxing {
    pub fn ketat(root_dir: PathBuf) -> Self {
        Self {
            direktori_diizinkan: vec![root_dir],
            izin_jaringan: false,
            izin_subprocess: false,
            batasan_memori_mb: Some(128),
        }
    }

    pub fn standar(root_dir: PathBuf) -> Self {
        Self {
            direktori_diizinkan: vec![root_dir],
            izin_jaringan: true,
            izin_subprocess: false,
            batasan_memori_mb: Some(512),
        }
    }

    /// Memeriksa apakah suatu jalur berkas diizinkan untuk diakses.
    pub fn periksa_akses_jalur(&self, jalur: &Path) -> Result<(), String> {
        let mut lolos = false;
        for diizinkan in &self.direktori_diizinkan {
            if jalur.starts_with(diizinkan) {
                lolos = true;
                break;
            }
        }

        // Cegah directory traversal "../"
        let komponen_str = jalur.to_string_lossy();
        if komponen_str.contains("../") || komponen_str.contains("..\\") {
            return Err(format!(
                "Pelanggaran Sandboxing: Upaya directory traversal terdeteksi pada '{:?}'",
                jalur
            ));
        }

        if !lolos {
            return Err(format!(
                "Pelanggaran Sandboxing: Akses ke jalur '{:?}' di luar ruang kerja yang diizinkan",
                jalur
            ));
        }

        Ok(())
    }

    /// Memeriksa izin akses jaringan.
    pub fn periksa_akses_jaringan(&self) -> Result<(), String> {
        if !self.izin_jaringan {
            Err("Pelanggaran Sandboxing: Akses soket jaringan diblokir oleh kebijakan keamanan".into())
        } else {
            Ok(())
        }
    }

    /// Memeriksa izin pemanggilan sub-proses eksternal.
    pub fn periksa_izin_subprocess(&self) -> Result<(), String> {
        if !self.izin_subprocess {
            Err("Pelanggaran Sandboxing: Pemanggilan sub-proses sistem diblokir oleh kebijakan keamanan".into())
        } else {
            Ok(())
        }
    }
}

/// Menjalankan pemindaian audit keamanan pada direktori proyek.
pub fn audit_proyek(jalur_proyek: &str) -> Result<LaporanAudit, String> {
    let mut laporan = LaporanAudit::baru();
    let root = Path::new(jalur_proyek);
    if !root.exists() {
        return Err(format!("Direktori proyek '{}' tidak ditemukan", jalur_proyek));
    }

    pindai_direktori_rekursif(root, &mut laporan)?;
    Ok(laporan)
}

fn pindai_direktori_rekursif(dir: &Path, laporan: &mut LaporanAudit) -> Result<(), String> {
    let entries = fs::read_dir(dir).map_err(|e| format!("Gagal membaca direktori: {}", e))?;

    for entry in entries.flatten() {
        let p = entry.path();
        if p.is_dir() {
            let nama = p.file_name().unwrap_or_default().to_string_lossy();
            if nama != ".git" && nama != "target" && nama != "node_modules" {
                pindai_direktori_rekursif(&p, laporan)?;
            }
        } else if p.is_file() {
            if let Some(ext) = p.extension() {
                if ext == "komo" {
                    laporan.total_berkas += 1;
                    pindai_berkas_komo(&p, laporan)?;
                } else if ext == "toml" && p.file_name().unwrap_or_default() == "komo.toml" {
                    pindai_manifes_komo(&p, laporan)?;
                }
            }
        }
    }

    Ok(())
}

fn pindai_berkas_komo(jalur: &Path, laporan: &mut LaporanAudit) -> Result<(), String> {
    let konten = match fs::read_to_string(jalur) {
        Ok(k) => k,
        Err(_) => return Ok(()),
    };

    let nama_berkas = jalur.to_string_lossy().to_string();

    for (no_baris, baris) in konten.lines().enumerate() {
        let baris_trim = baris.trim();

        // 1. Deteksi hardcoded secrets/passwords/tokens
        let lower = baris_trim.to_lowercase();
        if (lower.contains("password = \"")
            || lower.contains("kata_sandi = \"")
            || lower.contains("api_key = \"")
            || lower.contains("token_rahasia = \""))
            && !baris_trim.starts_with("//")
        {
            laporan.tambah_temuan(TemuanKeamanan {
                tingkat: TingkatKeparahan::Tinggi,
                kategori: "Kredensial Terbuka (Hardcoded Secret)".into(),
                pesan: "Ditemukan potensi kata sandi atau kunci API tersemat langsung di kode sumber".into(),
                berkas: nama_berkas.clone(),
                baris: no_baris + 1,
            });
        }

        // 2. Deteksi pemanggilan sistem berisiko jika ada os::eksekusi
        if lower.contains("os::eksekusi(") && !baris_trim.starts_with("//") {
            laporan.tambah_temuan(TemuanKeamanan {
                tingkat: TingkatKeparahan::Peringatan,
                kategori: "Injeksi Perintah Sistem".into(),
                pesan: "Pemanggilan sub-proses sistem berpotensi rentan terhadap injeksi perintah jika argumen tidak disanitasi".into(),
                berkas: nama_berkas.clone(),
                baris: no_baris + 1,
            });
        }
    }

    Ok(())
}

fn pindai_manifes_komo(jalur: &Path, laporan: &mut LaporanAudit) -> Result<(), String> {
    let konten = match fs::read_to_string(jalur) {
        Ok(k) => k,
        Err(_) => return Ok(()),
    };

    // Simulasi pemindaian database kerentanan supply-chain paket Komolang
    let daftar_paket_rentan = [
        ("paket_usang_lama", "< 0.5.0", "CVE-2025-1001: Buffer overflow pada pengurai teks"),
        ("kripto_bocor", "< 1.2.0", "CVE-2025-2002: Kunci enkripsi lemah pada PRNG"),
    ];

    for (nama_paket, versi_rentan, deskripsi) in daftar_paket_rentan {
        if konten.contains(&format!("{} =", nama_paket)) {
            laporan.tambah_temuan(TemuanKeamanan {
                tingkat: TingkatKeparahan::Kritis,
                kategori: "Supply Chain Vulnerability".into(),
                pesan: format!("Dependensi '{}' {} memiliki kerentanan: {}", nama_paket, versi_rentan, deskripsi),
                berkas: jalur.to_string_lossy().to_string(),
                baris: 1,
            });
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tes_sandboxing_akses_jalur_dan_traversal() {
        let root = PathBuf::from("/workspace/proyek");
        let sandbox = KebijakanSandboxing::ketat(root.clone());

        // Jalur di dalam root diizinkan
        let file_ok = root.join("src").join("utama.komo");
        assert!(sandbox.periksa_akses_jalur(&file_ok).is_ok());

        // Jalur di luar root ditolak
        let file_luar = PathBuf::from("/etc/passwd");
        assert!(sandbox.periksa_akses_jalur(&file_luar).is_err());

        // Directory traversal ditolak
        let file_traversal = root.join("..").join("rahasia.txt");
        assert!(sandbox.periksa_akses_jalur(&file_traversal).is_err());
    }

    #[test]
    fn tes_pemindai_audit_keamanan() {
        let temp_dir = std::env::temp_dir().join("tes_audit_keamanan_komolang");
        let _ = fs::create_dir_all(&temp_dir);

        let berkas_komo = temp_dir.join("berisiko.komo");
        let kode = r#"
// Berkas contoh pengujian
fungsi masuk() {
    biar password = "rahasia_terbuka_123"
    os::eksekusi("rm -rf /")
}
"#;
        fs::write(&berkas_komo, kode).unwrap();

        let laporan = audit_proyek(temp_dir.to_str().unwrap()).unwrap();
        assert_eq!(laporan.total_berkas, 1);
        assert!(!laporan.apakah_aman());
        assert!(laporan.jumlah_tinggi >= 1);
        assert!(laporan.jumlah_peringatan >= 1);

        let _ = fs::remove_dir_all(temp_dir);
    }
}
