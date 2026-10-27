// Copyright (c) kangpcode (Dhafa Nazula Permadi). Licensed under MIT or Apache-2.0.

//! Emitter Assembly ARM64 (AArch64) langsung dari KIR SSA.
//! Menghasilkan teks assembly GNU AAPCS64 syntax yang deterministik.

use std::collections::HashMap;
use std::fmt::Write;

use komo_ir::{FungsiIr, IdNilai, Instruksi, Konstanta, ModulIr, OpBiner, OpUnari, Terminator};

use crate::arm64::register::{POOL_REGISTER_ALOKASI_ARM64, REG_PARAMETER_ARM64};
use crate::regalloc::{
    alokasikan_register_linear_scan, HasilAlokasiRegister, LokasiNilai,
};

/// Menghasilkan seluruh kode assembly ARM64 untuk `ModulIr`.
pub fn emit_arm64(modul: &ModulIr) -> String {
    let mut out = String::new();
    let mut label_string: HashMap<String, String> = HashMap::new();
    let mut string_counter = 0;

    for f in &modul.fungsi {
        for b in &f.blok {
            for (_, inst) in &b.instruksi {
                if let Instruksi::Konst(Konstanta::Teks(s)) = inst {
                    if !label_string.contains_key(s) {
                        let lbl = format!(".LC_arm_{}", string_counter);
                        string_counter += 1;
                        label_string.insert(s.clone(), lbl);
                    }
                }
            }
        }
    }

    writeln!(out, "\t.arch armv8-a").unwrap();
    if !label_string.is_empty() || !modul.data_global.is_empty() {
        writeln!(out, "\t.section .rodata").unwrap();
        for (teks, label) in &label_string {
            writeln!(out, "{}:", label).unwrap();
            writeln!(out, "\t.string \"{}\"", teks.replace('"', "\\\"")).unwrap();
        }
        for g in &modul.data_global {
            writeln!(out, "\t.globl {}", g.nama).unwrap();
            writeln!(out, "{}:", g.nama).unwrap();
            if let Some(Konstanta::Bulat(n)) = &g.inisialisasi {
                writeln!(out, "\t.xword {}", n).unwrap();
            } else {
                writeln!(out, "\t.zero {}", g.tipe.ukuran_bita().max(8)).unwrap();
            }
        }
    }

    writeln!(out, "\t.text").unwrap();

    let mut ada_utama = false;
    let mut ada_main = false;

    for f in &modul.fungsi {
        if f.nama == "utama" {
            ada_utama = true;
        }
        if f.nama == "main" {
            ada_main = true;
        }
        emit_fungsi_arm64(&mut out, f, &label_string);
    }

    if ada_utama && !ada_main {
        writeln!(out, "\t.globl main").unwrap();
        writeln!(out, "\t.type main, %function").unwrap();
        writeln!(out, "main:").unwrap();
        writeln!(out, "\tstp x29, x30, [sp, -16]!").unwrap();
        writeln!(out, "\tmov x29, sp").unwrap();
        writeln!(out, "\tbl utama").unwrap();
        writeln!(out, "\tldp x29, x30, [sp], 16").unwrap();
        writeln!(out, "\tret").unwrap();
    }

    out
}

