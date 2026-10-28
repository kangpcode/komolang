// Copyright (c) kangpcode (Dhafa Nazula Permadi). Licensed under MIT or Apache-2.0.

//! Implementasi protokol WebSocket RFC 6455: Handshake, Framing, Text, Binary, Ping/Pong, dan Close.

use crate::http::{KodeStatus, ResponsHttp};

const WS_GUID: &str = "258EAFA5-E914-47DA-95CA-C5AB0DC85B11";

/// Opcode jenis frame WebSocket menurut RFC 6455.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OpcodeWs {
    Lanjutan = 0x0,
    Teks = 0x1,
    Biner = 0x2,
    Tutup = 0x8,
    Ping = 0x9,
    Pong = 0xA,
}

/// Struktur data frame pesan WebSocket.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FrameWs {
    pub final_frame: bool,
    pub opcode: OpcodeWs,
    pub payload: Vec<u8>,
}

impl FrameWs {
    pub fn teks(pesan: &str) -> Self {
        Self {
            final_frame: true,
            opcode: OpcodeWs::Teks,
            payload: pesan.as_bytes().to_vec(),
        }
    }

    pub fn biner(data: Vec<u8>) -> Self {
        Self {
            final_frame: true,
            opcode: OpcodeWs::Biner,
            payload: data,
        }
    }

    pub fn ping(payload: Vec<u8>) -> Self {
        Self {
            final_frame: true,
            opcode: OpcodeWs::Ping,
            payload,
        }
    }

    pub fn pong(payload: Vec<u8>) -> Self {
        Self {
            final_frame: true,
            opcode: OpcodeWs::Pong,
            payload,
        }
    }

    pub fn tutup() -> Self {
        Self {
            final_frame: true,
            opcode: OpcodeWs::Tutup,
            payload: Vec::new(),
        }
    }

    /// Mengodekan frame menjadi paket byte siap kirim ke soket jaringan.
    pub fn ke_bytes(&self) -> Vec<u8> {
        let mut out = Vec::new();

        let byte0 = if self.final_frame { 0x80 } else { 0x00 } | (self.opcode as u8 & 0x0F);
        out.push(byte0);

        let p_len = self.payload.len();
        if p_len < 126 {
            out.push(p_len as u8); // Mask bit 0 untuk server -> client
        } else if p_len <= 65535 {
            out.push(126);
            out.extend_from_slice(&(p_len as u16).to_be_bytes());
        } else {
            out.push(127);
            out.extend_from_slice(&(p_len as u64).to_be_bytes());
        }

        out.extend_from_slice(&self.payload);
        out
    }

    /// Mendekodekan frame dari aliran byte yang diterima.
    pub fn dari_bytes(bytes: &[u8]) -> Result<(Self, usize), String> {
        if bytes.len() < 2 {
            return Err("Ukuran frame terlalu pendek".into());
        }

        let byte0 = bytes[0];
        let final_frame = (byte0 & 0x80) != 0;
        let opcode_num = byte0 & 0x0F;

        let opcode = match opcode_num {
            0x0 => OpcodeWs::Lanjutan,
            0x1 => OpcodeWs::Teks,
            0x2 => OpcodeWs::Biner,
            0x8 => OpcodeWs::Tutup,
            0x9 => OpcodeWs::Ping,
            0xA => OpcodeWs::Pong,
            _ => return Err(format!("Opcode WebSocket tidak dikenal: 0x{:x}", opcode_num)),
        };

        let byte1 = bytes[1];
        let is_masked = (byte1 & 0x80) != 0;
        let mut len_code = (byte1 & 0x7F) as usize;
        let mut kursor = 2;

        if len_code == 126 {
            if bytes.len() < kursor + 2 {
                return Err("Frame terpotong pada pembacaan extended length 16".into());
            }
            len_code = u16::from_be_bytes([bytes[kursor], bytes[kursor + 1]]) as usize;
            kursor += 2;
        } else if len_code == 127 {
            if bytes.len() < kursor + 8 {
                return Err("Frame terpotong pada pembacaan extended length 64".into());
            }
            len_code = u64::from_be_bytes([
                bytes[kursor],
                bytes[kursor + 1],
                bytes[kursor + 2],
                bytes[kursor + 3],
                bytes[kursor + 4],
                bytes[kursor + 5],
                bytes[kursor + 6],
                bytes[kursor + 7],
            ]) as usize;
            kursor += 8;
        }

        let mask_key = if is_masked {
            if bytes.len() < kursor + 4 {
                return Err("Frame terpotong pada mask key".into());
            }
            let key = [
                bytes[kursor],
                bytes[kursor + 1],
                bytes[kursor + 2],
                bytes[kursor + 3],
            ];
            kursor += 4;
            Some(key)
        } else {
            None
        };

        if bytes.len() < kursor + len_code {
            return Err("Frame terpotong pada payload data".into());
        }

        let mut payload = bytes[kursor..kursor + len_code].to_vec();
        kursor += len_code;

        if let Some(mask) = mask_key {
            for (i, b) in payload.iter_mut().enumerate() {
                *b ^= mask[i % 4];
            }
        }

        Ok((
            Self {
                final_frame,
                opcode,
                payload,
            },
            kursor,
        ))
    }
}

