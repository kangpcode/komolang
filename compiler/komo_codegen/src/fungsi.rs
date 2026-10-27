// Copyright (c) kangpcode (Dhafa Nazula Permadi). Licensed under MIT or Apache-2.0.

//! Penurunan fungsi KIR ke instruksi Cranelift.
//! Menangani blok dasar SSA, alokasi register/nilai, instruksi aritmatika/logika,
//! akses memori (alloca/load/store), pemanggilan fungsi, dan terminator.

use std::collections::HashMap;

use cranelift_codegen::ir::condcodes::{FloatCC, IntCC};
use cranelift_codegen::ir::{
    types, AbiParam, Block, BlockArg, InstBuilder, MemFlagsData, StackSlotData, StackSlotKind, TrapCode,
    Type as ClType, Value as ClValue,
};
use cranelift_frontend::FunctionBuilder;
use cranelift_module::{FuncId, Linkage, Module};

use komo_ir::{FungsiIr, IdBlok, IdNilai, Instruksi, IrTipe, Konstanta, OpBiner, OpUnari, Terminator};

use crate::tipe::konversi_tipe_ke_cranelift;

/// Mengompilasi `FungsiIr` ke dalam Cranelift `Module`.
pub fn kompilasi_fungsi<M: Module>(
    module: &mut M,
    builder: &mut FunctionBuilder,
    fungsi_ir: &FungsiIr,
    peta_fungsi: &HashMap<String, FuncId>,
    peta_signature: &HashMap<String, cranelift_codegen::ir::Signature>,
    string_cache: &mut HashMap<String, ClValue>,
) -> Result<(), String> {
    let pointer_type = module.target_config().pointer_type();

    // 1. Buat seluruh blok Cranelift terlebih dahulu
    let mut peta_blok: HashMap<IdBlok, Block> = HashMap::new();
    for b in &fungsi_ir.blok {
        let cl_block = builder.create_block();
        peta_blok.insert(b.id, cl_block);
    }

    // 2. Analisis Phi nodes pada setiap blok dasar
    // Struktur: blok_id -> Vec<(id_nilai_phi, Vec<(id_nilai_input, id_blok_asal)>)>
    let mut blok_phi_nodes: HashMap<IdBlok, Vec<(IdNilai, Vec<(IdNilai, IdBlok)>)>> = HashMap::new();
    let mut peta_nilai: HashMap<IdNilai, ClValue> = HashMap::new();

    for b in &fungsi_ir.blok {
        let mut phis = Vec::new();
        for (hasil, inst) in &b.instruksi {
            if let Instruksi::Phi { tipe, masukan } = inst {
                if let Some(res_id) = hasil {
                    let cl_block = peta_blok[&b.id];
                    let cl_ty = konversi_tipe_ke_cranelift(tipe, pointer_type).unwrap_or(pointer_type);
                    let param_val = builder.append_block_param(cl_block, cl_ty);
                    peta_nilai.insert(*res_id, param_val);
                    phis.push((*res_id, masukan.clone()));
                }
            }
        }
        if !phis.is_empty() {
            blok_phi_nodes.insert(b.id, phis);
        }
    }

    // 3. Konfigurasi blok awal (entri fungsi)
    let cl_entry = peta_blok[&fungsi_ir.blok_awal];
    builder.append_block_params_for_function_params(cl_entry);
    builder.switch_to_block(cl_entry);

    let cl_entry_params = builder.block_params(cl_entry);
    for (i, param) in fungsi_ir.parameter.iter().enumerate() {
        peta_nilai.insert(param.nilai, cl_entry_params[i]);
    }

    // 4. Terjemahkan setiap blok dasar
    for b in &fungsi_ir.blok {
        let cl_block = peta_blok[&b.id];
        builder.switch_to_block(cl_block);

        for (hasil, inst) in &b.instruksi {
            match inst {
                Instruksi::Phi { .. } => {
                    // Sudah ditangani di atas sebagai block parameter
                }
                Instruksi::Konst(k) => {
                    let val = match k {
                        Konstanta::Bulat(n) => builder.ins().iconst(types::I64, *n),
                        Konstanta::Desimal(d) => builder.ins().f64const(*d),
                        Konstanta::Bool(b) => builder.ins().iconst(types::I8, if *b { 1 } else { 0 }),
                        Konstanta::Karakter(c) => builder.ins().iconst(types::I32, *c as u32 as i64),
                        Konstanta::Bita(b) => builder.ins().iconst(types::I8, *b as i64),
                        Konstanta::Nol | Konstanta::Kosong => builder.ins().iconst(pointer_type, 0),
                        Konstanta::Teks(s) => {
                            if let Some(existing) = string_cache.get(s) {
                                *existing
                            } else {
                                let mut data_ctx = cranelift_module::DataDescription::new();
                                let mut bytes = s.as_bytes().to_vec();
                                bytes.push(0); // Null-terminated C string
                                data_ctx.define(bytes.into_boxed_slice());
                                let data_id = module
                                    .declare_anonymous_data(true, false)
                                    .map_err(|e| format!("Gagal mendeklarasikan data teks: {}", e))?;
                                module
                                    .define_data(data_id, &data_ctx)
                                    .map_err(|e| format!("Gagal mendefinisikan data teks: {}", e))?;
                                let local_data = module.declare_data_in_func(data_id, builder.func);
                                let ptr_val = builder.ins().symbol_value(pointer_type, local_data);
                                string_cache.insert(s.clone(), ptr_val);
                                ptr_val
                            }
                        }
                    };
                    if let Some(res) = hasil {
                        peta_nilai.insert(*res, val);
                    }
                }

                Instruksi::Biner { op, kiri, kanan, tipe } => {
                    let is_float = *tipe == IrTipe::F64 || *tipe == IrTipe::F32;
                    let target_cl_ty = if *tipe == IrTipe::F32 {
                        types::F32
                    } else if *tipe == IrTipe::F64 {
                        types::F64
                    } else {
                        types::I64
                    };

                    let mut cl_kiri = peta_nilai.get(kiri).copied().unwrap_or_else(|| {
                        builder.ins().iconst(types::I64, 0)
                    });
                    let mut cl_kanan = peta_nilai.get(kanan).copied().unwrap_or_else(|| {
                        builder.ins().iconst(types::I64, 0)
                    });

                    let ty_kiri = builder.func.dfg.value_type(cl_kiri);
                    let ty_kanan = builder.func.dfg.value_type(cl_kanan);

                    if is_float {
                        if ty_kiri.is_int() {
                            cl_kiri = builder.ins().fcvt_from_sint(target_cl_ty, cl_kiri);
                        }
                        if ty_kanan.is_int() {
                            cl_kanan = builder.ins().fcvt_from_sint(target_cl_ty, cl_kanan);
                        }
                    } else if target_cl_ty.is_int() {
                        if ty_kiri.is_float() {
                            cl_kiri = builder.ins().fcvt_to_sint_sat(target_cl_ty, cl_kiri);
                        }
                        if ty_kanan.is_float() {
                            cl_kanan = builder.ins().fcvt_to_sint_sat(target_cl_ty, cl_kanan);
                        }
                    }

                    let val = match op {
                        OpBiner::Tambah => {
                            if *tipe == IrTipe::F64 || *tipe == IrTipe::F32 {
                                builder.ins().fadd(cl_kiri, cl_kanan)
                            } else {
                                builder.ins().iadd(cl_kiri, cl_kanan)
                            }
                        }
                        OpBiner::Kurang => {
                            if *tipe == IrTipe::F64 || *tipe == IrTipe::F32 {
                                builder.ins().fsub(cl_kiri, cl_kanan)
                            } else {
                                builder.ins().isub(cl_kiri, cl_kanan)
                            }
                        }
                        OpBiner::Kali => {
                            if *tipe == IrTipe::F64 || *tipe == IrTipe::F32 {
                                builder.ins().fmul(cl_kiri, cl_kanan)
                            } else {
                                builder.ins().imul(cl_kiri, cl_kanan)
                            }
                        }
                        OpBiner::Bagi => {
                            if *tipe == IrTipe::F64 || *tipe == IrTipe::F32 {
                                builder.ins().fdiv(cl_kiri, cl_kanan)
                            } else {
                                builder.ins().sdiv(cl_kiri, cl_kanan)
                            }
                        }
                        OpBiner::Sisa => builder.ins().srem(cl_kiri, cl_kanan),
                        OpBiner::DanBit => builder.ins().band(cl_kiri, cl_kanan),
                        OpBiner::AtauBit => builder.ins().bor(cl_kiri, cl_kanan),
                        OpBiner::XorBit => builder.ins().bxor(cl_kiri, cl_kanan),
                        OpBiner::GeserKiri => builder.ins().ishl(cl_kiri, cl_kanan),
                        OpBiner::GeserKanan => builder.ins().sshr(cl_kiri, cl_kanan),

                        OpBiner::SamaDengan => {
                            if *tipe == IrTipe::F64 || *tipe == IrTipe::F32 {
                                builder.ins().fcmp(FloatCC::Equal, cl_kiri, cl_kanan)
                            } else {
                                builder.ins().icmp(IntCC::Equal, cl_kiri, cl_kanan)
                            }
                        }
                        OpBiner::TidakSama => {
                            if *tipe == IrTipe::F64 || *tipe == IrTipe::F32 {
                                builder.ins().fcmp(FloatCC::NotEqual, cl_kiri, cl_kanan)
                            } else {
                                builder.ins().icmp(IntCC::NotEqual, cl_kiri, cl_kanan)
                            }
                        }
                        OpBiner::KurangDari => {
                            if *tipe == IrTipe::F64 || *tipe == IrTipe::F32 {
                                builder.ins().fcmp(FloatCC::LessThan, cl_kiri, cl_kanan)
                            } else {
                                builder.ins().icmp(IntCC::SignedLessThan, cl_kiri, cl_kanan)
                            }
                        }
                        OpBiner::KurangSama => {
                            if *tipe == IrTipe::F64 || *tipe == IrTipe::F32 {
                                builder.ins().fcmp(FloatCC::LessThanOrEqual, cl_kiri, cl_kanan)
                            } else {
                                builder.ins().icmp(IntCC::SignedLessThanOrEqual, cl_kiri, cl_kanan)
                            }
                        }
                        OpBiner::LebihDari => {
                            if *tipe == IrTipe::F64 || *tipe == IrTipe::F32 {
                                builder.ins().fcmp(FloatCC::GreaterThan, cl_kiri, cl_kanan)
                            } else {
                                builder.ins().icmp(IntCC::SignedGreaterThan, cl_kiri, cl_kanan)
                            }
                        }
                        OpBiner::LebihSama => {
                            if *tipe == IrTipe::F64 || *tipe == IrTipe::F32 {
                                builder.ins().fcmp(FloatCC::GreaterThanOrEqual, cl_kiri, cl_kanan)
                            } else {
                                builder.ins().icmp(IntCC::SignedGreaterThanOrEqual, cl_kiri, cl_kanan)
                            }
                        }
                    };
                    if let Some(res) = hasil {
                        peta_nilai.insert(*res, val);
                    }
                }

                Instruksi::Unari { op, operan, tipe } => {
                    let cl_operan = peta_nilai.get(operan).copied().unwrap_or_else(|| {
                        builder.ins().iconst(types::I64, 0)
                    });
                    let val = match op {
                        OpUnari::Negasi => {
                            if *tipe == IrTipe::F64 || *tipe == IrTipe::F32 {
                                builder.ins().fneg(cl_operan)
                            } else {
                                builder.ins().ineg(cl_operan)
                            }
                        }
                        OpUnari::Bukan => {
                            let satu = builder.ins().iconst(types::I8, 1);
                            builder.ins().bxor(cl_operan, satu)
                        }
                        OpUnari::BukanBit => builder.ins().bnot(cl_operan),
                    };
                    if let Some(res) = hasil {
                        peta_nilai.insert(*res, val);
                    }
                }

                Instruksi::Alloca(tipe) => {
                    let size = tipe.ukuran_bita().max(8) as u32;
                    let stack_slot = builder.create_sized_stack_slot(StackSlotData::new(
                        StackSlotKind::ExplicitSlot,
                        size,
                        3, // 8-byte alignment
                    ));
                    let ptr_val = builder.ins().stack_addr(pointer_type, stack_slot, 0);
                    if let Some(res) = hasil {
                        peta_nilai.insert(*res, ptr_val);
                    }
                }

                Instruksi::Muat { tipe, penunjuk } => {
                    let cl_ptr = peta_nilai.get(penunjuk).copied().unwrap_or_else(|| {
                        builder.ins().iconst(pointer_type, 0)
                    });
                    let cl_ty = konversi_tipe_ke_cranelift(tipe, pointer_type).unwrap_or(pointer_type);
                    let val = builder.ins().load(cl_ty, MemFlagsData::trusted(), cl_ptr, 0);
                    if let Some(res) = hasil {
                        peta_nilai.insert(*res, val);
                    }
                }

                Instruksi::Simpan { nilai, penunjuk } => {
                    let cl_val = peta_nilai.get(nilai).copied().unwrap_or_else(|| {
                        builder.ins().iconst(types::I64, 0)
                    });
                    let cl_ptr = peta_nilai.get(penunjuk).copied().unwrap_or_else(|| {
                        builder.ins().iconst(pointer_type, 0)
                    });
                    builder.ins().store(MemFlagsData::trusted(), cl_val, cl_ptr, 0);
                }

                Instruksi::AksesBidang { struktur, indeks } => {
                    let cl_struk = peta_nilai.get(struktur).copied().unwrap_or_else(|| {
                        builder.ins().iconst(pointer_type, 0)
                    });
                    let offset = (*indeks * 8) as i64;
                    let field_ptr = builder.ins().iadd_imm_s(cl_struk, offset);
                    if let Some(res) = hasil {
                        peta_nilai.insert(*res, field_ptr);
                    }
                }

                Instruksi::AksesIndeks { target, indeks } => {
                    let cl_target = peta_nilai.get(target).copied().unwrap_or_else(|| {
                        builder.ins().iconst(pointer_type, 0)
                    });
                    let cl_idx = peta_nilai.get(indeks).copied().unwrap_or_else(|| {
                        builder.ins().iconst(pointer_type, 0)
                    });
                    let elem_size = builder.ins().iconst(pointer_type, 8);
                    let byte_offset = builder.ins().imul(cl_idx, elem_size);
                    let elem_ptr = builder.ins().iadd(cl_target, byte_offset);
                    if let Some(res) = hasil {
                        peta_nilai.insert(*res, elem_ptr);
                    }
                }

                Instruksi::Cast { asal, tipe_tujuan } => {
                    let cl_asal = peta_nilai.get(asal).copied().unwrap_or_else(|| {
                        builder.ins().iconst(types::I64, 0)
                    });
                    let cl_dest_ty =
                        konversi_tipe_ke_cranelift(tipe_tujuan, pointer_type).unwrap_or(pointer_type);
                    let val = lakukan_cast(builder, cl_asal, cl_dest_ty);
                    if let Some(res) = hasil {
                        peta_nilai.insert(*res, val);
                    }
                }

                Instruksi::Panggil {
                    fungsi,
                    argumen,
                    tipe_kembali,
                } => {
                    let mut cl_args = Vec::new();
                    for a in argumen {
                        let arg_val = peta_nilai.get(a).copied().unwrap_or_else(|| {
                            builder.ins().iconst(pointer_type, 0)
                        });
                        cl_args.push(arg_val);
                    }

                    if fungsi == "komo_rt_format_teks" {
                        let first = cl_args.first().copied().unwrap_or_else(|| {
                            builder.ins().iconst(pointer_type, 0)
                        });
                        if let Some(res) = hasil {
                            peta_nilai.insert(*res, first);
                        }
                        continue;
                    }

                    let call_res = panggil_fungsi(
                        module,
                        builder,
                        fungsi,
                        &cl_args,
                        tipe_kembali,
                        pointer_type,
                        peta_fungsi,
                        peta_signature,
                    )?;

                    if let Some(res) = hasil {
                        if let Some(v) = call_res {
                            peta_nilai.insert(*res, v);
                        }
                    }
                }
            }
        }

        // Tangani Terminator pada akhir blok dasar
        if let Some(term) = &b.terminator {
            match term {
                Terminator::Lompat(target) => {
                    let cl_target = peta_blok[target];
                    let args = kumpulkan_phi_args(*target, b.id, &blok_phi_nodes, &peta_nilai);
                    builder.ins().jump(cl_target, &args);
                }
                Terminator::Cabang {
                    kondisi,
                    jika_benar,
                    jika_salah,
                } => {
                    let cond_val = peta_nilai.get(kondisi).copied().unwrap_or_else(|| {
                        builder.ins().iconst(types::I8, 0)
                    });
                    let cl_benar = peta_blok[jika_benar];
                    let cl_salah = peta_blok[jika_salah];
                    let args_benar =
                        kumpulkan_phi_args(*jika_benar, b.id, &blok_phi_nodes, &peta_nilai);
                    let args_salah =
                        kumpulkan_phi_args(*jika_salah, b.id, &blok_phi_nodes, &peta_nilai);

                    builder.ins().brif(cond_val, cl_benar, &args_benar, cl_salah, &args_salah);
                }
                Terminator::Kembali(Some(val)) => {
                    let ret_val = peta_nilai.get(val).copied().unwrap_or_else(|| {
                        builder.ins().iconst(types::I64, 0)
                    });
                    builder.ins().return_(&[ret_val]);
                }
                Terminator::Kembali(None) => {
                    builder.ins().return_(&[]);
                }
                Terminator::TidakBisa(_alasan) => {
                    builder.ins().trap(TrapCode::unwrap_user(1));
                }
            }
        }
    }

    // Seal semua blok setelah seluruh terminator diterjemahkan
    for b in &fungsi_ir.blok {
        let cl_block = peta_blok[&b.id];
        builder.seal_block(cl_block);
    }

    Ok(())
}

