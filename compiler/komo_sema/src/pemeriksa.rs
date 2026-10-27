// Copyright (c) kangpcode (Dhafa Nazula Permadi). Licensed under MIT or Apache-2.0.

//! Pemeriksa semantik inti (type checking, scope resolution, dan validasi ekspresi/pernyataan).

use crate::eksaustif::periksa_eksaustif;
use crate::ownership::PelacakOwnership;
use crate::simbol::{JenisSimbol, Simbol, TabelSimbol};
use crate::tipe::{tipe_dari_ast, SemTipe};
use komo_ast::*;
use komo_diag::{kode, Diagnostik};
use std::collections::HashMap;

pub struct PemeriksaSemantik {
    pub tabel: TabelSimbol,
    pub diagnostik: Vec<Diagnostik>,
    pub tipe_ekspresi: HashMap<Rentang, SemTipe>,
}

impl PemeriksaSemantik {
    pub fn baru() -> Self {
        Self {
            tabel: TabelSimbol::baru(),
            diagnostik: Vec::new(),
            tipe_ekspresi: HashMap::new(),
        }
    }

    /// Melakukan analisis semantik penuh pada berkas program (Two-Pass).
    pub fn periksa_berkas(&mut self, berkas: &BerkasProgram) {
        // Lintasan 1: Kumpulkan semua deklarasi tipe, fungsi, konstanta, dan metode tingkat atas
        self.kumpulkan_deklarasi_global(berkas);

        // Lintasan 2: Analisis badan fungsi, penugasan, ekspresi, dan aliran kendali
        self.analisis_isi_deklarasi(berkas);
    }

    fn lapor(&mut self, d: Diagnostik) {
        self.diagnostik.push(d);
    }

    /// Lintasan 1: Mengumpulkan tanda tangan simbol tingkat atas.
    fn kumpulkan_deklarasi_global(&mut self, berkas: &BerkasProgram) {
        // Daftarkan modul impor
        for imp in &berkas.impor {
            let nama_alias = if let Some(a) = &imp.alias {
                a.nama.clone()
            } else {
                imp.jalur
                    .split('/')
                    .next_back()
                    .unwrap_or(&imp.jalur)
                    .to_string()
            };
            let _ = self.tabel.tambah_simbol(Simbol {
                nama: nama_alias,
                jenis: JenisSimbol::Modul {
                    nama: imp.jalur.clone(),
                },
                rentang: imp.rentang,
            });
        }

        // Daftarkan tipe bentukan terlebih dahulu
        for dek in &berkas.deklarasi {
            if let Deklarasi::Tipe(t) = dek {
                let nama = t.nama.nama.clone();
                let generik: Vec<String> = t.generik.iter().map(|g| g.nama.clone()).collect();
                let tipe = match &t.isi {
                    IsiTipe::Struktur(bidang_ast) => {
                        let bidang = bidang_ast
                            .iter()
                            .map(|b| {
                                (
                                    b.nama.nama.clone(),
                                    tipe_dari_ast(&b.tipe, &|n| self.tabel.cari_tipe(n)),
                                    b.visibilitas,
                                )
                            })
                            .collect();
                        SemTipe::Struktur {
                            nama: nama.clone(),
                            generik,
                            bidang,
                        }
                    }
                    IsiTipe::Enum(varian_ast) => {
                        let varian = varian_ast
                            .iter()
                            .map(|v| {
                                let tipe_isi = v
                                    .tipe
                                    .iter()
                                    .map(|tp| tipe_dari_ast(tp, &|n| self.tabel.cari_tipe(n)))
                                    .collect();
                                (v.nama.nama.clone(), tipe_isi)
                            })
                            .collect();
                        SemTipe::Enum {
                            nama: nama.clone(),
                            generik,
                            varian,
                        }
                    }
                    IsiTipe::Antarmuka(metode_ast) => {
                        let metode = metode_ast
                            .iter()
                            .map(|m| {
                                let params = m
                                    .parameter
                                    .iter()
                                    .map(|p| tipe_dari_ast(&p.tipe, &|n| self.tabel.cari_tipe(n)))
                                    .collect();
                                let kembalian = if let Some(ret) = &m.tipe_kembalian {
                                    tipe_dari_ast(ret, &|n| self.tabel.cari_tipe(n))
                                } else {
                                    SemTipe::Kosong
                                };
                                (m.nama.nama.clone(), params, kembalian)
                            })
                            .collect();
                        SemTipe::Antarmuka {
                            nama: nama.clone(),
                            generik,
                            metode,
                        }
                    }
                    IsiTipe::Alias(alias_ast) => {
                        tipe_dari_ast(alias_ast, &|n| self.tabel.cari_tipe(n))
                    }
                };

                if let Err(d) = self.tabel.tambah_simbol(Simbol {
                    nama: nama.clone(),
                    jenis: JenisSimbol::Tipe {
                        tipe,
                        visibilitas: t.visibilitas,
                    },
                    rentang: t.rentang,
                }) {
                    self.lapor(d);
                }
            }
        }

        // Daftarkan tanda tangan fungsi dan metode
        for dek in &berkas.deklarasi {
            if let Deklarasi::Fungsi(f) = dek {
                let params: Vec<SemTipe> = f
                    .parameter
                    .iter()
                    .map(|p| tipe_dari_ast(&p.tipe, &|n| self.tabel.cari_tipe(n)))
                    .collect();
                let kembalian = if let Some(ret) = &f.tipe_kembalian {
                    tipe_dari_ast(ret, &|n| self.tabel.cari_tipe(n))
                } else {
                    SemTipe::Kosong
                };
                let tipe_fungsi = SemTipe::Fungsi {
                    parameter: params,
                    kembalian: Box::new(kembalian),
                };

                if let Some(penerima) = &f.penerima {
                    // Ini metode pada tipe penerima
                    let tipe_penerima = tipe_dari_ast(&penerima.tipe, &|n| self.tabel.cari_tipe(n));
                    let nama_penerima = match &tipe_penerima {
                        SemTipe::Pinjam { dalam, .. } => dalam.nama(),
                        lain => lain.nama(),
                    };
                    if let Err(d) = self.tabel.tambah_simbol(Simbol {
                        nama: format!("{}.{}", nama_penerima, f.nama.nama),
                        jenis: JenisSimbol::Metode {
                            tipe_penerima,
                            tipe: tipe_fungsi,
                            visibilitas: f.visibilitas,
                        },
                        rentang: f.rentang,
                    }) {
                        self.lapor(d);
                    }
                } else {
                    // Ini fungsi biasa
                    if let Err(d) = self.tabel.tambah_simbol(Simbol {
                        nama: f.nama.nama.clone(),
                        jenis: JenisSimbol::Fungsi {
                            tipe: tipe_fungsi,
                            visibilitas: f.visibilitas,
                        },
                        rentang: f.rentang,
                    }) {
                        self.lapor(d);
                    }
                }
            }
        }
    }

