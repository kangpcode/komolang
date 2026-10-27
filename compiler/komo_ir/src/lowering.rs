// Copyright (c) kangpcode (Dhafa Nazula Permadi). Licensed under MIT or Apache-2.0.

//! Lowering dari Abstract Syntax Tree (AST) dan Tabel Semantik ke KIR (Komo IR).
//! Menghasilkan representasi SSA dengan blok dasar, alloca untuk variabel lokal,
//! dan kontrol aliran eksplisit yang siap dioptimalkan oleh pass mem2reg.

use std::collections::HashMap;
use komo_ast::*;
use komo_sema::{HasilSemantik, SemTipe};

use crate::blok::BlokDasar;
use crate::fungsi::{FungsiIr, ParameterIr};
use crate::instruksi::{Instruksi, OpBiner, OpUnari, Terminator};
use crate::modul::{DataGlobal, ModulIr};
use crate::nilai::{IdBlok, IdNilai, Konstanta};
use crate::tipe::IrTipe;

/// Mengonversi BerkasProgram dan HasilSemantik menjadi ModulIr KIR.
pub fn turunkan_ke_kir(berkas: &BerkasProgram, sema: &HasilSemantik) -> ModulIr {
    let nama_modul = berkas.paket.nama.nama.clone();
    let mut modul = ModulIr::baru(nama_modul);

    // 1. Turunkan deklarasi konstanta dan variabel tingkat atas (global)
    for dek in &berkas.deklarasi {
        match &dek.jenis {
            JenisDeklarasi::Konstanta { nama, nilai, .. } => {
                let init = evaluasi_konstanta_literal(nilai);
                let tipe_ir = if let Some(k) = &init {
                    k.tipe()
                } else {
                    IrTipe::I64
                };
                modul.tambah_global(DataGlobal {
                    nama: nama.nama.clone(),
                    tipe: tipe_ir,
                    inisialisasi: init,
                    konstan: true,
                });
            }
            JenisDeklarasi::Variabel { nama, nilai, .. } => {
                let init = nilai.as_ref().and_then(evaluasi_konstanta_literal);
                let tipe_ir = if let Some(k) = &init {
                    k.tipe()
                } else {
                    IrTipe::I64
                };
                modul.tambah_global(DataGlobal {
                    nama: nama.nama.clone(),
                    tipe: tipe_ir,
                    inisialisasi: init,
                    konstan: false,
                });
            }
            _ => {}
        }
    }

    // 2. Turunkan fungsi dan metode
    for dek in &berkas.deklarasi {
        if let JenisDeklarasi::Fungsi(f) = &dek.jenis {
            let fungsi_ir = turunkan_fungsi(f, sema);
            modul.tambah_fungsi(fungsi_ir);
        }
    }

    modul
}

fn evaluasi_konstanta_literal(expr: &Ekspresi) -> Option<Konstanta> {
    match &expr.jenis {
        JenisEkspresi::Literal(l) => match l {
            Literal::Bulat(n) => Some(Konstanta::Bulat(*n)),
            Literal::Desimal(d) => Some(Konstanta::Desimal(*d)),
            Literal::Teks(s) => Some(Konstanta::Teks(s.clone())),
            Literal::Karakter(c) => Some(Konstanta::Karakter(*c)),
            Literal::Bool(b) => Some(Konstanta::Bool(*b)),
            Literal::Bita(b) => Some(Konstanta::Bita(*b)),
            Literal::Kosong => Some(Konstanta::Kosong),
        },
        _ => None,
    }
}

/// Konteks status penurunan fungsi ke KIR.
struct KonteksFungsi<'a> {
    sema: &'a HasilSemantik,
    fungsi: FungsiIr,
    blok_aktif: IdBlok,
    variabel_lokal: HashMap<String, (IdNilai, IrTipe)>,
    tumpukan_tunda: Vec<Ekspresi>,
}

