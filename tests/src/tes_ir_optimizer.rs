// Copyright (c) kangpcode (Dhafa Nazula Permadi). Licensed under MIT or Apache-2.0.

//! Pengujian untuk lintasan optimisasi KIR: Constant Folding, DCE, Mem2Reg, dan Inlining.

use komo_ir::{
    cetak_modul, turunkan_ke_kir, verifikasi_modul, FungsiIr, Instruksi, IrTipe, Konstanta,
    OpBiner, Pengoptimal, Terminator,
};
use komo_parser::urai;
use komo_sema::periksa;

#[test]
fn tes_optimasi_constant_folding_aritmatika() {
    let mut f = FungsiIr::baru("hitung_konstan".to_string(), IrTipe::I64);
    let v0 = f.alokasikan_nilai();
    f.blok[0].tambah_instruksi(Some(v0), Instruksi::Konst(Konstanta::Bulat(10)));
    let v1 = f.alokasikan_nilai();
    f.blok[0].tambah_instruksi(Some(v1), Instruksi::Konst(Konstanta::Bulat(20)));
    let v2 = f.alokasikan_nilai();
    f.blok[0].tambah_instruksi(
        Some(v2),
        Instruksi::Biner {
            op: OpBiner::Tambah,
            kiri: v0,
            kanan: v1,
            tipe: IrTipe::I64,
        },
    );
    f.blok[0].atur_terminator(Terminator::Kembali(Some(v2)));

    let pengoptimal = Pengoptimal::baru(2);
    pengoptimal.optimalkan_fungsi(&mut f);

    // Hasil harus dilipat menjadi 30
    let ada_30 = f.blok[0]
        .instruksi
        .iter()
        .any(|(_, inst)| matches!(inst, Instruksi::Konst(Konstanta::Bulat(30))));
    assert!(ada_30, "Harusnya terlipat menjadi konstanta 30");
}

#[test]
fn tes_optimasi_constant_folding_cabang() {
    let mut f = FungsiIr::baru("cabang_konstan".to_string(), IrTipe::Void);
    let v_true = f.alokasikan_nilai();
    f.blok[0].tambah_instruksi(Some(v_true), Instruksi::Konst(Konstanta::Bool(true)));
    let bb1 = f.buat_blok(Some("target_benar".to_string()));
    let bb2 = f.buat_blok(Some("target_salah".to_string()));
    f.cari_blok_mut(bb1)
        .unwrap()
        .atur_terminator(Terminator::Kembali(None));
    f.cari_blok_mut(bb2)
        .unwrap()
        .atur_terminator(Terminator::Kembali(None));

    f.blok[0].atur_terminator(Terminator::Cabang {
        kondisi: v_true,
        jika_benar: bb1,
        jika_salah: bb2,
    });

    let pengoptimal = Pengoptimal::baru(2);
    pengoptimal.optimalkan_fungsi(&mut f);

    // Cabang harus dilipat menjadi lompat langsung ke bb1
    assert_eq!(f.blok[0].terminator, Some(Terminator::Lompat(bb1)));
    // bb2 harus tereliminasi oleh DCE
    assert_eq!(f.blok.len(), 2);
    assert!(!f.blok.iter().any(|b| b.id == bb2));
}

#[test]
fn tes_optimasi_dead_code_elimination() {
    let mut f = FungsiIr::baru("inst_mati".to_string(), IrTipe::Void);
    let v_mati = f.alokasikan_nilai();
    f.blok[0].tambah_instruksi(Some(v_mati), Instruksi::Konst(Konstanta::Bulat(999))); // Nilai tidak pernah dipakai
    f.blok[0].atur_terminator(Terminator::Kembali(None));

    let pengoptimal = Pengoptimal::baru(2);
    pengoptimal.optimalkan_fungsi(&mut f);

    assert!(
        f.blok[0].instruksi.is_empty(),
        "Instruksi mati harus dihapus oleh DCE"
    );
}