    /// Lintasan 2: Memeriksa badan deklarasi secara mendalam.
    fn analisis_isi_deklarasi(&mut self, berkas: &BerkasProgram) {
        for dek in &berkas.deklarasi {
            match dek {
                Deklarasi::Konstanta(k) => {
                    let tipe_nilai = self.evaluasi_ekspresi(&k.nilai);
                    if let Some(t_ast) = &k.tipe {
                        let t_eksplisit = tipe_dari_ast(t_ast, &|n| self.tabel.cari_tipe(n));
                        if !tipe_nilai.cocok_dengan(&t_eksplisit) {
                            self.lapor(
                                Diagnostik::galat(
                                    kode::TIPE_TIDAK_COCOK,
                                    format!(
                                        "tipe nilai konstanta tidak cocok: dideklarasikan '{}', tetapi nilai bertipe '{}'",
                                        t_eksplisit.nama(),
                                        tipe_nilai.nama()
                                    ),
                                    k.nilai.rentang,
                                )
                                .dengan_saran("sesuaikan tipe konstanta atau nilainya"),
                            );
                        }
                    }
                    if let Err(d) = self.tabel.tambah_simbol(Simbol {
                        nama: k.nama.nama.clone(),
                        jenis: JenisSimbol::Konstanta { tipe: tipe_nilai },
                        rentang: k.rentang,
                    }) {
                        self.lapor(d);
                    }
                }
                Deklarasi::Variabel(v) => {
                    let tipe_nilai = self.evaluasi_ekspresi(&v.nilai);
                    if let Some(t_ast) = &v.tipe {
                        let t_eksplisit = tipe_dari_ast(t_ast, &|n| self.tabel.cari_tipe(n));
                        if !tipe_nilai.cocok_dengan(&t_eksplisit) {
                            self.lapor(
                                Diagnostik::galat(
                                    kode::TIPE_TIDAK_COCOK,
                                    format!(
                                        "tipe nilai variabel tidak cocok: dideklarasikan '{}', tetapi nilai bertipe '{}'",
                                        t_eksplisit.nama(),
                                        tipe_nilai.nama()
                                    ),
                                    v.nilai.rentang,
                                )
                                .dengan_saran("sesuaikan tipe variabel atau nilai awalnya"),
                            );
                        }
                    }
                    if let Err(d) = self.tabel.tambah_simbol(Simbol {
                        nama: v.nama.nama.clone(),
                        jenis: JenisSimbol::Variabel {
                            tipe: tipe_nilai,
                            mutabel: v.mutabel,
                            sudah_dipindah: false,
                            jumlah_pinjam: 0,
                            pinjam_mutabel: false,
                        },
                        rentang: v.rentang,
                    }) {
                        self.lapor(d);
                    }
                }
                Deklarasi::Fungsi(f) => {
                    let tipe_kembalian = if let Some(ret) = &f.tipe_kembalian {
                        tipe_dari_ast(ret, &|n| self.tabel.cari_tipe(n))
                    } else {
                        SemTipe::Kosong
                    };

                    self.tabel.masuk_lingkup_fungsi(tipe_kembalian);

                    // Masukkan penerima metode ke dalam lingkup fungsi
                    if let Some(penerima) = &f.penerima {
                        let tipe_penerima =
                            tipe_dari_ast(&penerima.tipe, &|n| self.tabel.cari_tipe(n));
                        let _ = self.tabel.tambah_simbol(Simbol {
                            nama: penerima.nama.nama.clone(),
                            jenis: JenisSimbol::Variabel {
                                tipe: tipe_penerima,
                                mutabel: matches!(
                                    penerima.tipe.jenis,
                                    JenisTipe::Pinjam { mutabel: true, .. }
                                ),
                                sudah_dipindah: false,
                                jumlah_pinjam: 0,
                                pinjam_mutabel: false,
                            },
                            rentang: penerima.nama.rentang,
                        });
                    }

                    // Masukkan parameter ke dalam lingkup fungsi
                    for p in &f.parameter {
                        let tipe_param = tipe_dari_ast(&p.tipe, &|n| self.tabel.cari_tipe(n));
                        if let Err(d) = self.tabel.tambah_simbol(Simbol {
                            nama: p.nama.nama.clone(),
                            jenis: JenisSimbol::Variabel {
                                tipe: tipe_param,
                                mutabel: false,
                                sudah_dipindah: false,
                                jumlah_pinjam: 0,
                                pinjam_mutabel: false,
                            },
                            rentang: p.nama.rentang,
                        }) {
                            self.lapor(d);
                        }
                    }

                    // Analisis isi badan fungsi
                    self.analisis_blok(&f.badan);

                    self.tabel.keluar_lingkup();
                }
                Deklarasi::Tipe(_) => {
                    // Sudah dikumpulkan pada lintasan 1
                }
            }
        }
    }