fn turunkan_fungsi(f: &DeklarasiFungsi, sema: &HasilSemantik) -> FungsiIr {
    // Tentukan nama fungsi IR
    let nama_ir = if let Some(penerima) = &f.penerima {
        let nama_penerima = tipe_ast_ke_nama(&penerima.tipe);
        format!("{}.{}", nama_penerima, f.nama.nama)
    } else {
        f.nama.nama.clone()
    };

    let tipe_kembali = f
        .tipe_kembali
        .as_ref()
        .map(|t| tipe_ast_ke_ir(t, sema))
        .unwrap_or(IrTipe::Void);

    let mut ctx = KonteksFungsi {
        sema,
        fungsi: FungsiIr::baru(nama_ir, tipe_kembali.clone()),
        blok_aktif: IdBlok(0),
        variabel_lokal: HashMap::new(),
        tumpukan_tunda: Vec::new(),
    };

    // Alokasikan parameter
    if let Some(penerima) = &f.penerima {
        let tipe_ir = tipe_ast_ke_ir(&penerima.tipe, sema);
        let param_val = ctx.fungsi.tambah_parameter(penerima.nama.nama.clone(), tipe_ir.clone());
        let alloca_ptr = ctx.alokasikan_instruksi(Instruksi::Alloca(tipe_ir.clone()));
        ctx.tambah_instruksi(Instruksi::Simpan {
            nilai: param_val,
            penunjuk: alloca_ptr,
        });
        ctx.variabel_lokal.insert(penerima.nama.nama.clone(), (alloca_ptr, tipe_ir));
    }

    for p in &f.parameter {
        let tipe_ir = tipe_ast_ke_ir(&p.tipe, sema);
        let param_val = ctx.fungsi.tambah_parameter(p.nama.nama.clone(), tipe_ir.clone());
        let alloca_ptr = ctx.alokasikan_instruksi(Instruksi::Alloca(tipe_ir.clone()));
        ctx.tambah_instruksi(Instruksi::Simpan {
            nilai: param_val,
            penunjuk: alloca_ptr,
        });
        ctx.variabel_lokal.insert(p.nama.nama.clone(), (alloca_ptr, tipe_ir));
    }

    // Turunkan isi blok badan fungsi jika ada
    if let Some(badan) = &f.badan {
        for stmt in &badan.pernyataan {
            ctx.turunkan_pernyataan(stmt);
        }
    }

    // Pastikan blok aktif ditutup terminator jika belum
    let blok = ctx.fungsi.cari_blok(ctx.blok_aktif).unwrap();
    if !blok.apakah_ditutup() {
        ctx.jalankan_tunda();
        let term = if tipe_kembali == IrTipe::Void {
            Terminator::Kembali(None)
        } else {
            // Nilai kembalian default/zero jika jalur jatuh sampai akhir tanpa return
            let zero_val = ctx.alokasikan_instruksi(Instruksi::Konst(Konstanta::Bulat(0)));
            Terminator::Kembali(Some(zero_val))
        };
        ctx.atur_terminator_blok_aktif(term);
    }

    ctx.fungsi
}

impl<'a> KonteksFungsi<'a> {
    fn alokasikan_instruksi(&mut self, inst: Instruksi) -> IdNilai {
        let id = self.fungsi.alokasikan_nilai();
        let blok = self.fungsi.cari_blok_mut(self.blok_aktif).unwrap();
        blok.tambah_instruksi(Some(id), inst);
        id
    }

    fn tambah_instruksi(&mut self, inst: Instruksi) {
        let blok = self.fungsi.cari_blok_mut(self.blok_aktif).unwrap();
        blok.tambah_instruksi(None, inst);
    }

    fn atur_terminator_blok_aktif(&mut self, term: Terminator) {
        let blok = self.fungsi.cari_blok_mut(self.blok_aktif).unwrap();
        blok.atur_terminator(term);
    }

    fn jalankan_tunda(&mut self) {
        let tunda_list = self.tumpukan_tunda.clone();
        for expr in tunda_list.iter().rev() {
            let _ = self.turunkan_ekspresi(expr);
        }
    }

