<!-- Copyright (c) kangpcode (Dhafa Nazula Permadi). Licensed under MIT or Apache-2.0. -->
<script setup>
import { ref } from 'vue';
import { useKomolang } from './komolang_bridge.js';

const { siap, mode, hitung_deret_fibonacci, cek_apakah_prima } = useKomolang();

// State interaktif Fibonacci
const inputFibo = ref(40);
const hasilFibo = ref(null);
const waktuFibo = ref(null);

// State interaktif Bilangan Prima
const inputPrima = ref(104729);
const hasilPrima = ref(null);
const waktuPrima = ref(null);

function jalankanFibonacci() {
  const n = parseInt(inputFibo.value, 10);
  if (isNaN(n) || n < 0) return;
  const mulai = performance.now();
  hasilFibo.value = hitung_deret_fibonacci(n);
  const selesai = performance.now();
  waktuFibo.value = (selesai - mulai).toFixed(3);
}

function jalankanCekPrima() {
  const n = parseInt(inputPrima.value, 10);
  if (isNaN(n) || n < 1) return;
  const mulai = performance.now();
  hasilPrima.value = cek_apakah_prima(n);
  const selesai = performance.now();
  waktuPrima.value = (selesai - mulai).toFixed(3);
}
</script>

<template>
  <div class="komo-app-container">
    <header class="app-header">
      <div class="header-badge">Komolang + Vue 3 Interop</div>
      <h1 class="header-title">Akselerasi WebAssembly Komolang</h1>
      <p class="header-subtitle">
        Integrasi komputasi intensif bahasa Komolang langsung ke antarmuka reaktif Vue 3 Composition API.
      </p>
      <div class="status-chip" :class="{ 'status-active': siap }">
        <span class="status-dot"></span>
        <span id="status-mesin">{{ siap ? mode : 'Menginisialisasi Engine Komolang...' }}</span>
      </div>
    </header>

    <main class="grid-cards">
      <!-- Kartu 1: Deret Fibonacci -->
      <section class="card glass-card" id="card-fibonacci">
        <div class="card-icon">⚡</div>
        <h2 class="card-title">Deret Fibonacci Komolang</h2>
        <p class="card-desc">
          Eksekusi iterasi berkecepatan native di Komolang dengan kompleksitas O(n) tanpa overhead JS stack frame.
        </p>

        <div class="form-group">
          <label for="input-fibo" class="input-label">Urutan Fibonacci (n):</label>
          <input
            id="input-fibo"
            type="number"
            v-model="inputFibo"
            min="0"
            max="92"
            class="input-control"
          />
        </div>

        <button
          id="btn-hitung-fibo"
          class="btn-primary"
          :disabled="!siap"
          @click="jalankanFibonacci"
        >
          Hitung Deret Fibonacci
        </button>

        <div v-if="hasilFibo !== null" class="result-box" id="hasil-fibo-container">
          <div class="result-label">Hasil Perhitungan:</div>
          <div class="result-value" id="hasil-fibo-nilai">{{ hasilFibo }}</div>
          <div class="result-metric" id="hasil-fibo-waktu">
            ⏱ Waktu Eksekusi: <strong>{{ waktuFibo }} ms</strong>
          </div>
        </div>
      </section>

      <!-- Kartu 2: Pengujian Bilangan Prima -->
      <section class="card glass-card" id="card-prima">
        <div class="card-icon">🛡️</div>
        <h2 class="card-title">Detektor Bilangan Prima</h2>
        <p class="card-desc">
          Verifikasi matematis keprimaan bilangan 64-bit menggunakan algoritma wheel factorization 6k ± 1 di Komolang.
        </p>

        <div class="form-group">
          <label for="input-prima" class="input-label">Angka yang Diuji:</label>
          <input
            id="input-prima"
            type="number"
            v-model="inputPrima"
            min="1"
            class="input-control"
          />
        </div>

        <button
          id="btn-cek-prima"
          class="btn-primary"
          :disabled="!siap"
          @click="jalankanCekPrima"
        >
          Verifikasi Keprimaan
        </button>

        <div v-if="hasilPrima !== null" class="result-box" id="hasil-prima-container">
          <div class="result-label">Status Keprimaan:</div>
          <div
            class="result-badge"
            :class="hasilPrima ? 'badge-prima' : 'badge-komposit'"
            id="hasil-prima-status"
          >
            {{ hasilPrima ? '✓ Bilangan Prima' : '✗ Bilangan Komposit' }}
          </div>
          <div class="result-metric" id="hasil-prima-waktu">
            ⏱ Waktu Eksekusi: <strong>{{ waktuPrima }} ms</strong>
          </div>
        </div>
      </section>
    </main>

    <footer class="app-footer">
      <p>Komolang UI & WebAssembly Interop Framework v0.1.0 • Lisensi MIT/Apache-2.0</p>
    </footer>
  </div>
</template>

<style scoped>
@import url('https://fonts.googleapis.com/css2?family=Outfit:wght@300;400;600;700&family=JetBrains+Mono:wght@400;600&display=swap');

