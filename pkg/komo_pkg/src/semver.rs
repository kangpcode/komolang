// Copyright (c) kangpcode (Dhafa Nazula Permadi). Licensed under MIT or Apache-2.0.

//! Pengurai dan pembanding versi semantik (Semantic Versioning 2.0.0).

use std::cmp::Ordering;
use std::fmt;

/// Representasi versi semantik terurai: `major.minor.patch[-prerelease][+build]`.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Versi {
    pub major: u64,
    pub minor: u64,
    pub patch: u64,
    pub prerelease: Option<String>,
    pub build: Option<String>,
}

impl Versi {
    pub fn baru(major: u64, minor: u64, patch: u64) -> Self {
        Self {
            major,
            minor,
            patch,
            prerelease: None,
            build: None,
        }
    }

    /// Mengurai string semver seperti "1.2.3", "0.4.0-rc.1", atau "2.1.0+build.42".
    pub fn urai(s: &str) -> Result<Self, String> {
        let s = s.trim();
        if s.is_empty() {
            return Err("String versi kosong".into());
        }

        // Pisahkan build metadata (+)
        let (sisa, build) = if let Some(pos) = s.find('+') {
            (&s[..pos], Some(s[pos + 1..].to_string()))
        } else {
            (s, None)
        };

        // Pisahkan prerelease (-)
        let (versi_utama, prerelease) = if let Some(pos) = sisa.find('-') {
            (&sisa[..pos], Some(sisa[pos + 1..].to_string()))
        } else {
            (sisa, None)
        };

        let bagian: Vec<&str> = versi_utama.split('.').collect();
        if bagian.is_empty() || bagian.len() > 3 {
            return Err(format!(
                "Format versi tidak valid: '{}' (diharapkan X.Y.Z atau X.Y atau X)",
                s
            ));
        }

        let major = bagian[0]
            .parse::<u64>()
            .map_err(|_| format!("Major bukan angka: '{}'", bagian[0]))?;
        let minor = if bagian.len() > 1 {
            bagian[1]
                .parse::<u64>()
                .map_err(|_| format!("Minor bukan angka: '{}'", bagian[1]))?
        } else {
            0
        };
        let patch = if bagian.len() > 2 {
            bagian[2]
                .parse::<u64>()
                .map_err(|_| format!("Patch bukan angka: '{}'", bagian[2]))?
        } else {
            0
        };

        Ok(Self {
            major,
            minor,
            patch,
            prerelease,
            build,
        })
    }
}

impl PartialOrd for Versi {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Versi {
    fn cmp(&self, other: &Self) -> Ordering {
        match self.major.cmp(&other.major) {
            Ordering::Equal => {}
            ord => return ord,
        }
        match self.minor.cmp(&other.minor) {
            Ordering::Equal => {}
            ord => return ord,
        }
        match self.patch.cmp(&other.patch) {
            Ordering::Equal => {}
            ord => return ord,
        }

        // SemVer 2.0: versi reguler lebih tinggi dari versi prerelease
        match (&self.prerelease, &other.prerelease) {
            (None, None) => Ordering::Equal,
            (Some(_), None) => Ordering::Less,
            (None, Some(_)) => Ordering::Greater,
            (Some(a), Some(b)) => a.cmp(b),
        }
    }
}

impl fmt::Display for Versi {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}.{}.{}", self.major, self.minor, self.patch)?;
        if let Some(ref pre) = self.prerelease {
            write!(f, "-{}", pre)?;
        }
        if let Some(ref b) = self.build {
            write!(f, "+{}", b)?;
        }
        Ok(())
    }
}

/// Batasan versi dependensi yang didukung oleh package manager Komolang.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum KebutuhanVersi {
    /// Bintang / versi apa pun: `*`
    Bebas,
    /// Versi persis: `=1.2.3` atau `1.2.3`
    Sama(Versi),
    /// Kompatibel semver (caret): `^1.2.3`
    /// (1.2.3 <= v < 2.0.0; atau 0.2.3 <= v < 0.3.0 jika 0.x.x)
    Kompatibel(Versi),
    /// Tilde (patch level): `~1.2.3` (1.2.3 <= v < 1.3.0)
    Tilde(Versi),
    /// Lebih besar sama dengan: `>=1.2.3`
    LebihBesarSama(Versi),
    /// Lebih kecil dari: `<2.0.0`
    LebihKecil(Versi),
    /// Rentang gabungan: `>=1.0.0, <2.0.0`
    Gabungan(Vec<KebutuhanVersi>),
}

