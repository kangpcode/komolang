// Copyright (c) kangpcode (Dhafa Nazula Permadi). Licensed under MIT or Apache-2.0.

//! Entity Component System (ECS) berkinerja tinggi berbasis memory layout berorientasi data.

use std::any::{Any, TypeId};
use std::collections::HashMap;

/// Identifier entitas unik dengan penomoran berurutan dan generasi.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Entitas {
    pub id: u32,
    pub generasi: u32,
}

impl Entitas {
    pub const fn baru(id: u32, generasi: u32) -> Self {
        Self { id, generasi }
    }
}

/// Trait penampung tipe komponen terhapus (type-erased component storage).
trait PenyimpanKomponenTrait: Send + Sync {
    fn hapus(&mut self, entitas_id: u32);
    fn as_any(&self) -> &dyn Any;
    fn as_any_mut(&mut self) -> &mut dyn Any;
}

/// Penyimpan komponen yang mengelompokkan data komponen ke dalam tabel dense array.
struct PenyimpanKomponen<T> {
    data: HashMap<u32, T>,
}

impl<T: 'static + Send + Sync> PenyimpanKomponen<T> {
    fn baru() -> Self {
        Self {
            data: HashMap::new(),
        }
    }
}

impl<T: 'static + Send + Sync> PenyimpanKomponenTrait for PenyimpanKomponen<T> {
    fn hapus(&mut self, entitas_id: u32) {
        self.data.remove(&entitas_id);
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}

/// Dunia ECS utama yang mengelola daur hidup entitas dan pemetaan komponen.
pub struct DuniaEcs {
    penghitung_id: u32,
    generasi: HashMap<u32, u32>,
    entitas_aktif: Vec<Entitas>,
    antrean_daur_ulang_id: Vec<u32>,
    penyimpan: HashMap<TypeId, Box<dyn PenyimpanKomponenTrait>>,
}

impl DuniaEcs {
    /// Membuat dunia ECS baru yang bersih.
    pub fn baru() -> Self {
        Self {
            penghitung_id: 1,
            generasi: HashMap::new(),
            entitas_aktif: Vec::new(),
            antrean_daur_ulang_id: Vec::new(),
            penyimpan: HashMap::new(),
        }
    }

    /// Membuat (spawn) entitas baru dan mengembalikan ID uniknya.
    pub fn buat_entitas(&mut self) -> Entitas {
        let (id, gen) = if let Some(id_lama) = self.antrean_daur_ulang_id.pop() {
            let gen = self.generasi.entry(id_lama).or_insert(0);
            *gen += 1;
            (id_lama, *gen)
        } else {
            let id = self.penghitung_id;
            self.penghitung_id += 1;
            self.generasi.insert(id, 0);
            (id, 0)
        };

        let entitas = Entitas::baru(id, gen);
        self.entitas_aktif.push(entitas);
        entitas
    }

    /// Menghapus entitas beserta seluruh komponen yang terpasang padanya.
    pub fn hapus_entitas(&mut self, entitas: Entitas) -> bool {
        if let Some(pos) = self.entitas_aktif.iter().position(|&e| e == entitas) {
            self.entitas_aktif.swap_remove(pos);

            // Hapus semua data komponen pada entitas ini
            for penyimpan in self.penyimpan.values_mut() {
                penyimpan.hapus(entitas.id);
            }

            self.antrean_daur_ulang_id.push(entitas.id);
            true
        } else {
            false
        }
    }

    /// Memeriksa apakah suatu entitas masih hidup di dunia ECS.
    pub fn apakah_hidup(&self, entitas: Entitas) -> bool {
        self.entitas_aktif.contains(&entitas)
    }

    /// Memasang atau memperbarui komponen pada suatu entitas.
    pub fn pasang_komponen<T: 'static + Send + Sync>(&mut self, entitas: Entitas, komponen: T) {
        if !self.apakah_hidup(entitas) {
            return;
        }

        let type_id = TypeId::of::<T>();
        let entry = self
            .penyimpan
            .entry(type_id)
            .or_insert_with(|| Box::new(PenyimpanKomponen::<T>::baru()));

        let penyimpan = entry
            .as_any_mut()
            .downcast_mut::<PenyimpanKomponen<T>>()
            .expect("Konversi penyimpan komponen gagal");