.komo-app-container {
  font-family: 'Outfit', sans-serif;
  color: #e2e8f0;
  background: radial-gradient(circle at 50% 10%, #1e1b4b 0%, #0f172a 60%, #020617 100%);
  min-height: 100vh;
  padding: 3rem 1.5rem;
  box-sizing: border-box;
  display: flex;
  flex-direction: column;
  align-items: center;
}

.app-header {
  text-align: center;
  max-width: 680px;
  margin-bottom: 3.5rem;
}

.header-badge {
  display: inline-block;
  padding: 0.35rem 0.85rem;
  background: rgba(99, 102, 241, 0.18);
  color: #818cf8;
  border: 1px solid rgba(99, 102, 241, 0.35);
  border-radius: 9999px;
  font-size: 0.85rem;
  font-weight: 600;
  letter-spacing: 0.05em;
  text-transform: uppercase;
  margin-bottom: 1.25rem;
}

.header-title {
  font-size: 2.75rem;
  font-weight: 700;
  line-height: 1.15;
  margin: 0 0 1rem;
  background: linear-gradient(135deg, #ffffff 30%, #a5b4fc 100%);
  -webkit-background-clip: text;
  -webkit-text-fill-color: transparent;
}

.header-subtitle {
  font-size: 1.125rem;
  color: #94a3b8;
  line-height: 1.6;
  margin: 0 0 1.5rem;
}

.status-chip {
  display: inline-flex;
  align-items: center;
  gap: 0.5rem;
  padding: 0.4rem 1rem;
  border-radius: 9999px;
  font-size: 0.875rem;
  background: rgba(15, 23, 42, 0.7);
  border: 1px solid #334155;
  color: #94a3b8;
  transition: all 0.3s ease;
}

.status-chip.status-active {
  border-color: rgba(52, 211, 153, 0.4);
  color: #6ee7b7;
  background: rgba(6, 78, 59, 0.25);
}

.status-dot {
  width: 8px;
  height: 8px;
  border-radius: 50%;
  background: #f59e0b;
}

.status-active .status-dot {
  background: #10b981;
  box-shadow: 0 0 10px #10b981;
}

.grid-cards {
  display: grid;
  grid-template-columns: repeat(auto-fit, minmax(320px, 1fr));
  gap: 2rem;
  width: 100%;
  max-width: 900px;
}

.glass-card {
  background: rgba(30, 41, 59, 0.65);
  backdrop-filter: blur(16px);
  -webkit-backdrop-filter: blur(16px);
  border: 1px solid rgba(255, 255, 255, 0.08);
  border-radius: 1.25rem;
  padding: 2rem;
  box-shadow: 0 20px 40px -15px rgba(0, 0, 0, 0.5);
  transition: transform 0.25s ease, box-shadow 0.25s ease, border-color 0.25s ease;
}

.glass-card:hover {
  transform: translateY(-4px);
  border-color: rgba(129, 140, 248, 0.4);
  box-shadow: 0 25px 50px -12px rgba(99, 102, 241, 0.25);
}

.card-icon {
  font-size: 2rem;
  margin-bottom: 1rem;
}

.card-title {
  font-size: 1.5rem;
  font-weight: 600;
  margin: 0 0 0.5rem;
  color: #f8fafc;
}

.card-desc {
  font-size: 0.95rem;
  color: #94a3b8;
  line-height: 1.5;
  margin: 0 0 1.5rem;
}

.form-group {
  margin-bottom: 1.25rem;
}

.input-label {
  display: block;
  font-size: 0.875rem;
  font-weight: 500;
  color: #cbd5e1;
  margin-bottom: 0.5rem;
}

.input-control {
  width: 100%;
  padding: 0.75rem 1rem;
  border-radius: 0.75rem;
  background: rgba(15, 23, 42, 0.85);
  border: 1px solid #334155;
  color: #f1f5f9;
  font-family: 'JetBrains Mono', monospace;
  font-size: 1.1rem;
  box-sizing: border-box;
  transition: border-color 0.2s ease, box-shadow 0.2s ease;
}

.input-control:focus {
  outline: none;
  border-color: #6366f1;
  box-shadow: 0 0 0 3px rgba(99, 102, 241, 0.3);
}

.btn-primary {
  width: 100%;
  padding: 0.875rem 1.25rem;
  border-radius: 0.75rem;
  border: none;
  background: linear-gradient(135deg, #4f46e5 0%, #7c3aed 100%);
  color: #ffffff;
  font-family: inherit;
  font-size: 1rem;
  font-weight: 600;
  cursor: pointer;
  transition: all 0.2s ease;
  box-shadow: 0 4px 14px rgba(79, 70, 229, 0.35);
}

.btn-primary:hover:not(:disabled) {
  background: linear-gradient(135deg, #4338ca 0%, #6d28d9 100%);
  transform: translateY(-1px);
  box-shadow: 0 6px 20px rgba(79, 70, 229, 0.5);
}

.btn-primary:disabled {
  opacity: 0.5;
  cursor: not-allowed;
}

.result-box {
  margin-top: 1.5rem;
  padding: 1.25rem;
  border-radius: 0.875rem;
  background: rgba(15, 23, 42, 0.9);
  border: 1px solid #334155;
}

.result-label {
  font-size: 0.8rem;
  color: #94a3b8;
  text-transform: uppercase;
  letter-spacing: 0.05em;
  margin-bottom: 0.35rem;
}

.result-value {
  font-family: 'JetBrains Mono', monospace;
  font-size: 1.35rem;
  font-weight: 600;
  color: #a5b4fc;
  word-break: break-all;
}

.result-badge {
  display: inline-block;
  padding: 0.35rem 0.85rem;
  border-radius: 0.5rem;
  font-size: 1rem;
  font-weight: 600;
}

.badge-prima {
  background: rgba(16, 185, 129, 0.2);
  color: #34d399;
  border: 1px solid rgba(52, 211, 153, 0.35);
}

.badge-komposit {
  background: rgba(239, 68, 68, 0.2);
  color: #f87171;
  border: 1px solid rgba(248, 113, 113, 0.35);
}

.result-metric {
  font-size: 0.875rem;
  color: #cbd5e1;
  margin-top: 0.75rem;
  padding-top: 0.5rem;
  border-top: 1px solid rgba(255, 255, 255, 0.08);
}

.app-footer {
  margin-top: 4rem;
  color: #64748b;
  font-size: 0.875rem;
  text-align: center;
}
</style>
