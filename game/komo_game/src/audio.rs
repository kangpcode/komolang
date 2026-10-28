// Copyright (c) kangpcode (Dhafa Nazula Permadi). Licensed under MIT or Apache-2.0.

//! Sistem audio prosedural, generator efek suara (SFX) retro, dan synthesizer WAV PCM.

use std::f32::consts::PI;
use std::fs;
use std::path::Path;

/// Bentuk gelombang osilator audio dasar.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BentukGelombang {
    Sinus,
    Kotak,
    Segitiga,
    Gerigi,
    Kebisingan, // White noise
}

/// Pembungkus kurva amplop ADSR (Attack, Decay, Sustain, Release) untuk modulasi dinamika suara.
#[derive(Debug, Clone, Copy)]
pub struct AmplopAdsr {
    pub durasi_attack: f32,  // detik
    pub durasi_decay: f32,   // detik
    pub tingkat_sustain: f32, // 0.0 - 1.0
    pub durasi_release: f32, // detik
}

impl AmplopAdsr {
    pub fn baru(attack: f32, decay: f32, sustain: f32, release: f32) -> Self {
        Self {
            durasi_attack: attack,
            durasi_decay: decay,
            tingkat_sustain: sustain.clamp(0.0, 1.0),
            durasi_release: release,
        }
    }

    /// Menghitung faktor amplifikasi (0.0 sampai 1.0) pada waktu tertentu dari durasi total nada.
    pub fn evaluasi(&self, waktu: f32, total_durasi: f32) -> f32 {
        if waktu < 0.0 || waktu > total_durasi {
            return 0.0;
        }

        // Tahap Attack
        if waktu < self.durasi_attack {
            if self.durasi_attack <= 0.0 {
                return 1.0;
            }
            return waktu / self.durasi_attack;
        }

        // Tahap Decay
        let waktu_setelah_attack = waktu - self.durasi_attack;
        if waktu_setelah_attack < self.durasi_decay {
            if self.durasi_decay <= 0.0 {
                return self.tingkat_sustain;
            }
            let rasio = waktu_setelah_attack / self.durasi_decay;
            return 1.0 - rasio * (1.0 - self.tingkat_sustain);
        }

        // Tahap Release (dihitung dari akhir durasi)
        let sisa_waktu = total_durasi - waktu;
        if sisa_waktu < self.durasi_release {
            if self.durasi_release <= 0.0 {
                return 0.0;
            }
            return self.tingkat_sustain * (sisa_waktu / self.durasi_release);
        }

        // Tahap Sustain
        self.tingkat_sustain
    }
}

/// Buffer klip audio berisi sampel gelombang suara dalam format floating point [-1.0, 1.0].
#[derive(Debug, Clone)]
pub struct KlipAudio {
    pub frekuensi_sampel: u32,
    pub sampel: Vec<f32>,
}

impl KlipAudio {
    pub fn baru(frekuensi_sampel: u32, sampel: Vec<f32>) -> Self {
        Self {
            frekuensi_sampel,
            sampel,
        }
    }

    /// Durasi total klip dalam hitungan detik.
    pub fn durasi_detik(&self) -> f32 {
        if self.frekuensi_sampel == 0 {
            0.0
        } else {
            self.sampel.len() as f32 / self.frekuensi_sampel as f32
        }
    }

    /// Mengonversi sampel menjadi data berkas biner standar RIFF WAVE (16-bit PCM, Mono).
    pub fn ke_wav_bytes(&self) -> Vec<u8> {
        let mut bytes = Vec::new();

        let num_channels: u16 = 1;
        let bits_per_sample: u16 = 16;
        let byte_rate = self.frekuensi_sampel * num_channels as u32 * (bits_per_sample / 8) as u32;
        let block_align = num_channels * (bits_per_sample / 8);
        let data_size = (self.sampel.len() * 2) as u32;
        let riff_chunk_size = 36 + data_size;

        // RIFF Header
        bytes.extend_from_slice(b"RIFF");
        bytes.extend_from_slice(&riff_chunk_size.to_le_bytes());
        bytes.extend_from_slice(b"WAVE");

        // "fmt " Sub-chunk
        bytes.extend_from_slice(b"fmt ");
        bytes.extend_from_slice(&16u32.to_le_bytes()); // Subchunk1Size (16 for PCM)
        bytes.extend_from_slice(&1u16.to_le_bytes()); // AudioFormat (1 for PCM)
        bytes.extend_from_slice(&num_channels.to_le_bytes());
        bytes.extend_from_slice(&self.frekuensi_sampel.to_le_bytes());
        bytes.extend_from_slice(&byte_rate.to_le_bytes());
        bytes.extend_from_slice(&block_align.to_le_bytes());
        bytes.extend_from_slice(&bits_per_sample.to_le_bytes());

        // "data" Sub-chunk
        bytes.extend_from_slice(b"data");
        bytes.extend_from_slice(&data_size.to_le_bytes());

        for &s in &self.sampel {
            // Soft-clipping dan konversi ke signed 16-bit integer (-32768 s/d 32767)
            let clampled = s.clamp(-1.0, 1.0);
            let pcm = (clampled * 32767.0) as i16;
            bytes.extend_from_slice(&pcm.to_le_bytes());
        }

        bytes
    }

