# Kebijakan Kompatibilitas Bahasa & Proses RFC Komolang

Dokumen ini mendefinisikan komitmen stabilitas resmi bahasa pemrograman **Komolang**, aturan versi semantik (SemVer), serta tata kelola evolusi bahasa melalui proses **RFC (Request for Comments)**.

---

## 1. Komitmen SemVer 2.0

Komolang mematuhi spesifikasi [Semantic Versioning 2.0.0](https://semver.org/lang/id/):

Format versi: **`MAJOR.MINOR.PATCH`**

1. **`PATCH` (0.1.X)**:
   - Perbaikan bug, peningkatan performa compiler, optimasi memori, atau pembaruan dokumentasi.
   - Menjamin 100% kompatibilitas mundur (*backward compatible*) tanpa perubahan perilaku API publik.

2. **`MINOR` (0.X.0)**:
   - Penambahan kata kunci baru (yang tidak menimbulkan ambiguitas), fungsi pustaka standar baru, fitur CLI baru, atau optimasi codegen baru.
   - Seluruh kode yang valid pada versi minor sebelumnya dijamin tetap dapat dikompilasi tanpa galat (*zero breaking changes*).

3. **`MAJOR` (X.0.0)**:
   - Perubahan drastis (*breaking changes*) pada tata bahasa AST, sistem tipe dasar, atau penghapusan modul pustaka standar yang telah didepresiasi.
   - Rilis Major wajib menyertakan panduan migrasi otomatis (*code migration tool* `komo migrasi`).

---

## 2. Siklus Depresiasi (*Deprecation Policy*)

Sebelum sebuah API atau fitur dihapus:
1. Fitur diberi anotasi peringatan (*deprecation warning*) minimal selama **2 rilis Minor berturut-turut**.
2. Kompilator memberikan pesan peringatan bersahabat beserta alternatif pengganti yang disarankan.
3. Fitur hanya boleh dihapus total pada lompatan versi **Major berikutnya**.

---

## 3. Proses RFC (Request for Comments)

Setiap usulan fitur bahasa baru, sintaks baru, atau perubahan besar pada arsitektur compiler wajib melalui proses RFC:

```
[ Ide / Diskusi ] ──> [ Draf RFC ] ──> [ Review Komunitas & Tim Inti ] ──> [ Diterima (Accepted) ] ──> [ Implementasi ]
```

### Tahapan RFC:
1. **Fork & Buat Cabang**: Salin draf usulan dari `docs/rfc/0000-template-rfc.md`.
2. **Klarifikasi Motivasi**: Jelaskan masalah nyata yang diselesaikan dan alternatif desain yang telah dipertimbangkan.
3. **Konsensus Publik**: Pembahasan terbuka di repositori Komolang.
4. **Keputusan Tim Arsitektur**: Pengesahan status (*Accepted*, *Postponed*, atau *Rejected*).