    /// Menganalisis pernyataan di dalam suatu blok leksikal.
    pub fn analisis_blok(&mut self, blok: &Blok) {
        for p in &blok.pernyataan {
            self.analisis_pernyataan(p);
        }
    }

    /// Menganalisis satu pernyataan.
    pub fn analisis_pernyataan(&mut self, p: &Pernyataan) {
        match &p.jenis {
            JenisPernyataan::Variabel {
                mutabel,
                nama,
                tipe,
                nilai,
            } => {
                let tipe_nilai = self.evaluasi_ekspresi(nilai);
                let tipe_final = if let Some(t_ast) = tipe {
                    let t_eksplisit = tipe_dari_ast(t_ast, &|n| self.tabel.cari_tipe(n));
                    if !tipe_nilai.cocok_dengan(&t_eksplisit) {
                        self.lapor(
                            Diagnostik::galat(
                                kode::TIPE_TIDAK_COCOK,
                                format!(
                                    "tipe nilai tidak cocok dengan anotasi variabel: diharapkan '{}', tetapi nilai bertipe '{}'",
                                    t_eksplisit.nama(),
                                    tipe_nilai.nama()
                                ),
                                nilai.rentang,
                            )
                            .dengan_saran("pastikan tipe nilai sesuai dengan anotasi tipe"),
                        );
                    }
                    t_eksplisit
                } else {
                    tipe_nilai
                };

                if let Err(d) = self.tabel.tambah_simbol(Simbol {
                    nama: nama.nama.clone(),
                    jenis: JenisSimbol::Variabel {
                        tipe: tipe_final,
                        mutabel: *mutabel,
                        sudah_dipindah: false,
                        jumlah_pinjam: 0,
                        pinjam_mutabel: false,
                    },
                    rentang: nama.rentang,
                }) {
                    self.lapor(d);
                }
            }
            JenisPernyataan::Tugas {
                target,
                operator: _,
                nilai,
            } => {
                let tipe_nilai = self.evaluasi_ekspresi(nilai);

                // Periksa validitas mutasi target
                if let JenisEkspresi::Pengenal(id) = &target.jenis {
                    if let Err(d) =
                        PelacakOwnership::periksa_mutasi(&mut self.tabel, &id.nama, id.rentang)
                    {
                        self.lapor(d);
                    }
                }

                let tipe_target = self.evaluasi_ekspresi(target);
                if !tipe_nilai.cocok_dengan(&tipe_target) {
                    self.lapor(
                        Diagnostik::galat(
                            kode::TIPE_TIDAK_COCOK,
                            format!(
                                "tidak dapat menugaskan tipe '{}' ke target bertipe '{}'",
                                tipe_nilai.nama(),
                                tipe_target.nama()
                            ),
                            p.rentang,
                        )
                        .dengan_saran("pastikan tipe nilai sisi kanan sesuai dengan tipe target"),
                    );
                }
            }
            JenisPernyataan::Jika {
                kondisi,
                maka,
                lainnya,
            } => {
                let tipe_kondisi = self.evaluasi_ekspresi(kondisi);
                if !tipe_kondisi.cocok_dengan(&SemTipe::Bool) {
                    self.lapor(
                        Diagnostik::galat(
                            kode::KONDISI_BUKAN_BOOL,
                            format!(
                                "kondisi pernyataan 'jika' harus bertipe 'bool', tetapi ditemukan '{}'",
                                tipe_kondisi.nama()
                            ),
                            kondisi.rentang,
                        )
                        .dengan_saran("gunakan ekspresi perbandingan atau boolean"),
                    );
                }

                self.tabel.masuk_lingkup();
                self.analisis_blok(maka);
                self.tabel.keluar_lingkup();

                if let Some(l) = lainnya {
                    self.tabel.masuk_lingkup();
                    self.analisis_pernyataan(l);
                    self.tabel.keluar_lingkup();
                }
            }
            JenisPernyataan::Selagi { kondisi, badan } => {
                let tipe_kondisi = self.evaluasi_ekspresi(kondisi);
                if !tipe_kondisi.cocok_dengan(&SemTipe::Bool) {
                    self.lapor(
                        Diagnostik::galat(
                            kode::KONDISI_BUKAN_BOOL,
                            format!(
                                "kondisi pengulangan 'selagi' harus bertipe 'bool', tetapi ditemukan '{}'",
                                tipe_kondisi.nama()
                            ),
                            kondisi.rentang,
                        )
                        .dengan_saran("gunakan ekspresi perbandingan atau boolean"),
                    );
                }

                self.tabel.masuk_lingkup();
                self.analisis_blok(badan);
                self.tabel.keluar_lingkup();
            }
            JenisPernyataan::Untuk {
                variabel,
                koleksi,
                badan,
            } => {
                let tipe_koleksi = self.evaluasi_ekspresi(koleksi);
                let tipe_elemen = match tipe_koleksi {
                    SemTipe::Larik(item) => *item,
                    SemTipe::Bulat => SemTipe::Bulat,
                    SemTipe::Galat => SemTipe::Galat,
                    _ => {
                        self.lapor(
                            Diagnostik::galat(
                                kode::TIPE_TIDAK_COCOK,
                                format!(
                                    "pengulangan 'untuk' memerlukan larik atau rentang angka, tetapi ditemukan '{}'",
                                    tipe_koleksi.nama()
                                ),
                                koleksi.rentang,
                            )
                            .dengan_saran("gunakan rentang angka (misal '0..5') atau variabel bertipe larik"),
                        );
                        SemTipe::Galat
                    }
                };

                self.tabel.masuk_lingkup();
                let _ = self.tabel.tambah_simbol(Simbol {
                    nama: variabel.nama.clone(),
                    jenis: JenisSimbol::Variabel {
                        tipe: tipe_elemen,
                        mutabel: false,
                        sudah_dipindah: false,
                        jumlah_pinjam: 0,
                        pinjam_mutabel: false,
                    },
                    rentang: variabel.rentang,
                });
                self.analisis_blok(badan);
                self.tabel.keluar_lingkup();
            }
            JenisPernyataan::Pilih {
                nilai,
                kasus,
                lainnya,
            } => {
                let tipe_nilai = self.evaluasi_ekspresi(nilai);
                for k in kasus {
                    let tipe_kasus = self.evaluasi_ekspresi(&k.nilai);
                    if !tipe_kasus.cocok_dengan(&tipe_nilai) {
                        self.lapor(
                            Diagnostik::galat(
                                kode::TIPE_TIDAK_COCOK,
                                format!(
                                    "nilai kasus bertipe '{}' tidak cocok dengan tipe pemilih '{}'",
                                    tipe_kasus.nama(),
                                    tipe_nilai.nama()
                                ),
                                k.nilai.rentang,
                            )
                            .dengan_saran("sesuaikan tipe nilai kasus"),
                        );
                    }
                    self.tabel.masuk_lingkup();
                    for s in &k.badan {
                        self.analisis_pernyataan(s);
                    }
                    self.tabel.keluar_lingkup();
                }

                if let Some(l) = lainnya {
                    self.tabel.masuk_lingkup();
                    for s in l {
                        self.analisis_pernyataan(s);
                    }
                    self.tabel.keluar_lingkup();
                }
            }
            JenisPernyataan::Cocok { nilai, cabang } => {
                let tipe_nilai = self.evaluasi_ekspresi(nilai);

                // Periksa kelengkapan (exhaustiveness) cabang pencocokan
                if let Err(d) = periksa_eksaustif(&tipe_nilai, cabang, p.rentang) {
                    self.lapor(d);
                }

                for c in cabang {
                    self.tabel.masuk_lingkup();
                    self.daftarkan_pola_ke_lingkup(&c.pola, &tipe_nilai);
                    self.analisis_pernyataan(&c.badan);
                    self.tabel.keluar_lingkup();
                }
            }
            JenisPernyataan::Tunda(e) => {
                self.evaluasi_ekspresi(e);
            }
            JenisPernyataan::TundaBlok(b) => {
                self.tabel.masuk_lingkup();
                self.analisis_blok(b);
                self.tabel.keluar_lingkup();
            }
            JenisPernyataan::Jalankan(e) => {
                self.evaluasi_ekspresi(e);
            }
            JenisPernyataan::TidakAman(b) => {
                self.tabel.masuk_lingkup();
                self.analisis_blok(b);
                self.tabel.keluar_lingkup();
            }
            JenisPernyataan::Kembali(opt_e) => {
                if !self.tabel.dalam_fungsi() {
                    self.lapor(
                        Diagnostik::galat(
                            kode::RETURN_DI_LUAR_FUNGSI,
                            "pernyataan 'kembali' hanya diizinkan di dalam fungsi",
                            p.rentang,
                        )
                        .dengan_saran("hapus pernyataan 'kembali' di luar fungsi"),
                    );
                    return;
                }

                let tipe_kembali = if let Some(e) = opt_e {
                    self.evaluasi_ekspresi(e)
                } else {
                    SemTipe::Kosong
                };

                if let Some(diharapkan) = self.tabel.tipe_kembalian_fungsi_aktif() {
                    if !tipe_kembali.cocok_dengan(&diharapkan) {
                        self.lapor(
                            Diagnostik::galat(
                                kode::RETURN_TIDAK_COCOK,
                                format!(
                                    "tipe nilai kembalian '{}' tidak cocok dengan deklarasi fungsi yang mengharapkan '{}'",
                                    tipe_kembali.nama(),
                                    diharapkan.nama()
                                ),
                                p.rentang,
                            )
                            .dengan_saran("sesuaikan nilai kembalian atau tipe kembalian fungsi"),
                        );
                    }
                }
            }
            JenisPernyataan::Blok(b) => {
                self.tabel.masuk_lingkup();
                self.analisis_blok(b);
                self.tabel.keluar_lingkup();
            }
            JenisPernyataan::Ekspresi(e) => {
                self.evaluasi_ekspresi(e);
            }
        }
    }

