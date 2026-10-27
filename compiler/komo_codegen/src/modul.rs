// Copyright (c) kangpcode (Dhafa Nazula Permadi). Licensed under MIT or Apache-2.0.

//! Kompilator tingkat modul yang menurunkan `ModulIr` menjadi berkas objek native (ELF, Mach-O, PE/COFF).
//! Mendukung arsitektur x86_64 dan AArch64 (ARM64) dengan informasi debug DWARF/PDB.

use std::collections::HashMap;

use cranelift_codegen::ir::{types, AbiParam, InstBuilder};
use cranelift_frontend::{FunctionBuilder, FunctionBuilderContext};
use cranelift_module::{DataDescription, FuncId, Linkage, Module};
use cranelift_object::{ObjectBuilder, ObjectModule};

use komo_ir::{DataGlobal, Konstanta, ModulIr};

use crate::fungsi::kompilasi_fungsi;
use crate::target::KonfigurasiTarget;
use crate::tipe::konversi_tipe_ke_cranelift;

/// Hasil kompilasi modul native ke dalam bentuk biner objek.
#[derive(Debug, Clone)]
pub struct HasilKompilasiObjek {
    /// Bita berkas objek ELF / Mach-O / COFF hasil emisi Cranelift.
    pub bytes_objek: Vec<u8>,
    /// Nama modul.
    pub nama_modul: String,
    /// Target konfigurasi yang digunakan.
    pub target: KonfigurasiTarget,
}

/// Mengompilasi `ModulIr` menjadi berkas objek native (.o / .obj) menggunakan `ObjectModule`.
pub fn kompilasi_ke_objek(
    modul: &ModulIr,
    target: &KonfigurasiTarget,
) -> Result<HasilKompilasiObjek, String> {
    let isa = target.bangun_isa()?;
    let pointer_type = isa.pointer_type();

    let builder = ObjectBuilder::new(
        isa,
        modul.nama.clone(),
        cranelift_module::default_libcall_names(),
    )
    .map_err(|e| format!("Gagal menginisialisasi ObjectBuilder: {}", e))?;

    let mut object_module = ObjectModule::new(builder);

    // 1. Emit seluruh data global dan konstanta modul
    for global in &modul.data_global {
        emit_data_global(&mut object_module, global)?;
    }

    // 2. Deklarasikan seluruh signature fungsi pada modul
    let mut peta_fungsi: HashMap<String, FuncId> = HashMap::new();
    for f in &modul.fungsi {
        let mut sig = object_module.make_signature();
        for param in &f.parameter {
            if let Some(ty) = konversi_tipe_ke_cranelift(&param.tipe, pointer_type) {
                sig.params.push(AbiParam::new(ty));
            }
        }
        if let Some(ret_ty) = konversi_tipe_ke_cranelift(&f.tipe_kembali, pointer_type) {
            sig.returns.push(AbiParam::new(ret_ty));
        }

        let linkage = Linkage::Export;
        let func_id = object_module
            .declare_function(&f.nama, linkage, &sig)
            .map_err(|e| format!("Gagal mendeklarasikan fungsi '{}': {}", f.nama, e))?;
        peta_fungsi.insert(f.nama.clone(), func_id);
    }

    // 3. Bangun dan kompilasi badan setiap fungsi
    let mut ctx = object_module.make_context();
    let mut builder_ctx = FunctionBuilderContext::new();
    let mut string_cache = HashMap::new();

    for f in &modul.fungsi {
        let func_id = peta_fungsi[&f.nama];
        ctx.func.signature = object_module.declarations().get_function_decl(func_id).signature.clone();

        {
            let mut builder = FunctionBuilder::new(&mut ctx.func, &mut builder_ctx);
            kompilasi_fungsi(
                &mut object_module,
                &mut builder,
                f,
                &peta_fungsi,
                &mut string_cache,
            )?;
            builder.finalize();
        }

        object_module
            .define_function(func_id, &mut ctx)
            .map_err(|e| format!("Gagal mengompilasi badan fungsi '{}': {}", f.nama, e))?;
        object_module.clear_context(&mut ctx);
    }

    // 4. Jika terdapat fungsi `utama` dan belum ada fungsi bernama literal `main`,
    // buatkan pembungkus (entrypoint wrapper) C standar `main() -> i32`
    if peta_fungsi.contains_key("utama") && !peta_fungsi.contains_key("main") {
        buat_wrapper_c_main(
            &mut object_module,
            &peta_fungsi,
            &mut ctx,
            &mut builder_ctx,
            modul,
        )?;
    }

    // 5. Selesaikan modul dan hasilkan bita berkas objek
    let product = object_module.finish();
    let bytes_objek = product
        .emit()
        .map_err(|e| format!("Gagal mengemisikan berkas objek: {}", e))?;

    Ok(HasilKompilasiObjek {
        bytes_objek,
        nama_modul: modul.nama.clone(),
        target: target.clone(),
    })
}