    fn turunkan_pernyataan(&mut self, stmt: &Pernyataan) {
        // Jika blok saat ini sudah ditutup terminator (misalnya setelah 'kembali'),
        // abaikan instruksi yang tidak terjangkau (dead code)
        if self.fungsi.cari_blok(self.blok_aktif).unwrap().apakah_ditutup() {
            return;
        }

        match &stmt.jenis {
            JenisPernyataan::Variabel { nama, nilai, tipe, .. } => {
                let tipe_ir = if let Some(t) = tipe {
                    tipe_ast_ke_ir(t, self.sema)
                } else if let Some(n) = nilai {
                    self.ambil_tipe_ekspresi(n)
                } else {
                    IrTipe::I64
                };

                let alloca_ptr = self.alokasikan_instruksi(Instruksi::Alloca(tipe_ir.clone()));
                if let Some(n) = nilai {
                    let val = self.turunkan_ekspresi(n);
                    self.tambah_instruksi(Instruksi::Simpan {
                        nilai: val,
                        penunjuk: alloca_ptr,
                    });
                }
                self.variabel_lokal.insert(nama.nama.clone(), (alloca_ptr, tipe_ir));
            }

            JenisPernyataan::Tugas { target, nilai } => {
                let val = self.turunkan_ekspresi(nilai);
                let ptr = self.ambil_alamat_lvalue(target);
                self.tambah_instruksi(Instruksi::Simpan {
                    nilai: val,
                    penunjuk: ptr,
                });
            }

            JenisPernyataan::TugasGabungan { target, operator, nilai } => {
                let ptr = self.ambil_alamat_lvalue(target);
                let t_ir = self.ambil_tipe_ekspresi(target);
                let lama = self.alokasikan_instruksi(Instruksi::Muat {
                    tipe: t_ir.clone(),
                    penunjuk: ptr,
                });
                let rhs = self.turunkan_ekspresi(nilai);
                let op = match operator {
                    OperatorTugas::TambahSama => OpBiner::Tambah,
                    OperatorTugas::KurangSama => OpBiner::Kurang,
                    OperatorTugas::KaliSama => OpBiner::Kali,
                    OperatorTugas::BagiSama => OpBiner::Bagi,
                    OperatorTugas::SisaSama => OpBiner::Sisa,
                };
                let baru = self.alokasikan_instruksi(Instruksi::Biner {
                    op,
                    kiri: lama,
                    kanan: rhs,
                    tipe: t_ir,
                });
                self.tambah_instruksi(Instruksi::Simpan {
                    nilai: baru,
                    penunjuk: ptr,
                });
            }

            JenisPernyataan::Ekspresi(e) => {
                let _ = self.turunkan_ekspresi(e);
            }

            JenisPernyataan::Blok(b) => {
                for s in &b.pernyataan {
                    self.turunkan_pernyataan(s);
                }
            }

            JenisPernyataan::Jika { kondisi, maka, cabang_lain, lainnya } => {
                let val_kondisi = self.turunkan_ekspresi(kondisi);

                let bb_maka = self.fungsi.buat_blok(Some("jika_maka".to_string()));
                let bb_berikutnya = self.fungsi.buat_blok(Some("jika_berikutnya".to_string()));
                let bb_gabung = self.fungsi.buat_blok(Some("jika_gabung".to_string()));

                let bb_salah_target = if cabang_lain.is_empty() && lainnya.is_none() {
                    bb_gabung
                } else {
                    bb_berikutnya
                };

                self.atur_terminator_blok_aktif(Terminator::Cabang {
                    kondisi: val_kondisi,
                    jika_benar: bb_maka,
                    jika_salah: bb_salah_target,
                });

                // Cabang maka
                self.blok_aktif = bb_maka;
                for s in &maka.pernyataan {
                    self.turunkan_pernyataan(s);
                }
                if !self.fungsi.cari_blok(self.blok_aktif).unwrap().apakah_ditutup() {
                    self.atur_terminator_blok_aktif(Terminator::Lompat(bb_gabung));
                }

                // Cabang jika_lain dan lainnya
                if !cabang_lain.is_empty() || lainnya.is_some() {
                    self.blok_aktif = bb_berikutnya;
                    let mut sisa_lain = lainnya.as_ref();

                    if !cabang_lain.is_empty() {
                        for cab in cabang_lain {
                            let k = self.turunkan_ekspresi(&cab.kondisi);
                            let bb_cab_maka = self.fungsi.buat_blok(Some("jika_lain_maka".to_string()));
                            let bb_cab_salah = self.fungsi.buat_blok(Some("jika_lain_salah".to_string()));

                            self.atur_terminator_blok_aktif(Terminator::Cabang {
                                kondisi: k,
                                jika_benar: bb_cab_maka,
                                jika_salah: bb_cab_salah,
                            });

                            self.blok_aktif = bb_cab_maka;
                            for s in &cab.badan.pernyataan {
                                self.turunkan_pernyataan(s);
                            }
                            if !self.fungsi.cari_blok(self.blok_aktif).unwrap().apakah_ditutup() {
                                self.atur_terminator_blok_aktif(Terminator::Lompat(bb_gabung));
                            }

                            self.blok_aktif = bb_cab_salah;
                        }
                    }

                    if let Some(lain) = sisa_lain {
                        for s in &lain.pernyataan {
                            self.turunkan_pernyataan(s);
                        }
                    }
                    if !self.fungsi.cari_blok(self.blok_aktif).unwrap().apakah_ditutup() {
                        self.atur_terminator_blok_aktif(Terminator::Lompat(bb_gabung));
                    }
                }

                self.blok_aktif = bb_gabung;
            }

            JenisPernyataan::Selagi { kondisi, badan } => {
                let bb_kondisi = self.fungsi.buat_blok(Some("selagi_kondisi".to_string()));
                let bb_badan = self.fungsi.buat_blok(Some("selagi_badan".to_string()));
                let bb_keluar = self.fungsi.buat_blok(Some("selagi_keluar".to_string()));

                self.atur_terminator_blok_aktif(Terminator::Lompat(bb_kondisi));

                self.blok_aktif = bb_kondisi;
                let val_kondisi = self.turunkan_ekspresi(kondisi);
                self.atur_terminator_blok_aktif(Terminator::Cabang {
                    kondisi: val_kondisi,
                    jika_benar: bb_badan,
                    jika_salah: bb_keluar,
                });

                self.blok_aktif = bb_badan;
                for s in &badan.pernyataan {
                    self.turunkan_pernyataan(s);
                }
                if !self.fungsi.cari_blok(self.blok_aktif).unwrap().apakah_ditutup() {
                    self.atur_terminator_blok_aktif(Terminator::Lompat(bb_kondisi));
                }

                self.blok_aktif = bb_keluar;
            }

            JenisPernyataan::Untuk { variabel, sumber, badan } => {
                // Untuk rentang: dari..sampai
                if let JenisEkspresi::Rentang { dari, sampai, inklusif } = &sumber.jenis {
                    let iter_val = dari
                        .as_ref()
                        .map(|d| self.turunkan_ekspresi(d))
                        .unwrap_or_else(|| self.alokasikan_instruksi(Instruksi::Konst(Konstanta::Bulat(0))));

                    let iter_ptr = self.alokasikan_instruksi(Instruksi::Alloca(IrTipe::I64));
                    self.tambah_instruksi(Instruksi::Simpan {
                        nilai: iter_val,
                        penunjuk: iter_ptr,
                    });
                    self.variabel_lokal.insert(variabel.nama.clone(), (iter_ptr, IrTipe::I64));

                    let bb_kondisi = self.fungsi.buat_blok(Some("untuk_kondisi".to_string()));
                    let bb_badan = self.fungsi.buat_blok(Some("untuk_badan".to_string()));
                    let bb_keluar = self.fungsi.buat_blok(Some("untuk_keluar".to_string()));

                    self.atur_terminator_blok_aktif(Terminator::Lompat(bb_kondisi));

                    self.blok_aktif = bb_kondisi;
                    let current_iter = self.alokasikan_instruksi(Instruksi::Muat {
                        tipe: IrTipe::I64,
                        penunjuk: iter_ptr,
                    });

                    let batas_val = sampai
                        .as_ref()
                        .map(|s| self.turunkan_ekspresi(s))
                        .unwrap_or_else(|| self.alokasikan_instruksi(Instruksi::Konst(Konstanta::Bulat(i64::MAX))));

                    let op_cmp = if *inklusif {
                        OpBiner::KurangSama
                    } else {
                        OpBiner::KurangDari
                    };

                    let cond_val = self.alokasikan_instruksi(Instruksi::Biner {
                        op: op_cmp,
                        kiri: current_iter,
                        kanan: batas_val,
                        tipe: IrTipe::I64,
                    });

                    self.atur_terminator_blok_aktif(Terminator::Cabang {
                        kondisi: cond_val,
                        jika_benar: bb_badan,
                        jika_salah: bb_keluar,
                    });

                    self.blok_aktif = bb_badan;
                    for s in &badan.pernyataan {
                        self.turunkan_pernyataan(s);
                    }

                    // Increment iter
                    if !self.fungsi.cari_blok(self.blok_aktif).unwrap().apakah_ditutup() {
                        let cur = self.alokasikan_instruksi(Instruksi::Muat {
                            tipe: IrTipe::I64,
                            penunjuk: iter_ptr,
                        });
                        let one = self.alokasikan_instruksi(Instruksi::Konst(Konstanta::Bulat(1)));
                        let next = self.alokasikan_instruksi(Instruksi::Biner {
                            op: OpBiner::Tambah,
                            kiri: cur,
                            kanan: one,
                            tipe: IrTipe::I64,
                        });
                        self.tambah_instruksi(Instruksi::Simpan {
                            nilai: next,
                            penunjuk: iter_ptr,
                        });
                        self.atur_terminator_blok_aktif(Terminator::Lompat(bb_kondisi));
                    }

                    self.blok_aktif = bb_keluar;
                } else {
                    // Iterasi umum pada larik
                    let arr_val = self.turunkan_ekspresi(sumber);
                    let bb_badan = self.fungsi.buat_blok(Some("untuk_larik_badan".to_string()));
                    let bb_keluar = self.fungsi.buat_blok(Some("untuk_larik_keluar".to_string()));

                    self.atur_terminator_blok_aktif(Terminator::Lompat(bb_badan));
                    self.blok_aktif = bb_badan;
                    for s in &badan.pernyataan {
                        self.turunkan_pernyataan(s);
                    }
                    if !self.fungsi.cari_blok(self.blok_aktif).unwrap().apakah_ditutup() {
                        self.atur_terminator_blok_aktif(Terminator::Lompat(bb_keluar));
                    }
                    self.blok_aktif = bb_keluar;
                }
            }

            JenisPernyataan::Cocok { nilai, cabang } => {
                let target_val = self.turunkan_ekspresi(nilai);
                let bb_gabung = self.fungsi.buat_blok(Some("cocok_gabung".to_string()));

                for (i, c) in cabang.iter().enumerate() {
                    let bb_badan = self.fungsi.buat_blok(Some(format!("cocok_badan_{}", i)));
                    let bb_berikutnya = self.fungsi.buat_blok(Some(format!("cocok_uji_{}", i + 1)));

                    // Uji pola
                    let cocok_kondisi = match &c.pola.jenis {
                        JenisPola::Wildcard => {
                            self.alokasikan_instruksi(Instruksi::Konst(Konstanta::Bool(true)))
                        }
                        JenisPola::Literal(lit) => {
                            let lit_val = self.turunkan_literal(lit);
                            self.alokasikan_instruksi(Instruksi::Biner {
                                op: OpBiner::SamaDengan,
                                kiri: target_val,
                                kanan: lit_val,
                                tipe: IrTipe::I64,
                            })
                        }
                        JenisPola::Pengenal(id) => {
                            // Binding identifier: cocokkan dan alokasikan variabel lokal
                            let alloca_ptr = self.alokasikan_instruksi(Instruksi::Alloca(IrTipe::I64));
                            self.tambah_instruksi(Instruksi::Simpan {
                                nilai: target_val,
                                penunjuk: alloca_ptr,
                            });
                            self.variabel_lokal.insert(id.nama.clone(), (alloca_ptr, IrTipe::I64));
                            self.alokasikan_instruksi(Instruksi::Konst(Konstanta::Bool(true)))
                        }
                        _ => self.alokasikan_instruksi(Instruksi::Konst(Konstanta::Bool(true))),
                    };

                    self.atur_terminator_blok_aktif(Terminator::Cabang {
                        kondisi: cocok_kondisi,
                        jika_benar: bb_badan,
                        jika_salah: bb_berikutnya,
                    });

                    self.blok_aktif = bb_badan;
                    self.turunkan_pernyataan(&c.aksi);
                    if !self.fungsi.cari_blok(self.blok_aktif).unwrap().apakah_ditutup() {
                        self.atur_terminator_blok_aktif(Terminator::Lompat(bb_gabung));
                    }

                    self.blok_aktif = bb_berikutnya;
                }

                if !self.fungsi.cari_blok(self.blok_aktif).unwrap().apakah_ditutup() {
                    self.atur_terminator_blok_aktif(Terminator::Lompat(bb_gabung));
                }

                self.blok_aktif = bb_gabung;
            }

            JenisPernyataan::Pilih { kasus, lainnya } => {
                let bb_gabung = self.fungsi.buat_blok(Some("pilih_gabung".to_string()));
                for (i, k) in kasus.iter().enumerate() {
                    let bb_kasus = self.fungsi.buat_blok(Some(format!("pilih_kasus_{}", i)));
                    let bb_next = self.fungsi.buat_blok(Some(format!("pilih_next_{}", i)));

                    // Simulasikan polling saluran
                    let dummy_ready = self.alokasikan_instruksi(Instruksi::Konst(Konstanta::Bool(true)));
                    self.atur_terminator_blok_aktif(Terminator::Cabang {
                        kondisi: dummy_ready,
                        jika_benar: bb_kasus,
                        jika_salah: bb_next,
                    });

                    self.blok_aktif = bb_kasus;
                    for s in &k.badan.pernyataan {
                        self.turunkan_pernyataan(s);
                    }
                    if !self.fungsi.cari_blok(self.blok_aktif).unwrap().apakah_ditutup() {
                        self.atur_terminator_blok_aktif(Terminator::Lompat(bb_gabung));
                    }

                    self.blok_aktif = bb_next;
                }

                if let Some(lain) = lainnya {
                    for s in &lain.pernyataan {
                        self.turunkan_pernyataan(s);
                    }
                }
                if !self.fungsi.cari_blok(self.blok_aktif).unwrap().apakah_ditutup() {
                    self.atur_terminator_blok_aktif(Terminator::Lompat(bb_gabung));
                }

                self.blok_aktif = bb_gabung;
            }

            JenisPernyataan::Kembali(e) => {
                self.jalankan_tunda();
                let term = if let Some(expr) = e {
                    let val = self.turunkan_ekspresi(expr);
                    Terminator::Kembali(Some(val))
                } else {
                    Terminator::Kembali(None)
                };
                self.atur_terminator_blok_aktif(term);
            }

            JenisPernyataan::Tunda(e) => {
                self.tumpukan_tunda.push(*e.clone());
            }

            JenisPernyataan::Jalankan(e) => {
                let _ = self.turunkan_ekspresi(e);
            }

            JenisPernyataan::TidakAman(b) => {
                for s in &b.pernyataan {
                    self.turunkan_pernyataan(s);
                }
            }
        }
    }