impl KebutuhanVersi {
    /// Mengurai string ekspresi kebutuhan versi.
    pub fn urai(s: &str) -> Result<Self, String> {
        let s = s.trim();
        if s == "*" || s.is_empty() {
            return Ok(KebutuhanVersi::Bebas);
        }

        if s.contains(',') {
            let mut list = Vec::new();
            for part in s.split(',') {
                list.push(Self::urai(part.trim())?);
            }
            return Ok(KebutuhanVersi::Gabungan(list));
        }

        if let Some(sisa) = s.strip_prefix('^') {
            let v = Versi::urai(sisa)?;
            return Ok(KebutuhanVersi::Kompatibel(v));
        }

        if let Some(sisa) = s.strip_prefix('~') {
            let v = Versi::urai(sisa)?;
            return Ok(KebutuhanVersi::Tilde(v));
        }

        if let Some(sisa) = s.strip_prefix(">=") {
            let v = Versi::urai(sisa)?;
            return Ok(KebutuhanVersi::LebihBesarSama(v));
        }

        if let Some(sisa) = s.strip_prefix('<') {
            let v = Versi::urai(sisa)?;
            return Ok(KebutuhanVersi::LebihKecil(v));
        }

        if let Some(sisa) = s.strip_prefix('=') {
            let v = Versi::urai(sisa)?;
            return Ok(KebutuhanVersi::Sama(v));
        }

        // Default jika tanpa prefiks adalah kompatibel SemVer (misal "1.2.3" setara "^1.2.3")
        let v = Versi::urai(s)?;
        Ok(KebutuhanVersi::Kompatibel(v))
    }

    /// Memeriksa apakah sebuah `Versi` konkret memenuhi kebutuhan versi ini.
    pub fn cocok_dengan(&self, target: &Versi) -> bool {
        match self {
            KebutuhanVersi::Bebas => true,
            KebutuhanVersi::Sama(v) => target == v,
            KebutuhanVersi::LebihBesarSama(v) => target >= v,
            KebutuhanVersi::LebihKecil(v) => target < v,
            KebutuhanVersi::Tilde(v) => {
                if target < v {
                    return false;
                }
                target.major == v.major && target.minor == v.minor
            }
            KebutuhanVersi::Kompatibel(v) => {
                if target < v {
                    return false;
                }
                if v.major > 0 {
                    target.major == v.major
                } else if v.minor > 0 {
                    target.major == 0 && target.minor == v.minor
                } else {
                    target == v
                }
            }
            KebutuhanVersi::Gabungan(daftar) => {
                daftar.iter().all(|syarat| syarat.cocok_dengan(target))
            }
        }
    }
}

impl fmt::Display for KebutuhanVersi {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            KebutuhanVersi::Bebas => write!(f, "*"),
            KebutuhanVersi::Sama(v) => write!(f, "={}", v),
            KebutuhanVersi::Kompatibel(v) => write!(f, "^{}", v),
            KebutuhanVersi::Tilde(v) => write!(f, "~{}", v),
            KebutuhanVersi::LebihBesarSama(v) => write!(f, ">={}", v),
            KebutuhanVersi::LebihKecil(v) => write!(f, "<{}", v),
            KebutuhanVersi::Gabungan(list) => {
                let s = list
                    .iter()
                    .map(|k| k.to_string())
                    .collect::<Vec<_>>()
                    .join(", ");
                write!(f, "{}", s)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tes_urai_versi_dan_urut() {
        let v1 = Versi::urai("1.0.0").unwrap();
        let v2 = Versi::urai("1.2.0").unwrap();
        let v3 = Versi::urai("1.2.3").unwrap();
        let v4 = Versi::urai("2.0.0").unwrap();

        assert!(v1 < v2);
        assert!(v2 < v3);
        assert!(v3 < v4);
    }

    #[test]
    fn tes_kecocokan_kebutuhan_versi() {
        let req_caret = KebutuhanVersi::urai("^1.2.3").unwrap();
        assert!(req_caret.cocok_dengan(&Versi::urai("1.2.3").unwrap()));
        assert!(req_caret.cocok_dengan(&Versi::urai("1.2.9").unwrap()));
        assert!(req_caret.cocok_dengan(&Versi::urai("1.9.0").unwrap()));
        assert!(!req_caret.cocok_dengan(&Versi::urai("2.0.0").unwrap()));
        assert!(!req_caret.cocok_dengan(&Versi::urai("1.2.2").unwrap()));

        let req_tilde = KebutuhanVersi::urai("~1.2.3").unwrap();
        assert!(req_tilde.cocok_dengan(&Versi::urai("1.2.4").unwrap()));
        assert!(!req_tilde.cocok_dengan(&Versi::urai("1.3.0").unwrap()));

        let req_range = KebutuhanVersi::urai(">=1.0.0, <2.0.0").unwrap();
        assert!(req_range.cocok_dengan(&Versi::urai("1.5.0").unwrap()));
        assert!(!req_range.cocok_dengan(&Versi::urai("2.0.0").unwrap()));
    }
}
