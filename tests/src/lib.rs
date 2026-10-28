// Copyright (c) kangpcode (Dhafa Nazula Permadi). Licensed under MIT or Apache-2.0.

//! Test suite integrasi dan konformansi Komolang.

#[cfg(test)]
mod tes_fuzzing;
#[cfg(test)]
mod tes_ir_optimizer;
#[cfg(test)]
mod tes_ir_valid;
#[cfg(test)]
mod tes_ir_verifier;
#[cfg(test)]
mod tes_leksikal;
#[cfg(test)]
mod tes_pengurai_galat;
#[cfg(test)]
mod tes_pengurai_valid;
#[cfg(test)]
mod tes_pretty_printer;
#[cfg(test)]
mod tes_semantik_galat;
#[cfg(test)]
mod tes_semantik_valid;
#[cfg(test)]
mod tes_codegen_native;
#[cfg(test)]
mod tes_codegen_asm;
#[cfg(test)]
mod tes_runtime;
#[cfg(test)]
mod tes_std;
#[cfg(test)]
mod tes_cli_tooling;
#[cfg(test)]
mod tes_paket_registry;
#[cfg(test)]
mod tes_web_framework;
#[cfg(test)]
mod tes_ui_interop;
#[cfg(test)]
mod tes_game_engine;
#[cfg(test)]
mod tes_lintas_platform;

#[cfg(test)]
mod tests {
    use komo_diag::render_semua;
    use komo_parser::urai;
    use komo_runtime::KonteksRuntime;
    use std::fs;
    use std::path::Path;

    #[test]
    fn tes_seluruh_contoh_kode() {
        let contoh_dir = Path::new(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .unwrap()
            .join("examples");
        let berkas_contoh = vec![
            "halo_dunia.komo",
            "perhitungan.komo",
            "sintaks_lengkap.komo",
        ];

        for nama_berkas in berkas_contoh {
            let jalur = contoh_dir.join(nama_berkas);
            assert!(jalur.exists(), "Berkas contoh {:?} harus ada", jalur);

            let kode = fs::read_to_string(&jalur).expect("Harus dapat membaca berkas contoh");
            let hasil = urai(&kode);
            assert!(
                hasil.berhasil(),
                "Penguraian berkas {:?} gagal:\n{}",
                jalur,
                render_semua(&hasil.diagnostik, nama_berkas, &kode)
            );

            let hasil_sema = komo_sema::periksa(&hasil.berkas);
            assert!(
                hasil_sema.berhasil(),
                "Analisis semantik berkas {:?} gagal:\n{}",
                jalur,
                render_semua(&hasil_sema.diagnostik, nama_berkas, &kode)
            );

            assert_eq!(hasil.berkas.paket.nama.nama, "utama");
        }
    }

    #[test]
    fn tes_koneksi_runtime_dan_alokator() {
        let rt = KonteksRuntime::baru();
        assert!(rt.aktif);
        assert!(!rt.versi.is_empty());
    }
}
