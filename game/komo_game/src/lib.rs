// Copyright (c) kangpcode (Dhafa Nazula Permadi). Licensed under MIT or Apache-2.0.

//! Pustaka Engine Game dan Multimedia Mandiri untuk Komolang (`komo_game`).
//!
//! Mencakup Entity Component System (ECS), loop deterministik 60 FPS, profiler latensi frame,
//! penanganan input keyboard/mouse terpadu, synthesizer efek suara (SFX) retro dengan ekspor WAV,
//! simulasi fisika 2D (AABB collision & impulse resolution), 2D Sprite/Tilemap/Particle renderer,
//! serta lapisan abstraksi renderer 3D dengan z-buffer software.

pub mod audio;
pub mod demo_game;
pub mod ecs;
pub mod fisika2d;
pub mod input;
pub mod loop_game;
pub mod renderer2d;
pub mod renderer3d;

// Re-ekspor komponen penting untuk kemudahan penggunaan
pub use audio::{AmplopAdsr, BentukGelombang, KlipAudio, MixerAudio, Synthesizer};
pub use demo_game::{DemoGame2D, DemoGame3D};
pub use ecs::{DuniaEcs, Entitas};
pub use fisika2d::{BadanKaku2D, Kecepatan2D, KotakTabrakan2D, Transformasi2D};
pub use input::{KodeTombol, ManajerInput, TombolMouse};
pub use loop_game::{MesinGame, ProfilerFrame, SiklusGame, StatistikFrame, DELTA_DETIK_60_FPS};
pub use renderer2d::{EmitterPartikel, Kamera2D, Partikel, Renderer2D, Sprite2D, Tilemap2D};
pub use renderer3d::{Matriks4, Mesh3D, Renderer3D, Vektor3, Vertex3D, ZBuffer};
