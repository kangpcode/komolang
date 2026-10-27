// Copyright (c) kangpcode (Dhafa Nazula Permadi). Licensed under MIT or Apache-2.0.

//! Pengujian diagnostik dan penolakan IR tidak valid oleh KIR Verifier.

use komo_ir::{
    verifikasi_fungsi, FungsiIr, IdBlok, IdNilai, Instruksi, IrTipe, Konstanta, OpBiner, Terminator,
};

#[test]
fn tes_verifier_tolak_blok_tanpa_terminator() {
    let f = FungsiIr::baru("tanpa_term".to_string(), IrTipe::Void);
    // f.blok[0] belum memiliki terminator
    let res = verifikasi_fungsi(&f);
    assert!(res.is_err());
    let pesan = format!("{:?}", res.err().unwrap());
    assert!(pesan.contains("tidak diakhiri dengan terminator"));
}

#[test]
fn tes_verifier_tolak_lompat_ke_blok_tak_dikenal() {
    let mut f = FungsiIr::baru("lompat_ngawur".to_string(), IrTipe::Void);
    f.blok[0].atur_terminator(Terminator::Lompat(IdBlok(999))); // Blok 999 tidak ada
    let res = verifikasi_fungsi(&f);
    assert!(res.is_err());
    let pesan = format!("{:?}", res.err().unwrap());
    assert!(pesan.contains("merujuk ke blok penerus bb999 yang tidak ada"));
}

#[test]
fn tes_verifier_tolak_nilai_digunakan_sebelum_didefinisikan() {
    let mut f = FungsiIr::baru("use_before_def".to_string(), IrTipe::Void);
    // Menggunakan %5 padahal belum pernah dibuat
    f.blok[0].tambah_instruksi(
        Some(IdNilai(0)),
        Instruksi::Biner {
            op: OpBiner::Tambah,
            kiri: IdNilai(5),
            kanan: IdNilai(5),
            tipe: IrTipe::I64,
        },
    );
    f.blok[0].atur_terminator(Terminator::Kembali(None));
    let res = verifikasi_fungsi(&f);
    assert!(res.is_err());
    let pesan = format!("{:?}", res.err().unwrap());
    assert!(pesan.contains("digunakan sebelum didefinisikan"));
}

#[test]
fn tes_verifier_tolak_definisi_ganda_id_nilai() {
    let mut f = FungsiIr::baru("double_def".to_string(), IrTipe::Void);
    f.blok[0].tambah_instruksi(Some(IdNilai(0)), Instruksi::Konst(Konstanta::Bulat(1)));
    f.blok[0].tambah_instruksi(Some(IdNilai(0)), Instruksi::Konst(Konstanta::Bulat(2))); // ID 0 lagi!
    f.blok[0].atur_terminator(Terminator::Kembali(None));
    let res = verifikasi_fungsi(&f);
    assert!(res.is_err());
    let pesan = format!("{:?}", res.err().unwrap());
    assert!(pesan.contains("didefinisikan lebih dari sekali"));
}

#[test]
fn tes_verifier_tolak_phi_setelah_non_phi() {
    let mut f = FungsiIr::baru("phi_salah_tempat".to_string(), IrTipe::Void);
    let val_k = f.alokasikan_nilai();
    f.blok[0].tambah_instruksi(Some(val_k), Instruksi::Konst(Konstanta::Bulat(10)));
    // Instruksi phi ditaruh setelah konstanta
    let val_phi = f.alokasikan_nilai();
    f.blok[0].tambah_instruksi(
        Some(val_phi),
        Instruksi::Phi {
            tipe: IrTipe::I64,
            masukan: vec![],
        },
    );
    f.blok[0].atur_terminator(Terminator::Kembali(None));
    let res = verifikasi_fungsi(&f);
    assert!(res.is_err());
    let pesan = format!("{:?}", res.err().unwrap());
    assert!(pesan.contains("harus berada di awal blok sebelum instruksi non-phi"));
}

#[test]
fn tes_verifier_tolak_kondisi_percabangan_bukan_i1() {
    let mut f = FungsiIr::baru("cabang_bukan_bool".to_string(), IrTipe::Void);
    let val_bukan_bool = f.alokasikan_nilai();
    f.blok[0].tambah_instruksi(
        Some(val_bukan_bool),
        Instruksi::Konst(Konstanta::Bulat(42)), // i64 bukan i1
    );
    let bb1 = f.buat_blok(None);
    let bb2 = f.buat_blok(None);
    f.cari_blok_mut(bb1)
        .unwrap()
        .atur_terminator(Terminator::Kembali(None));
    f.cari_blok_mut(bb2)
        .unwrap()
        .atur_terminator(Terminator::Kembali(None));

    f.blok[0].atur_terminator(Terminator::Cabang {
        kondisi: val_bukan_bool,
        jika_benar: bb1,
        jika_salah: bb2,
    });

    let res = verifikasi_fungsi(&f);
    assert!(res.is_err());
    let pesan = format!("{:?}", res.err().unwrap());
    assert!(pesan.contains("harus bertipe 'i1'"));
}

#[test]
fn tes_verifier_tolak_tipe_kembali_tidak_cocok() {
    let mut f = FungsiIr::baru("ret_mismatch".to_string(), IrTipe::I64);
    let val_float = f.alokasikan_nilai();
    f.blok[0].tambah_instruksi(
        Some(val_float),
        Instruksi::Konst(Konstanta::Desimal(1.5)), // f64 padahal fungsi kembalikan i64
    );
    f.blok[0].atur_terminator(Terminator::Kembali(Some(val_float)));

    let res = verifikasi_fungsi(&f);
    assert!(res.is_err());
    let pesan = format!("{:?}", res.err().unwrap());
    assert!(pesan.contains("tetapi fungsi mengembalikan 'i64'"));
}