    fn turunkan_ekspresi(&mut self, expr: &Ekspresi) -> IdNilai {
        match &expr.jenis {
            JenisEkspresi::Literal(l) => self.turunkan_literal(l),

            JenisEkspresi::Pengenal(id) => {
                if let Some((ptr, tipe)) = self.variabel_lokal.get(&id.nama).cloned() {
                    self.alokasikan_instruksi(Instruksi::Muat {
                        tipe,
                        penunjuk: ptr,
                    })
                } else {
                    // Mungkin konstanta global atau konstanta literal
                    self.alokasikan_instruksi(Instruksi::Konst(Konstanta::Bulat(0)))
                }
            }

            JenisEkspresi::Biner { kiri, operator, kanan } => {
                let lhs = self.turunkan_ekspresi(kiri);
                let rhs = self.turunkan_ekspresi(kanan);
                let tipe_ir = self.ambil_tipe_ekspresi(kiri);

                let op = match operator {
                    OperatorBiner::Tambah => OpBiner::Tambah,
                    OperatorBiner::Kurang => OpBiner::Kurang,
                    OperatorBiner::Kali => OpBiner::Kali,
                    OperatorBiner::Bagi => OpBiner::Bagi,
                    OperatorBiner::Sisa => OpBiner::Sisa,
                    OperatorBiner::SamaDengan => OpBiner::SamaDengan,
                    OperatorBiner::TidakSama => OpBiner::TidakSama,
                    OperatorBiner::KurangDari => OpBiner::KurangDari,
                    OperatorBiner::KurangSama => OpBiner::KurangSama,
                    OperatorBiner::LebihDari => OpBiner::LebihDari,
                    OperatorBiner::LebihSama => OpBiner::LebihSama,
                    OperatorBiner::DanLogika | OperatorBiner::DanBit => OpBiner::DanBit,
                    OperatorBiner::AtauLogika | OperatorBiner::AtauBit => OpBiner::AtauBit,
                    OperatorBiner::XorBit => OpBiner::XorBit,
                    OperatorBiner::GeserKiri => OpBiner::GeserKiri,
                    OperatorBiner::GeserKanan => OpBiner::GeserKanan,
                };

                self.alokasikan_instruksi(Instruksi::Biner {
                    op,
                    kiri: lhs,
                    kanan: rhs,
                    tipe: tipe_ir,
                })
            }

            JenisEkspresi::Unari { operator, operan } => {
                let val = self.turunkan_ekspresi(operan);
                let tipe_ir = self.ambil_tipe_ekspresi(operan);
                let op = match operator {
                    OperatorUnari::Negasi => OpUnari::Negasi,
                    OperatorUnari::Bukan => OpUnari::Bukan,
                    OperatorUnari::BukanBit => OpUnari::BukanBit,
                };
                self.alokasikan_instruksi(Instruksi::Unari {
                    op,
                    operan: val,
                    tipe: tipe_ir,
                })
            }

            JenisEkspresi::Panggilan { fungsi, argumen } => {
                let mut nilai_arg = Vec::new();
                let nama_fungsi = match &fungsi.jenis {
                    JenisEkspresi::Pengenal(id) => id.nama.clone(),
                    JenisEkspresi::AksesBidang { objek, bidang } => {
                        // Pemanggilan metode obj.metode()
                        let obj_ptr = self.turunkan_ekspresi(objek);
                        nilai_arg.push(obj_ptr);
                        let tipe_obj = self.ambil_tipe_ekspresi(objek);
                        format!("{}.{}", tipe_obj, bidang.nama)
                    }
                    _ => "panggilan_tak_dikenal".to_string(),
                };

                for a in argumen {
                    nilai_arg.push(self.turunkan_ekspresi(a));
                }

                let ret_tipe = self.ambil_tipe_ekspresi(expr);
                self.alokasikan_instruksi(Instruksi::Panggil {
                    fungsi: nama_fungsi,
                    argumen: nilai_arg,
                    tipe_kembali: ret_tipe,
                })
            }

            JenisEkspresi::AksesBidang { objek, bidang } => {
                let obj_ptr = self.turunkan_ekspresi(objek);
                let tipe_hasil = self.ambil_tipe_ekspresi(expr);
                let field_ptr = self.alokasikan_instruksi(Instruksi::AksesBidang {
                    struktur: obj_ptr,
                    indeks: 0, // Dihitung dari layout bidang di pass semantik
                });
                self.alokasikan_instruksi(Instruksi::Muat {
                    tipe: tipe_hasil,
                    penunjuk: field_ptr,
                })
            }

            JenisEkspresi::Indeks { objek, indeks } => {
                let target_ptr = self.turunkan_ekspresi(objek);
                let idx_val = self.turunkan_ekspresi(indeks);
                let elem_tipe = self.ambil_tipe_ekspresi(expr);
                let elem_ptr = self.alokasikan_instruksi(Instruksi::AksesIndeks {
                    target: target_ptr,
                    indeks: idx_val,
                });
                self.alokasikan_instruksi(Instruksi::Muat {
                    tipe: elem_tipe,
                    penunjuk: elem_ptr,
                })
            }

            JenisEkspresi::Larik(elemen) => {
                let t_elemen = if let Some(first) = elemen.first() {
                    self.ambil_tipe_ekspresi(first)
                } else {
                    IrTipe::I64
                };
                let larik_ptr = self.alokasikan_instruksi(Instruksi::Alloca(IrTipe::Larik(
                    Box::new(t_elemen),
                    elemen.len(),
                )));
                for (i, e) in elemen.iter().enumerate() {
                    let v = self.turunkan_ekspresi(e);
                    let idx_val = self.alokasikan_instruksi(Instruksi::Konst(Konstanta::Bulat(i as i64)));
                    let elem_ptr = self.alokasikan_instruksi(Instruksi::AksesIndeks {
                        target: larik_ptr,
                        indeks: idx_val,
                    });
                    self.tambah_instruksi(Instruksi::Simpan {
                        nilai: v,
                        penunjuk: elem_ptr,
                    });
                }
                larik_ptr
            }

            JenisEkspresi::InisialisasiStruktur { bidang, .. } => {
                let struct_tipe = self.ambil_tipe_ekspresi(expr);
                let struct_ptr = self.alokasikan_instruksi(Instruksi::Alloca(struct_tipe));
                for (i, (_, nilai_bidang)) in bidang.iter().enumerate() {
                    let v = self.turunkan_ekspresi(nilai_bidang);
                    let field_ptr = self.alokasikan_instruksi(Instruksi::AksesBidang {
                        struktur: struct_ptr,
                        indeks: i,
                    });
                    self.tambah_instruksi(Instruksi::Simpan {
                        nilai: v,
                        penunjuk: field_ptr,
                    });
                }
                struct_ptr
            }

            JenisEkspresi::InterpolasiTeks(bagian) => {
                // String formatting desugared to runtime build
                let mut argumen = Vec::new();
                for b in bagian {
                    match b {
                        BagianInterpolasi::Teks(s) => {
                            let k = self.alokasikan_instruksi(Instruksi::Konst(Konstanta::Teks(s.clone())));
                            argumen.push(k);
                        }
                        BagianInterpolasi::Ekspresi(e) => {
                            let v = self.turunkan_ekspresi(e);
                            argumen.push(v);
                        }
                    }
                }
                self.alokasikan_instruksi(Instruksi::Panggil {
                    fungsi: "komo_rt_format_teks".to_string(),
                    argumen,
                    tipe_kembali: IrTipe::Ptr,
                })
            }

            JenisEkspresi::Pindah(e) => self.turunkan_ekspresi(e),

            JenisEkspresi::Pinjam { ekspresi, .. } => {
                self.ambil_alamat_lvalue(ekspresi)
            }

            JenisEkspresi::Rentang { dari, sampai, .. } => {
                let start = dari.as_ref().map(|d| self.turunkan_ekspresi(d))
                    .unwrap_or_else(|| self.alokasikan_instruksi(Instruksi::Konst(Konstanta::Bulat(0))));
                let end = sampai.as_ref().map(|s| self.turunkan_ekspresi(s))
                    .unwrap_or_else(|| self.alokasikan_instruksi(Instruksi::Konst(Konstanta::Bulat(0))));
                // Rentang sebagai struct { i64, i64 }
                let range_ptr = self.alokasikan_instruksi(Instruksi::Alloca(IrTipe::Struktur(vec![IrTipe::I64, IrTipe::I64])));
                let f0 = self.alokasikan_instruksi(Instruksi::AksesBidang { struktur: range_ptr, indeks: 0 });
                self.tambah_instruksi(Instruksi::Simpan { nilai: start, penunjuk: f0 });
                let f1 = self.alokasikan_instruksi(Instruksi::AksesBidang { struktur: range_ptr, indeks: 1 });
                self.tambah_instruksi(Instruksi::Simpan { nilai: end, penunjuk: f1 });
                range_ptr
            }
        }
    }

