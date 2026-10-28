// Copyright (c) kangpcode (Dhafa Nazula Permadi). Licensed under MIT or Apache-2.0.

//! Server Language Server Protocol (LSP) Komolang berbasis JSON-RPC 2.0.

use std::collections::HashMap;
use std::io::{self, BufRead, Read, Write};

use komo_ast::cetak_berkas;
use komo_parser::urai;
use komo_sema::periksa;

/// Menjalankan loop utama LSP server melalui I/O standar (stdin / stdout).
pub fn jalankan_server_lsp() {
    let mut stdin = io::stdin().lock();
    let mut stdout = io::stdout();

    let mut dokumen_aktif: HashMap<String, String> = HashMap::new();

    loop {
        // 1. Baca header Content-Length
        let mut panjang_konten: usize = 0;
        let mut baris_header = String::new();

        loop {
            baris_header.clear();
            if stdin.read_line(&mut baris_header).unwrap_or(0) == 0 {
                return; // EOF
            }

            let trim = baris_header.trim();
            if trim.is_empty() {
                break; // Akhir header, muatan isi JSON dimulai
            }

            if let Some(val) = trim.strip_prefix("Content-Length:") {
                panjang_konten = val.trim().parse::<usize>().unwrap_or(0);
            }
        }

        if panjang_konten == 0 {
            continue;
        }

        // 2. Baca muatan JSON
        let mut buffer = vec![0u8; panjang_konten];
        if stdin.read_exact(&mut buffer).is_err() {
            return;
        }

        let pesan_json = match String::from_utf8(buffer) {
            Ok(s) => s,
            Err(_) => continue,
        };

        // 3. Tangani permintaan LSP
        let (respons, notifikasi) = proses_permintaan_lsp(&pesan_json, &mut dokumen_aktif);
        for notif in notifikasi {
            kirim_pesan_mentah(&mut stdout, &notif);
        }
        if let Some(resp) = respons {
            kirim_pesan_mentah(&mut stdout, &resp);
        }
    }
}