        penyimpan.data.insert(entitas.id, komponen);
    }

    /// Mengambil referensi imutabel ke komponen entitas.
    pub fn ambil_komponen<T: 'static>(&self, entitas: Entitas) -> Option<&T> {
        if !self.apakah_hidup(entitas) {
            return None;
        }

        let type_id = TypeId::of::<T>();
        let penyimpan = self.penyimpan.get(&type_id)?;
        let data = penyimpan
            .as_any()
            .downcast_ref::<PenyimpanKomponen<T>>()?;

        data.data.get(&entitas.id)
    }

    /// Mengambil referensi mutabel ke komponen entitas.
    pub fn ambil_komponen_mut<T: 'static>(&mut self, entitas: Entitas) -> Option<&mut T> {
        if !self.apakah_hidup(entitas) {
            return None;
        }

        let type_id = TypeId::of::<T>();
        let penyimpan = self.penyimpan.get_mut(&type_id)?;
        let data = penyimpan
            .as_any_mut()
            .downcast_mut::<PenyimpanKomponen<T>>()?;

        data.data.get_mut(&entitas.id)
    }

    /// Memeriksa apakah entitas memiliki tipe komponen tertentu.
    pub fn punya_komponen<T: 'static>(&self, entitas: Entitas) -> bool {
        self.ambil_komponen::<T>(entitas).is_some()
    }

    /// Mencopot komponen tertentu dari entitas.
    pub fn copot_komponen<T: 'static>(&mut self, entitas: Entitas) -> Option<T> {
        let type_id = TypeId::of::<T>();
        let penyimpan = self.penyimpan.get_mut(&type_id)?;
        let data = penyimpan
            .as_any_mut()
            .downcast_mut::<PenyimpanKomponen<T>>()?;

        data.data.remove(&entitas.id)
    }

    /// Menjalankan kueri untuk seluruh entitas yang memiliki tipe komponen T.
    pub fn kueri_1<T: 'static>(&self) -> Vec<(Entitas, &T)> {
        let type_id = TypeId::of::<T>();
        let mut hasil = Vec::new();

        if let Some(penyimpan) = self.penyimpan.get(&type_id) {
            if let Some(data) = penyimpan.as_any().downcast_ref::<PenyimpanKomponen<T>>() {
                for &e in &self.entitas_aktif {
                    if let Some(komp) = data.data.get(&e.id) {
                        hasil.push((e, komp));
                    }
                }
            }
        }

        hasil
    }

    /// Menjalankan kueri untuk seluruh entitas yang memiliki pasangan komponen (A, B).
    pub fn kueri_2<A: 'static, B: 'static>(&self) -> Vec<(Entitas, &A, &B)> {
        let id_a = TypeId::of::<A>();
        let id_b = TypeId::of::<B>();
        let mut hasil = Vec::new();

        if let (Some(penyimpan_a), Some(penyimpan_b)) =
            (self.penyimpan.get(&id_a), self.penyimpan.get(&id_b))
        {
            if let (Some(data_a), Some(data_b)) = (
                penyimpan_a.as_any().downcast_ref::<PenyimpanKomponen<A>>(),
                penyimpan_b.as_any().downcast_ref::<PenyimpanKomponen<B>>(),
            ) {
                for &e in &self.entitas_aktif {
                    if let (Some(komp_a), Some(komp_b)) =
                        (data_a.data.get(&e.id), data_b.data.get(&e.id))
                    {
                        hasil.push((e, komp_a, komp_b));
                    }
                }
            }
        }

        hasil
    }

    /// Mengembalikan jumlah entitas aktif saat ini.
    pub fn jumlah_entitas(&self) -> usize {
        self.entitas_aktif.len()
    }

    /// Daftar seluruh entitas aktif.
    pub fn daftar_entitas(&self) -> &[Entitas] {
        &self.entitas_aktif
    }
}

impl Default for DuniaEcs {
    fn default() -> Self {
        Self::baru()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Debug, PartialEq, Clone)]
    struct Posisi {
        x: f32,
        y: f32,
    }

    #[derive(Debug, PartialEq, Clone)]
    struct Kecepatan {
        vx: f32,
        vy: f32,
    }

    #[derive(Debug, PartialEq, Clone)]
    struct Nama(String);

    #[test]
    fn tes_daur_hidup_entitas_dan_komponen() {
        let mut dunia = DuniaEcs::baru();

        let e1 = dunia.buat_entitas();
        let e2 = dunia.buat_entitas();

        dunia.pasang_komponen(e1, Posisi { x: 10.0, y: 20.0 });
        dunia.pasang_komponen(e1, Kecepatan { vx: 1.0, vy: 0.0 });
        dunia.pasang_komponen(e1, Nama("Pemain".into()));

        dunia.pasang_komponen(e2, Posisi { x: 50.0, y: 100.0 });
        dunia.pasang_komponen(e2, Nama("Musuh".into()));

        assert_eq!(dunia.jumlah_entitas(), 2);
        assert!(dunia.punya_komponen::<Kecepatan>(e1));
        assert!(!dunia.punya_komponen::<Kecepatan>(e2));

        // Mutasi komponen
        if let Some(pos) = dunia.ambil_komponen_mut::<Posisi>(e1) {
            pos.x += 5.0;
        }
        assert_eq!(dunia.ambil_komponen::<Posisi>(e1).unwrap().x, 15.0);

        // Kueri 2 komponen
        let hasil = dunia.kueri_2::<Posisi, Kecepatan>();
        assert_eq!(hasil.len(), 1);
        assert_eq!(hasil[0].0, e1);
        assert_eq!(hasil[0].1.x, 15.0);
        assert_eq!(hasil[0].2.vx, 1.0);

        // Hapus entitas
        assert!(dunia.hapus_entitas(e1));
        assert_eq!(dunia.jumlah_entitas(), 1);
        assert!(!dunia.apakah_hidup(e1));
        assert!(dunia.ambil_komponen::<Posisi>(e1).is_none());
    }
}