    fn daftarkan_pola_ke_lingkup(&mut self, pola: &Pola, tipe_nilai: &SemTipe) {
        match &pola.jenis {
            JenisPola::Pengenal(id) => {
                let _ = self.tabel.tambah_simbol(Simbol {
                    nama: id.nama.clone(),
                    jenis: JenisSimbol::Variabel {
                        tipe: tipe_nilai.clone(),
                        mutabel: false,
                        sudah_dipindah: false,
                        jumlah_pinjam: 0,
                        pinjam_mutabel: false,
                    },
                    rentang: id.rentang,
                });
            }
            JenisPola::Varian { nama, argumen } => {
                // Untuk hasil<T, E>
                if let SemTipe::Hasil { sukses, gagal } = tipe_nilai {
                    if nama.nama.eq_ignore_ascii_case("sukses") && !argumen.is_empty() {
                        self.daftarkan_pola_ke_lingkup(&argumen[0], sukses);
                    } else if nama.nama.eq_ignore_ascii_case("gagal") && !argumen.is_empty() {
                        self.daftarkan_pola_ke_lingkup(&argumen[0], gagal);
                    }
                } else if let SemTipe::Mungkin(dalam) = tipe_nilai {
                    if nama.nama.eq_ignore_ascii_case("ada") && !argumen.is_empty() {
                        self.daftarkan_pola_ke_lingkup(&argumen[0], dalam);
                    }
                } else if let SemTipe::Enum { varian, .. } = tipe_nilai {
                    if let Some((_, tipe_isi)) = varian
                        .iter()
                        .find(|(v, _)| v.eq_ignore_ascii_case(&nama.nama))
                    {
                        for (i, p) in argumen.iter().enumerate() {
                            if let Some(t) = tipe_isi.get(i) {
                                self.daftarkan_pola_ke_lingkup(p, t);
                            }
                        }
                    }
                }
            }
            _ => {}
        }
    }

