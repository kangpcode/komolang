// Copyright (c) kangpcode (Dhafa Nazula Permadi). Licensed under MIT or Apache-2.0.

//! Framework interoperabilitas JavaScript & WebAssembly (WASM) untuk integrasi Vue.js dan React.

use std::collections::HashMap;

/// Nilai data universal untuk transfer dua arah antara Komolang dan JavaScript.
#[derive(Debug, Clone, PartialEq)]
pub enum NilaiJs {
    Nihil,
    Bool(bool),
    Bulat(i64),
    Desimal(f64),
    Teks(String),
    Larik(Vec<NilaiJs>),
    Objek(HashMap<String, NilaiJs>),
}

impl NilaiJs {
    /// Mengonversi nilai menjadi representasi string JSON yang siap dikonsumsi JavaScript.
    pub fn ke_json(&self) -> String {
        match self {
            NilaiJs::Nihil => "null".to_string(),
            NilaiJs::Bool(b) => if *b { "true" } else { "false" }.to_string(),
            NilaiJs::Bulat(n) => n.to_string(),
            NilaiJs::Desimal(f) => format!("{:.4}", f),
            NilaiJs::Teks(s) => format!("\"{}\"", s.replace('"', "\\\"")),
            NilaiJs::Larik(l) => {
                let items = l.iter().map(|item| item.ke_json()).collect::<Vec<_>>().join(", ");
                format!("[{}]", items)
            }
            NilaiJs::Objek(o) => {
                let entries = o
                    .iter()
                    .map(|(k, v)| format!("\"{}\": {}", k, v.ke_json()))
                    .collect::<Vec<_>>()
                    .join(", ");
                format!("{{{}}}", entries)
            }
        }
    }
}

/// Metadata fungsi yang diekspor dari Komolang ke modul WASM.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DeskripsiFungsiWasm {
    pub nama: String,
    pub parameter: Vec<(String, String)>, // (nama_param, tipe_param)
    pub tipe_kembali: String,
    pub deskripsi: String,
}

/// Menghasilkan modul wrapper JavaScript untuk composable Vue.js 3 (`useKomolang`).
pub fn buat_binding_vue(modul_wasm: &str, daftar_fungsi: &[DeskripsiFungsiWasm]) -> String {
    let mut js = String::new();
    js.push_str("// Copyright (c) kangpcode. Dihasilkan otomatis oleh KomoUI Interop Framework.\n");
    js.push_str("// Binding resmi Komolang WASM untuk Vue.js 3 (Composition API).\n\n");
    js.push_str("import { ref, onMounted } from 'vue';\n\n");

    js.push_str("let instanceWasm = null;\n\n");

    js.push_str(&format!(
        r#"export async function inisialisasiKomolang(jalurWasm = '/{}') {{
    if (instanceWasm) return instanceWasm;
    const response = await fetch(jalurWasm);
    const bytes = await response.arrayBuffer();
    const {{ instance }} = await WebAssembly.instantiate(bytes, {{
        env: {{
            komo_cetak: (ptr, len) => {{
                // Interop log output
            }}
        }}
    }});
    instanceWasm = instance.exports;
    return instanceWasm;
}}

export function useKomolang() {{
    const siap = ref(false);
    const galat = ref(null);

    onMounted(async () => {{
        try {{
            await inisialisasiKomolang();
            siap.value = true;
        }} catch (e) {{
            galat.value = e;
        }}
    }});

"#,
        modul_wasm
    ));

    for f in daftar_fungsi {
        let params_str = f
            .parameter
            .iter()
            .map(|(p, _)| p.as_str())
            .collect::<Vec<_>>()
            .join(", ");

        js.push_str(&format!(
            "    /**\n     * {}\n     */\n",
            f.deskripsi
        ));
        js.push_str(&format!(
            r#"    const {} = ({}) => {{
        if (!instanceWasm) throw new Error("Modul Komolang WASM belum siap!");
        return instanceWasm.{}({});
    }};

"#,
            f.nama, params_str, f.nama, params_str
        ));
    }

    let exports_str = daftar_fungsi
        .iter()
        .map(|f| format!("        {},", f.nama))
        .collect::<Vec<_>>()
        .join("\n");

    js.push_str(&format!(
        r#"    return {{
        siap,
        galat,
{}
    }};
}}
"#,
        exports_str
    ));

    js
}

/// Menghasilkan file deklarasi TypeScript (`.d.ts`) untuk keamanan tipe data di IDE modern.
pub fn buat_definisi_typescript(daftar_fungsi: &[DeskripsiFungsiWasm]) -> String {
    let mut ts = String::new();
    ts.push_str("// Definisi tipe TypeScript untuk Komolang WebAssembly Bridge\n\n");
    ts.push_str("import type { Ref } from 'vue';\n\n");
    ts.push_str("export function inisialisasiKomolang(jalurWasm?: string): Promise<any>;\n\n");
    ts.push_str("export interface AntarmukaKomolang {\n");
    ts.push_str("    siap: Ref<boolean>;\n");
    ts.push_str("    galat: Ref<Error | null>;\n");

    for f in daftar_fungsi {
        let params_ts = f
            .parameter
            .iter()
            .map(|(nama, tipe)| {
                let ts_type = match tipe.as_str() {
                    "bulat" | "desimal" => "number",
                    "bool" => "boolean",
                    "teks" => "string",
                    _ => "any",
                };
                format!("{}: {}", nama, ts_type)
            })
            .collect::<Vec<_>>()
            .join(", ");

        let ret_ts = match f.tipe_kembali.as_str() {
            "bulat" | "desimal" => "number",
            "bool" => "boolean",
            "teks" => "string",
            _ => "any",
        };

        ts.push_str(&format!(
            "    {}({}): {};\n",
            f.nama, params_ts, ret_ts
        ));
    }

    ts.push_str("}\n\n");
    ts.push_str("export function useKomolang(): AntarmukaKomolang;\n");
    ts
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tes_serialisasi_nilai_js() {
        let mut map = HashMap::new();
        map.insert("nama".into(), NilaiJs::Teks("Komolang".into()));
        map.insert("versi".into(), NilaiJs::Bulat(1));
        map.insert("aktif".into(), NilaiJs::Bool(true));

        let obj = NilaiJs::Objek(map);
        let json = obj.ke_json();

        assert!(json.contains("\"nama\": \"Komolang\""));
        assert!(json.contains("\"versi\": 1"));
        assert!(json.contains("\"aktif\": true"));
    }

    #[test]
    fn tes_generator_binding_vue() {
        let funcs = vec![
            DeskripsiFungsiWasm {
                nama: "hitung_faktorial".into(),
                parameter: vec![("n".into(), "bulat".into())],
                tipe_kembali: "bulat".into(),
                deskripsi: "Menghitung faktorial bilangan dengan algoritma native Komolang".into(),
            },
            DeskripsiFungsiWasm {
                nama: "enkripsi_sha256".into(),
                parameter: vec![("pesan".into(), "teks".into())],
                tipe_kembali: "teks".into(),
                deskripsi: "Enkripsi teks ke SHA-256".into(),
            },
        ];

        let js = buat_binding_vue("aplikasi.wasm", &funcs);
        assert!(js.contains("export function useKomolang()"));
        assert!(js.contains("const hitung_faktorial = (n) =>"));
        assert!(js.contains("const enkripsi_sha256 = (pesan) =>"));

        let ts = buat_definisi_typescript(&funcs);
        assert!(ts.contains("hitung_faktorial(n: number): number;"));
        assert!(ts.contains("enkripsi_sha256(pesan: string): string;"));
    }
}
