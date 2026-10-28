# Panduan Integrasi Editor & IDE untuk Komolang

Komolang dilengkapi dengan server **Language Server Protocol (LSP)** bawaan mandiri yang dapat dijalankan melalui perintah:
```bash
komo lsp
```
Server ini berkomunikasi via JSON-RPC 2.0 standar melalui `stdin`/`stdout`, sehingga kompatibel dengan semua editor modern yang mendukung LSP.

---

## 1. Neovim (Bawaan `vim.lsp` / `nvim-lspconfig`)

Tambahkan konfigurasi berikut ke konfigurasi Neovim Anda (misal `init.lua` atau `lua/plugins/lsp.lua`):

```lua
-- Daftarkan jenis berkas .komo
vim.filetype.add({
  extension = {
    komo = 'komo',
  },
})

-- Konfigurasi LSP Komolang
local lspconfig = require('lspconfig')
local configs = require('lspconfig.configs')

if not configs.komo_lsp then
  configs.komo_lsp = {
    default_config = {
      cmd = { 'komo', 'lsp' },
      filetypes = { 'komo' },
      root_dir = lspconfig.util.root_pattern('komo.toml', '.git'),
      settings = {},
    },
  }
end

lspconfig.komo_lsp.setup{
  on_attach = function(client, bufnr)
    -- Format saat menyimpan berkas
    vim.api.nvim_create_autocmd("BufWritePre", {
      buffer = bufnr,
      callback = function()
        vim.lsp.buf.format({ async = false })
      end,
    })
  end,
}
```

---

## 2. Helix Editor

Tambahkan cuplikan konfigurasi berikut ke berkas konfigurasi bahasa Helix Anda (`~/.config/helix/languages.toml`):

```toml
[[language]]
name = "komo"
scope = "source.komo"
injection-regex = "komo"
file-types = ["komo"]
comment-token = "//"
block-comment-tokens = { start = "/*", end = "*/" }
indent = { tab-width = 4, unit = "    " }
language-servers = [ "komo-lsp" ]

[language-server.komo-lsp]
command = "komo"
args = ["lsp"]
```

---

## 3. Zed Editor

Tambahkan konfigurasi eksternal LSP ke `~/.config/zed/settings.json`:

```json
{
  "languages": {
    "Komolang": {
      "tab_size": 4,
      "format_on_save": "on"
    }
  },
  "lsp": {
    "komo-lsp": {
      "binary": {
        "path": "komo",
        "arguments": ["lsp"]
      }
    }
  }
}
```

---

## 4. Visual Studio Code

Gunakan ekstensi resmi di `editors/vscode/` atau konfigurasikan klien generic LSP untuk mengeksekusi:
```json
{
  "komolang.lsp.jalur": "komo",
  "editor.formatOnSave": true
}
```

---

## Fitur LSP yang Didukung

| Fitur | Deskripsi |
|---|---|
| `textDocument/didOpen` | Inisialisasi analisis dokumen aktif |
| `textDocument/didChange` | Diagnostik sintaks & semantik langsung (*real-time error squiggles*) |
| `textDocument/completion` | Pelengkapan kode otomatis untuk 30 kata kunci dan tipe bawaan |
| `textDocument/hover` | Penjelasan tipe dan dokumentasi simbol |
| `textDocument/formatting` | Pemformatan dokumen kanonik idempoten berbasis `komo format` |