    fn turunkan_literal(&mut self, l: &Literal) -> IdNilai {
        let konst = match l {
            Literal::Bulat(n) => Konstanta::Bulat(*n),
            Literal::Desimal(d) => Konstanta::Desimal(*d),
            Literal::Teks(s) => Konstanta::Teks(s.clone()),
            Literal::Karakter(c) => Konstanta::Karakter(*c),
            Literal::Bool(b) => Konstanta::Bool(*b),
            Literal::Bita(b) => Konstanta::Bita(*b),
            Literal::Kosong => Konstanta::Kosong,
        };
        self.alokasikan_instruksi(Instruksi::Konst(konst))
    }

    fn ambil_alamat_lvalue(&mut self, expr: &Ekspresi) -> IdNilai {
        match &expr.jenis {
            JenisEkspresi::Pengenal(id) => {
                if let Some((ptr, _)) = self.variabel_lokal.get(&id.nama) {
                    *ptr
                } else {
                    self.alokasikan_instruksi(Instruksi::Alloca(IrTipe::I64))
                }
            }
            JenisEkspresi::AksesBidang { objek, .. } => {
                let obj_ptr = self.turunkan_ekspresi(objek);
                self.alokasikan_instruksi(Instruksi::AksesBidang {
                    struktur: obj_ptr,
                    indeks: 0,
                })
            }
            JenisEkspresi::Indeks { objek, indeks } => {
                let target_ptr = self.turunkan_ekspresi(objek);
                let idx_val = self.turunkan_ekspresi(indeks);
                self.alokasikan_instruksi(Instruksi::AksesIndeks {
                    target: target_ptr,
                    indeks: idx_val,
                })
            }
            _ => self.alokasikan_instruksi(Instruksi::Alloca(IrTipe::I64)),
        }
    }

