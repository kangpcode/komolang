// Copyright (c) kangpcode (Dhafa Nazula Permadi). Licensed under MIT or Apache-2.0.

//! Lowering dari Abstract Syntax Tree (AST) dan Tabel Semantik ke KIR (Komo IR).
//! Menghasilkan representasi SSA dengan blok dasar, alloca untuk variabel lokal,
//! dan kontrol aliran eksplisit yang siap dioptimalkan oleh pass mem2reg.

use komo_ast::*;
use komo_sema::HasilSemantik;
use std::collections::HashMap;

use crate::fungsi::FungsiIr;
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
        match dek {
            Deklarasi::Konstanta(k) => {
                let init = evaluasi_konstanta_literal(&k.nilai);
                let tipe_ir = if let Some(c) = &init {
                    c.tipe()
                } else {
                    IrTipe::I64
                };
                modul.tambah_global(DataGlobal {
                    nama: k.nama.nama.clone(),
                    tipe: tipe_ir,
                    inisialisasi: init,
                    konstan: true,
                });
            }
            Deklarasi::Variabel(v) => {
                let init = evaluasi_konstanta_literal(&v.nilai);
                let tipe_ir = if let Some(c) = &init {
                    c.tipe()
                } else {
                    IrTipe::I64
                };
                modul.tambah_global(DataGlobal {
                    nama: v.nama.nama.clone(),
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
        if let Deklarasi::Fungsi(f) = dek {
            let fungsi_ir = turunkan_fungsi(f, sema);
            modul.tambah_fungsi(fungsi_ir);
        }
    }

    modul
}

fn evaluasi_konstanta_literal(expr: &Ekspresi) -> Option<Konstanta> {
    match &expr.jenis {
        JenisEkspresi::Bulat(n) => Some(Konstanta::Bulat(*n as i64)),
        JenisEkspresi::Desimal(d) => Some(Konstanta::Desimal(*d)),
        JenisEkspresi::Bool(b) => Some(Konstanta::Bool(*b)),
        JenisEkspresi::Rune(c) => Some(Konstanta::Karakter(*c)),
        JenisEkspresi::Kosong => Some(Konstanta::Kosong),
        JenisEkspresi::Teks(parts) => {
            if parts.len() == 1 {
                if let BagianTeks::Teks(s) = &parts[0] {
                    return Some(Konstanta::Teks(s.clone()));
                }
            } else if parts.is_empty() {
                return Some(Konstanta::Teks(String::new()));
            }
            None
        }
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
    let nama_ir = if let Some(penerima) = &f.penerima {
        let nama_penerima = tipe_ast_ke_nama(&penerima.tipe);
        format!("{}.{}", nama_penerima, f.nama.nama)
    } else {
        f.nama.nama.clone()
    };

    let tipe_kembali = f
        .tipe_kembalian
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
        let param_val = ctx
            .fungsi
            .tambah_parameter(penerima.nama.nama.clone(), tipe_ir.clone());
        let alloca_ptr = ctx.alokasikan_instruksi(Instruksi::Alloca(tipe_ir.clone()));
        ctx.tambah_instruksi(Instruksi::Simpan {
            nilai: param_val,
            penunjuk: alloca_ptr,
        });
        ctx.variabel_lokal
            .insert(penerima.nama.nama.clone(), (alloca_ptr, tipe_ir));
    }

    for p in &f.parameter {
        let tipe_ir = tipe_ast_ke_ir(&p.tipe, sema);
        let param_val = ctx
            .fungsi
            .tambah_parameter(p.nama.nama.clone(), tipe_ir.clone());
        let alloca_ptr = ctx.alokasikan_instruksi(Instruksi::Alloca(tipe_ir.clone()));
        ctx.tambah_instruksi(Instruksi::Simpan {
            nilai: param_val,
            penunjuk: alloca_ptr,
        });
        ctx.variabel_lokal
            .insert(p.nama.nama.clone(), (alloca_ptr, tipe_ir));
    }

    // Turunkan isi blok badan fungsi
    for stmt in &f.badan.pernyataan {
        ctx.turunkan_pernyataan(stmt);
    }

    // Pastikan blok aktif ditutup terminator jika belum
    let blok = ctx.fungsi.cari_blok(ctx.blok_aktif).unwrap();
    if !blok.apakah_ditutup() {
        ctx.jalankan_tunda();
        let term = if tipe_kembali == IrTipe::Void {
            Terminator::Kembali(None)
        } else {
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
        if self
            .fungsi
            .cari_blok(self.blok_aktif)
            .unwrap()
            .apakah_ditutup()
        {
            return;
        }

        match &stmt.jenis {
            JenisPernyataan::Variabel {
                nama, tipe, nilai, ..
            } => {
                let tipe_ir = if let Some(t) = tipe {
                    tipe_ast_ke_ir(t, self.sema)
                } else {
                    self.ambil_tipe_ekspresi(nilai)
                };

                let alloca_ptr = self.alokasikan_instruksi(Instruksi::Alloca(tipe_ir.clone()));
                let val = self.turunkan_ekspresi(nilai);
                self.tambah_instruksi(Instruksi::Simpan {
                    nilai: val,
                    penunjuk: alloca_ptr,
                });
                self.variabel_lokal
                    .insert(nama.nama.clone(), (alloca_ptr, tipe_ir));
            }

            JenisPernyataan::Tugas {
                target,
                operator,
                nilai,
            } => {
                let ptr = self.ambil_alamat_lvalue(target);
                let val = self.turunkan_ekspresi(nilai);

                let nilai_disimpan = match operator {
                    OperatorTugas::Tetap => val,
                    op_gabungan => {
                        let t_ir = self.ambil_tipe_ekspresi(target);
                        let lama = self.alokasikan_instruksi(Instruksi::Muat {
                            tipe: t_ir.clone(),
                            penunjuk: ptr,
                        });
                        let op = match op_gabungan {
                            OperatorTugas::Tambah => OpBiner::Tambah,
                            OperatorTugas::Kurang => OpBiner::Kurang,
                            OperatorTugas::Kali => OpBiner::Kali,
                            OperatorTugas::Bagi => OpBiner::Bagi,
                            OperatorTugas::Sisa => OpBiner::Sisa,
                            OperatorTugas::Tetap => unreachable!(),
                        };
                        self.alokasikan_instruksi(Instruksi::Biner {
                            op,
                            kiri: lama,
                            kanan: val,
                            tipe: t_ir,
                        })
                    }
                };

                self.tambah_instruksi(Instruksi::Simpan {
                    nilai: nilai_disimpan,
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

            JenisPernyataan::Jika {
                kondisi,
                maka,
                lainnya,
            } => {
                let val_kondisi = self.turunkan_ekspresi(kondisi);

                let bb_maka = self.fungsi.buat_blok(Some("jika_maka".to_string()));
                let bb_gabung = self.fungsi.buat_blok(Some("jika_gabung".to_string()));
                let bb_salah = if lainnya.is_some() {
                    self.fungsi.buat_blok(Some("jika_salah".to_string()))
                } else {
                    bb_gabung
                };

                self.atur_terminator_blok_aktif(Terminator::Cabang {
                    kondisi: val_kondisi,
                    jika_benar: bb_maka,
                    jika_salah: bb_salah,
                });

                // Cabang maka
                self.blok_aktif = bb_maka;
                for s in &maka.pernyataan {
                    self.turunkan_pernyataan(s);
                }
                if !self
                    .fungsi
                    .cari_blok(self.blok_aktif)
                    .unwrap()
                    .apakah_ditutup()
                {
                    self.atur_terminator_blok_aktif(Terminator::Lompat(bb_gabung));
                }

                // Cabang salah / lainnya
                if let Some(lain) = lainnya {
                    self.blok_aktif = bb_salah;
                    self.turunkan_pernyataan(lain);
                    if !self
                        .fungsi
                        .cari_blok(self.blok_aktif)
                        .unwrap()
                        .apakah_ditutup()
                    {
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
                if !self
                    .fungsi
                    .cari_blok(self.blok_aktif)
                    .unwrap()
                    .apakah_ditutup()
                {
                    self.atur_terminator_blok_aktif(Terminator::Lompat(bb_kondisi));
                }

                self.blok_aktif = bb_keluar;
            }

            JenisPernyataan::Untuk {
                variabel,
                koleksi,
                badan,
            } => {
                if let JenisEkspresi::Rentang {
                    awal,
                    akhir,
                    inklusif,
                } = &koleksi.jenis
                {
                    let iter_val = self.turunkan_ekspresi(awal);
                    let iter_ptr = self.alokasikan_instruksi(Instruksi::Alloca(IrTipe::I64));
                    self.tambah_instruksi(Instruksi::Simpan {
                        nilai: iter_val,
                        penunjuk: iter_ptr,
                    });
                    self.variabel_lokal
                        .insert(variabel.nama.clone(), (iter_ptr, IrTipe::I64));

                    let bb_kondisi = self.fungsi.buat_blok(Some("untuk_kondisi".to_string()));
                    let bb_badan = self.fungsi.buat_blok(Some("untuk_badan".to_string()));
                    let bb_keluar = self.fungsi.buat_blok(Some("untuk_keluar".to_string()));

                    self.atur_terminator_blok_aktif(Terminator::Lompat(bb_kondisi));

                    self.blok_aktif = bb_kondisi;
                    let cur_iter = self.alokasikan_instruksi(Instruksi::Muat {
                        tipe: IrTipe::I64,
                        penunjuk: iter_ptr,
                    });
                    let batas_val = self.turunkan_ekspresi(akhir);

                    let op_cmp = if *inklusif {
                        OpBiner::KurangSama
                    } else {
                        OpBiner::KurangDari
                    };

                    let cond_val = self.alokasikan_instruksi(Instruksi::Biner {
                        op: op_cmp,
                        kiri: cur_iter,
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

                    if !self
                        .fungsi
                        .cari_blok(self.blok_aktif)
                        .unwrap()
                        .apakah_ditutup()
                    {
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
                    let _ = self.turunkan_ekspresi(koleksi);
                    let bb_badan = self
                        .fungsi
                        .buat_blok(Some("untuk_koleksi_badan".to_string()));
                    let bb_keluar = self
                        .fungsi
                        .buat_blok(Some("untuk_koleksi_keluar".to_string()));

                    self.atur_terminator_blok_aktif(Terminator::Lompat(bb_badan));
                    self.blok_aktif = bb_badan;
                    for s in &badan.pernyataan {
                        self.turunkan_pernyataan(s);
                    }
                    if !self
                        .fungsi
                        .cari_blok(self.blok_aktif)
                        .unwrap()
                        .apakah_ditutup()
                    {
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

                    let cocok_kondisi = match &c.pola.jenis {
                        JenisPola::Wildcard => {
                            self.alokasikan_instruksi(Instruksi::Konst(Konstanta::Bool(true)))
                        }
                        JenisPola::Literal(lit_expr) => {
                            let lit_val = self.turunkan_ekspresi(lit_expr);
                            self.alokasikan_instruksi(Instruksi::Biner {
                                op: OpBiner::SamaDengan,
                                kiri: target_val,
                                kanan: lit_val,
                                tipe: IrTipe::I64,
                            })
                        }
                        JenisPola::Pengenal(id) => {
                            let alloca_ptr =
                                self.alokasikan_instruksi(Instruksi::Alloca(IrTipe::I64));
                            self.tambah_instruksi(Instruksi::Simpan {
                                nilai: target_val,
                                penunjuk: alloca_ptr,
                            });
                            self.variabel_lokal
                                .insert(id.nama.clone(), (alloca_ptr, IrTipe::I64));
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
                    self.turunkan_pernyataan(&c.badan);
                    if !self
                        .fungsi
                        .cari_blok(self.blok_aktif)
                        .unwrap()
                        .apakah_ditutup()
                    {
                        self.atur_terminator_blok_aktif(Terminator::Lompat(bb_gabung));
                    }

                    self.blok_aktif = bb_berikutnya;
                }

                if !self
                    .fungsi
                    .cari_blok(self.blok_aktif)
                    .unwrap()
                    .apakah_ditutup()
                {
                    self.atur_terminator_blok_aktif(Terminator::Lompat(bb_gabung));
                }

                self.blok_aktif = bb_gabung;
            }

            JenisPernyataan::Pilih { kasus, lainnya, .. } => {
                let bb_gabung = self.fungsi.buat_blok(Some("pilih_gabung".to_string()));
                for (i, k) in kasus.iter().enumerate() {
                    let bb_kasus = self.fungsi.buat_blok(Some(format!("pilih_kasus_{}", i)));
                    let bb_next = self.fungsi.buat_blok(Some(format!("pilih_next_{}", i)));

                    let dummy_ready =
                        self.alokasikan_instruksi(Instruksi::Konst(Konstanta::Bool(true)));
                    self.atur_terminator_blok_aktif(Terminator::Cabang {
                        kondisi: dummy_ready,
                        jika_benar: bb_kasus,
                        jika_salah: bb_next,
                    });

                    self.blok_aktif = bb_kasus;
                    for s in &k.badan {
                        self.turunkan_pernyataan(s);
                    }
                    if !self
                        .fungsi
                        .cari_blok(self.blok_aktif)
                        .unwrap()
                        .apakah_ditutup()
                    {
                        self.atur_terminator_blok_aktif(Terminator::Lompat(bb_gabung));
                    }

                    self.blok_aktif = bb_next;
                }

                if let Some(lain) = lainnya {
                    for s in lain {
                        self.turunkan_pernyataan(s);
                    }
                }
                if !self
                    .fungsi
                    .cari_blok(self.blok_aktif)
                    .unwrap()
                    .apakah_ditutup()
                {
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
                self.tumpukan_tunda.push(e.clone());
            }

            JenisPernyataan::TundaBlok(b) => {
                for s in &b.pernyataan {
                    if let JenisPernyataan::Ekspresi(e) = &s.jenis {
                        self.tumpukan_tunda.push(e.clone());
                    }
                }
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
            JenisEkspresi::Bulat(n) => {
                self.alokasikan_instruksi(Instruksi::Konst(Konstanta::Bulat(*n as i64)))
            }
            JenisEkspresi::Desimal(d) => {
                self.alokasikan_instruksi(Instruksi::Konst(Konstanta::Desimal(*d)))
            }
            JenisEkspresi::Bool(b) => {
                self.alokasikan_instruksi(Instruksi::Konst(Konstanta::Bool(*b)))
            }
            JenisEkspresi::Rune(c) => {
                self.alokasikan_instruksi(Instruksi::Konst(Konstanta::Karakter(*c)))
            }
            JenisEkspresi::Kosong => self.alokasikan_instruksi(Instruksi::Konst(Konstanta::Kosong)),

            JenisEkspresi::Teks(parts) => {
                if parts.len() == 1 {
                    if let BagianTeks::Teks(s) = &parts[0] {
                        return self
                            .alokasikan_instruksi(Instruksi::Konst(Konstanta::Teks(s.clone())));
                    }
                }
                let mut argumen = Vec::new();
                for b in parts {
                    match b {
                        BagianTeks::Teks(s) => {
                            let k = self
                                .alokasikan_instruksi(Instruksi::Konst(Konstanta::Teks(s.clone())));
                            argumen.push(k);
                        }
                        BagianTeks::Ekspresi(e) => {
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

            JenisEkspresi::Pengenal(id) => {
                if let Some((ptr, tipe)) = self.variabel_lokal.get(&id.nama).cloned() {
                    self.alokasikan_instruksi(Instruksi::Muat {
                        tipe,
                        penunjuk: ptr,
                    })
                } else {
                    self.alokasikan_instruksi(Instruksi::Konst(Konstanta::Bulat(0)))
                }
            }

            JenisEkspresi::Biner {
                kiri,
                operator,
                kanan,
            } => {
                let lhs = self.turunkan_ekspresi(kiri);
                let rhs = self.turunkan_ekspresi(kanan);
                let tipe_ir = self.ambil_tipe_ekspresi(kiri);

                let op = match operator {
                    OperatorBiner::Tambah => OpBiner::Tambah,
                    OperatorBiner::Kurang => OpBiner::Kurang,
                    OperatorBiner::Kali => OpBiner::Kali,
                    OperatorBiner::Bagi => OpBiner::Bagi,
                    OperatorBiner::Sisa => OpBiner::Sisa,
                    OperatorBiner::Sama => OpBiner::SamaDengan,
                    OperatorBiner::TidakSama => OpBiner::TidakSama,
                    OperatorBiner::LebihKecil => OpBiner::KurangDari,
                    OperatorBiner::LebihKecilSama => OpBiner::KurangSama,
                    OperatorBiner::LebihBesar => OpBiner::LebihDari,
                    OperatorBiner::LebihBesarSama => OpBiner::LebihSama,
                    OperatorBiner::Dan => OpBiner::DanBit,
                    OperatorBiner::Atau => OpBiner::AtauBit,
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
                match operator {
                    OperatorUnari::Negatif => self.alokasikan_instruksi(Instruksi::Unari {
                        op: OpUnari::Negasi,
                        operan: val,
                        tipe: tipe_ir,
                    }),
                    OperatorUnari::Bukan => self.alokasikan_instruksi(Instruksi::Unari {
                        op: OpUnari::Bukan,
                        operan: val,
                        tipe: tipe_ir,
                    }),
                    OperatorUnari::Pinjam | OperatorUnari::PinjamMutabel => {
                        self.ambil_alamat_lvalue(operan)
                    }
                    OperatorUnari::Pindah => val,
                }
            }

            JenisEkspresi::Panggil {
                fungsi, argumen, ..
            } => {
                let mut nilai_arg = Vec::new();
                let nama_fungsi = match &fungsi.jenis {
                    JenisEkspresi::Pengenal(id) => id.nama.clone(),
                    JenisEkspresi::Bidang { objek, nama } => {
                        if let JenisEkspresi::Pengenal(obj_id) = &objek.jenis {
                            if !self.variabel_lokal.contains_key(&obj_id.nama) {
                                // Pemanggilan fungsi dari modul impor (misal `cetak.baris`)
                                format!("{}.{}", obj_id.nama, nama.nama)
                            } else {
                                let obj_ptr = self.turunkan_ekspresi(objek);
                                nilai_arg.push(obj_ptr);
                                let tipe_obj = self.ambil_tipe_ekspresi(objek);
                                format!("{}.{}", tipe_obj, nama.nama)
                            }
                        } else {
                            let obj_ptr = self.turunkan_ekspresi(objek);
                            nilai_arg.push(obj_ptr);
                            let tipe_obj = self.ambil_tipe_ekspresi(objek);
                            format!("{}.{}", tipe_obj, nama.nama)
                        }
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

            JenisEkspresi::Bidang { objek, .. } => {
                let obj_ptr = self.turunkan_ekspresi(objek);
                let tipe_hasil = self.ambil_tipe_ekspresi(expr);
                let field_ptr = self.alokasikan_instruksi(Instruksi::AksesBidang {
                    struktur: obj_ptr,
                    indeks: 0,
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
                    let idx_val =
                        self.alokasikan_instruksi(Instruksi::Konst(Konstanta::Bulat(i as i64)));
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

            JenisEkspresi::Struktur { bidang, .. } => {
                let struct_tipe = self.ambil_tipe_ekspresi(expr);
                let struct_ptr = self.alokasikan_instruksi(Instruksi::Alloca(struct_tipe));
                for (i, b) in bidang.iter().enumerate() {
                    let v = self.turunkan_ekspresi(&b.nilai);
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

            JenisEkspresi::Rentang { awal, akhir, .. } => {
                let start = self.turunkan_ekspresi(awal);
                let end = self.turunkan_ekspresi(akhir);
                let range_ptr =
                    self.alokasikan_instruksi(Instruksi::Alloca(IrTipe::Struktur(vec![
                        IrTipe::I64,
                        IrTipe::I64,
                    ])));
                let f0 = self.alokasikan_instruksi(Instruksi::AksesBidang {
                    struktur: range_ptr,
                    indeks: 0,
                });
                self.tambah_instruksi(Instruksi::Simpan {
                    nilai: start,
                    penunjuk: f0,
                });
                let f1 = self.alokasikan_instruksi(Instruksi::AksesBidang {
                    struktur: range_ptr,
                    indeks: 1,
                });
                self.tambah_instruksi(Instruksi::Simpan {
                    nilai: end,
                    penunjuk: f1,
                });
                range_ptr
            }
        }
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
            JenisEkspresi::Bidang { objek, .. } => {
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
        if let Some(sem_tipe) = self.sema.tipe_ekspresi.get(&expr.rentang) {
            IrTipe::dari_sem_tipe(sem_tipe)
        } else {
            match &expr.jenis {
                JenisEkspresi::Bulat(_) => IrTipe::I64,
                JenisEkspresi::Desimal(_) => IrTipe::F64,
                JenisEkspresi::Bool(_) => IrTipe::I1,
                JenisEkspresi::Teks(_) => IrTipe::Ptr,
                _ => IrTipe::I64,
            }
        }
    }
}

fn tipe_ast_ke_nama(t: &Tipe) -> String {
    match &t.jenis {
        JenisTipe::Nama { nama, .. } => nama.nama.clone(),
        JenisTipe::Pinjam { dalam, .. } => tipe_ast_ke_nama(dalam),
    }
}

fn tipe_ast_ke_ir(t: &Tipe, sema: &HasilSemantik) -> IrTipe {
    let sem = komo_sema::tipe_dari_ast(t, &|n| sema.tabel.cari_tipe(n));
    IrTipe::dari_sem_tipe(&sem)
}