fn emit_fungsi_arm64(out: &mut String, f: &FungsiIr, label_string: &HashMap<String, String>) {
    let regalloc = alokasikan_register_linear_scan(f, POOL_REGISTER_ALOKASI_ARM64.len());
    let nama_asm = f.nama.replace('.', "_");

    writeln!(out, "\t.globl {}", nama_asm).unwrap();
    writeln!(out, "\t.type {}, %function", nama_asm).unwrap();
    writeln!(out, "{}:", nama_asm).unwrap();

    // Stack frame size minimal 16 bita dan kelipatan 16 (AAPCS64)
    let frame_size = ((regalloc.ukuran_stack + 16 + 15) / 16) * 16;
    writeln!(out, "\tstp x29, x30, [sp, -{}]!", frame_size).unwrap();
    writeln!(out, "\tmov x29, sp").unwrap();

    // Salin argumen parameter dari x0..x7 ke lokasi register/stack masing-masing
    for (i, p) in f.parameter.iter().enumerate() {
        if i < REG_PARAMETER_ARM64.len() {
            let reg_param = REG_PARAMETER_ARM64[i];
            let dest_loc = format_lokasi_arm64(p.nilai, &regalloc);
            if dest_loc.starts_with('[') {
                writeln!(out, "\tstr {}, {}", reg_param.sebagai_str(), dest_loc).unwrap();
            } else {
                writeln!(out, "\tmov {}, {}", dest_loc, reg_param.sebagai_str()).unwrap();
            }
        }
    }

    for b in &f.blok {
        writeln!(out, ".L_arm_{}_{}:", nama_asm, b.id.0).unwrap();

        for (hasil, inst) in &b.instruksi {
            match inst {
                Instruksi::Phi { .. } => {}
                Instruksi::Konst(k) => {
                    if let Some(res) = hasil {
                        let dest = format_lokasi_arm64(*res, &regalloc);
                        match k {
                            Konstanta::Bulat(n) => {
                                emit_simpan_nilai_arm64(out, &format!("\tmov x0, #{}", n), &dest);
                            }
                            Konstanta::Bool(b) => {
                                emit_simpan_nilai_arm64(
                                    out,
                                    &format!("\tmov x0, #{}", if *b { 1 } else { 0 }),
                                    &dest,
                                );
                            }
                            Konstanta::Karakter(c) => {
                                emit_simpan_nilai_arm64(out, &format!("\tmov x0, #{}", *c as u32), &dest);
                            }
                            Konstanta::Bita(b) => {
                                emit_simpan_nilai_arm64(out, &format!("\tmov x0, #{}", b), &dest);
                            }
                            Konstanta::Nol | Konstanta::Kosong | Konstanta::Desimal(_) => {
                                emit_simpan_nilai_arm64(out, "\tmov x0, xzr", &dest);
                            }
                            Konstanta::Teks(s) => {
                                if let Some(lbl) = label_string.get(s) {
                                    writeln!(out, "\tadrp x0, {}", lbl).unwrap();
                                    writeln!(out, "\tadd x0, x0, :lo12:{}", lbl).unwrap();
                                    emit_simpan_nilai_arm64(out, "", &dest);
                                }
                            }
                        }
                    }
                }

                Instruksi::Biner { op, kiri, kanan, .. } => {
                    let loc_kiri = format_lokasi_arm64(*kiri, &regalloc);
                    let loc_kanan = format_lokasi_arm64(*kanan, &regalloc);
                    let dest = hasil
                        .map(|r| format_lokasi_arm64(r, &regalloc))
                        .unwrap_or_else(|| "x0".to_string());

                    // Muat kiri ke x0, kanan ke x1
                    emit_muat_ke_reg_arm64(out, &loc_kiri, "x0");
                    emit_muat_ke_reg_arm64(out, &loc_kanan, "x1");

                    match op {
                        OpBiner::Tambah => {
                            writeln!(out, "\tadd x0, x0, x1").unwrap();
                        }
                        OpBiner::Kurang => {
                            writeln!(out, "\tsub x0, x0, x1").unwrap();
                        }
                        OpBiner::Kali => {
                            writeln!(out, "\tmul x0, x0, x1").unwrap();
                        }
                        OpBiner::Bagi => {
                            writeln!(out, "\tsdiv x0, x0, x1").unwrap();
                        }
                        OpBiner::Sisa => {
                            // sdiv x2, x0, x1; msub x0, x2, x1, x0
                            writeln!(out, "\tsdiv x2, x0, x1").unwrap();
                            writeln!(out, "\tmsub x0, x2, x1, x0").unwrap();
                        }
                        OpBiner::DanBit => {
                            writeln!(out, "\tand x0, x0, x1").unwrap();
                        }
                        OpBiner::AtauBit => {
                            writeln!(out, "\torr x0, x0, x1").unwrap();
                        }
                        OpBiner::XorBit => {
                            writeln!(out, "\teor x0, x0, x1").unwrap();
                        }
                        OpBiner::GeserKiri => {
                            writeln!(out, "\tlsl x0, x0, x1").unwrap();
                        }
                        OpBiner::GeserKanan => {
                            writeln!(out, "\tasr x0, x0, x1").unwrap();
                        }
                        // Perbandingan
                        OpBiner::SamaDengan => {
                            writeln!(out, "\tcmp x0, x1").unwrap();
                            writeln!(out, "\tcset x0, eq").unwrap();
                        }
                        OpBiner::TidakSama => {
                            writeln!(out, "\tcmp x0, x1").unwrap();
                            writeln!(out, "\tcset x0, ne").unwrap();
                        }
                        OpBiner::KurangDari => {
                            writeln!(out, "\tcmp x0, x1").unwrap();
                            writeln!(out, "\tcset x0, lt").unwrap();
                        }
                        OpBiner::KurangSama => {
                            writeln!(out, "\tcmp x0, x1").unwrap();
                            writeln!(out, "\tcset x0, le").unwrap();
                        }
                        OpBiner::LebihDari => {
                            writeln!(out, "\tcmp x0, x1").unwrap();
                            writeln!(out, "\tcset x0, gt").unwrap();
                        }
                        OpBiner::LebihSama => {
                            writeln!(out, "\tcmp x0, x1").unwrap();
                            writeln!(out, "\tcset x0, ge").unwrap();
                        }
                    }

                    emit_simpan_nilai_arm64(out, "", &dest);
                }

                Instruksi::Unari { op, operan, .. } => {
                    let loc_op = format_lokasi_arm64(*operan, &regalloc);
                    let dest = hasil
                        .map(|r| format_lokasi_arm64(r, &regalloc))
                        .unwrap_or_else(|| "x0".to_string());
                    emit_muat_ke_reg_arm64(out, &loc_op, "x0");
                    match op {
                        OpUnari::Negasi => {
                            writeln!(out, "\tneg x0, x0").unwrap();
                        }
                        OpUnari::Bukan => {
                            writeln!(out, "\teor x0, x0, #1").unwrap();
                        }
                        OpUnari::BukanBit => {
                            writeln!(out, "\tmvn x0, x0").unwrap();
                        }
                    }
                    emit_simpan_nilai_arm64(out, "", &dest);
                }

                Instruksi::Alloca(_) => {
                    if let Some(res) = hasil {
                        let dest = format_lokasi_arm64(*res, &regalloc);
                        writeln!(out, "\tsub x0, x29, #16").unwrap();
                        emit_simpan_nilai_arm64(out, "", &dest);
                    }
                }

                Instruksi::Muat { penunjuk, .. } => {
                    let loc_ptr = format_lokasi_arm64(*penunjuk, &regalloc);
                    let dest = hasil
                        .map(|r| format_lokasi_arm64(r, &regalloc))
                        .unwrap_or_else(|| "x0".to_string());
                    emit_muat_ke_reg_arm64(out, &loc_ptr, "x1");
                    writeln!(out, "\tldr x0, [x1]").unwrap();
                    emit_simpan_nilai_arm64(out, "", &dest);
                }

                Instruksi::Simpan { nilai, penunjuk } => {
                    let loc_val = format_lokasi_arm64(*nilai, &regalloc);
                    let loc_ptr = format_lokasi_arm64(*penunjuk, &regalloc);
                    emit_muat_ke_reg_arm64(out, &loc_val, "x0");
                    emit_muat_ke_reg_arm64(out, &loc_ptr, "x1");
                    writeln!(out, "\tstr x0, [x1]").unwrap();
                }

                Instruksi::AksesBidang { struktur, indeks } => {
                    let loc_struk = format_lokasi_arm64(*struktur, &regalloc);
                    let dest = hasil
                        .map(|r| format_lokasi_arm64(r, &regalloc))
                        .unwrap_or_else(|| "x0".to_string());
                    let offset = (*indeks * 8) as i64;
                    emit_muat_ke_reg_arm64(out, &loc_struk, "x0");
                    writeln!(out, "\tadd x0, x0, #{}", offset).unwrap();
                    emit_simpan_nilai_arm64(out, "", &dest);
                }

                Instruksi::AksesIndeks { target, indeks } => {
                    let loc_target = format_lokasi_arm64(*target, &regalloc);
                    let loc_idx = format_lokasi_arm64(*indeks, &regalloc);
                    let dest = hasil
                        .map(|r| format_lokasi_arm64(r, &regalloc))
                        .unwrap_or_else(|| "x0".to_string());
                    emit_muat_ke_reg_arm64(out, &loc_target, "x0");
                    emit_muat_ke_reg_arm64(out, &loc_idx, "x1");
                    writeln!(out, "\tadd x0, x0, x1, lsl #3").unwrap();
                    emit_simpan_nilai_arm64(out, "", &dest);
                }

                Instruksi::Cast { asal, .. } => {
                    let loc_asal = format_lokasi_arm64(*asal, &regalloc);
                    let dest = hasil
                        .map(|r| format_lokasi_arm64(r, &regalloc))
                        .unwrap_or_else(|| "x0".to_string());
                    emit_muat_ke_reg_arm64(out, &loc_asal, "x0");
                    emit_simpan_nilai_arm64(out, "", &dest);
                }

                Instruksi::Panggil { fungsi, argumen, .. } => {
                    if fungsi == "komo_rt_format_teks" {
                        if let Some(res) = hasil {
                            let dest = format_lokasi_arm64(*res, &regalloc);
                            if let Some(first_arg) = argumen.first() {
                                let loc = format_lokasi_arm64(*first_arg, &regalloc);
                                emit_muat_ke_reg_arm64(out, &loc, "x0");
                                emit_simpan_nilai_arm64(out, "", &dest);
                            }
                        }
                        continue;
                    }

                    for (i, arg) in argumen.iter().enumerate() {
                        if i < REG_PARAMETER_ARM64.len() {
                            let loc_arg = format_lokasi_arm64(*arg, &regalloc);
                            let target_reg = REG_PARAMETER_ARM64[i];
                            emit_muat_ke_reg_arm64(out, &loc_arg, target_reg.sebagai_str());
                        }
                    }

                    let nama_panggil = match fungsi.as_str() {
                        "cetak.baris" | "std/cetak.baris" => "komo_std_cetak_baris",
                        "cetak.teks" | "std/cetak.teks" => "komo_std_cetak_teks",
                        "cetak.bulat" => "komo_std_cetak_bulat",
                        "cetak.desimal" => "komo_std_cetak_desimal",
                        lain => lain,
                    };

                    writeln!(out, "\tbl {}", nama_panggil.replace('.', "_")).unwrap();

                    if let Some(res) = hasil {
                        let dest = format_lokasi_arm64(*res, &regalloc);
                        emit_simpan_nilai_arm64(out, "", &dest);
                    }
                }
            }
        }

        if let Some(term) = &b.terminator {
            match term {
                Terminator::Lompat(target) => {
                    writeln!(out, "\tb .L_arm_{}_{}", nama_asm, target.0).unwrap();
                }
                Terminator::Cabang {
                    kondisi,
                    jika_benar,
                    jika_salah,
                } => {
                    let loc_cond = format_lokasi_arm64(*kondisi, &regalloc);
                    emit_muat_ke_reg_arm64(out, &loc_cond, "x0");
                    writeln!(out, "\tcbnz x0, .L_arm_{}_{}", nama_asm, jika_benar.0).unwrap();
                    writeln!(out, "\tb .L_arm_{}_{}", nama_asm, jika_salah.0).unwrap();
                }
                Terminator::Kembali(Some(val)) => {
                    let loc_val = format_lokasi_arm64(*val, &regalloc);
                    emit_muat_ke_reg_arm64(out, &loc_val, "x0");
                    writeln!(out, "\tldp x29, x30, [sp], {}", frame_size).unwrap();
                    writeln!(out, "\tret").unwrap();
                }
                Terminator::Kembali(None) => {
                    writeln!(out, "\tmov x0, xzr").unwrap();
                    writeln!(out, "\tldp x29, x30, [sp], {}", frame_size).unwrap();
                    writeln!(out, "\tret").unwrap();
                }
                Terminator::TidakBisa(_) => {
                    writeln!(out, "\tbrk #1000").unwrap();
                }
            }
        }
    }

    writeln!(out).unwrap();
}

