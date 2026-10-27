// Copyright (c) kangpcode (Dhafa Nazula Permadi). Licensed under MIT or Apache-2.0.

//! Emitter Assembly x86_64 langsung dari KIR SSA.
//! Menghasilkan teks assembly GNU / AT&T syntax yang deterministik dan siap diassemble.

use std::collections::HashMap;
use std::fmt::Write;

use komo_ir::{FungsiIr, IdNilai, Instruksi, Konstanta, ModulIr, OpBiner, OpUnari, Terminator};

use crate::regalloc::alokasikan_register_linear_scan;
use crate::x86_64::register::{POOL_REGISTER_ALOKASI_X86, REG_PARAMETER_X86};

/// Menghasilkan seluruh kode assembly x86_64 untuk `ModulIr`.
pub fn emit_x86_64(modul: &ModulIr) -> String {
    let mut out = String::new();
    let mut label_string: HashMap<String, String> = HashMap::new();
    let mut string_counter = 0;

    // 1. Kumpulkan seluruh literal string untuk bagian .rodata
    for f in &modul.fungsi {
        for b in &f.blok {
            for (_, inst) in &b.instruksi {
                if let Instruksi::Konst(Konstanta::Teks(s)) = inst {
                    if !label_string.contains_key(s) {
                        let lbl = format!(".LC_komo_{}", string_counter);
                        string_counter += 1;
                        label_string.insert(s.clone(), lbl);
                    }
                }
            }
        }
    }

    // 2. Tulis header dan bagian .rodata jika ada string literal atau data global
    writeln!(out, "\t.file \"{}.komo\"", modul.nama).unwrap();
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
                writeln!(out, "\t.quad {}", n).unwrap();
            } else {
                writeln!(out, "\t.zero {}", g.tipe.ukuran_bita().max(8)).unwrap();
            }
        }
    }

    // 3. Tulis bagian .text untuk seluruh fungsi
    writeln!(out, "\t.text").unwrap();

    let mut ada_fungsi_utama = false;
    let mut ada_fungsi_main = false;

    for f in &modul.fungsi {
        if f.nama == "utama" {
            ada_fungsi_utama = true;
        }
        if f.nama == "main" {
            ada_fungsi_main = true;
        }
        emit_fungsi_x86_64(&mut out, f, &label_string);
    }

    // 4. Jika terdapat fungsi `utama` dan belum ada fungsi `main`,
    // buatkan pembungkus C entrypoint standar `main`
    if ada_fungsi_utama && !ada_fungsi_main {
        writeln!(out, "\t.globl main").unwrap();
        writeln!(out, "\t.type main, @function").unwrap();
        writeln!(out, "main:").unwrap();
        writeln!(out, "\tpushq %rbp").unwrap();
        writeln!(out, "\tmovq %rsp, %rbp").unwrap();
        writeln!(out, "\tcall utama").unwrap();
        writeln!(out, "\tmovq %rbp, %rsp").unwrap();
        writeln!(out, "\tpopq %rbp").unwrap();
        writeln!(out, "\tretq").unwrap();
    }

    // 5. Tandai non-executable stack untuk keamanan standar OS
    writeln!(out, "\t.section .note.GNU-stack,\"\",@progbits").unwrap();

    out
}

struct TataLetakStack {
    peta_nilai: HashMap<IdNilai, i64>,
    peta_alloca: HashMap<IdNilai, i64>,
    frame_size: usize,
}

fn hitung_tata_letak_stack(f: &FungsiIr) -> TataLetakStack {
    let mut peta_nilai: HashMap<IdNilai, i64> = HashMap::new();
    let mut peta_alloca: HashMap<IdNilai, i64> = HashMap::new();
    let mut slot_counter: usize = 0;

    // 1. Slot untuk setiap parameter fungsi
    for p in &f.parameter {
        slot_counter += 1;
        peta_nilai.insert(p.nilai, (slot_counter * 8) as i64);
    }

    // 2. Slot untuk setiap nilai SSA dan buffer Alloca
    for b in &f.blok {
        for (hasil, inst) in &b.instruksi {
            if let Some(res) = hasil {
                if !peta_nilai.contains_key(res) {
                    slot_counter += 1;
                    peta_nilai.insert(*res, (slot_counter * 8) as i64);
                }
            }
            if let (Some(res), Instruksi::Alloca(tipe)) = (hasil, inst) {
                let bytes = tipe.ukuran_bita().max(8);
                let slots = (bytes + 7) / 8;
                slot_counter += slots;
                peta_alloca.insert(*res, (slot_counter * 8) as i64);
            }
        }
    }

    let total_bytes = slot_counter * 8;
    let frame_size = ((total_bytes + 15) / 16) * 16;
    let frame_size = frame_size.max(16);

    TataLetakStack {
        peta_nilai,
        peta_alloca,
        frame_size,
    }
}