/// Menghasilkan respons jabat tangan (Handshake Upgrade) HTTP 101 Switching Protocols.
pub fn buat_respons_handshake_ws(client_key: &str) -> ResponsHttp {
    let accept_key = hitung_accept_key(client_key);
    ResponsHttp::baru(KodeStatus(101))
        .dengan_header("Upgrade", "websocket")
        .dengan_header("Connection", "Upgrade")
        .dengan_header("Sec-WebSocket-Accept", &accept_key)
}

/// Menghitung nilai header `Sec-WebSocket-Accept` via SHA-1 dan Base64.
pub fn hitung_accept_key(client_key: &str) -> String {
    let gabungan = format!("{}{}", client_key.trim(), WS_GUID);
    let digest = sha1_bytes(gabungan.as_bytes());
    base64_encode(&digest)
}

// -----------------------------------------------------------------------------
// Implementasi Mandiri SHA-1 (FIPS 180-1) dan Base64 murni tanpa dependensi
// -----------------------------------------------------------------------------

fn sha1_bytes(pesan: &[u8]) -> [u8; 20] {
    let mut h0 = 0x67452301u32;
    let mut h1 = 0xEFCDAB89u32;
    let mut h2 = 0x98BADCFEu32;
    let mut h3 = 0x10325476u32;
    let mut h4 = 0xC3D2E1F0u32;

    let ml = (pesan.len() as u64) * 8;
    let mut buffer = pesan.to_vec();
    buffer.push(0x80);

    while (buffer.len() % 64) != 56 {
        buffer.push(0);
    }
    buffer.extend_from_slice(&ml.to_be_bytes());

    for chunk in buffer.chunks_exact(64) {
        let mut w = [0u32; 80];
        for i in 0..16 {
            w[i] = u32::from_be_bytes([
                chunk[i * 4],
                chunk[i * 4 + 1],
                chunk[i * 4 + 2],
                chunk[i * 4 + 3],
            ]);
        }
        for i in 16..80 {
            w[i] = (w[i - 3] ^ w[i - 8] ^ w[i - 14] ^ w[i - 16]).rotate_left(1);
        }

        let mut a = h0;
        let mut b = h1;
        let mut c = h2;
        let mut d = h3;
        let mut e = h4;

        for i in 0..80 {
            let (f, k) = match i {
                0..=19 => ((b & c) | ((!b) & d), 0x5A827999u32),
                20..=39 => (b ^ c ^ d, 0x6ED9EBA1u32),
                40..=59 => ((b & c) | (b & d) | (c & d), 0x8F1BBCDCu32),
                _ => (b ^ c ^ d, 0xCA62C1D6u32),
            };

            let temp = a
                .rotate_left(5)
                .wrapping_add(f)
                .wrapping_add(e)
                .wrapping_add(k)
                .wrapping_add(w[i]);
            e = d;
            d = c;
            c = b.rotate_left(30);
            b = a;
            a = temp;
        }

        h0 = h0.wrapping_add(a);
        h1 = h1.wrapping_add(b);
        h2 = h2.wrapping_add(c);
        h3 = h3.wrapping_add(d);
        h4 = h4.wrapping_add(e);
    }

    let mut out = [0u8; 20];
    out[0..4].copy_from_slice(&h0.to_be_bytes());
    out[4..8].copy_from_slice(&h1.to_be_bytes());
    out[8..12].copy_from_slice(&h2.to_be_bytes());
    out[12..16].copy_from_slice(&h3.to_be_bytes());
    out[16..20].copy_from_slice(&h4.to_be_bytes());
    out
}

const BASE64_CHARS: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";

fn base64_encode(input: &[u8]) -> String {
    let mut out = String::new();
    let mut i = 0;

    while i < input.len() {
        let b0 = input[i];
        let b1 = if i + 1 < input.len() { input[i + 1] } else { 0 };
        let b2 = if i + 2 < input.len() { input[i + 2] } else { 0 };

        let idx0 = (b0 >> 2) as usize;
        let idx1 = (((b0 & 0x03) << 4) | (b1 >> 4)) as usize;
        let idx2 = (((b1 & 0x0F) << 2) | (b2 >> 6)) as usize;
        let idx3 = (b2 & 0x3F) as usize;

        out.push(BASE64_CHARS[idx0] as char);
        out.push(BASE64_CHARS[idx1] as char);

        if i + 1 < input.len() {
            out.push(BASE64_CHARS[idx2] as char);
        } else {
            out.push('=');
        }

        if i + 2 < input.len() {
            out.push(BASE64_CHARS[idx3] as char);
        } else {
            out.push('=');
        }

        i += 3;
    }

    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tes_handshake_accept_key() {
        // Vektor uji RFC 6455
        let client_key = "dGhlIHNhbXBsZSBub25jZQ==";
        let accept = hitung_accept_key(client_key);
        assert_eq!(accept, "s3pPLMBiTxaQ9kYGzzhZRbK+xOo=");
    }

    #[test]
    fn tes_encoding_dan_decoding_frame_ws() {
        let frame = FrameWs::teks("Halo Komo WebSocket!");
        let bytes = frame.ke_bytes();

        let (frame_hasil, dibaca) = FrameWs::dari_bytes(&bytes).unwrap();
        assert_eq!(dibaca, bytes.len());
        assert_eq!(frame_hasil.opcode, OpcodeWs::Teks);
        assert_eq!(
            std::str::from_utf8(&frame_hasil.payload).unwrap(),
            "Halo Komo WebSocket!"
        );
    }
}
