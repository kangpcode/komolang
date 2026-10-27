// Copyright (c) kangpcode (Dhafa Nazula Permadi). Licensed under MIT or Apache-2.0.

//! Mesin eksekusi Just-In-Time (JIT) in-memory untuk Komolang.
//! Memungkinkan pengujian dan eksekusi instan tanpa memerlukan linker eksternal.

use std::collections::HashMap;

use cranelift_codegen::ir::AbiParam;
use cranelift_frontend::{FunctionBuilder, FunctionBuilderContext};
use cranelift_jit::{JITBuilder, JITModule};
use cranelift_module::{FuncId, Linkage, Module};

use komo_ir::ModulIr;

use crate::fungsi::kompilasi_fungsi;
use crate::runtime::{
    komo_rt_format_teks, komo_std_cetak_baris, komo_std_cetak_bulat, komo_std_cetak_desimal,
    komo_std_cetak_teks,
};
use crate::tipe::konversi_tipe_ke_cranelift;

/// Mesin eksekutor program Komolang secara JIT in-memory.
pub struct EksekutorJit {
    module: JITModule,
    peta_fungsi: HashMap<String, FuncId>,
    tipe_kembali_utama: Option<komo_ir::IrTipe>,
}

impl EksekutorJit {
    /// Membuat eksekutor JIT baru dan mengompilasi `ModulIr` ke dalam memori eksekusi.
    pub fn baru(modul: &ModulIr) -> Result<Self, String> {
        let mut builder = JITBuilder::new(cranelift_module::default_libcall_names())
            .map_err(|e| format!("Gagal membuat JITBuilder: {}", e))?;

        // Daftarkan simbol runtime resmi ke dalam tabel lookup JIT
        builder.symbol("komo_std_cetak_baris", komo_std_cetak_baris as *const u8);
        builder.symbol("komo_std_cetak_teks", komo_std_cetak_teks as *const u8);
        builder.symbol("komo_std_cetak_bulat", komo_std_cetak_bulat as *const u8);
        builder.symbol("komo_std_cetak_desimal", komo_std_cetak_desimal as *const u8);
        builder.symbol("komo_rt_format_teks", komo_rt_format_teks as *const u8);
        builder.symbol("puts", libc::puts as *const u8);

        let mut jit_module = JITModule::new(builder);
        let pointer_type = jit_module.target_config().pointer_type();

        // 1. Deklarasikan seluruh signature fungsi
        let mut peta_fungsi: HashMap<String, FuncId> = HashMap::new();
        let mut peta_signature: HashMap<String, cranelift_codegen::ir::Signature> = HashMap::new();
        for f in &modul.fungsi {
            let mut sig = jit_module.make_signature();
            for param in &f.parameter {
                if let Some(ty) = konversi_tipe_ke_cranelift(&param.tipe, pointer_type) {
                    sig.params.push(AbiParam::new(ty));
                }
            }
            if let Some(ret_ty) = konversi_tipe_ke_cranelift(&f.tipe_kembali, pointer_type) {
                sig.returns.push(AbiParam::new(ret_ty));
            }

            peta_signature.insert(f.nama.clone(), sig.clone());
            let func_id = jit_module
                .declare_function(&f.nama, Linkage::Export, &sig)
                .map_err(|e| format!("Gagal mendeklarasikan fungsi JIT '{}': {}", f.nama, e))?;
            peta_fungsi.insert(f.nama.clone(), func_id);
        }

        // 2. Bangun dan kompilasi badan fungsi
        let mut ctx = jit_module.make_context();
        let mut builder_ctx = FunctionBuilderContext::new();
        let mut string_cache = HashMap::new();

        for f in &modul.fungsi {
            let func_id = peta_fungsi[&f.nama];
            ctx.func.signature = jit_module.declarations().get_function_decl(func_id).signature.clone();

            {
                let mut builder = FunctionBuilder::new(&mut ctx.func, &mut builder_ctx);
                kompilasi_fungsi(
                    &mut jit_module,
                    &mut builder,
                    f,
                    &peta_fungsi,
                    &peta_signature,
                    &mut string_cache,
                )?;
                builder.finalize(jit_module.target_config());
            }

            if let Err(e) = jit_module.define_function(func_id, &mut ctx) {
                let rincian = match cranelift_codegen::verifier::verify_function(&ctx.func, jit_module.isa()) {
                    Ok(_) => format!("{}", e),
                    Err(verif) => format!("{}: {}", e, verif),
                };
                return Err(format!("Gagal mengompilasi fungsi JIT '{}': {}", f.nama, rincian));
            }
            jit_module.clear_context(&mut ctx);
        }

        // 3. Finalisasi kode mesin di memori
        jit_module
            .finalize_definitions()
            .map_err(|e| format!("Gagal memfinalisasi definisi JIT: {}", e))?;

        let tipe_kembali_utama = modul
            .cari_fungsi("utama")
            .or_else(|| modul.cari_fungsi("main"))
            .map(|f| f.tipe_kembali.clone());

        Ok(Self {
            module: jit_module,
            peta_fungsi,
            tipe_kembali_utama,
        })
    }

