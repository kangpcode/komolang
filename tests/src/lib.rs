// Copyright (c) kangpcode (Dhafa Nazula Permadi). Licensed under MIT or Apache-2.0.

//! Test suite integrasi dan konformansi Komolang.

#[cfg(test)]
mod tests {
    use komo_lexer::Pemindai;
    use komo_parser::Pengurai;
    use komo_runtime::KonteksRuntime;
    use std::fs;
    use std::path::Path;

    #[test]
    fn tes_seluruh_contoh_kode() {
        let contoh_dir = Path::new(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .unwrap()
            .join("examples");
        let berkas_contoh = vec!["halo_dunia.komo", "perhitungan.komo"];

        for nama_berkas in berkas_contoh {
            let jalur = contoh_dir.join(nama_berkas);
            assert!(jalur.exists(), "Berkas contoh {:?} harus ada", jalur);

            let kode = fs::read_to_string(&jalur).expect("Harus dapat membaca berkas contoh");
            let mut pemindai = Pemindai::baru(&kode);
            let tokens = pemindai
                .pindai_semua()
                .expect("Pemindaian leksikal harus berhasil");

            let mut pengurai = Pengurai::baru(tokens);
            let ast = pengurai
                .urai_berkas()
                .expect("Penguraian AST harus berhasil");

            assert_eq!(ast.paket.nama.nama, "utama");
        }
    }

    #[test]
    fn tes_koneksi_runtime_dan_alokator() {
        let rt = KonteksRuntime::baru();
        assert!(rt.aktif);
        assert!(!rt.versi.is_empty());
    }
}