    fn ambil_tipe_ekspresi(&self, expr: &Ekspresi) -> IrTipe {
        if let Some(sem_tipe) = self.sema.tabel_tipe.get(&expr.rentang) {
            IrTipe::dari_sem_tipe(sem_tipe)
        } else {
            match &expr.jenis {
                JenisEkspresi::Literal(Literal::Bulat(_)) => IrTipe::I64,
                JenisEkspresi::Literal(Literal::Desimal(_)) => IrTipe::F64,
                JenisEkspresi::Literal(Literal::Bool(_)) => IrTipe::I1,
                JenisEkspresi::Literal(Literal::Teks(_)) => IrTipe::Ptr,
                _ => IrTipe::I64,
            }
        }
    }
}

fn tipe_ast_ke_nama(t: &Tipe) -> String {
    match &t.jenis {
        JenisTipe::Sederhana(id) => id.nama.clone(),
        JenisTipe::Generik { nama, .. } => nama.nama.clone(),
        JenisTipe::Pinjam { dalam, .. } => tipe_ast_ke_nama(dalam),
        _ => "Objek".to_string(),
    }
}

fn tipe_ast_ke_ir(t: &Tipe, sema: &HasilSemantik) -> IrTipe {
    if let Some(sem) = sema.tabel_tipe.get(&t.rentang) {
        IrTipe::dari_sem_tipe(sem)
    } else {
        match &t.jenis {
            JenisTipe::Sederhana(id) => match id.nama.as_str() {
                "bulat" => IrTipe::I64,
                "desimal" => IrTipe::F64,
                "teks" => IrTipe::Ptr,
                "bool" => IrTipe::I1,
                "karakter" => IrTipe::Char,
                "bita" => IrTipe::I8,
                "kosong" => IrTipe::Void,
                _ => IrTipe::Ptr,
            },
            JenisTipe::Pinjam { .. } => IrTipe::Ptr,
            JenisTipe::Larik { .. } => IrTipe::Struktur(vec![IrTipe::Ptr, IrTipe::I64, IrTipe::I64]),
            _ => IrTipe::I64,
        }
    }
}
