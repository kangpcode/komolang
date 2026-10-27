// Copyright (c) kangpcode (Dhafa Nazula Permadi). Licensed under MIT or Apache-2.0.

//! Verifier KIR untuk memvalidasi integritas representasi SSA.
//! Memastikan bentuk SSA sah, setiap blok berakhiran tepat satu terminator,
//! dominansi definisi nilai sebelum penggunaan, dan konsistensi tipe.

use std::collections::{HashMap, HashSet};
use crate::fungsi::FungsiIr;
use crate::instruksi::{Instruksi, OpBiner, OpUnari, Terminator};
use crate::modul::ModulIr;
use crate::nilai::{IdBlok, IdNilai};
use crate::tipe::IrTipe;

/// Kesalahan verifikasi pada KIR.
#[derive(Debug, Clone, PartialEq)]
pub struct GalatVerifikasi {
    pub fungsi: String,
    pub blok: Option<IdBlok>,
    pub pesan: String,
}

impl std::fmt::Display for GalatVerifikasi {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if let Some(b) = self.blok {
            write!(f, "[KIR Verifier] Fungsi '@{}', Blok {}: {}", self.fungsi, b, self.pesan)
        } else {
            write!(f, "[KIR Verifier] Fungsi '@{}': {}", self.fungsi, self.pesan)
        }
    }
}

/// Memverifikasi seluruh modul KIR.
pub fn verifikasi_modul(modul: &ModulIr) -> Result<(), Vec<GalatVerifikasi>> {
    let mut semua_galat = Vec::new();

    for f in &modul.fungsi {
        if let Err(galat_fungsi) = verifikasi_fungsi(f) {
            semua_galat.extend(galat_fungsi);
        }
    }

    if semua_galat.is_empty() {
        Ok(())
    } else {
        Err(semua_galat)
    }
}

