# Bab 7: Pengembangan Web & Layanan RESTful (`komoweb`)

Crate `komo_web` menghadirkan framework backend mandiri berkinerja puluhan ribu permintaan per detik dengan konsumsi memori minimal.

---

## 1. Membangun REST API Pertama

```komo
paket utama

impor komo_web::{ServerKomo, Router, ResponsHttp, MetodeHttp}

publik fungsi utama() {
    var router = Router::baru()

    // Endpoint GET statis
    router.daftarkan(MetodeHttp::GET, "/", fungsi(req) {
        kembali ResponsHttp::teks("Selamat datang di API Komolang! 🦎")
    })

    // Endpoint dengan parameter URL dinamis
    router.daftarkan(MetodeHttp::GET, "/pengguna/:id", fungsi(req) {
        biar id = req.parameter("id")
        kembali ResponsHttp::json(format!("{{\"id\": \"{}\", \"status\": \"aktif\"}}", id))
    })

    biar server = ServerKomo::baru(router).dengan_port(8080)
    server.layani()
}
```

---

## 2. Server Pengembangan dengan Hot Reload

Jalankan perintah berikut di folder proyek web Anda:

```bash
komo layani . --port=8080 --hot-reload
```

Server akan langsung memuat ulang kode secara otomatis setiap kali Anda menyimpan perubahan pada berkas `.komo` tanpa perlu memulai ulang proses secara manual!
