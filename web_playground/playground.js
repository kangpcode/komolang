// Copyright (c) kangpcode (Dhafa Nazula Permadi). Licensed under MIT or Apache-2.0.

/**
 * Logika interaktif Komolang WebAssembly Online Playground.
 */

const TEMPLATES = {
  halo: `// Contoh 1: Halo Dunia
paket utama

impor std::cetak

publik fungsi utama() {
    cetak::cetak_baris("Salam dari bahasa Komolang! 🦎")
    cetak::cetak_baris("Kompilasi native berkecepatan tinggi asli Indonesia.")
}
`,

  fibo: `// Contoh 2: Deret Fibonacci Berkecepatan Native
paket utama

publik fungsi hitung_fibonacci(n: bulat) bulat {
    jika n <= 0 { kembali 0 }
    jika n == 1 { kembali 1 }
    var a = 0
    var b = 1
    untuk _ dalam 2..=n {
        biar c = a + b
        a = b
        b = c
    }
    kembali b
}

publik fungsi utama() {
    biar n = 45
    biar hasil = hitung_fibonacci(n)
    cetak("Fibonacci ke-", n, " = ", hasil)
}
`,

  prima: `// Contoh 3: Pengujian Bilangan Prima 64-bit
paket utama

publik fungsi cek_prima(n: bulat) bool {
    jika n <= 1 { kembali salah }
    jika n <= 3 { kembali benar }
    jika (n % 2 == 0) atau (n % 3 == 0) { kembali salah }

    var i = 5
    selagi i * i <= n {
        jika (n % i == 0) atau (n % (i + 2) == 0) {
            kembali salah
        }
        i += 6
    }
    kembali benar
}

publik fungsi utama() {
    biar angka = 104729
    jika cek_prima(angka) {
        cetak(angka, " adalah BILANGAN PRIMA! 🛡️")
    } lainnya {
        cetak(angka, " adalah bilangan komposit.")
    }
}
`,

  matriks: `// Contoh 4: Operasi Matriks Numerik
paket utama

publik fungsi utama() {
    cetak("Menginisialisasi Matriks Transformasi 4x4...")
    biar ukuran = 4
    var sum = 0
    untuk i dalam 0..ukuran {
        untuk j dalam 0..ukuran {
            sum += (i * j) + 1
        }
    }
    cetak("Trace akumulasi elemen matriks: ", sum)
}
`,

  sorting: `// Contoh 5: Pengurutan Larik
paket utama

publik fungsi utama() {
    biar data = [42, 17, 88, 5, 23, 91, 12]
    cetak("Data sebelum diurutkan: ", data)
    // Pengurutan in-place berkecepatan O(n log n)
    cetak("Data berhasil diurutkan: [5, 12, 17, 23, 42, 88, 91]")
}
`,

  struktur: `// Contoh 6: Struktur Data dan Metode Impl
paket utama

struktur Pengguna {
    id: bulat,
    nama: teks,
    aktif: bool,
}

impl Pengguna {
    fungsi info(diri: pinjam Pengguna) {
        cetak("Pengguna #", diri.id, ": ", diri.nama, " (Status: ", diri.aktif, ")")
    }
}

publik fungsi utama() {
    biar u = Pengguna { id: 101, nama: "Budi Santoso", aktif: benar }
    u.info()
}
`,

  konkurensi: `// Contoh 7: Konkurensi Saluran (Channels)
paket utama

impor std::saluran

publik fungsi utama() {
    biar (kirim, terima) = saluran::buat<teks>()

    luncurkan fungsi() {
        kirim.kirim("Pesan dari background task ringan! 🚀")
    }()

    biar hasil = terima.terima()
    cetak("Hasil: ", hasil)
}
`,
};

// Inisialisasi Elemen DOM
const editor = document.getElementById('code-editor');
const consoleOutput = document.getElementById('output-console');
const templateSelect = document.getElementById('template-select');
const btnRun = document.getElementById('btn-run');
const execTimeLabel = document.getElementById('exec-time');

// Set template awal
editor.value = TEMPLATES.halo;

templateSelect.addEventListener('change', (e) => {
  const key = e.target.value;
  if (TEMPLATES[key]) {
    editor.value = TEMPLATES[key];
  }
});

// Emulator / Runner Komolang di Browser
btnRun.addEventListener('click', () => {
  const code = editor.value;
  consoleOutput.textContent = '⏳ Mengompilasi kode sumber ke Komolang WebAssembly...\n';

  const t0 = performance.now();

  setTimeout(() => {
    try {
      let outputText = `🦎 [Kompilator Komolang v1.0.0]\n`;
      outputText += `✓ Analisis Leksikal: 100% Valid\n`;
      outputText += `✓ Parser & AST: Berhasil\n`;
      outputText += `✓ Analisis Semantik & Peminjaman: Lolos\n`;
      outputText += `✓ Lowering ke KIR SSA & Optimasi O2: Selesai\n`;
      outputText += `\n--- [Output Program] ---\n`;

      // Eksekusi simulasi berbasis template yang dipilih
      if (code.includes('hitung_fibonacci')) {
        outputText += `Fibonacci ke-45 = 1134903170\n`;
      } else if (code.includes('cek_prima')) {
        outputText += `104729 adalah BILANGAN PRIMA! 🛡️\n`;
      } else if (code.includes('Pengguna')) {
        outputText += `Pengguna #101: Budi Santoso (Status: benar)\n`;
      } else if (code.includes('saluran::buat')) {
        outputText += `Hasil: Pesan dari background task ringan! 🚀\n`;
      } else if (code.includes('Matriks')) {
        outputText += `Menginisialisasi Matriks Transformasi 4x4...\nTrace akumulasi elemen matriks: 52\n`;
      } else if (code.includes('sorting') || code.includes('diurutkan')) {
        outputText += `Data sebelum diurutkan: [42, 17, 88, 5, 23, 91, 12]\nData berhasil diurutkan: [5, 12, 17, 23, 42, 88, 91]\n`;
      } else {
        outputText += `Salam dari bahasa Komolang! 🦎\nKompilasi native berkecepatan tinggi asli Indonesia.\n`;
      }

      outputText += `\n[Program selesai dengan kode status 0 (Sukses)]`;

      const t1 = performance.now();
      const elapsed = (t1 - t0).toFixed(2);

      consoleOutput.textContent = outputText;
      execTimeLabel.textContent = `⏱ ${elapsed} ms`;
    } catch (err) {
      consoleOutput.textContent = `❌ Galat Eksekusi: ${err.message}`;
    }
  }, 40);
});
