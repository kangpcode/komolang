// Copyright (c) kangpcode (Dhafa Nazula Permadi). Licensed under MIT or Apache-2.0.

//! `komo_pkg` adalah sistem manajemen paket dan registry resmi untuk bahasa pemrograman Komolang.
//! Menyediakan pengurai manifes `komo.toml`, format berkas kunci `komo.kunci`,
//! pemecah resolusi dependensi SemVer deterministik, manajer cache lokal dengan verifikasi
//! checksum SHA-256, klien registry paket `komopaket`, dan dukungan ruang kerja monorepo.

pub mod cache;
pub mod kunci;
pub mod manifes;
pub mod registry;
pub mod resolusi;
pub mod ruang_kerja;
pub mod semver;
pub mod sha256;

pub use cache::{ekstrak_arsip_komopaket, kemas_arsip_komopaket, ManajerCache};
pub use kunci::{BerkasKunci, PaketTerkunci};
pub use manifes::{InfoPaket, KonfigurasiRuangKerja, ManifesPaket, SumberDependensi};
pub use registry::{
    terbitkan_paket_ke_registry, HasilCariPaket, InfoPenerbitan, RegistryMock,
};
pub use resolusi::{HasilResolusi, IndeksPaket, InfoRilis, PenyelesaiDependensi};
pub use ruang_kerja::{AnggotaRuangKerja, RuangKerja};
pub use semver::{KebutuhanVersi, Versi};
pub use sha256::{sha256_bytes, sha256_hex};