/// Memproses satu pesan JSON-RPC LSP dan menghasilkan respons (jika berupa request dengan id)
/// serta daftar pesan notifikasi (misal publishDiagnostics).
pub fn proses_permintaan_lsp(
    pesan: &str,
    dokumen: &mut HashMap<String, String>,
) -> (Option<String>, Vec<String>) {
    let id_opt = ekstrak_id_json(pesan);
    let metode = ekstrak_string_json(pesan, "method");
    let mut notifikasi = Vec::new();
    let mut respons = None;

    match metode.as_str() {
        "initialize" => {
            let res = r#"{
                "capabilities": {
                    "textDocumentSync": 1,
                    "completionProvider": {
                        "resolveProvider": false,
                        "triggerCharacters": [".", " "]
                    },
                    "hoverProvider": true,
                    "documentFormattingProvider": true
                },
                "serverInfo": {
                    "name": "komo-lsp",
                    "version": "0.1.0"
                }
            }"#;
            respons = Some(format_respons(id_opt, res));
        }
        "initialized" => {
            // Notifikasi klien bahwa inisialisasi selesai
        }
        "shutdown" => {
            respons = Some(format_respons(id_opt, "null"));
        }
        "exit" => {
            // Permintaan keluar
        }
        "textDocument/didOpen" => {
            if let Some(uri) = ekstrak_uri_dokumen(pesan) {
                if let Some(teks) = ekstrak_teks_dokumen(pesan) {
                    dokumen.insert(uri.clone(), teks.clone());
                    notifikasi.push(buat_pesan_diagnostik(&uri, &teks));
                }
            }
        }
        "textDocument/didChange" => {
            if let Some(uri) = ekstrak_uri_dokumen(pesan) {
                if let Some(teks) = ekstrak_teks_dokumen(pesan) {
                    dokumen.insert(uri.clone(), teks.clone());
                    notifikasi.push(buat_pesan_diagnostik(&uri, &teks));
                }
            }
        }
        "textDocument/completion" => {
            let items = r#"[
                {"label": "fungsi", "kind": 14, "detail": "Deklarasi fungsi"},
                {"label": "publik", "kind": 14, "detail": "Penentu visibilitas publik"},
                {"label": "tipe", "kind": 14, "detail": "Deklarasi tipe data"},
                {"label": "struktur", "kind": 14, "detail": "Tipe bentukan struktur"},
                {"label": "antarmuka", "kind": 14, "detail": "Tipe antarmuka / interface"},
                {"label": "enum", "kind": 14, "detail": "Tipe enumerasi aljabar"},
                {"label": "konst", "kind": 14, "detail": "Deklarasi nilai konstanta"},
                {"label": "biar", "kind": 14, "detail": "Variabel lokal imutabel"},
                {"label": "var", "kind": 14, "detail": "Variabel mutabel"},
                {"label": "jika", "kind": 14, "detail": "Percabangan kondisi jika"},
                {"label": "lainnya", "kind": 14, "detail": "Cabang fallback lainnya"},
                {"label": "selagi", "kind": 14, "detail": "Pengulangan bersyarat"},
                {"label": "untuk", "kind": 14, "detail": "Pengulangan iterator elemen"},
                {"label": "dalam", "kind": 14, "detail": "Kata kunci iterator rentang/koleksi"},
                {"label": "cocok", "kind": 14, "detail": "Pencocokan pola eksaustif"},
                {"label": "kembali", "kind": 14, "detail": "Nilai kembalian fungsi"},
                {"label": "tunda", "kind": 14, "detail": "Penundaan eksekusi hingga akhir lingkup (defer)"},
                {"label": "jalankan", "kind": 14, "detail": "Peluncuran green task konkurensi"},
                {"label": "saluran", "kind": 14, "detail": "Saluran komunikasi antartask (channel)"},
                {"label": "benar", "kind": 14, "detail": "Literal boolean true"},
                {"label": "salah", "kind": 14, "detail": "Literal boolean false"},
                {"label": "bulat", "kind": 7, "detail": "Tipe bilangan bulat 64-bit"},
                {"label": "desimal", "kind": 7, "detail": "Tipe bilangan pecahan ganda 64-bit"},
                {"label": "teks", "kind": 7, "detail": "Tipe untaian karakter UTF-8"},
                {"label": "bool", "kind": 7, "detail": "Tipe nilai kebenaran logika"},
                {"label": "larik", "kind": 7, "detail": "Tipe koleksi berurutan dinamis"}
            ]"#;
            let resp = format!(r#"{{"isIncomplete": false, "items": {}}}"#, items);
            respons = Some(format_respons(id_opt, &resp));
        }
        "textDocument/hover" => {
            let res = r#"{
                "contents": {
                    "kind": "markdown",
                    "value": "**Komolang Official Language Server**\n\nBahasa pemrograman sistem modern berbahasa Indonesia."
                }
            }"#;
            respons = Some(format_respons(id_opt, res));
        }
        "textDocument/formatting" => {
            if let Some(uri) = ekstrak_uri_dokumen(pesan) {
                if let Some(teks) = dokumen.get(&uri) {
                    let hasil = urai(teks);
                    if hasil.berhasil() {
                        let terformat = cetak_berkas(&hasil.berkas);
                        let baris_total = teks.lines().count();
                        let resp = format!(
                            r#"[{{
                                "range": {{
                                    "start": {{"line": 0, "character": 0}},
                                    "end": {{"line": {}, "character": 0}}
                                }},
                                "newText": {}
                            }}]"#,
                            baris_total + 1,
                            escape_json(&terformat)
                        );
                        return (Some(format_respons(id_opt, &resp)), notifikasi);
                    }
                }
            }
            respons = Some(format_respons(id_opt, "[]"));
        }
        _ => {
            if id_opt.is_some() {
                respons = Some(format_respons(id_opt, "null"));
            }
        }
    }

    (respons, notifikasi)
}

