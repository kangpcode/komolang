#!/usr/bin/env bash
# Copyright (c) kangpcode (Dhafa Nazula Permadi). Licensed under MIT or Apache-2.0.
# Skrip instalasi resmi Komolang untuk Linux dan macOS.

set -e

KOMO_VERSI="0.1.0"
KOMO_DIR="${HOME}/.komo"
KOMO_BIN="${KOMO_DIR}/bin"

echo "🦎 Memulai instalasi Komolang (Komodo Language) v${KOMO_VERSI}..."

OS="$(uname -s | tr '[:upper:]' '[:lower:]')"
ARCH="$(uname -m)"

case "${OS}" in
    linux)
        case "${ARCH}" in
            x86_64) TARGET="x86_64-unknown-linux-gnu" ;;
            aarch64|arm64) TARGET="aarch64-unknown-linux-gnu" ;;
            *) echo "Arsitektur tidak didukung: ${ARCH}"; exit 1 ;;
        esac
        ;;
    darwin)
        case "${ARCH}" in
            x86_64) TARGET="x86_64-apple-darwin" ;;
            arm64) TARGET="aarch64-apple-darwin" ;;
            *) echo "Arsitektur tidak didukung: ${ARCH}"; exit 1 ;;
        esac
        ;;
    *)
        echo "Sistem operasi tidak didukung oleh install.sh: ${OS}. Gunakan install.ps1 untuk Windows."
        exit 1
        ;;
esac

echo "📦 Target platform terdeteksi: ${TARGET}"
mkdir -p "${KOMO_BIN}"

TARBALL="komolang-v${KOMO_VERSI}-${TARGET}.tar.gz"
URL="https://github.com/kangpcode/komolang/releases/download/v${KOMO_VERSI}/${TARBALL}"

echo "⬇️  Mengunduh rilis resmi Komolang..."
# Jika biner lokal target/release/komo sudah ada di build mesin saat ini, salin langsung
if [ -f "target/release/komo" ]; then
    cp "target/release/komo" "${KOMO_BIN}/komo"
    chmod +x "${KOMO_BIN}/komo"
    echo "⚡ Menggunakan biner lokal yang telah dibangun."
elif command -v curl >/dev/null 2>&1; then
    curl -fsSL "${URL}" -o "/tmp/${TARBALL}" || true
    if [ -f "/tmp/${TARBALL}" ]; then
        tar -xzf "/tmp/${TARBALL}" -C "${KOMO_BIN}"
        chmod +x "${KOMO_BIN}/komo"
    fi
fi

# Konfigurasi PATH di file profil shell jika belum ada
SHELL_RC="${HOME}/.bashrc"
if [ -f "${HOME}/.zshrc" ]; then
    SHELL_RC="${HOME}/.zshrc"
fi

if ! grep -q "KOMO_DIR" "${SHELL_RC}" 2>/dev/null; then
    echo "" >> "${SHELL_RC}"
    echo "# Komolang Environment Path" >> "${SHELL_RC}"
    echo "export PATH=\"${KOMO_BIN}:\$PATH\"" >> "${SHELL_RC}"
    echo "export KOMO_HOME=\"${KOMO_DIR}\"" >> "${SHELL_RC}"
    echo "✨ Variabel PATH telah ditambahkan ke ${SHELL_RC}"
fi

echo "✅ Instalasi Komolang selesai!"
echo "🚀 Jalankan: 'source ${SHELL_RC}' lalu ketik 'komo bantuan'"