    /// Menjalankan fungsi `utama()` dan mengembalikan hasilnya sebagai integer 64-bit.
    pub fn jalankan_utama(&self) -> Result<i64, String> {
        let func_id = self
            .peta_fungsi
            .get("utama")
            .or_else(|| self.peta_fungsi.get("main"))
            .ok_or_else(|| "Fungsi 'utama' atau 'main' tidak ditemukan dalam modul".to_string())?;

        let code_ptr = self.module.get_finalized_function(*func_id);
        if code_ptr.is_null() {
            return Err("Penunjuk kode fungsi 'utama' bernilai null".to_string());
        }

        if self.tipe_kembali_utama == Some(komo_ir::IrTipe::Void) {
            let f: unsafe extern "C" fn() = unsafe { std::mem::transmute(code_ptr) };
            unsafe { f() };
            Ok(0)
        } else {
            let f: unsafe extern "C" fn() -> i64 = unsafe { std::mem::transmute(code_ptr) };
            let hasil = unsafe { f() };
            Ok(hasil)
        }
    }

    /// Menjalankan fungsi dengan satu parameter integer (misalnya `fibonacci(n)`).
    pub fn jalankan_fungsi_i64_ke_i64(&self, nama: &str, arg: i64) -> Result<i64, String> {
        let func_id = self
            .peta_fungsi
            .get(nama)
            .ok_or_else(|| format!("Fungsi '{}' tidak ditemukan", nama))?;

        let code_ptr = self.module.get_finalized_function(*func_id);
        if code_ptr.is_null() {
            return Err(format!("Penunjuk fungsi '{}' bernilai null", nama));
        }

        let f: unsafe extern "C" fn(i64) -> i64 = unsafe { std::mem::transmute(code_ptr) };
        let hasil = unsafe { f(arg) };
        Ok(hasil)
    }

    /// Menjalankan fungsi dengan dua parameter integer (misalnya `tambah(a, b)`).
    pub fn jalankan_fungsi_i64_2_ke_i64(&self, nama: &str, arg1: i64, arg2: i64) -> Result<i64, String> {
        let func_id = self
            .peta_fungsi
            .get(nama)
            .ok_or_else(|| format!("Fungsi '{}' tidak ditemukan", nama))?;

        let code_ptr = self.module.get_finalized_function(*func_id);
        if code_ptr.is_null() {
            return Err(format!("Penunjuk fungsi '{}' bernilai null", nama));
        }

        let f: unsafe extern "C" fn(i64, i64) -> i64 = unsafe { std::mem::transmute(code_ptr) };
        let hasil = unsafe { f(arg1, arg2) };
        Ok(hasil)
    }
}
