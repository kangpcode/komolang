// Copyright (c) kangpcode (Dhafa Nazula Permadi). Licensed under MIT or Apache-2.0.

//! Dump teks representasi KIR yang mudah dibaca manusia (`--dump-kir`).
//! Mengonversi ModulIr dan FungsiIr menjadi format SSA tekstual yang terstruktur.

use std::fmt::Write;
use crate::fungsi::FungsiIr;
use crate::instruksi::{Instruksi, Terminator};
use crate::modul::ModulIr;

/// Menghasilkan representasi string SSA lengkap untuk suatu modul KIR.
pub fn cetak_modul(modul: &ModulIr) -> String {
    let mut keluar = String::new();
    let _ = writeln!(keluar, "; Komo Intermediate Representation (KIR)");
    let _ = writeln!(keluar, "modul \"{}\"\n", modul.nama);

    for g in &modul.data_global {
        let modifier = if g.konstan { "konstanta" } else { "global" };
        if let Some(init) = &g.inisialisasi {
            let _ = writeln!(keluar, "{} @{}: {} = {}", modifier, g.nama, g.tipe, init);
        } else {
            let _ = writeln!(keluar, "{} @{}: {}", modifier, g.nama, g.tipe);
        }
    }

    if !modul.data_global.is_empty() {
        let _ = writeln!(keluar);
    }

    for (i, f) in modul.fungsi.iter().enumerate() {
        if i > 0 {
            let _ = writeln!(keluar);
        }
        keluar.push_str(&cetak_fungsi(f));
    }

    keluar
}

/// Menghasilkan representasi string SSA untuk suatu fungsi KIR.
pub fn cetak_fungsi(f: &FungsiIr) -> String {
    let mut keluar = String::new();
    let _ = write!(keluar, "fungsi @{}(", f.nama);
    for (i, p) in f.parameter.iter().enumerate() {
        if i > 0 {
            let _ = write!(keluar, ", ");
        }
        let _ = write!(keluar, "{}: {}", p.nilai, p.tipe);
    }
    let _ = writeln!(keluar, ") -> {} {{", f.tipe_kembali);

    for b in &f.blok {
        if let Some(label) = &b.label {
            let _ = writeln!(keluar, "{}: // {}", b.id, label);
        } else {
            let _ = writeln!(keluar, "{}:", b.id);
        }

        for (hasil, inst) in &b.instruksi {
            let _ = write!(keluar, "    ");
            if let Some(h) = hasil {
                let _ = write!(keluar, "{} = ", h);
            }
            cetak_instruksi(&mut keluar, inst);
            let _ = writeln!(keluar);
        }

        let _ = write!(keluar, "    ");
        if let Some(term) = &b.terminator {
            cetak_terminator(&mut keluar, term);
        } else {
            let _ = write!(keluar, "<blok belum ditutup>");
        }
        let _ = writeln!(keluar);
    }

    let _ = writeln!(keluar, "}}");
    keluar
}

fn cetak_instruksi(out: &mut String, inst: &Instruksi) {
    match inst {
        Instruksi::Konst(k) => {
            let _ = write!(out, "konst {} : {}", k, k.tipe());
        }
        Instruksi::Biner { op, kiri, kanan, tipe } => {
            let _ = write!(out, "biner.{} {}, {} : {}", op, kiri, kanan, tipe);
        }
        Instruksi::Unari { op, operan, tipe } => {
            let _ = write!(out, "unari.{} {} : {}", op, operan, tipe);
        }
        Instruksi::Alloca(t) => {
            let _ = write!(out, "alloca {}", t);
        }
        Instruksi::Muat { tipe, penunjuk } => {
            let _ = write!(out, "muat {}, {}", tipe, penunjuk);
        }
        Instruksi::Simpan { nilai, penunjuk } => {
            let _ = write!(out, "simpan {}, {}", nilai, penunjuk);
        }
        Instruksi::AksesBidang { struktur, indeks } => {
            let _ = write!(out, "akses_bidang {}, {}", struktur, indeks);
        }
        Instruksi::AksesIndeks { target, indeks } => {
            let _ = write!(out, "akses_indeks {}, {}", target, indeks);
        }
        Instruksi::Panggil { fungsi, argumen, tipe_kembali } => {
            let _ = write!(out, "panggil @{}(", fungsi);
            for (i, a) in argumen.iter().enumerate() {
                if i > 0 {
                    let _ = write!(out, ", ");
                }
                let _ = write!(out, "{}", a);
            }
            let _ = write!(out, ") : {}", tipe_kembali);
        }
        Instruksi::Phi { tipe, masukan } => {
            let _ = write!(out, "phi {} ", tipe);
            for (i, (val, blk)) in masukan.iter().enumerate() {
                if i > 0 {
                    let _ = write!(out, ", ");
                }
                let _ = write!(out, "[{}, {}]", val, blk);
            }
        }
        Instruksi::Cast { asal, tipe_tujuan } => {
            let _ = write!(out, "cast {} ke {}", asal, tipe_tujuan);
        }
    }
}

fn cetak_terminator(out: &mut String, term: &Terminator) {
    match term {
        Terminator::Lompat(target) => {
            let _ = write!(out, "lompat {}", target);
        }
        Terminator::Cabang { kondisi, jika_benar, jika_salah } => {
            let _ = write!(out, "cabang {}, {}, {}", kondisi, jika_benar, jika_salah);
        }
        Terminator::Kembali(Some(val)) => {
            let _ = write!(out, "kembali {}", val);
        }
        Terminator::Kembali(None) => {
            let _ = write!(out, "kembali");
        }
        Terminator::TidakBisa(pesan) => {
            let _ = write!(out, "tidak_bisa \"{}\"", pesan.replace('"', "\\\""));
        }
    }
}