fn buat_pesan_diagnostik(uri: &str, teks: &str) -> String {
    let mut diag_json = Vec::new();

    let hasil_urai = urai(teks);
    for d in &hasil_urai.diagnostik {
        let b = d.rentang.baris.saturating_sub(1);
        let k = d.rentang.kolom.saturating_sub(1);
        let baris_d = format!(
            r#"{{"range": {{"start": {{"line": {}, "character": {}}}, "end": {{"line": {}, "character": {}}}}}, "severity": 1, "source": "komo", "message": {}}}"#,
            b, k, b, k + 1,
            escape_json(&d.pesan)
        );
        diag_json.push(baris_d);
    }

    if hasil_urai.berhasil() {
        let hasil_sema = periksa(&hasil_urai.berkas);
        for d in &hasil_sema.diagnostik {
            let b = d.rentang.baris.saturating_sub(1);
            let k = d.rentang.kolom.saturating_sub(1);
            let baris_d = format!(
                r#"{{"range": {{"start": {{"line": {}, "character": {}}}, "end": {{"line": {}, "character": {}}}}}, "severity": 1, "source": "komo", "message": {}}}"#,
                b, k, b, k + 1,
                escape_json(&d.pesan)
            );
            diag_json.push(baris_d);
        }
    }

    format!(
        r#"{{"jsonrpc": "2.0", "method": "textDocument/publishDiagnostics", "params": {{"uri": "{}", "diagnostics": [{}]}}}}"#,
        uri,
        diag_json.join(", ")
    )
}

fn format_respons(id: Option<String>, hasil_json: &str) -> String {
    let id_str = id.unwrap_or_else(|| "1".into());
    format!(
        r#"{{"jsonrpc": "2.0", "id": {}, "result": {}}}"#,
        id_str, hasil_json
    )
}

fn kirim_pesan_mentah(stdout: &mut io::Stdout, muatan: &str) {
    let panjang = muatan.len();
    let paket = format!("Content-Length: {}\r\n\r\n{}", panjang, muatan);
    let _ = stdout.write_all(paket.as_bytes());
    let _ = stdout.flush();
}

fn escape_json(s: &str) -> String {
    let mut hasil = String::from("\"");
    for c in s.chars() {
        match c {
            '"' => hasil.push_str("\\\""),
            '\\' => hasil.push_str("\\\\"),
            '\n' => hasil.push_str("\\n"),
            '\r' => hasil.push_str("\\r"),
            '\t' => hasil.push_str("\\t"),
            c => hasil.push(c),
        }
    }
    hasil.push('"');
    hasil
}

fn ekstrak_string_json(sumber: &str, kunci: &str) -> String {
    let pola = format!("\"{}\":", kunci);
    if let Some(pos) = sumber.find(&pola) {
        let sisa = &sumber[pos + pola.len()..].trim_start();
        if sisa.starts_with('"') {
            if let Some(akhir) = sisa[1..].find('"') {
                return sisa[1..1 + akhir].to_string();
            }
        }
    }
    String::new()
}

fn ekstrak_id_json(sumber: &str) -> Option<String> {
    if let Some(pos) = sumber.find("\"id\":") {
        let sisa = &sumber[pos + 5..].trim_start();
        let mut akhir = 0;
        for c in sisa.chars() {
            if c == ',' || c == '}' || c.is_whitespace() {
                break;
            }
            akhir += c.len_utf8();
        }
        if akhir > 0 {
            return Some(sisa[..akhir].to_string());
        }
    }
    None
}

fn ekstrak_uri_dokumen(sumber: &str) -> Option<String> {
    let val = ekstrak_string_json(sumber, "uri");
    if !val.is_empty() {
        Some(val)
    } else {
        None
    }
}

fn ekstrak_teks_dokumen(sumber: &str) -> Option<String> {
    if let Some(pos) = sumber.find("\"text\":") {
        let sisa = &sumber[pos + 7..].trim_start();
        if sisa.starts_with('"') {
            let mut res = String::new();
            let mut escape = false;
            for c in sisa[1..].chars() {
                if escape {
                    match c {
                        'n' => res.push('\n'),
                        'r' => res.push('\r'),
                        't' => res.push('\t'),
                        '\\' => res.push('\\'),
                        '"' => res.push('"'),
                        _ => res.push(c),
                    }
                    escape = false;
                } else if c == '\\' {
                    escape = true;
                } else if c == '"' {
                    return Some(res);
                } else {
                    res.push(c);
                }
            }
        }
    }
    None
}
