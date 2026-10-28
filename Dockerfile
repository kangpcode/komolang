# Copyright (c) kangpcode (Dhafa Nazula Permadi). Licensed under MIT or Apache-2.0.
# Multi-stage Dockerfile untuk build dan runtime container Komolang.

# Stage 1: Pembangun Biner (Builder)
FROM rust:1.80-slim-bullseye AS builder

WORKDIR /usr/src/komolang

# Instal dependensi pembangunan
RUN apt-get update && apt-get install -y --no-install-recommends \
    build-essential \
    pkg-config \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*

COPY . .

# Bangun seluruh paket dalam mode rilis penuh
RUN cargo build --release -p komo

# Stage 2: Runtime Minimal Container
FROM debian:bullseye-slim

LABEL maintainer="Dhafa Nazula Permadi <kangpcode@gmail.com>"
LABEL org.opencontainers.image.title="Komolang"
LABEL org.opencontainers.image.description="Bahasa pemrograman kompilasi native berkecepatan tinggi asli Indonesia"

RUN apt-get update && apt-get install -y --no-install-recommends \
    ca-certificates \
    curl \
    && rm -rf /var/lib/apt/lists/*

COPY --from=builder /usr/src/komolang/target/release/komo /usr/local/bin/komo

RUN chmod +x /usr/local/bin/komo

# Direktori kerja pengguna
WORKDIR /workspace

# Verifikasi kompilator
RUN komo versi

ENTRYPOINT ["komo"]
CMD ["bantuan"]
