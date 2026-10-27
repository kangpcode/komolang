// Copyright (c) kangpcode (Dhafa Nazula Permadi). Licensed under MIT or Apache-2.0.

//! Pengecek kepemilikan (ownership) dan peminjaman (borrow checker) sederhana untuk Komolang.
//! Menegakkan aturan mutabilitas (`biar` vs `var`), mencegah penggunaan setelah nilai dipindah
//! (use-after-move), dan memvalidasi aturan peminjaman eksklusif (`pinjam var`).

use crate::simbol::{JenisSimbol, TabelSimbol};
use komo_ast::Rentang;
use komo_diag::{kode, Diagnostik};

pub struct PelacakOwnership;

impl PelacakOwnership {
    /// Memvalidasi mutasi variabel pada pernyataan tugas (`x = ...`, `x += ...`).
    pub fn periksa_mutasi(
        tabel: &mut TabelSimbol,
        nama: &str,
        rentang: Rentang,
    ) -> Result<(), Diagnostik> {
        if let Some(simbol) = tabel.cari_simbol_mut(nama) {
            match &mut simbol.jenis {
                JenisSimbol::Variabel {
                    mutabel,
                    sudah_dipindah,
                    jumlah_pinjam,
                    pinjam_mutabel,
                    ..
                } => {
                    if *sudah_dipindah {
                        return Err(Diagnostik::galat(
                            kode::PENGGUNAAN_SETELAH_PINDAH,
                            format!(
                                "tidak dapat mengubah variabel '{}' karena nilainya telah dipindahkan",
                                nama
                            ),
                            rentang,
                        )
                        .dengan_saran("hindari penggunaan variabel setelah kepemilikannya dipindahkan"));
                    }

                    if !*mutabel {
                        return Err(Diagnostik::galat(
                            kode::MUTASI_VARIABEL_IMUTABEL,
                            format!(
                                "tidak dapat mengubah variabel imutabel '{}'; dideklarasikan dengan 'biar'",
                                nama
                            ),
                            rentang,
                        )
                        .dengan_saran(format!(
                            "ubah deklarasi menjadi 'var {}' jika nilainya perlu diubah",
                            nama
                        )));
                    }

                    if *jumlah_pinjam > 0 || *pinjam_mutabel {
                        return Err(Diagnostik::galat(
                            kode::PEMINJAMAN_KONFLIK,
                            format!(
                                "tidak dapat mengubah '{}' saat sedang dipinjam oleh operasi lain",
                                nama
                            ),
                            rentang,
                        )
                        .dengan_saran("selesaikan peminjaman sebelum melakukan mutasi"));
                    }

                    Ok(())
                }
                JenisSimbol::Konstanta { .. } => Err(Diagnostik::galat(
                    kode::MUTASI_VARIABEL_IMUTABEL,
                    format!("tidak dapat mengubah konstanta '{}'", nama),
                    rentang,
                )),
                _ => Ok(()),
            }
        } else {
            Err(Diagnostik::galat(
                kode::SIMBOL_TIDAK_DITEMUKAN,
                format!("variabel '{}' tidak ditemukan", nama),
                rentang,
            ))
        }
    }

    /// Memvalidasi penggunaan variabel (apakah nilainya sudah pernah dipindah).
    pub fn periksa_penggunaan(
        tabel: &TabelSimbol,
        nama: &str,
        rentang: Rentang,
    ) -> Result<(), Diagnostik> {
        if let Some(simbol) = tabel.cari_simbol(nama) {
            if let JenisSimbol::Variabel { sudah_dipindah, .. } = &simbol.jenis {
                if *sudah_dipindah {
                    return Err(Diagnostik::galat(
                        kode::PENGGUNAAN_SETELAH_PINDAH,
                        format!(
                            "penggunaan variabel '{}' yang nilainya telah dipindahkan",
                            nama
                        ),
                        rentang,
                    )
                    .dengan_saran("nilai telah dipindahkan; buat salinan baru atau hindari penggunaan ulang"));
                }
            }
        }
        Ok(())
    }