    /// Menyimpan klip ke berkas file format .wav.
    pub fn simpan_wav(&self, jalur: &Path) -> std::io::Result<()> {
        let bytes = self.ke_wav_bytes();
        fs::write(jalur, bytes)
    }
}

/// Synthesizer audio prosedural mandiri.
pub struct Synthesizer {
    pub frekuensi_sampel: u32,
}

impl Synthesizer {
    pub fn baru(frekuensi_sampel: u32) -> Self {
        Self { frekuensi_sampel }
    }

    /// Menghasilkan nada dengan frekuensi tertentu, bentuk gelombang, dan amplop ADSR.
    pub fn hasilkan_nada(
        &self,
        frekuensi: f32,
        durasi_detik: f32,
        bentuk: BentukGelombang,
        amplop: AmplopAdsr,
    ) -> KlipAudio {
        let total_sampel = (self.frekuensi_sampel as f32 * durasi_detik) as usize;
        let mut sampel = Vec::with_capacity(total_sampel);

        let mut rng_state: u64 = 123456789; // LCG untuk noise

        for i in 0..total_sampel {
            let t = i as f32 / self.frekuensi_sampel as f32;
            let fase = (t * frekuensi) % 1.0;

            let gelombang_mentah = match bentuk {
                BentukGelombang::Sinus => (2.0 * PI * fase).sin(),
                BentukGelombang::Kotak => if fase < 0.5 { 1.0 } else { -1.0 },
                BentukGelombang::Segitiga => 4.0 * (fase - 0.5).abs() - 1.0,
                BentukGelombang::Gerigi => 2.0 * fase - 1.0,
                BentukGelombang::Kebisingan => {
                    rng_state = rng_state.wrapping_mul(6364136223846793005).wrapping_add(1);
                    ((rng_state >> 33) as f32 / 2147483648.0) - 1.0
                }
            };

            let gain = amplop.evaluasi(t, durasi_detik);
            sampel.push(gelombang_mentah * gain);
        }

        KlipAudio::baru(self.frekuensi_sampel, sampel)
    }

    /// Efek suara lompat retro (pitch sweep ke atas).
    pub fn sfx_lompat(&self) -> KlipAudio {
        let durasi = 0.16;
        let total = (self.frekuensi_sampel as f32 * durasi) as usize;
        let mut sampel = Vec::with_capacity(total);

        for i in 0..total {
            let t = i as f32 / self.frekuensi_sampel as f32;
            let progress = t / durasi;
            let freq = 180.0 + progress * 420.0; // 180 Hz -> 600 Hz
            let fase = (t * freq) % 1.0;
            let val = if fase < 0.5 { 0.7 } else { -0.7 };
            let env = 1.0 - progress;
            sampel.push(val * env);
        }

        KlipAudio::baru(self.frekuensi_sampel, sampel)
    }

    /// Efek suara tembakan laser (pitch sweep ke bawah cepat).
    pub fn sfx_laser(&self) -> KlipAudio {
        let durasi = 0.12;
        let total = (self.frekuensi_sampel as f32 * durasi) as usize;
        let mut sampel = Vec::with_capacity(total);

        for i in 0..total {
            let t = i as f32 / self.frekuensi_sampel as f32;
            let progress = t / durasi;
            let freq = 880.0 * (1.0 - progress * 0.7); // 880 Hz -> 264 Hz
            let fase = (t * freq) % 1.0;
            let val = 2.0 * fase - 1.0; // Gerigi
            let env = 1.0 - progress;
            sampel.push(val * env);
        }

        KlipAudio::baru(self.frekuensi_sampel, sampel)
    }

