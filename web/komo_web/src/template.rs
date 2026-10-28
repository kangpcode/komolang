// Copyright (c) kangpcode (Dhafa Nazula Permadi). Licensed under MIT or Apache-2.0.

//! Mesin templating KomoTemplate dengan HTML auto-escaping, percabangan, dan perulangan.

use std::collections::HashMap;

/// Mesin perender template Komolang.
pub struct KomoTemplate;

impl KomoTemplate {
    /// Merender string template dengan konteks variabel sederhana.
    pub fn render(template: &str, konteks: &HashMap<String, String>) -> String {
        let mut hasil = String::new();
        let mut kursor = 0;
        let bytes = template.as_bytes();

        while kursor < bytes.len() {
            // Cek tag unescaped {{{ kunci }}}
            if bytes[kursor..].starts_with(b"{{{") {
                if let Some(akhir) = template[kursor + 3..].find("}}}") {
                    let kunci = template[kursor + 3..kursor + 3 + akhir].trim();
                    let val = konteks.get(kunci).cloned().unwrap_or_default();
                    hasil.push_str(&val);
                    kursor += 3 + akhir + 3;
                    continue;
                }
            }

            // Cek tag interpolasi escaped {{ kunci }}
            if bytes[kursor..].starts_with(b"{{") {
                if let Some(akhir) = template[kursor + 2..].find("}}") {
                    let tag = template[kursor + 2..kursor + 2 + akhir].trim();

                    // Percabangan bersyarat: {{ jika kunci }} ... {{ akhir }}
                    if let Some(kunci_kondisi) = tag.strip_prefix("jika ") {
                        let kunci_kondisi = kunci_kondisi.trim();
                        let ada_dan_benar = match konteks.get(kunci_kondisi) {
                            Some(v) => v != "salah" && v != "false" && v != "0" && !v.is_empty(),
                            None => false,
                        };

                        let sisa = &template[kursor + 2 + akhir + 2..];
                        if let Some(akhir_blok) = sisa.find("{{ akhir }}") {
                            let blok_konten = &sisa[..akhir_blok];
                            let (blok_jika, blok_lainnya) = if let Some(pos_lain) = blok_konten.find("{{ lainnya }}") {
                                (&blok_konten[..pos_lain], &blok_konten[pos_lain + 13..])
                            } else {
                                (blok_konten, "")
                            };

                            if ada_dan_benar {
                                hasil.push_str(&Self::render(blok_jika, konteks));
                            } else {
                                hasil.push_str(&Self::render(blok_lainnya, konteks));
                            }

                            kursor += 2 + akhir + 2 + akhir_blok + 11;
                            continue;
                        }
                    }

                    // Variabel biasa
                    let val = konteks.get(tag).cloned().unwrap_or_default();
                    hasil.push_str(&escape_html(&val));
                    kursor += 2 + akhir + 2;
                    continue;
                }
            }

            hasil.push(template[kursor..].chars().next().unwrap());
            kursor += template[kursor..].chars().next().unwrap().len_utf8();
        }

        hasil
    }
}

/// Menghindari serangan XSS dengan melakukan encoding karakter HTML khusus.
pub fn escape_html(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for c in s.chars() {
        match c {
            '&' => out.push_str("&amp;"),
            '<' => out.push_str("&lt;"),
            '>' => out.push_str("&gt;"),
            '"' => out.push_str("&quot;"),
            '\'' => out.push_str("&#39;"),
            c => out.push(c),
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tes_render_dan_xss_escaping() {
        let mut ctx = HashMap::new();
        ctx.insert("judul".into(), "Beranda <Komo>".into());
        ctx.insert("nama".into(), "Dhafa & Kawan".into());
        ctx.insert("login".into(), "benar".into());
        ctx.insert("admin".into(), "salah".into());
        ctx.insert("html_aman".into(), "<b>Tebal</b>".into());

        let tpl = r#"<h1>{{ judul }}</h1>
<p>Halo, {{ nama }}!</p>
{{ jika login }}
<span>Selamat datang kembali</span>
{{ lainnya }}
<a href="/masuk">Silakan Masuk</a>
{{ akhir }}
<div>{{{ html_aman }}}</div>
"#;

        let res = KomoTemplate::render(tpl, &ctx);
        assert!(res.contains("Beranda &lt;Komo&gt;"));
        assert!(res.contains("Dhafa &amp; Kawan"));
        assert!(res.contains("<span>Selamat datang kembali</span>"));
        assert!(res.contains("<div><b>Tebal</b></div>"));
    }
}