fn format_lokasi_arm64(id: IdNilai, regalloc: &HasilAlokasiRegister) -> String {
    match regalloc.peta_lokasi.get(&id) {
        Some(LokasiNilai::Register(idx)) => {
            let reg = POOL_REGISTER_ALOKASI_ARM64[*idx % POOL_REGISTER_ALOKASI_ARM64.len()];
            reg.sebagai_str().to_string()
        }
        Some(LokasiNilai::Stack(offset)) => {
            format!("[x29, #{}]", offset)
        }
        None => "[x29, #-16]".to_string(),
    }
}

fn emit_muat_ke_reg_arm64(out: &mut String, src_loc: &str, target_reg: &str) {
    if src_loc == target_reg {
        return;
    }
    if src_loc.starts_with('[') {
        writeln!(out, "\tldr {}, {}", target_reg, src_loc).unwrap();
    } else {
        writeln!(out, "\tmov {}, {}", target_reg, src_loc).unwrap();
    }
}

fn emit_simpan_nilai_arm64(out: &mut String, prefix_inst: &str, dest_loc: &str) {
    if !prefix_inst.is_empty() {
        writeln!(out, "{}", prefix_inst).unwrap();
    }
    if dest_loc == "x0" {
        return;
    }
    if dest_loc.starts_with('[') {
        writeln!(out, "\tstr x0, {}", dest_loc).unwrap();
    } else {
        writeln!(out, "\tmov {}, x0", dest_loc).unwrap();
    }
}
