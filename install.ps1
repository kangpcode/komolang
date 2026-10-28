# Copyright (c) kangpcode (Dhafa Nazula Permadi). Licensed under MIT or Apache-2.0.
# Skrip instalasi resmi Komolang untuk Windows PowerShell.

$ErrorActionPreference = 'Stop'

$KomoVersi = "0.1.0"
$KomoDir = Join-Path $HOME ".komo"
$KomoBin = Join-Path $KomoDir "bin"

Write-Host "🦎 Memulai instalasi Komolang (Komodo Language) v$KomoVersi untuk Windows..." -ForegroundColor Cyan

if (-not (Test-Path $KomoBin)) {
    New-Item -ItemType Directory -Path $KomoBin -Force | Out-Null
}

$Target = "x86_64-pc-windows-msvc"
$ZipName = "komolang-v$KomoVersi-$Target.zip"
$Url = "https://github.com/kangpcode/komolang/releases/download/v$KomoVersi/$ZipName"

if (Test-Path "target\release\komo.exe") {
    Copy-Item "target\release\komo.exe" (Join-Path $KomoBin "komo.exe") -Force
    Write-Host "⚡ Menggunakan biner lokal yang telah dibangun." -ForegroundColor Green
} else {
    Write-Host "⬇️  Mengunduh rilis resmi dari GitHub..." -ForegroundColor Cyan
    $ZipPath = Join-Path $env:TEMP $ZipName
    Invoke-WebRequest -Uri $Url -OutFile $ZipPath
    Expand-Archive -Path $ZipPath -DestinationPath $KomoBin -Force
}

# Tambahkan ke User PATH jika belum ada
$CurrentPath = [Environment]::GetEnvironmentVariable("Path", "User")
if ($CurrentPath -notlike "*$KomoBin*") {
    [Environment]::SetEnvironmentVariable("Path", "$CurrentPath;$KomoBin", "User")
    Write-Host "✨ Direktori $KomoBin telah ditambahkan ke User PATH." -ForegroundColor Yellow
}

Write-Host "✅ Instalasi Komolang berhasil!" -ForegroundColor Green
Write-Host "🚀 Buka terminal baru dan ketik 'komo bantuan'" -ForegroundColor Cyan