fn kumpulkan_phi_args(
    target_blok: IdBlok,
    asal_blok: IdBlok,
    blok_phi_nodes: &HashMap<IdBlok, Vec<(IdNilai, Vec<(IdNilai, IdBlok)>)>>,
    peta_nilai: &HashMap<IdNilai, ClValue>,
) -> Vec<BlockArg> {
    let mut args = Vec::new();
    if let Some(phis) = blok_phi_nodes.get(&target_blok) {
        for (_, masukan) in phis {
            let mut val_opt = None;
            for (in_val, in_blk) in masukan {
                if *in_blk == asal_blok {
                    val_opt = peta_nilai.get(in_val).copied();
                    break;
                }
            }
            if let Some(v) = val_opt {
                args.push(BlockArg::from(v));
            }
        }
    }
    args
}

fn lakukan_cast(builder: &mut FunctionBuilder, val: ClValue, dest_ty: ClType) -> ClValue {
    let src_ty = builder.func.dfg.value_type(val);
    if src_ty == dest_ty {
        return val;
    }

    if src_ty.is_int() && dest_ty.is_int() {
        if src_ty.bits() < dest_ty.bits() {
            builder.ins().sextend(dest_ty, val)
        } else {
            builder.ins().ireduce(dest_ty, val)
        }
    } else if src_ty.is_int() && dest_ty.is_float() {
        builder.ins().fcvt_from_sint(dest_ty, val)
    } else if src_ty.is_float() && dest_ty.is_int() {
        builder.ins().fcvt_to_sint_sat(dest_ty, val)
    } else if src_ty.is_float() && dest_ty.is_float() {
        if src_ty.bits() < dest_ty.bits() {
            builder.ins().fpromote(dest_ty, val)
        } else {
            builder.ins().fdemote(dest_ty, val)
        }
    } else {
        val
    }
}