/// Memverifikasi satu fungsi KIR.
pub fn verifikasi_fungsi(f: &FungsiIr) -> Result<(), Vec<GalatVerifikasi>> {
    let mut galat = Vec::new();
    let nama_fungsi = f.nama.clone();

    // 1. Periksa keberadaan blok awal
    let id_semua_blok: HashSet<IdBlok> = f.blok.iter().map(|b| b.id).collect();
    if !id_semua_blok.contains(&f.blok_awal) {
        galat.push(GalatVerifikasi {
            fungsi: nama_fungsi.clone(),
            blok: None,
            pesan: format!("Blok awal {} tidak ditemukan di dalam fungsi", f.blok_awal),
        });
    }

    // 2. Kumpulkan pendahulu tiap blok
    let pendahulu = f.hitung_pendahulu();

    // 3. Lacak definisi nilai untuk memeriksa keunikan SSA dan tipe
    let mut tipe_nilai: HashMap<IdNilai, IrTipe> = HashMap::new();
    let mut terdefinisi: HashSet<IdNilai> = HashSet::new();

    // Parameter fungsi terdefinisi di awal
    for p in &f.parameter {
        if !terdefinisi.insert(p.nilai) {
            galat.push(GalatVerifikasi {
                fungsi: nama_fungsi.clone(),
                blok: Some(f.blok_awal),
                pesan: format!("ID nilai parameter {} didefinisikan lebih dari sekali", p.nilai),
            });
        }
        tipe_nilai.insert(p.nilai, p.tipe.clone());
    }

    // 4. Periksa setiap blok dasar
    for b in &f.blok {
        // Blok harus memiliki terminator
        let term = match &b.terminator {
            Some(t) => t,
            None => {
                galat.push(GalatVerifikasi {
                    fungsi: nama_fungsi.clone(),
                    blok: Some(b.id),
                    pesan: "Blok dasar tidak diakhiri dengan terminator".to_string(),
                });
                continue;
            }
        };

        // Periksa apakah penerus terminator valid
        for p in term.ambil_penerus() {
            if !id_semua_blok.contains(&p) {
                galat.push(GalatVerifikasi {
                    fungsi: nama_fungsi.clone(),
                    blok: Some(b.id),
                    pesan: format!("Terminator merujuk ke blok penerus {} yang tidak ada", p),
                });
            }
        }

        // Periksa tipe kondisi pada percabangan
        if let Terminator::Cabang { kondisi, .. } = term {
            if let Some(t_kondisi) = tipe_nilai.get(kondisi) {
                if *t_kondisi != IrTipe::I1 {
                    galat.push(GalatVerifikasi {
                        fungsi: nama_fungsi.clone(),
                        blok: Some(b.id),
                        pesan: format!("Kondisi percabangan {} harus bertipe 'i1', ditemukan '{}'", kondisi, t_kondisi),
                    });
                }
            }
        }

        // Periksa tipe kembalian
        if let Terminator::Kembali(val_opt) = term {
            match val_opt {
                Some(val) => {
                    if let Some(t_val) = tipe_nilai.get(val) {
                        if *t_val != f.tipe_kembali && f.tipe_kembali != IrTipe::Void {
                            galat.push(GalatVerifikasi {
                                fungsi: nama_fungsi.clone(),
                                blok: Some(b.id),
                                pesan: format!("Nilai kembalian {} bertipe '{}', tetapi fungsi mengembalikan '{}'", val, t_val, f.tipe_kembali),
                            });
                        }
                    }
                }
                None => {
                    if f.tipe_kembali != IrTipe::Void {
                        galat.push(GalatVerifikasi {
                            fungsi: nama_fungsi.clone(),
                            blok: Some(b.id),
                            pesan: format!("Terminator 'kembali' tanpa nilai, padahal fungsi mengembalikan '{}'", f.tipe_kembali),
                        });
                    }
                }
            }
        }

        // Periksa urutan instruksi di dalam blok
        let mut fasa_phi = true;

        for (hasil_opt, inst) in &b.instruksi {
            if let Instruksi::Phi { masukan, tipe } = inst {
                if !fasa_phi {
                    galat.push(GalatVerifikasi {
                        fungsi: nama_fungsi.clone(),
                        blok: Some(b.id),
                        pesan: "Instruksi 'phi' harus berada di awal blok sebelum instruksi non-phi".to_string(),
                    });
                }

                // Periksa pendahulu phi
                let blok_pendahulu = pendahulu.get(&b.id).cloned().unwrap_or_default();
                for (_, blk_asal) in masukan {
                    if !blok_pendahulu.contains(blk_asal) {
                        galat.push(GalatVerifikasi {
                            fungsi: nama_fungsi.clone(),
                            blok: Some(b.id),
                            pesan: format!("Blok asal {} pada node 'phi' bukan merupakan pendahulu langsung dari blok {}", blk_asal, b.id),
                        });
                    }
                }
            } else {
                fasa_phi = false;
            }

            // Periksa bahwa operan yang digunakan sudah terdefinisi (kecuali operan phi yang boleh berasal dari back-edge loop)
            if !matches!(inst, Instruksi::Phi { .. }) {
                for use_val in inst.ambil_penggunaan() {
                    if !terdefinisi.contains(&use_val) {
                        galat.push(GalatVerifikasi {
                            fungsi: nama_fungsi.clone(),
                            blok: Some(b.id),
                            pesan: format!("Nilai {} digunakan sebelum didefinisikan", use_val),
                        });
                    }
                }
            }

            // Catat hasil instruksi ke himpunan nilai terdefinisi
            if let Some(hasil) = hasil_opt {
                if !terdefinisi.insert(*hasil) {
                    galat.push(GalatVerifikasi {
                        fungsi: nama_fungsi.clone(),
                        blok: Some(b.id),
                        pesan: format!("Nilai SSA {} didefinisikan lebih dari sekali", hasil),
                    });
                }
                if let Some(t) = inst.tipe_hasil() {
                    tipe_nilai.insert(*hasil, t);
                }
            }
        }

        // Periksa operan terminator
        for use_val in term.ambil_penggunaan() {
            if !terdefinisi.contains(&use_val) {
                galat.push(GalatVerifikasi {
                    fungsi: nama_fungsi.clone(),
                    blok: Some(b.id),
                    pesan: format!("Nilai {} digunakan pada terminator sebelum didefinisikan", use_val),
                });
            }
        }
    }

    if galat.is_empty() {
        Ok(())
    } else {
        Err(galat)
    }
}