    /// Mengevaluasi dan memverifikasi tipe suatu ekspresi.
    pub fn evaluasi_ekspresi(&mut self, e: &Ekspresi) -> SemTipe {
        let tipe = match &e.jenis {
            JenisEkspresi::Bulat(_) => SemTipe::Bulat,
            JenisEkspresi::Desimal(_) => SemTipe::Desimal,
            JenisEkspresi::Teks(bagian) => {
                for b in bagian {
                    if let BagianTeks::Ekspresi(sub) = b {
                        self.evaluasi_ekspresi(sub);
                    }
                }
                SemTipe::Teks
            }
            JenisEkspresi::Rune(_) => SemTipe::Rune,
            JenisEkspresi::Bool(_) => SemTipe::Bool,
            JenisEkspresi::Kosong => SemTipe::Kosong,
            JenisEkspresi::Pengenal(id) => {
                if let Some(simbol) = self.tabel.cari_simbol(&id.nama) {
                    let tipe = simbol.tipe();
                    if let Err(d) =
                        PelacakOwnership::periksa_penggunaan(&self.tabel, &id.nama, id.rentang)
                    {
                        self.lapor(d);
                    }
                    tipe
                } else {
                    self.lapor(
                        Diagnostik::galat(
                            kode::SIMBOL_TIDAK_DITEMUKAN,
                            format!("nama '{}' tidak ditemukan dalam lingkup ini", id.nama),
                            id.rentang,
                        )
                        .dengan_saran(
                            "pastikan nama variabel, fungsi, atau tipe sudah dideklarasikan",
                        ),
                    );
                    SemTipe::Galat
                }
            }
            JenisEkspresi::Biner {
                operator,
                kiri,
                kanan,
            } => {
                let t_kiri = self.evaluasi_ekspresi(kiri);
                let t_kanan = self.evaluasi_ekspresi(kanan);

                match operator {
                    OperatorBiner::Tambah => {
                        if t_kiri == SemTipe::Teks && t_kanan == SemTipe::Teks {
                            SemTipe::Teks
                        } else if t_kiri.adalah_angka() && t_kanan.adalah_angka() {
                            if t_kiri.adalah_desimal() || t_kanan.adalah_desimal() {
                                SemTipe::Desimal
                            } else {
                                t_kiri
                            }
                        } else {
                            self.lapor(Diagnostik::galat(
                                kode::OPERATOR_TIDAK_DIDUKUNG,
                                format!(
                                    "operator '+' tidak didukung untuk tipe '{}' dan '{}'",
                                    t_kiri.nama(),
                                    t_kanan.nama()
                                ),
                                e.rentang,
                            ));
                            SemTipe::Galat
                        }
                    }
                    OperatorBiner::Kurang
                    | OperatorBiner::Kali
                    | OperatorBiner::Bagi
                    | OperatorBiner::Sisa => {
                        if t_kiri.adalah_angka() && t_kanan.adalah_angka() {
                            if t_kiri.adalah_desimal() || t_kanan.adalah_desimal() {
                                SemTipe::Desimal
                            } else {
                                t_kiri
                            }
                        } else {
                            self.lapor(Diagnostik::galat(
                                kode::OPERATOR_TIDAK_DIDUKUNG,
                                format!("operator aritmatika hanya didukung untuk tipe angka, bukan '{}' dan '{}'", t_kiri.nama(), t_kanan.nama()),
                                e.rentang,
                            ));
                            SemTipe::Galat
                        }
                    }
                    OperatorBiner::Sama | OperatorBiner::TidakSama => {
                        let cocok = if t_kiri.adalah_angka() && t_kanan.adalah_angka() {
                            true
                        } else {
                            t_kiri.cocok_dengan(&t_kanan) || t_kanan.cocok_dengan(&t_kiri)
                        };
                        if !cocok {
                            self.lapor(Diagnostik::galat(
                                kode::TIPE_TIDAK_COCOK,
                                format!("perbandingan kesetaraan memerlukan tipe yang cocok, ditemukan '{}' dan '{}'", t_kiri.nama(), t_kanan.nama()),
                                e.rentang,
                            ));
                        }
                        SemTipe::Bool
                    }
                    OperatorBiner::LebihKecil
                    | OperatorBiner::LebihKecilSama
                    | OperatorBiner::LebihBesar
                    | OperatorBiner::LebihBesarSama => {
                        if !t_kiri.adalah_angka() || !t_kanan.adalah_angka() {
                            self.lapor(Diagnostik::galat(
                                kode::OPERATOR_TIDAK_DIDUKUNG,
                                format!("operator perbandingan hanya didukung untuk tipe angka, ditemukan '{}' dan '{}'", t_kiri.nama(), t_kanan.nama()),
                                e.rentang,
                            ));
                        }
                        SemTipe::Bool
                    }
                    OperatorBiner::Dan | OperatorBiner::Atau => {
                        if !t_kiri.cocok_dengan(&SemTipe::Bool)
                            || !t_kanan.cocok_dengan(&SemTipe::Bool)
                        {
                            self.lapor(Diagnostik::galat(
                                kode::OPERATOR_TIDAK_DIDUKUNG,
                                format!("operator logika memerlukan operan 'bool', ditemukan '{}' dan '{}'", t_kiri.nama(), t_kanan.nama()),
                                e.rentang,
                            ));
                        }
                        SemTipe::Bool
                    }
                }
            }
            JenisEkspresi::Unari { operator, operan } => {
                let t_operan = self.evaluasi_ekspresi(operan);
                match operator {
                    OperatorUnari::Negatif => {
                        if !t_operan.adalah_angka() {
                            self.lapor(Diagnostik::galat(
                                kode::OPERATOR_TIDAK_DIDUKUNG,
                                format!("operator negatif '-' hanya didukung untuk tipe angka, bukan '{}'", t_operan.nama()),
                                e.rentang,
                            ));
                            SemTipe::Galat
                        } else {
                            t_operan
                        }
                    }
                    OperatorUnari::Bukan => {
                        if !t_operan.cocok_dengan(&SemTipe::Bool) {
                            self.lapor(Diagnostik::galat(
                                kode::OPERATOR_TIDAK_DIDUKUNG,
                                format!(
                                    "operator logika bukan '!' memerlukan tipe 'bool', bukan '{}'",
                                    t_operan.nama()
                                ),
                                e.rentang,
                            ));
                            SemTipe::Galat
                        } else {
                            SemTipe::Bool
                        }
                    }
                    OperatorUnari::Pinjam => {
                        if let JenisEkspresi::Pengenal(id) = &operan.jenis {
                            if let Err(d) = PelacakOwnership::lakukan_peminjaman(
                                &mut self.tabel,
                                &id.nama,
                                false,
                                id.rentang,
                            ) {
                                self.lapor(d);
                            }
                        }
                        SemTipe::Pinjam {
                            mutabel: false,
                            dalam: Box::new(t_operan),
                        }
                    }
                    OperatorUnari::PinjamMutabel => {
                        if let JenisEkspresi::Pengenal(id) = &operan.jenis {
                            if let Err(d) = PelacakOwnership::lakukan_peminjaman(
                                &mut self.tabel,
                                &id.nama,
                                true,
                                id.rentang,
                            ) {
                                self.lapor(d);
                            }
                        }
                        SemTipe::Pinjam {
                            mutabel: true,
                            dalam: Box::new(t_operan),
                        }
                    }
                    OperatorUnari::Pindah => {
                        if let JenisEkspresi::Pengenal(id) = &operan.jenis {
                            if let Err(d) = PelacakOwnership::tandai_dipindah(
                                &mut self.tabel,
                                &id.nama,
                                id.rentang,
                            ) {
                                self.lapor(d);
                            }
                        }
                        t_operan
                    }
                }
            }
            JenisEkspresi::Panggil {
                fungsi,
                argumen_tipe,
                argumen,
            } => {
                let tipe_argumen: Vec<SemTipe> =
                    argumen.iter().map(|a| self.evaluasi_ekspresi(a)).collect();

                // 1. Cek panggilan konstruktor/fungsi bawaan seperti `saluran`, `sukses`, `gagal`, `ada`
                if let JenisEkspresi::Pengenal(id) = &fungsi.jenis {
                    match id.nama.as_str() {
                        "saluran" => {
                            let dalam = if !argumen_tipe.is_empty() {
                                tipe_dari_ast(&argumen_tipe[0], &|n| self.tabel.cari_tipe(n))
                            } else {
                                SemTipe::Bulat
                            };
                            return SemTipe::Saluran(Box::new(dalam));
                        }
                        "sukses" => {
                            let isi = tipe_argumen.first().cloned().unwrap_or(SemTipe::Kosong);
                            return SemTipe::Hasil {
                                sukses: Box::new(isi),
                                gagal: Box::new(SemTipe::Teks),
                            };
                        }
                        "gagal" => {
                            let err = tipe_argumen.first().cloned().unwrap_or(SemTipe::Teks);
                            return SemTipe::Hasil {
                                sukses: Box::new(SemTipe::Desimal),
                                gagal: Box::new(err),
                            };
                        }
                        "ada" => {
                            let isi = tipe_argumen.first().cloned().unwrap_or(SemTipe::Kosong);
                            return SemTipe::Mungkin(Box::new(isi));
                        }
                        _ => {}
                    }
                }

                // 2. Cek pemanggilan metode atau fungsi biasa
                let t_fungsi = self.evaluasi_ekspresi(fungsi);
                match t_fungsi {
                    SemTipe::Fungsi {
                        parameter,
                        kembalian,
                    } => {
                        if parameter.len() != tipe_argumen.len() {
                            self.lapor(
                                Diagnostik::galat(
                                    kode::JUMLAH_ARGUMEN_SALAH,
                                    format!(
                                        "pemanggilan memerlukan {} argumen, tetapi diberikan {}",
                                        parameter.len(),
                                        tipe_argumen.len()
                                    ),
                                    e.rentang,
                                )
                                .dengan_saran("periksa daftar parameter fungsi yang dipanggil"),
                            );
                        } else {
                            for (p_tipe, a_tipe) in parameter.iter().zip(tipe_argumen.iter()) {
                                if !a_tipe.cocok_dengan(p_tipe) {
                                    self.lapor(
                                        Diagnostik::galat(
                                            kode::TIPE_TIDAK_COCOK,
                                            format!(
                                                "tipe argumen tidak cocok: diharapkan '{}', tetapi diberikan '{}'",
                                                p_tipe.nama(),
                                                a_tipe.nama()
                                            ),
                                            e.rentang,
                                        )
                                        .dengan_saran("sesuaikan tipe argumen yang dioper"),
                                    );
                                }
                            }
                        }
                        *kembalian
                    }
                    SemTipe::Galat => SemTipe::Galat,
                    lain => {
                        self.lapor(
                            Diagnostik::galat(
                                kode::BUKAN_FUNGSI,
                                format!(
                                    "tipe '{}' bukan fungsi dan tidak dapat dipanggil",
                                    lain.nama()
                                ),
                                fungsi.rentang,
                            )
                            .dengan_saran(
                                "hanya fungsi atau metode yang dapat dipanggil dengan '()'",
                            ),
                        );
                        SemTipe::Galat
                    }
                }
            }
            JenisEkspresi::Bidang { objek, nama } => {
                // Cek apakah ini pemanggilan modul std (seperti `cetak.baris`)
                if let JenisEkspresi::Pengenal(id) = &objek.jenis {
                    if let Some(s) = self.tabel.cari_simbol(&id.nama) {
                        if let JenisSimbol::Modul { .. } = &s.jenis {
                            // Fungsi pustaka modul bawaan (misal cetak.baris)
                            return SemTipe::Fungsi {
                                parameter: vec![SemTipe::Generik("T".into())],
                                kembalian: Box::new(SemTipe::Kosong),
                            };
                        }
                    }
                }

                let t_objek = self.evaluasi_ekspresi(objek);
                let tipe_inti = match &t_objek {
                    SemTipe::Pinjam { dalam, .. } => *dalam.clone(),
                    lain => lain.clone(),
                };

                // Cek metode bawaan saluran (`kirim`, `terima`, `tutup`)
                if let SemTipe::Saluran(dalam) = &tipe_inti {
                    match nama.nama.as_str() {
                        "kirim" => {
                            return SemTipe::Fungsi {
                                parameter: vec![*dalam.clone()],
                                kembalian: Box::new(SemTipe::Kosong),
                            };
                        }
                        "terima" => {
                            return SemTipe::Fungsi {
                                parameter: vec![],
                                kembalian: dalam.clone(),
                            };
                        }
                        "tutup" => {
                            return SemTipe::Fungsi {
                                parameter: vec![],
                                kembalian: Box::new(SemTipe::Kosong),
                            };
                        }
                        _ => {}
                    }
                }

                // Cek metode kustom yang terdaftar pada tipe ini: `{tipe}.{metode}`
                let kunci_metode = format!("{}.{}", tipe_inti.nama(), nama.nama);
                if let Some(simbol) = self.tabel.cari_simbol(&kunci_metode) {
                    if let JenisSimbol::Metode { tipe, .. } = &simbol.jenis {
                        return tipe.clone();
                    }
                }

                // Cek bidang pada struktur
                if let SemTipe::Struktur { bidang, .. } = &tipe_inti {
                    if let Some((_, t_bidang, _)) = bidang.iter().find(|(b, _, _)| b == &nama.nama)
                    {
                        return t_bidang.clone();
                    } else {
                        self.lapor(
                            Diagnostik::galat(
                                kode::BIDANG_TIDAK_DITEMUKAN,
                                format!(
                                    "struktur '{}' tidak memiliki bidang atau metode bernama '{}'",
                                    tipe_inti.nama(),
                                    nama.nama
                                ),
                                nama.rentang,
                            )
                            .dengan_saran("periksa penulisan nama bidang atau deklarasi struktur"),
                        );
                        return SemTipe::Galat;
                    }
                }

                if tipe_inti != SemTipe::Galat {
                    self.lapor(
                        Diagnostik::galat(
                            kode::BUKAN_STRUKTUR,
                            format!(
                                "tipe '{}' bukan struktur dan tidak memiliki bidang",
                                tipe_inti.nama()
                            ),
                            objek.rentang,
                        )
                        .dengan_saran(
                            "akses bidang '.' hanya berlaku untuk tipe struktur atau modul",
                        ),
                    );
                }
                SemTipe::Galat
            }
            JenisEkspresi::Indeks { objek, indeks } => {
                let t_objek = self.evaluasi_ekspresi(objek);
                let t_indeks = self.evaluasi_ekspresi(indeks);

                match t_objek {
                    SemTipe::Larik(item) => {
                        if !t_indeks.adalah_bulat() {
                            self.lapor(
                                Diagnostik::galat(
                                    kode::INDEKS_BUKAN_BULAT,
                                    format!(
                                        "indeks larik harus berupa bilangan bulat, bukan '{}'",
                                        t_indeks.nama()
                                    ),
                                    indeks.rentang,
                                )
                                .dengan_saran("gunakan angka bulat untuk indeks larik"),
                            );
                        }
                        *item
                    }
                    SemTipe::Peta { kunci, nilai } => {
                        if !t_indeks.cocok_dengan(&kunci) {
                            self.lapor(
                                Diagnostik::galat(
                                    kode::TIPE_TIDAK_COCOK,
                                    format!(
                                        "kunci peta bertipe '{}', tetapi diberikan indeks bertipe '{}'",
                                        kunci.nama(),
                                        t_indeks.nama()
                                    ),
                                    indeks.rentang,
                                )
                                .dengan_saran("gunakan tipe kunci yang cocok"),
                            );
                        }
                        *nilai
                    }
                    SemTipe::Galat => SemTipe::Galat,
                    lain => {
                        self.lapor(
                            Diagnostik::galat(
                                kode::BUKAN_LARIK,
                                format!("tipe '{}' tidak dapat diindeks dengan '[]'", lain.nama()),
                                objek.rentang,
                            )
                            .dengan_saran(
                                "operasi pengindeksan hanya didukung untuk larik dan peta",
                            ),
                        );
                        SemTipe::Galat
                    }
                }
            }
            JenisEkspresi::Struktur { nama, bidang } => {
                if let Some(t_def) = self.tabel.cari_tipe(&nama.nama) {
                    if let SemTipe::Struktur {
                        bidang: def_bidang, ..
                    } = &t_def
                    {
                        for b in bidang {
                            let t_nilai = self.evaluasi_ekspresi(&b.nilai);
                            if let Some((_, t_diharapkan, _)) =
                                def_bidang.iter().find(|(n, _, _)| n == &b.nama.nama)
                            {
                                if !t_nilai.cocok_dengan(t_diharapkan) {
                                    self.lapor(
                                        Diagnostik::galat(
                                            kode::TIPE_TIDAK_COCOK,
                                            format!(
                                                "tipe bidang '{}.{}' adalah '{}', tetapi diberikan nilai bertipe '{}'",
                                                nama.nama,
                                                b.nama.nama,
                                                t_diharapkan.nama(),
                                                t_nilai.nama()
                                            ),
                                            b.nilai.rentang,
                                        )
                                        .dengan_saran("sesuaikan tipe nilai inisialisasi bidang"),
                                    );
                                }
                            } else {
                                self.lapor(
                                    Diagnostik::galat(
                                        kode::BIDANG_TIDAK_DITEMUKAN,
                                        format!(
                                            "struktur '{}' tidak memiliki bidang '{}'",
                                            nama.nama, b.nama.nama
                                        ),
                                        b.nama.rentang,
                                    )
                                    .dengan_saran("periksa deklarasi bidang struktur"),
                                );
                            }
                        }
                        t_def
                    } else {
                        self.lapor(
                            Diagnostik::galat(
                                kode::BUKAN_STRUKTUR,
                                format!("'{}' bukan tipe struktur", nama.nama),
                                nama.rentang,
                            )
                            .dengan_saran("hanya tipe struktur yang dapat diinisialisasi dengan sintaks '{...}'"),
                        );
                        SemTipe::Galat
                    }
                } else {
                    self.lapor(
                        Diagnostik::galat(
                            kode::TIPE_TIDAK_DITEMUKAN,
                            format!("tipe struktur '{}' tidak ditemukan", nama.nama),
                            nama.rentang,
                        )
                        .dengan_saran("pastikan struktur sudah dideklarasikan"),
                    );
                    SemTipe::Galat
                }
            }
            JenisEkspresi::Larik(item) => {
                if item.is_empty() {
                    SemTipe::Larik(Box::new(SemTipe::Galat))
                } else {
                    let t_pertama = self.evaluasi_ekspresi(&item[0]);
                    for el in &item[1..] {
                        let t_el = self.evaluasi_ekspresi(el);
                        if !t_el.cocok_dengan(&t_pertama) {
                            self.lapor(
                                Diagnostik::galat(
                                    kode::TIPE_TIDAK_COCOK,
                                    format!(
                                        "elemen larik harus homogen bertipe '{}', tetapi ditemukan elemen bertipe '{}'",
                                        t_pertama.nama(),
                                        t_el.nama()
                                    ),
                                    el.rentang,
                                )
                                .dengan_saran("pastikan seluruh elemen larik memiliki tipe yang sama"),
                            );
                        }
                    }
                    SemTipe::Larik(Box::new(t_pertama))
                }
            }
            JenisEkspresi::Rentang { awal, akhir, .. } => {
                let t_awal = self.evaluasi_ekspresi(awal);
                let t_akhir = self.evaluasi_ekspresi(akhir);
                if !t_awal.adalah_bulat() || !t_akhir.adalah_bulat() {
                    self.lapor(
                        Diagnostik::galat(
                            kode::OPERATOR_TIDAK_DIDUKUNG,
                            format!(
                                "operator rentang '..' memerlukan nilai bilangan bulat, ditemukan '{}' dan '{}'",
                                t_awal.nama(),
                                t_akhir.nama()
                            ),
                            e.rentang,
                        )
                        .dengan_saran("gunakan angka bulat untuk batas rentang"),
                    );
                }
                SemTipe::Bulat
            }
        };

        self.tipe_ekspresi.insert(e.rentang, tipe.clone());
        tipe
    }
}
