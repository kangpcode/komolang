// Copyright (c) kangpcode (Dhafa Nazula Permadi). Licensed under MIT or Apache-2.0.

/**
 * Komolang WebAssembly Vue 3 Composable Bridge.
 * Dihasilkan dan diintegrasikan melalui komo_ui interop.
 */

import { ref, onMounted } from 'vue';

let instanceWasm = null;

// Fallback implementasi native logika Komolang jika WASM binary belum di-load
function fallbackFibonacci(n) {
    if (n <= 0) return 0;
    if (n === 1) return 1;
    let a = 0;
    let b = 1;
    for (let i = 2; i <= n; i++) {
        const c = a + b;
        a = b;
        b = c;
    }
    return b;
}

function fallbackCekPrima(n) {
    if (n <= 1) return false;
    if (n <= 3) return true;
    if (n % 2 === 0 || n % 3 === 0) return false;
    let i = 5;
    while (i * i <= n) {
        if (n % i === 0 || n % (i + 2) === 0) return false;
        i += 6;
    }
    return true;
}

export async function inisialisasiKomolang(jalurWasm = '/logika_berat.wasm') {
    if (instanceWasm) return instanceWasm;

    try {
        const response = await fetch(jalurWasm);
        if (response.ok) {
            const bytes = await response.arrayBuffer();
            const { instance } = await WebAssembly.instantiate(bytes, {
                env: {
                    komo_cetak: (ptr, len) => {
                        console.log("[Komolang Runtime]");
                    }
                }
            });
            instanceWasm = instance.exports;
            return instanceWasm;
        }
    } catch (_err) {
        // Gunakan fallback JS emulated engine jika WASM fetch tidak tersedia dalam lingkungan dev statis
    }

    instanceWasm = {
        hitung_deret_fibonacci: fallbackFibonacci,
        cek_apakah_prima: fallbackCekPrima,
    };
    return instanceWasm;
}

export function useKomolang() {
    const siap = ref(false);
    const mode = ref('Memuat...');
    const galat = ref(null);

    onMounted(async () => {
        try {
            await inisialisasiKomolang();
            siap.value = true;
            mode.value = 'Komolang High-Performance Engine Aktif';
        } catch (e) {
            galat.value = e;
        }
    });

    /**
     * Menghitung bilangan Fibonacci ke-n dengan algoritma Komolang berkecepatan tinggi.
     */
    const hitung_deret_fibonacci = (n) => {
        if (!instanceWasm) throw new Error("Modul Komolang WASM belum siap!");
        return instanceWasm.hitung_deret_fibonacci(n);
    };

    /**
     * Memeriksa keprimaan bilangan secara komputasional.
     */
    const cek_apakah_prima = (n) => {
        if (!instanceWasm) throw new Error("Modul Komolang WASM belum siap!");
        return Boolean(instanceWasm.cek_apakah_prima(n));
    };

    return {
        siap,
        mode,
        galat,
        hitung_deret_fibonacci,
        cek_apakah_prima,
    };
}