fn panggil_fungsi<M: Module>(
    module: &mut M,
    builder: &mut FunctionBuilder,
    nama_fungsi: &str,
    argumen: &[ClValue],
    tipe_kembali: &IrTipe,
    pointer_type: ClType,
    peta_fungsi: &HashMap<String, FuncId>,
    peta_signature: &HashMap<String, cranelift_codegen::ir::Signature>,
) -> Result<Option<ClValue>, String> {
    // Normalisasi nama fungsi jika berasal dari modul std seperti `cetak.baris`
    let nama_target = match nama_fungsi {
        "cetak.baris" | "std/cetak.baris" => "komo_std_cetak_baris",
        "cetak.teks" | "std/cetak.teks" => "komo_std_cetak_teks",
        "cetak.bulat" => "komo_std_cetak_bulat",
        "cetak.desimal" => "komo_std_cetak_desimal",
        lain => lain,
    };

    // Sesuaikan tipe argumen sesuai signature target jika sudah diketahui
    let mut cl_args = argumen.to_vec();
    if let Some(target_sig) = peta_signature
        .get(nama_target)
        .or_else(|| peta_signature.get(nama_fungsi))
    {
        for (i, arg) in cl_args.iter_mut().enumerate() {
            if i < target_sig.params.len() {
                let expected_ty = target_sig.params[i].value_type;
                let actual_ty = builder.func.dfg.value_type(*arg);
                if expected_ty != actual_ty {
                    *arg = lakukan_cast(builder, *arg, expected_ty);
                }
            }
        }
    }

    let func_id = if let Some(id) = peta_fungsi.get(nama_target).or_else(|| peta_fungsi.get(nama_fungsi)) {
        *id
    } else {
        // Deklarasikan sebagai fungsi eksternal runtime jika belum terdefinisi
        let mut sig = module.make_signature();
        for arg in &cl_args {
            let ty = builder.func.dfg.value_type(*arg);
            sig.params.push(AbiParam::new(ty));
        }
        if let Some(ret_ty) = konversi_tipe_ke_cranelift(tipe_kembali, pointer_type) {
            sig.returns.push(AbiParam::new(ret_ty));
        }
        module
            .declare_function(nama_target, Linkage::Import, &sig)
            .map_err(|e| format!("Gagal mendeklarasikan fungsi eksternal '{}': {}", nama_target, e))?
    };

    let local_func = module.declare_func_in_func(func_id, builder.func);
    let call_inst = builder.ins().call(local_func, &cl_args);
    let results = builder.inst_results(call_inst);

    if results.is_empty() {
        Ok(None)
    } else {
        Ok(Some(results[0]))
    }
}