    /// Menandai variabel sebagai telah dipindahkan kepemilikannya (`pindah x`).
    pub fn tandai_dipindah(
        tabel: &mut TabelSimbol,
        nama: &str,
        rentang: Rentang,
    ) -> Result<(), Diagnostik> {
        if let Some(simbol) = tabel.cari_simbol_mut(nama) {
            match &mut simbol.jenis {
                JenisSimbol::Variabel {
                    tipe,
                    sudah_dipindah,
                    jumlah_pinjam,
                    pinjam_mutabel,
                    ..
                } => {
                    if tipe.adalah_copy() {
                        // Tipe copy (primitif) tidak berpindah kepemilikan
                        return Ok(());
                    }

                    if *sudah_dipindah {
                        return Err(Diagnostik::galat(
                            kode::PENGGUNAAN_SETELAH_PINDAH,
                            format!("nilai '{}' sudah dipindahkan sebelumnya", nama),
                            rentang,
                        ));
                    }

                    if *jumlah_pinjam > 0 || *pinjam_mutabel {
                        return Err(Diagnostik::galat(
                            kode::PEMINJAMAN_KONFLIK,
                            format!(
                                "tidak dapat memindahkan '{}' saat sedang dipinjam",
                                nama
                            ),
                            rentang,
                        ));
                    }

                    *sudah_dipindah = true;
                    Ok(())
                }
                _ => Ok(()),
            }
        } else {
            Ok(())
        }
    }

    /// Melakukan peminjaman (`pinjam x` atau `pinjam var x`).
    pub fn lakukan_peminjaman(
        tabel: &mut TabelSimbol,
        nama: &str,
        mutabel: bool,
        rentang: Rentang,
    ) -> Result<(), Diagnostik> {
        if let Some(simbol) = tabel.cari_simbol_mut(nama) {
            match &mut simbol.jenis {
                JenisSimbol::Variabel {
                    mutabel: var_mutabel,
                    sudah_dipindah,
                    jumlah_pinjam,
                    pinjam_mutabel,
                    ..
                } => {
                    if *sudah_dipindah {
                        return Err(Diagnostik::galat(
                            kode::PENGGUNAAN_SETELAH_PINDAH,
                            format!(
                                "tidak dapat meminjam '{}' karena nilainya telah dipindahkan",
                                nama
                            ),
                            rentang,
                        ));
                    }

                    if mutabel {
                        if !*var_mutabel {
                            return Err(Diagnostik::galat(
                                kode::MUTASI_VARIABEL_IMUTABEL,
                                format!(
                                    "tidak dapat meminjam mutabel 'pinjam var {}' pada variabel imutabel",
                                    nama
                                ),
                                rentang,
                            )
                            .dengan_saran(format!(
                                "deklarasikan variabel dengan 'var {}' agar dapat dipinjam secara mutabel",
                                nama
                            )));
                        }

                        if *jumlah_pinjam > 0 || *pinjam_mutabel {
                            return Err(Diagnostik::galat(
                                kode::PEMINJAMAN_KONFLIK,
                                format!(
                                    "tidak dapat meminjam mutabel '{}' karena sedang ada peminjaman aktif lain",
                                    nama
                                ),
                                rentang,
                            ));
                        }

                        *pinjam_mutabel = true;
                    } else {
                        if *pinjam_mutabel {
                            return Err(Diagnostik::galat(
                                kode::PEMINJAMAN_KONFLIK,
                                format!(
                                    "tidak dapat meminjam imutabel '{}' karena sedang dipinjam secara mutabel",
                                    nama
                                ),
                                rentang,
                            ));
                        }
                        *jumlah_pinjam += 1;
                    }

                    Ok(())
                }
                _ => Ok(()),
            }
        } else {
            Ok(())
        }
    }
}