fn emit_data_global<M: Module>(module: &mut M, global: &DataGlobal) -> Result<FuncId, String> {
    let mut data_ctx = DataDescription::new();
    if let Some(init) = &global.inisialisasi {
        match init {
            Konstanta::Bulat(n) => data_ctx.define(Box::new(n.to_ne_bytes())),
            Konstanta::Desimal(d) => data_ctx.define(Box::new(d.to_ne_bytes())),
            Konstanta::Bool(b) => data_ctx.define(Box::new([if *b { 1 } else { 0 }])),
            Konstanta::Karakter(c) => data_ctx.define(Box::new((*c as u32).to_ne_bytes())),
            Konstanta::Bita(b) => data_ctx.define(Box::new([*b])),
            Konstanta::Teks(s) => {
                let mut bytes = s.as_bytes().to_vec();
                bytes.push(0);
                data_ctx.define(bytes.into_boxed_slice());
            }
            Konstanta::Nol | Konstanta::Kosong => {
                data_ctx.define_zeroinit(global.tipe.ukuran_bita().max(8));
            }
        }
    } else {
        data_ctx.define_zeroinit(global.tipe.ukuran_bita().max(8));
    }

    let data_id = module
        .declare_data(&global.nama, Linkage::Export, !global.konstan, false)
        .map_err(|e| format!("Gagal mendeklarasikan global '{}': {}", global.nama, e))?;

    module
        .define_data(data_id, &data_ctx)
        .map_err(|e| format!("Gagal mendefinisikan global '{}': {}", global.nama, e))?;

    // FuncId dummy placeholder
    Ok(FuncId::from_u32(0))
}

fn buat_wrapper_c_main<M: Module>(
    module: &mut M,
    peta_fungsi: &HashMap<String, FuncId>,
    ctx: &mut cranelift_codegen::Context,
    builder_ctx: &mut FunctionBuilderContext,
    modul: &ModulIr,
) -> Result<(), String> {
    let mut sig = module.make_signature();
    sig.returns.push(AbiParam::new(types::I32));

    let main_id = module
        .declare_function("main", Linkage::Export, &sig)
        .map_err(|e| format!("Gagal mendeklarasikan fungsi 'main': {}", e))?;

    ctx.func.signature = sig;
    {
        let mut builder = FunctionBuilder::new(&mut ctx.func, builder_ctx);
        let entry = builder.create_block();
        builder.switch_to_block(entry);
        builder.seal_block(entry);

        let utama_id = peta_fungsi["utama"];
        let local_utama = module.declare_func_in_func(utama_id, builder.func);
        let call_inst = builder.ins().call(local_utama, &[]);
        let results = builder.inst_results(call_inst);

        let fungsi_utama_ir = modul.cari_fungsi("utama").unwrap();
        let ret_val = if !results.is_empty() && fungsi_utama_ir.tipe_kembali != komo_ir::IrTipe::Void {
            let res = results[0];
            let res_ty = builder.func.dfg.value_type(res);
            if res_ty == types::I32 {
                res
            } else if res_ty.is_int() && res_ty.bits() > 32 {
                builder.ins().ireduce(types::I32, res)
            } else if res_ty.is_int() {
                builder.ins().sextend(types::I32, res)
            } else {
                builder.ins().iconst(types::I32, 0)
            }
        } else {
            builder.ins().iconst(types::I32, 0)
        };

        builder.ins().return_(&[ret_val]);
        builder.finalize();
    }

    module
        .define_function(main_id, ctx)
        .map_err(|e| format!("Gagal mendefinisikan fungsi pembungkus 'main': {}", e))?;
    module.clear_context(ctx);

    Ok(())
}
