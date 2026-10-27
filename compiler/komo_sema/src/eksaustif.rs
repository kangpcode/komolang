// Copyright (c) kangpcode (Dhafa Nazula Permadi). Licensed under MIT or Apache-2.0.

//! Pengecekan eksaustif (kelengkapan cakupan varian) untuk pernyataan `cocok` (pattern matching).
//! Memastikan seluruh varian enum, tipe hasil, atau mungkin tertangani atau memiliki cabang `_`.

use crate::tipe::SemTipe;
use komo_ast::{CabangPola, JenisPola, Pola, Rentang};
use komo_diag::{kode, Diagnostik};
use std::collections::HashSet;

/// Memeriksa apakah seluruh cabang pada pernyataan `cocok` sudah eksaustif.
pub fn periksa_eksaustif(
    tipe_nilai: &SemTipe,
    cabang: &[CabangPola],
    rentang_cocok: Rentang,
) -> Result<(), Diagnostik> {
    // 1. Jika ada cabang penampung semua (wildcard `_` atau variabel penampung), maka pasti eksaustif.
    for c in cabang {
        if adalah_penampung_semua(&c.pola) {
            return Ok(());
        }
    }

    // 2. Periksa kelengkapan per jenis tipe
    match tipe_nilai {
        SemTipe::Hasil { .. } => {
            let mut sukses = false;
            let mut gagal = false;

            for c in cabang {
                let nama = nama_varian_pola(&c.pola);
                if let Some(n) = nama {
                    if n.eq_ignore_ascii_case("sukses") {
                        sukses = true;
                    } else if n.eq_ignore_ascii_case("gagal") {
                        gagal = true;
                    }
                }
            }

            let mut belum = Vec::new();
            if !sukses {
                belum.push("'sukses'");
            }
            if !gagal {
                belum.push("'gagal'");
            }

            if !belum.is_empty() {
                return Err(Diagnostik::galat(
                    kode::CABANG_TIDAK_EKSAUSTIF,
                    format!(
                        "pencocokan 'cocok' pada tipe 'hasil' belum eksaustif: varian {} belum ditangani",
                        belum.join(", ")
                    ),
                    rentang_cocok,
                )
                .dengan_saran(format!(
                    "tambahkan cabang untuk {} atau gunakan cabang penampung '_'",
                    belum.join(" dan ")
                )));
            }
        }

        SemTipe::Mungkin { .. } => {
            let mut ada = false;
            let mut kosong = false;

            for c in cabang {
                let nama = nama_varian_pola(&c.pola);
                if let Some(n) = nama {
                    if n.eq_ignore_ascii_case("ada") {
                        ada = true;
                    } else if n.eq_ignore_ascii_case("kosong") {
                        kosong = true;
                    }
                }
            }

            let mut belum = Vec::new();
            if !ada {
                belum.push("'ada'");
            }
            if !kosong {
                belum.push("'kosong'");
            }

            if !belum.is_empty() {
                return Err(Diagnostik::galat(
                    kode::CABANG_TIDAK_EKSAUSTIF,
                    format!(
                        "pencocokan 'cocok' pada tipe 'mungkin' belum eksaustif: varian {} belum ditangani",
                        belum.join(", ")
                    ),
                    rentang_cocok,
                )
                .dengan_saran(format!(
                    "tambahkan cabang untuk {} atau gunakan cabang penampung '_'",
                    belum.join(" dan ")
                )));
            }
        }

        SemTipe::Enum { nama, varian, .. } => {
            let mut varian_tercakup = HashSet::new();

            for c in cabang {
                if let Some(n) = nama_varian_pola(&c.pola) {
                    for (v, _) in varian {
                        if v.eq_ignore_ascii_case(&n) {
                            varian_tercakup.insert(v.clone());
                        }
                    }
                }
            }

            let belum_tercakup: Vec<&str> = varian
                .iter()
                .filter(|(v, _)| !varian_tercakup.contains(v))
                .map(|(v, _)| v.as_str())
                .collect();

            if !belum_tercakup.is_empty() {
                let daftar_belum = belum_tercakup
                    .iter()
                    .map(|v| format!("'{}'", v))
                    .collect::<Vec<_>>()
                    .join(", ");
                return Err(Diagnostik::galat(
                    kode::CABANG_TIDAK_EKSAUSTIF,
                    format!(
                        "pencocokan 'cocok' pada enum '{}' belum eksaustif: varian {} belum ditangani",
                        nama, daftar_belum
                    ),
                    rentang_cocok,
                )
                .dengan_saran(format!(
                    "tambahkan cabang untuk {} atau tambahkan pola penampung '_'",
                    daftar_belum
                )));
            }
        }

        SemTipe::Bool => {
            let mut benar = false;
            let mut salah = false;

            for c in cabang {
                if let JenisPola::Literal(e) = &c.pola.jenis {
                    if let komo_ast::JenisEkspresi::Bool(b) = e.jenis {
                        if b {
                            benar = true;
                        } else {
                            salah = true;
                        }
                    }
                }
            }

            if !benar || !salah {
                return Err(Diagnostik::galat(
                    kode::CABANG_TIDAK_EKSAUSTIF,
                    "pencocokan 'cocok' pada nilai 'bool' harus menangani kedua kasus 'benar' dan 'salah'",
                    rentang_cocok,
                )
                .dengan_saran("tambahkan cabang yang belum tercakup atau tambahkan '_'"));
            }
        }

        _ => {
            // Untuk tipe dengan domain tak terbatas (misal bulat, teks), tanpa `_` tidak mungkin eksaustif
            return Err(Diagnostik::galat(
                kode::CABANG_TIDAK_EKSAUSTIF,
                format!(
                    "pencocokan 'cocok' pada tipe '{}' memerlukan cabang penampung '_'",
                    tipe_nilai.nama()
                ),
                rentang_cocok,
            )
            .dengan_saran("tambahkan cabang '_: ...' untuk menangani nilai lainnya"));
        }
    }

    Ok(())
}

fn adalah_penampung_semua(pola: &Pola) -> bool {
    matches!(pola.jenis, JenisPola::Wildcard)
}

fn nama_varian_pola(pola: &Pola) -> Option<String> {
    match &pola.jenis {
        JenisPola::Varian { nama, .. } => Some(nama.nama.clone()),
        JenisPola::Pengenal(p) => Some(p.nama.clone()),
        _ => None,
    }
}
