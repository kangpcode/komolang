// Copyright (c) kangpcode (Dhafa Nazula Permadi). Licensed under MIT or Apache-2.0.

//! `komo_web` adalah webserver dan framework web native berperforma tinggi untuk bahasa pemrograman Komolang.
//! Menyediakan abstraksi HTTP/1.1, router berbasis Radix Tree berparameter dan wildcard,
//! pipeline middleware berantai (CORS, Logger, Rate Limiter, Panic Recovery, Session),
//! pelayan berkas statis dengan ETag & proteksi traversal, mesin KomoTemplate dengan HTML auto-escaping,
//! dukungan protokol WebSocket RFC 6455, dan server engine multi-threaded dengan runner Hot-Reload.

pub mod http;
pub mod middleware;
pub mod router;
pub mod server;
pub mod statis;
pub mod template;
pub mod websocket;

pub use http::{KodeStatus, MetodeHttp, PermintaanHttp, ResponsHttp};
pub use middleware::{
    CorsMiddleware, Middleware, PembatasLajuMiddleware, PemulihanMiddleware, PencatatMiddleware,
    SesiMiddleware,
};
pub use router::{PenanganRute, Router};
pub use server::{HotReloadWatcher, ServerKomo};
pub use statis::PelayanBerkasStatis;
pub use template::{escape_html, KomoTemplate};
pub use websocket::{buat_respons_handshake_ws, FrameWs, OpcodeWs};
