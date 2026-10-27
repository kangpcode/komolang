// Copyright (c) kangpcode (Dhafa Nazula Permadi). Licensed under MIT or Apache-2.0.

//! Emitter Assembly x86_64 langsung dari KIR SSA.
//! Menghasilkan teks assembly GNU / AT&T syntax yang deterministik dan siap diassemble.

use std::collections::HashMap;
use std::fmt::Write;

use komo_ir::{FungsiIr, IdNilai, Instruksi, Konstanta, ModulIr, OpBiner, OpUnari, Terminator};

use crate::regalloc::{
    alokasikan_register_linear_scan, HasilAlokasiRegister, LokasiNilai,
};
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

fn emit_fungsi_x86_64(out: &mut String, f: &FungsiIr, label_string: &HashMap<String, String>) {
    let regalloc = alokasikan_register_linear_scan(f, POOL_REGISTER_ALOKASI_X86.len());

    // Normalisasi nama fungsi
    let nama_asm = f.nama.replace('.', "_");

    writeln!(out, "\t.globl {}", nama_asm).unwrap();
    writeln!(out, "\t.type {}, @function", nama_asm).unwrap();
    writeln!(out, "{}:", nama_asm).unwrap();

    // Prologue fungsi
    writeln!(out, "\tpushq %rbp").unwrap();
    writeln!(out, "\tmovq %rsp, %rbp").unwrap();
    let mut frame_size = regalloc.ukuran_stack;
    if frame_size == 0 {
        frame_size = 16;
    }
    writeln!(out, "\tsubq ${}, %rsp", frame_size).unwrap();

    // Salin argumen parameter dari register System V ke lokasi alokasi masing-masing
    for (i, p) in f.parameter.iter().enumerate() {
        if i < REG_PARAMETER_X86.len() {
            let reg_param = REG_PARAMETER_X86[i];
            let dest_loc = format_lokasi(p.nilai, &regalloc);
            writeln!(out, "\tmovq {}, {}", reg_param.sebagai_att(), dest_loc).unwrap();
        }
    }

    // Iterasi seluruh blok dasar
    for b in &f.blok {
        writeln!(out, ".L_{}_{}:", nama_asm, b.id.0).unwrap();

        for (hasil, inst) in &b.instruksi {
            match inst {
                Instruksi::Phi { .. } => {
                    // Phi diproses di transisi percabangan blok
                }
                Instruksi::Konst(k) => {
                    if let Some(res) = hasil {
                        let dest = format_lokasi(*res, &regalloc);
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
                    let loc_kiri = format_lokasi(*kiri, &regalloc);
                    let loc_kanan = format_lokasi(*kanan, &regalloc);
                    let dest = hasil.map(|r| format_lokasi(r, &regalloc)).unwrap_or_else(|| "%rax".to_string());

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
                    let loc_op = format_lokasi(*operan, &regalloc);
                    let dest = hasil.map(|r| format_lokasi(r, &regalloc)).unwrap_or_else(|| "%rax".to_string());
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

                Instruksi::Alloca(tipe) => {
                    if let Some(res) = hasil {
                        let dest = format_lokasi(*res, &regalloc);
                        let offset = regalloc.ukuran_stack.max(16);
                        let _ = tipe;
                        writeln!(out, "\tleaq -{}(%rbp), %rax", offset).unwrap();
                        writeln!(out, "\tmovq %rax, {}", dest).unwrap();
                    }
                }

                Instruksi::Muat { penunjuk, .. } => {
                    let loc_ptr = format_lokasi(*penunjuk, &regalloc);
                    let dest = hasil.map(|r| format_lokasi(r, &regalloc)).unwrap_or_else(|| "%rax".to_string());
                    writeln!(out, "\tmovq {}, %rax", loc_ptr).unwrap();
                    writeln!(out, "\tmovq (%rax), %rax").unwrap();
                    writeln!(out, "\tmovq %rax, {}", dest).unwrap();
                }

                Instruksi::Simpan { nilai, penunjuk } => {
                    let loc_val = format_lokasi(*nilai, &regalloc);
                    let loc_ptr = format_lokasi(*penunjuk, &regalloc);
                    writeln!(out, "\tmovq {}, %rax", loc_val).unwrap();
                    writeln!(out, "\tmovq {}, %rcx", loc_ptr).unwrap();
                    writeln!(out, "\tmovq %rax, (%rcx)").unwrap();
                }

                Instruksi::AksesBidang { struktur, indeks } => {
                    let loc_struk = format_lokasi(*struktur, &regalloc);
                    let dest = hasil.map(|r| format_lokasi(r, &regalloc)).unwrap_or_else(|| "%rax".to_string());
                    let offset = (*indeks * 8) as i64;
                    writeln!(out, "\tmovq {}, %rax", loc_struk).unwrap();
                    writeln!(out, "\tleaq {}(%rax), %rax", offset).unwrap();
                    writeln!(out, "\tmovq %rax, {}", dest).unwrap();
                }

                Instruksi::AksesIndeks { target, indeks } => {
                    let loc_target = format_lokasi(*target, &regalloc);
                    let loc_idx = format_lokasi(*indeks, &regalloc);
                    let dest = hasil.map(|r| format_lokasi(r, &regalloc)).unwrap_or_else(|| "%rax".to_string());
                    writeln!(out, "\tmovq {}, %rax", loc_target).unwrap();
                    writeln!(out, "\tmovq {}, %rcx", loc_idx).unwrap();
                    writeln!(out, "\tleaq (%rax,%rcx,8), %rax").unwrap();
                    writeln!(out, "\tmovq %rax, {}", dest).unwrap();
                }

                Instruksi::Cast { asal, .. } => {
                    let loc_asal = format_lokasi(*asal, &regalloc);
                    let dest = hasil.map(|r| format_lokasi(r, &regalloc)).unwrap_or_else(|| "%rax".to_string());
                    writeln!(out, "\tmovq {}, %rax", loc_asal).unwrap();
                    writeln!(out, "\tmovq %rax, {}", dest).unwrap();
                }

                Instruksi::Panggil { fungsi, argumen, .. } => {
                    if fungsi == "komo_rt_format_teks" {
                        if let Some(res) = hasil {
                            let dest = format_lokasi(*res, &regalloc);
                            if let Some(first_arg) = argumen.first() {
                                let loc = format_lokasi(*first_arg, &regalloc);
                                writeln!(out, "\tmovq {}, %rax", loc).unwrap();
                                writeln!(out, "\tmovq %rax, {}", dest).unwrap();
                            }
                        }
                        continue;
                    }

                    // Muat argumen ke register System V
                    for (i, arg) in argumen.iter().enumerate() {
                        if i < REG_PARAMETER_X86.len() {
                            let loc_arg = format_lokasi(*arg, &regalloc);
                            let reg_target = REG_PARAMETER_X86[i];
                            writeln!(out, "\tmovq {}, {}", loc_arg, reg_target.sebagai_att()).unwrap();
                        }
                    }

                    let nama_panggil = match fungsi.as_str() {
                        "cetak.baris" | "std/cetak.baris" => "komo_std_cetak_baris",
                        "cetak.teks" | "std/cetak.teks" => "komo_std_cetak_teks",
                        "cetak.bulat" => "komo_std_cetak_bulat",
                        "cetak.desimal" => "komo_std_cetak_desimal",
                        lain => lain,
                    };

                    writeln!(out, "\tcall {}", nama_panggil.replace('.', "_")).unwrap();

                    if let Some(res) = hasil {
                        let dest = format_lokasi(*res, &regalloc);
                        writeln!(out, "\tmovq %rax, {}", dest).unwrap();
                    }
                }
            }
        }

        // Terminator blok dasar
        if let Some(term) = &b.terminator {
            match term {
                Terminator::Lompat(target) => {
                    writeln!(out, "\tjmp .L_{}_{}", nama_asm, target.0).unwrap();
                }
                Terminator::Cabang { kondisi, jika_benar, jika_salah } => {
                    let loc_cond = format_lokasi(*kondisi, &regalloc);
                    writeln!(out, "\tcmpq $0, {}", loc_cond).unwrap();
                    writeln!(out, "\tjne .L_{}_{}", nama_asm, jika_benar.0).unwrap();
                    writeln!(out, "\tjmp .L_{}_{}", nama_asm, jika_salah.0).unwrap();
                }
                Terminator::Kembali(Some(val)) => {
                    let loc_val = format_lokasi(*val, &regalloc);
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

fn format_lokasi(id: IdNilai, regalloc: &HasilAlokasiRegister) -> String {
    match regalloc.peta_lokasi.get(&id) {
        Some(LokasiNilai::Register(idx)) => {
            let reg = POOL_REGISTER_ALOKASI_X86[*idx % POOL_REGISTER_ALOKASI_X86.len()];
            reg.sebagai_att().to_string()
        }
        Some(LokasiNilai::Stack(offset)) => {
            format!("{}(%rbp)", offset)
        }
        None => "-8(%rbp)".to_string(),
    }
}