#[test]
fn tes_optimasi_mem2reg_promosi_alokasi() {
    let mut f = FungsiIr::baru("mem2reg_test".to_string(), IrTipe::I64);
    let ptr = f.alokasikan_nilai();
    f.blok[0].tambah_instruksi(Some(ptr), Instruksi::Alloca(IrTipe::I64));
    let val_init = f.alokasikan_nilai();
    f.blok[0].tambah_instruksi(Some(val_init), Instruksi::Konst(Konstanta::Bulat(42)));
    f.blok[0].tambah_instruksi(
        None,
        Instruksi::Simpan {
            nilai: val_init,
            penunjuk: ptr,
        },
    );
    let val_load = f.alokasikan_nilai();
    f.blok[0].tambah_instruksi(
        Some(val_load),
        Instruksi::Muat {
            tipe: IrTipe::I64,
            penunjuk: ptr,
        },
    );
    f.blok[0].atur_terminator(Terminator::Kembali(Some(val_load)));

    let pengoptimal = Pengoptimal::baru(2);
    pengoptimal.optimalkan_fungsi(&mut f);

    // Alloca, simpan, muat harus tereliminasi
    let ada_alloca = f.blok[0]
        .instruksi
        .iter()
        .any(|(_, i)| matches!(i, Instruksi::Alloca(_)));
    let ada_simpan = f.blok[0]
        .instruksi
        .iter()
        .any(|(_, i)| matches!(i, Instruksi::Simpan { .. }));
    let ada_muat = f.blok[0]
        .instruksi
        .iter()
        .any(|(_, i)| matches!(i, Instruksi::Muat { .. }));

    assert!(!ada_alloca, "Alloca harus dihilangkan oleh mem2reg");
    assert!(!ada_simpan, "Simpan harus dihilangkan oleh mem2reg");
    assert!(!ada_muat, "Muat harus dihilangkan oleh mem2reg");
    assert_eq!(
        f.blok[0].terminator,
        Some(Terminator::Kembali(Some(val_init)))
    );
}

#[test]
fn tes_optimasi_inlining_fungsi_kecil() {
    let kode = r#"
        paket inline_test

        fungsi tambah_satu(x: bulat) bulat {
            kembali x + 1
        }

        fungsi hitung(n: bulat) bulat {
            biar hasil = tambah_satu(n)
            kembali hasil
        }
    "#;

    let hasil_urai = urai(kode);
    let hasil_sema = periksa(&hasil_urai.berkas);
    let mut modul = turunkan_ke_kir(&hasil_urai.berkas, &hasil_sema);

    let pengoptimal = Pengoptimal::baru(2);
    pengoptimal.optimalkan_modul(&mut modul);

    assert!(verifikasi_modul(&modul).is_ok());

    let _dump = cetak_modul(&modul);
    // Fungsi @hitung tidak lagi memanggil @tambah_satu karena sudah di-inline
    let hitung_ir = modul.cari_fungsi("hitung").unwrap();
    let memanggil_tambah_satu = hitung_ir.blok.iter().any(|b| {
        b.instruksi.iter().any(|(_, inst)| {
            matches!(inst, Instruksi::Panggil { fungsi, .. } if fungsi == "tambah_satu")
        })
    });
    assert!(
        !memanggil_tambah_satu,
        "Pemanggilan tambah_satu harus sudah di-inline ke dalam hitung"
    );
}

#[test]
fn tes_optimasi_end_to_end_sintaks_lengkap() {
    let kode = include_str!("../../examples/sintaks_lengkap.komo");
    let hasil_urai = urai(kode);
    let hasil_sema = periksa(&hasil_urai.berkas);
    let mut modul = turunkan_ke_kir(&hasil_urai.berkas, &hasil_sema);

    let pengoptimal = Pengoptimal::baru(2);
    pengoptimal.optimalkan_modul(&mut modul);

    // Memastikan modul yang dioptimalkan tetap lolos verifikasi KIR
    let verif = verifikasi_modul(&modul);
    assert!(
        verif.is_ok(),
        "IR yang dioptimalkan harus tetap sah:\n{:?}",
        verif.err()
    );
}
