// Copyright (c) kangpcode (Dhafa Nazula Permadi). Licensed under MIT or Apache-2.0.

//! State reaktif (Sinyal) dan rekonsiliasi pohon UI (Virtual UI Diffing).

use std::sync::{Arc, Mutex};

use crate::komponen::ElemenUi;

/// Sinyal reaktif thread-safe yang memicu listener saat nilainya bermutasi.
#[derive(Clone)]
pub struct Sinyal<T: Clone> {
    nilai: Arc<Mutex<T>>,
    pendengar: Arc<Mutex<Vec<Arc<dyn Fn(&T) + Send + Sync>>>>,
}

impl<T: Clone + 'static> Sinyal<T> {
    pub fn baru(nilai_awal: T) -> Self {
        Self {
            nilai: Arc::new(Mutex::new(nilai_awal)),
            pendengar: Arc::new(Mutex::new(Vec::new())),
        }
    }

    /// Mengambil salinan nilai saat ini.
    pub fn ambil(&self) -> T {
        self.nilai.lock().unwrap().clone()
    }

    /// Memperbarui nilai dan memberitahu seluruh pendengar.
    pub fn setel(&self, nilai_baru: T) {
        {
            let mut val = self.nilai.lock().unwrap();
            *val = nilai_baru.clone();
        }

        let daftar = self.pendengar.lock().unwrap();
        for cb in daftar.iter() {
            cb(&nilai_baru);
        }
    }

    /// Mendaftarkan fungsi pengamat yang dipanggil setiap kali nilai berubah.
    pub fn dengarkan(&self, aksi: impl Fn(&T) + Send + Sync + 'static) {
        let mut daftar = self.pendengar.lock().unwrap();
        daftar.push(Arc::new(aksi));
    }
}

/// Jenis operasi rekonsiliasi diff pada pohon UI.
#[derive(Debug, Clone, PartialEq)]
pub enum OperasiDiff {
    Sama,
    PerbaruiTeks { jalur: Vec<usize>, teks_baru: String },
    GantiElemen { jalur: Vec<usize>, elemen_baru: ElemenUi },
}

/// Menghitung perbedaan antara pohon elemen lama dan pohon elemen baru.
pub fn bedakan_pohon(lama: &ElemenUi, baru: &ElemenUi) -> Vec<OperasiDiff> {
    let mut hasil = Vec::new();
    let mut jalur = Vec::new();
    bedakan_rekursif(lama, baru, &mut jalur, &mut hasil);
    hasil
}

fn bedakan_rekursif(
    lama: &ElemenUi,
    baru: &ElemenUi,
    jalur: &mut Vec<usize>,
    perubahan: &mut Vec<OperasiDiff>,
) {
    if lama == baru {
        return;
    }

    match (lama, baru) {
        (
            ElemenUi::Teks { isi: t_lama, .. },
            ElemenUi::Teks { isi: t_baru, .. },
        ) if t_lama != t_baru => {
            perubahan.push(OperasiDiff::PerbaruiTeks {
                jalur: jalur.clone(),
                teks_baru: t_baru.clone(),
            });
        }
        (
            ElemenUi::Wadah { anak: anak_lama, .. },
            ElemenUi::Wadah { anak: anak_baru, .. },
        ) => {
            let min_len = anak_lama.len().min(anak_baru.len());
            for i in 0..min_len {
                jalur.push(i);
                bedakan_rekursif(&anak_lama[i], &anak_baru[i], jalur, perubahan);
                jalur.pop();
            }

            if anak_lama.len() != anak_baru.len() {
                perubahan.push(OperasiDiff::GantiElemen {
                    jalur: jalur.clone(),
                    elemen_baru: baru.clone(),
                });
            }
        }
        _ => {
            perubahan.push(OperasiDiff::GantiElemen {
                jalur: jalur.clone(),
                elemen_baru: baru.clone(),
            });
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::komponen::{teks, wadah_kolom};

    #[test]
    fn tes_sinyal_reaktif() {
        let hitungan = Sinyal::baru(0);
        let log = Arc::new(Mutex::new(Vec::new()));
        let log_clone = log.clone();

        hitungan.dengarkan(move |v| {
            log_clone.lock().unwrap().push(*v);
        });

        assert_eq!(hitungan.ambil(), 0);

        hitungan.setel(5);
        hitungan.setel(12);

        assert_eq!(hitungan.ambil(), 12);
        let catatan = log.lock().unwrap().clone();
        assert_eq!(catatan, vec![5, 12]);
    }

    #[test]
    fn tes_diffing_pohon_ui() {
        let pohon_a = wadah_kolom(vec![teks("Versi A"), teks("Statik")]);
        let pohon_b = wadah_kolom(vec![teks("Versi B"), teks("Statik")]);

        let diff = bedakan_pohon(&pohon_a, &pohon_b);
        assert_eq!(diff.len(), 1);

        match &diff[0] {
            OperasiDiff::PerbaruiTeks { jalur, teks_baru } => {
                assert_eq!(jalur, &vec![0]);
                assert_eq!(teks_baru, "Versi B");
            }
            _ => panic!("Ekspektasi PerbaruiTeks"),
        }
    }
}