    /// Efek suara ledakan (kebisingan berfrekuensi rendah yang meluruh).
    pub fn sfx_ledakan(&self) -> KlipAudio {
        let durasi = 0.35;
        let total = (self.frekuensi_sampel as f32 * durasi) as usize;
        let mut sampel = Vec::with_capacity(total);
        let mut rng: u64 = 987654321;

        for i in 0..total {
            let t = i as f32 / self.frekuensi_sampel as f32;
            let progress = t / durasi;

            rng = rng.wrapping_mul(6364136223846793005).wrapping_add(1);
            let noise = ((rng >> 33) as f32 / 2147483648.0) - 1.0;

            let env = (1.0 - progress).powf(1.8);
            sampel.push(noise * env);
        }

        KlipAudio::baru(self.frekuensi_sampel, sampel)
    }

    /// Efek suara perolehan koin / item (arpeggio cerah dua nada).
    pub fn sfx_koin(&self) -> KlipAudio {
        let durasi_nada = 0.08;
        let durasi_total = durasi_nada * 2.0;
        let total = (self.frekuensi_sampel as f32 * durasi_total) as usize;
        let mut sampel = Vec::with_capacity(total);

        let f1 = 987.77;  // B5
        let f2 = 1318.51; // E6

        for i in 0..total {
            let t = i as f32 / self.frekuensi_sampel as f32;
            let (freq, t_sub) = if t < durasi_nada {
                (f1, t)
            } else {
                (f2, t - durasi_nada)
            };

            let fase = (t * freq) % 1.0;
            let val = if fase < 0.5 { 0.6 } else { -0.6 };
            let env = 1.0 - (t_sub / durasi_nada);
            sampel.push(val * env);
        }

        KlipAudio::baru(self.frekuensi_sampel, sampel)
    }
}

impl Default for Synthesizer {
    fn default() -> Self {
        Self::baru(44100)
    }
}

/// Mixer audio multi-saluran untuk menggabungkan suara secara bersamaan.
pub struct MixerAudio {
    pub klip_aktif: Vec<(KlipAudio, usize)>, // (klip, posisi_sampel)
    pub volume_master: f32,
}

impl MixerAudio {
    pub fn baru() -> Self {
        Self {
            klip_aktif: Vec::new(),
            volume_master: 1.0,
        }
    }

    /// Memulai pemutaran klip audio di saluran baru.
    pub fn putar(&mut self, klip: KlipAudio) {
        self.klip_aktif.push((klip, 0));
    }

    /// Menghasilkan blok sampel berikutnya sebanyak `jumlah_sampel`.
    pub fn sintesis_blok(&mut self, jumlah_sampel: usize) -> Vec<f32> {
        let mut output = vec![0.0f32; jumlah_sampel];

        self.klip_aktif.retain_mut(|(klip, pos)| {
            let sisa_klip = klip.sampel.len() - *pos;
            let ambil = sisa_klip.min(jumlah_sampel);

            for i in 0..ambil {
                output[i] += klip.sampel[*pos + i];
            }

            *pos += ambil;
            *pos < klip.sampel.len()
        });

        for s in output.iter_mut() {
            *s = (*s * self.volume_master).clamp(-1.0, 1.0);
        }

        output
    }
}

impl Default for MixerAudio {
    fn default() -> Self {
        Self::baru()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tes_sintesis_audio_dan_wav_riff() {
        let synth = Synthesizer::baru(44100);
        let nada = synth.hasilkan_nada(
            440.0,
            0.1,
            BentukGelombang::Sinus,
            AmplopAdsr::baru(0.01, 0.02, 0.8, 0.02),
        );

        assert_eq!(nada.frekuensi_sampel, 44100);
        assert_eq!(nada.sampel.len(), 4410);

        let wav = nada.ke_wav_bytes();
        assert!(wav.starts_with(b"RIFF"));
        assert!(&wav[8..12] == b"WAVE");
        assert!(&wav[12..16] == b"fmt ");
        assert!(&wav[36..40] == b"data");
        assert_eq!(wav.len(), 44 + (4410 * 2));
    }

    #[test]
    fn tes_sfx_retro() {
        let synth = Synthesizer::baru(22050);
        let lompat = synth.sfx_lompat();
        let koin = synth.sfx_koin();
        let ledakan = synth.sfx_ledakan();

        assert!(!lompat.sampel.is_empty());
        assert!(!koin.sampel.is_empty());
        assert!(!ledakan.sampel.is_empty());

        let mut mixer = MixerAudio::baru();
        mixer.putar(koin);
        mixer.putar(lompat);

        let blok = mixer.sintesis_blok(1024);
        assert_eq!(blok.len(), 1024);
        assert!(blok.iter().any(|&s| s != 0.0));
    }
}