fn format_lokasi(id: IdNilai, layout: &TataLetakStack) -> String {
    let offset = layout.peta_nilai.get(&id).copied().unwrap_or(8);
    format!("-{}(%rbp)", offset)
}

fn emit_resolusi_phi_x86(
    out: &mut String,
    f: &FungsiIr,
    bb_asal_id: komo_ir::IdBlok,
    bb_tujuan_id: komo_ir::IdBlok,
    layout: &TataLetakStack,
) {
    if let Some(target_bb) = f.blok.iter().find(|b| b.id == bb_tujuan_id) {
        for (hasil, inst) in &target_bb.instruksi {
            if let (Some(dest_id), Instruksi::Phi { masukan, .. }) = (hasil, inst) {
                for (val, blok_id) in masukan {
                    if *blok_id == bb_asal_id {
                        let loc_src = format_lokasi(*val, layout);
                        let loc_dest = format_lokasi(*dest_id, layout);
                        writeln!(out, "\tmovq {}, %rax", loc_src).unwrap();
                        writeln!(out, "\tmovq %rax, {}", loc_dest).unwrap();
                    }
                }
            }
        }
    }
}

fn emit_fungsi_x86_64(out: &mut String, f: &FungsiIr, label_string: &HashMap<String, String>) {
    let _regalloc = alokasikan_register_linear_scan(f, POOL_REGISTER_ALOKASI_X86.len());
    let layout = hitung_tata_letak_stack(f);

    // Normalisasi nama fungsi
    let nama_asm = f.nama.replace('.', "_");

    writeln!(out, "\t.globl {}", nama_asm).unwrap();
    writeln!(out, "\t.type {}, @function", nama_asm).unwrap();
    writeln!(out, "{}:", nama_asm).unwrap();

    // Prologue fungsi
    writeln!(out, "\tpushq %rbp").unwrap();
    writeln!(out, "\tmovq %rsp, %rbp").unwrap();
    writeln!(out, "\tsubq ${}, %rsp", layout.frame_size).unwrap();

    // Salin argumen parameter dari register System V ke lokasi alokasi masing-masing
    for (i, p) in f.parameter.iter().enumerate() {
        if i < REG_PARAMETER_X86.len() {
            let reg_param = REG_PARAMETER_X86[i];
            let dest_loc = format_lokasi(p.nilai, &layout);
            writeln!(out, "\tmovq {}, {}", reg_param.sebagai_att(), dest_loc).unwrap();
        }
    }

    // Iterasi seluruh blok dasar
    for b in &f.blok {
        writeln!(out, ".L_{}_{}:", nama_asm, b.id.0).unwrap();

        for (hasil, inst) in &b.instruksi {
            match inst {
                Instruksi::Phi { .. } => {
                    // Phi diproses di transisi percabangan terminator
                }
                Instruksi::Konst(k) => {
                    if let Some(res) = hasil {
                        let dest = format_lokasi(*res, &layout);
                        match k {
                            Konstanta::Bulat(n) => {
                                writeln!(out, "\tmovq ${}, {}", n, dest).unwrap();
                            }
                            Konstanta::Bool(b) => {
                                writeln!(out, "\tmovq ${}, {}", if *b { 1 } else { 0 }, dest).unwrap();
                            }
                            Konstanta::Karakter(c) => {
                                writeln!(out, "\tmovq ${}, {}", *c as u32, dest).unwrap();
                            }
                            Konstanta::Bita(b) => {
                                writeln!(out, "\tmovq ${}, {}", b, dest).unwrap();
                            }
                            Konstanta::Nol | Konstanta::Kosong => {
                                writeln!(out, "\tmovq $0, {}", dest).unwrap();
                            }
                            Konstanta::Teks(s) => {
                                if let Some(lbl) = label_string.get(s) {
                                    writeln!(out, "\tleaq {}(%rip), %rax", lbl).unwrap();
                                    writeln!(out, "\tmovq %rax, {}", dest).unwrap();
                                }
                            }
                            Konstanta::Desimal(_) => {
                                writeln!(out, "\tmovq $0, {}", dest).unwrap();
                            }
                        }
                    }
                }

                Instruksi::Biner { op, kiri, kanan, .. } => {
                    let loc_kiri = format_lokasi(*kiri, &layout);
                    let loc_kanan = format_lokasi(*kanan, &layout);
                    let dest = hasil.map(|r| format_lokasi(r, &layout)).unwrap_or_else(|| "%rax".to_string());

                    // Muat operan kiri ke %rax
                    writeln!(out, "\tmovq {}, %rax", loc_kiri).unwrap();

                    match op {
                        OpBiner::Tambah => {
                            writeln!(out, "\taddq {}, %rax", loc_kanan).unwrap();
                            writeln!(out, "\tmovq %rax, {}", dest).unwrap();
                        }
                        OpBiner::Kurang => {
                            writeln!(out, "\tsubq {}, %rax", loc_kanan).unwrap();
                            writeln!(out, "\tmovq %rax, {}", dest).unwrap();
                        }
                        OpBiner::Kali => {
                            writeln!(out, "\timulq {}, %rax", loc_kanan).unwrap();
                            writeln!(out, "\tmovq %rax, {}", dest).unwrap();
                        }
                        OpBiner::Bagi => {
                            writeln!(out, "\tcqto").unwrap();
                            writeln!(out, "\tmovq {}, %rcx", loc_kanan).unwrap();
                            writeln!(out, "\tidivq %rcx").unwrap();
                            writeln!(out, "\tmovq %rax, {}", dest).unwrap();
                        }
                        OpBiner::Sisa => {
                            writeln!(out, "\tcqto").unwrap();
                            writeln!(out, "\tmovq {}, %rcx", loc_kanan).unwrap();
                            writeln!(out, "\tidivq %rcx").unwrap();
                            writeln!(out, "\tmovq %rdx, {}", dest).unwrap();
                        }
                        OpBiner::DanBit => {
                            writeln!(out, "\tandq {}, %rax", loc_kanan).unwrap();
                            writeln!(out, "\tmovq %rax, {}", dest).unwrap();
                        }
                        OpBiner::AtauBit => {
                            writeln!(out, "\torq {}, %rax", loc_kanan).unwrap();
                            writeln!(out, "\tmovq %rax, {}", dest).unwrap();
                        }
                        OpBiner::XorBit => {
                            writeln!(out, "\txorq {}, %rax", loc_kanan).unwrap();
                            writeln!(out, "\tmovq %rax, {}", dest).unwrap();
                        }
                        OpBiner::GeserKiri => {
                            writeln!(out, "\tmovq {}, %rcx", loc_kanan).unwrap();
                            writeln!(out, "\tshlq %cl, %rax").unwrap();
                            writeln!(out, "\tmovq %rax, {}", dest).unwrap();
                        }
                        OpBiner::GeserKanan => {
                            writeln!(out, "\tmovq {}, %rcx", loc_kanan).unwrap();
                            writeln!(out, "\tsarq %cl, %rax").unwrap();
                            writeln!(out, "\tmovq %rax, {}", dest).unwrap();
                        }
                        // Perbandingan integer
                        OpBiner::SamaDengan => {
                            writeln!(out, "\tcmpq {}, %rax", loc_kanan).unwrap();
                            writeln!(out, "\tsete %al").unwrap();
                            writeln!(out, "\tmovzbq %al, %rax").unwrap();
                            writeln!(out, "\tmovq %rax, {}", dest).unwrap();
                        }
                        OpBiner::TidakSama => {
                            writeln!(out, "\tcmpq {}, %rax", loc_kanan).unwrap();
                            writeln!(out, "\tsetne %al").unwrap();
                            writeln!(out, "\tmovzbq %al, %rax").unwrap();
                            writeln!(out, "\tmovq %rax, {}", dest).unwrap();
                        }
                        OpBiner::KurangDari => {
                            writeln!(out, "\tcmpq {}, %rax", loc_kanan).unwrap();
                            writeln!(out, "\tsetl %al").unwrap();
                            writeln!(out, "\tmovzbq %al, %rax").unwrap();
                            writeln!(out, "\tmovq %rax, {}", dest).unwrap();
                        }
                        OpBiner::KurangSama => {
                            writeln!(out, "\tcmpq {}, %rax", loc_kanan).unwrap();
                            writeln!(out, "\tsetle %al").unwrap();
                            writeln!(out, "\tmovzbq %al, %rax").unwrap();
                            writeln!(out, "\tmovq %rax, {}", dest).unwrap();
                        }
                        OpBiner::LebihDari => {
                            writeln!(out, "\tcmpq {}, %rax", loc_kanan).unwrap();
                            writeln!(out, "\tsetg %al").unwrap();
                            writeln!(out, "\tmovzbq %al, %rax").unwrap();
                            writeln!(out, "\tmovq %rax, {}", dest).unwrap();
                        }
                        OpBiner::LebihSama => {
                            writeln!(out, "\tcmpq {}, %rax", loc_kanan).unwrap();
                            writeln!(out, "\tsetge %al").unwrap();
                            writeln!(out, "\tmovzbq %al, %rax").unwrap();
                            writeln!(out, "\tmovq %rax, {}", dest).unwrap();
                        }
                    }
                }

                Instruksi::Unari { op, operan, .. } => {
                    let loc_op = format_lokasi(*operan, &layout);
                    let dest = hasil.map(|r| format_lokasi(r, &layout)).unwrap_or_else(|| "%rax".to_string());
                    writeln!(out, "\tmovq {}, %rax", loc_op).unwrap();
                    match op {
                        OpUnari::Negasi => {
                            writeln!(out, "\tnegq %rax").unwrap();
                        }
                        OpUnari::Bukan => {
                            writeln!(out, "\txorq $1, %rax").unwrap();
                        }
                        OpUnari::BukanBit => {
                            writeln!(out, "\tnotq %rax").unwrap();
                        }
                    }
                    writeln!(out, "\tmovq %rax, {}", dest).unwrap();
                }

                Instruksi::Alloca(_) => {
                    if let Some(res) = hasil {
                        let dest = format_lokasi(*res, &layout);
                        let offset = layout.peta_alloca.get(res).copied().unwrap_or(16);
                        writeln!(out, "\tleaq -{}(%rbp), %rax", offset).unwrap();
                        writeln!(out, "\tmovq %rax, {}", dest).unwrap();
                    }
                }

                Instruksi::Muat { penunjuk, .. } => {
                    let loc_ptr = format_lokasi(*penunjuk, &layout);
                    let dest = hasil.map(|r| format_lokasi(r, &layout)).unwrap_or_else(|| "%rax".to_string());
                    writeln!(out, "\tmovq {}, %rax", loc_ptr).unwrap();
                    writeln!(out, "\tmovq (%rax), %rax").unwrap();
                    writeln!(out, "\tmovq %rax, {}", dest).unwrap();
                }

                Instruksi::Simpan { nilai, penunjuk } => {
                    let loc_val = format_lokasi(*nilai, &layout);
                    let loc_ptr = format_lokasi(*penunjuk, &layout);
                    writeln!(out, "\tmovq {}, %rax", loc_val).unwrap();
                    writeln!(out, "\tmovq {}, %rcx", loc_ptr).unwrap();
                    writeln!(out, "\tmovq %rax, (%rcx)").unwrap();
                }

                Instruksi::AksesBidang { struktur, indeks } => {
                    let loc_struk = format_lokasi(*struktur, &layout);
                    let dest = hasil.map(|r| format_lokasi(r, &layout)).unwrap_or_else(|| "%rax".to_string());
                    let offset = (*indeks * 8) as i64;
                    writeln!(out, "\tmovq {}, %rax", loc_struk).unwrap();
                    writeln!(out, "\tleaq {}(%rax), %rax", offset).unwrap();
                    writeln!(out, "\tmovq %rax, {}", dest).unwrap();
                }

                Instruksi::AksesIndeks { target, indeks } => {
                    let loc_target = format_lokasi(*target, &layout);
                    let loc_idx = format_lokasi(*indeks, &layout);
                    let dest = hasil.map(|r| format_lokasi(r, &layout)).unwrap_or_else(|| "%rax".to_string());
                    writeln!(out, "\tmovq {}, %rax", loc_target).unwrap();
                    writeln!(out, "\tmovq {}, %rcx", loc_idx).unwrap();
                    writeln!(out, "\tleaq (%rax,%rcx,8), %rax").unwrap();
                    writeln!(out, "\tmovq %rax, {}", dest).unwrap();
                }

                Instruksi::Cast { asal, .. } => {
                    let loc_asal = format_lokasi(*asal, &layout);
                    let dest = hasil.map(|r| format_lokasi(r, &layout)).unwrap_or_else(|| "%rax".to_string());
                    writeln!(out, "\tmovq {}, %rax", loc_asal).unwrap();
                    writeln!(out, "\tmovq %rax, {}", dest).unwrap();
                }

                Instruksi::Panggil { fungsi, argumen, .. } => {
                    if fungsi == "komo_rt_format_teks" {
                        if let Some(res) = hasil {
                            let dest = format_lokasi(*res, &layout);
                            if let Some(first_arg) = argumen.first() {
                                let loc = format_lokasi(*first_arg, &layout);
                                writeln!(out, "\tmovq {}, %rax", loc).unwrap();
                                writeln!(out, "\tmovq %rax, {}", dest).unwrap();
                            }
                        }
                        continue;
                    }

                    // Muat argumen ke register System V
                    for (i, arg) in argumen.iter().enumerate() {
                        if i < REG_PARAMETER_X86.len() {
                            let loc_arg = format_lokasi(*arg, &layout);
                            let reg_target = REG_PARAMETER_X86[i];
                            writeln!(out, "\tmovq {}, {}", loc_arg, reg_target.sebagai_att()).unwrap();
                        }
                    }

                    let nama_panggil = match fungsi.as_str() {
                        "cetak.baris" | "std/cetak.baris" => "komo_std_cetak_baris",
                        "cetak.teks" | "std/cetak.teks" => "komo_std_cetak_teks",
                        "cetak.bulat" | "std/cetak.bulat" => "komo_std_cetak_bulat",
                        "cetak.desimal" | "std/cetak.desimal" => "komo_std_cetak_desimal",
                        lain => lain,
                    };

                    writeln!(out, "\tcall {}", nama_panggil.replace('.', "_")).unwrap();

                    if let Some(res) = hasil {
                        let dest = format_lokasi(*res, &layout);
                        writeln!(out, "\tmovq %rax, {}", dest).unwrap();
                    }
                }
            }
        }

        // Terminator blok dasar
        if let Some(term) = &b.terminator {
            match term {
                Terminator::Lompat(target) => {
                    emit_resolusi_phi_x86(out, f, b.id, *target, &layout);
                    writeln!(out, "\tjmp .L_{}_{}", nama_asm, target.0).unwrap();
                }
                Terminator::Cabang { kondisi, jika_benar, jika_salah } => {
                    let loc_cond = format_lokasi(*kondisi, &layout);
                    writeln!(out, "\tmovq {}, %rax", loc_cond).unwrap();
                    writeln!(out, "\tcmpq $0, %rax").unwrap();
                    writeln!(out, "\tjne .L_cabang_benar_{}_{}", nama_asm, b.id.0).unwrap();
                    // Cabang salah
                    emit_resolusi_phi_x86(out, f, b.id, *jika_salah, &layout);
                    writeln!(out, "\tjmp .L_{}_{}", nama_asm, jika_salah.0).unwrap();
                    // Cabang benar
                    writeln!(out, ".L_cabang_benar_{}_{}:", nama_asm, b.id.0).unwrap();
                    emit_resolusi_phi_x86(out, f, b.id, *jika_benar, &layout);
                    writeln!(out, "\tjmp .L_{}_{}", nama_asm, jika_benar.0).unwrap();
                }
                Terminator::Kembali(Some(val)) => {
                    let loc_val = format_lokasi(*val, &layout);
                    writeln!(out, "\tmovq {}, %rax", loc_val).unwrap();
                    writeln!(out, "\tmovq %rbp, %rsp").unwrap();
                    writeln!(out, "\tpopq %rbp").unwrap();
                    writeln!(out, "\tretq").unwrap();
                }
                Terminator::Kembali(None) => {
                    writeln!(out, "\tmovq $0, %rax").unwrap();
                    writeln!(out, "\tmovq %rbp, %rsp").unwrap();
                    writeln!(out, "\tpopq %rbp").unwrap();
                    writeln!(out, "\tretq").unwrap();
                }
                Terminator::TidakBisa(_) => {
                    writeln!(out, "\tud2").unwrap();
                }
            }
        }
    }

    writeln!(out).unwrap();
}
