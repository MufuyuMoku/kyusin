# PROGRESS — KyuSin

Diperbarui di akhir setiap milestone (SPEC §9). Entri terbaru di atas.

## Status milestone

| M | Status | Sesi/tanggal | Catatan |
|---|--------|--------------|---------|
| M0 | selesai | 2026-09-27 | Kerangka, tema, CRT, navigasi, konsol, registry, runner + tes tutorial, CI lisensi. MPL-2.0 dijawab di Rev. 4 (D-022) |
| M0b | selesai | 2026-09-27 | Efek CRT samar + slider, reduced motion baru, checkbox jujur, dua bahasa + cek terjemahan di CI |
| M0c | selesai | 2026-09-27 | Tiga mode boot (Verbose, Sinematik, Sapaan) + Mati |
| M1 | selesai | 2026-09-28 | Kontrak final, provably fair + verify, replay, Reversi memenuhi §7 |
| M1b | selesai | 2026-09-28 | Papan grid CSS + sprite piksel, komponen papan bersama, tes jendela asli tauri-driver di CI Windows, sapaan baru + bentuk jamak |
| M2 | selesai | 2026-09-28 | Catur: mesin (cozy-chess), SAN/PGN, 4 level terkalibrasi Stockfish 19, jam host, tutorial, seret-lepas + klik, tes jendela asli |
| M2b | selesai | 2026-09-29 | Catur level 5–6 (mesin kedua; ≈1826, ≈2093 vs Stockfish), tangga Reversi 4 level, aturan tangga ≤ 400 di CI (SPEC Rev. 10) |
| M3 | selesai | 2026-09-28 | Profil tunggal, Glicko-2 (rating lokal), riwayat, statistik; kalibrasi antar-bot Reversi. Sebelumnya: perbaikan Rev. 9 (menu jeda, penundaan, kursor) |
| M4 | belum mulai | | |
| M5 | belum mulai | | |
| M6 | belum mulai | | |
| M7 | belum mulai | | |
| M8 | belum mulai | | |
| M9 | belum mulai | | |
| M10 | belum mulai | | |
| M11 | belum mulai | | |
| M12 | belum mulai | | |

Status: `belum mulai` · `berjalan` · `selesai` · `terblokir`

## Status game (Definition of Done, SPEC §7)

Kolom 1–9 mengikuti poin DoD: 1 aturan+tes, 2 bot, 3 visual, 4 perintah teks, 5 tampilan, 6 tutorial+man, 7 tes tutorial, 8 replay, 9 manifest.

| Game | 1 | 2 | 3 | 4 | 5 | 6 | 7 | 8 | 9 | Selesai |
|------|---|---|---|---|---|---|---|---|---|---------|
| Reversi | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ya (M1); rating lokal di M3 |
| Catur | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ya (M2); rating lokal di M3 |

## Log sesi

### 2026-09-29 — M2b: level catur 5–6 dan tangga level (SPEC Rev. 10)
- Hasil uji klien atas perbaikan M2 dan M3: sesuai; D-048–D-052 disetujui. SPEC disunting menjadi Revisi 10 (D-053): aturan tangga level merata (§8), M2b diperluas (§9), entri §12.
- Pemeriksaan CI baru: selisih rating dua level bot berurutan ≤ 400 untuk setiap game kompetitif (level terbawah yang hanya punya batas atas dikecualikan). Saat dipasang, pemeriksaan ini langsung menangkap Reversi level 2 → 3 (selisih 789).
- Reversi (D-054): empat level, kalibrasi lokal antar-bot 1000 partai per pasangan: 1000 / 1344 / 1680 / 2001 (selisih 344 / 336 / 321).
- Catur level 5–6 (D-054): mesin kedua (PVS, tabel transposisi, null move, LMR, evaluasi bertahap); level 1–4 tidak diubah. Tiga kandidat batas node diukur paralel terhadap Stockfish 19 di GitHub Actions (cabang `m2b`, run 36506334919 / 36506342288 / 36506349760). Terpilih 10 ribu / 40 ribu node: level 5 ≈1826 (95%: 1748–1904), level 6 ≈2093 (2016–2170). Tangga catur: (level 1 < 1320) / 1283 / 1459 / 1604 / 1826 / 2093.
- Alat: `calibrate.yml` mendapat input `levels`, `merge`, `nodes`; `calibrate_internal --pair A,B` untuk adu dua level.
- Tes: 135 tes Rust (termasuk tangga level, mesin kedua: evaluasi simetris, mat, determinisme; kekuatan level 5 > 4 dan 6 > 5), 34 tes UI.
- Belum/sisa: tidak ada layar baru, jadi tidak ada tes jendela asli baru; pilihan level di layar catur kini 6 dan Reversi 4 (tes jendela asli yang ada tetap memakai level 1).
- Langkah berikutnya: M4.

### 2026-09-28 — Perbaikan hasil uji M2 (SPEC Rev. 9) dan M3
- SPEC Revisi 9 ditimpa (D-048): menu jeda + penundaan, kursor mengikuti interaksi terakhir, milestone M2b.
- Perbaikan bug hasil uji M2 (D-051), lulus CI sebelum M3 dimulai (run 36418674352):
  - (a) `Esc`/[ KELUAR ]/`back` di tengah pertandingan membuka menu jeda (fokus di Lanjutkan; Tunda & keluar; Menyerah dengan konfirmasi); jam dan bot berhenti selama jeda. Penundaan disimpan (satu per game) dan dilanjutkan dari layar game dengan posisi, langkah, dan jam yang sama; menutup jendela menunda otomatis. Reversi mendapat `resign` (tes aturan lebih dulu, commit 66e46e4).
  - (b) Kursor: posisi awal sekali saat papan dipasang (catur e2/e7, Reversi d4), pindah ke petak yang diklik/tempat bidak dilepas, tidak ikut langkah lawan, tersembunyi saat mouse dipakai dan muncul lagi di posisi terakhir dengan panah. Penyebab bug: posisi awal kursor dihitung ulang setiap kali daftar langkah berubah.
  - Tes jendela asli baru (`pause.e2e.mjs`): Esc → menu jeda dengan fokus di Lanjutkan, jam berhenti, Tunda & keluar lalu Lanjutkan mengembalikan posisi/langkah/jam yang sama (catur 5+0 dan Reversi); urutan pilih b1 → pilih g1 → f3 → panah menaruh kursor di f3 (Reversi: di petak yang diklik).
  - Lain-lain: pembacaan `DevToolsActivePort` di runner e2e diulang saat masih dikunci WebView2 (flaky EBUSY). Email commit memakai alamat noreply GitHub karena push ditolak (D-049).
- M3 (D-050, D-052):
  - Glicko-2 sendiri di `kyusin-core` (tes contoh Glickman ditulis lebih dulu, commit 622b43a).
  - Rating lawan bot dari kalibrasi: catur dari Stockfish (D-047), Reversi dari kalibrasi antar-bot baru (`calibrate_internal`, level 1 = 1000): level 2 ≈1344 ±33, level 3 ≈2133 ±99 (1000 partai per pasangan).
  - Store skema 3: profil tunggal, rating per game, riwayat; replay lama masuk riwayat tanpa rating.
  - Hasil dan rating dicatat saat pertandingan selesai (termasuk menyerah dan pertandingan tertunda yang dibuang); yang masih tertunda tidak dihitung.
  - UI: layar Profil (ganti nama), layar Statistik (tabel rating lokal ±RD, partai, M–S–K, terbaik; riwayat 30 terakhir dengan perubahan rating dan tautan replay), perubahan rating di akhir pertandingan, "rating lokal ≈…" di pilihan level, perintah konsol `profile` dan `stats`. Sapaan `name` dan `last_game` aktif dari data profil.
  - Tes jendela asli baru (`profile.e2e.mjs`): ganti nama, tetap setelah muat ulang; tabel statistik berisi catur dan Reversi dengan rating lokal, kolom selaras dan tidak meluber di tiga tema; riwayat berisi perubahan rating dan membuka replay.
- Tes: 126 tes Rust, 34 tes UI. Tes jendela asli (CI Windows, run 36421347259) lulus: Reversi, catur, menu jeda/penundaan/kursor, profil dan statistik (tabel berisi catur 751 ±249 dan Reversi 812 ±237 setelah partai uji yang kalah dari bot level 1). Satu run sebelumnya (36420598006) gagal sekali di langkah klik catur yang lama dan tidak terulang; tes kini mencetak event dan keadaan papan bila itu terjadi lagi.
- Belum/sisa: RD tidak bertambah selama tidak bermain (belum diperlukan). Ringkasan menang/kalah terhadap bandar di statistik menyusul bersama game casino (M4). Menu jeda LAN (hanya konfirmasi keluar) menyusul saat LAN ada.
- Langkah berikutnya: M2b (level catur 5–6) sesuai urutan §9 Rev. 9, lalu M4.

### 2026-09-28 — M2: catur
- Keputusan klien: D-041 (sprite, input visual, kalibrasi di workflow, tes jendela asli). Keputusan developer: D-042–D-046.
- Tes aturan ditulis dan di-commit lebih dulu (commit e81941f), lalu mesinnya.
- Dikerjakan:
  - Mesin aturan di atas cozy-chess (MIT): SAN kanonik + alias koordinat, rokade/en passant/promosi, skakmat, pat, remis otomatis (tiga kali, 50 langkah, bahan tidak cukup), menyerah, waktu habis. Metode kontrak opsional `canonical` untuk alias.
  - PGN: ekspor dari replay (tampil + salin), impor ke penampil replay dengan pemeriksaan setiap langkah.
  - Bot 4 level (alpha-beta + quiescence, dibatasi kedalaman/node). Jam 5+0, 10+5, 15+10, dihitung host; waktu habis tercatat di replay.
  - Tutorial dua bahasa (e4, Nf3, Bc4, O-O, aturan khusus) + `man`.
  - UI: enam sprite piksel 12×12 (putih penuh, hitam garis tepi otomatis), klik-pilih-lalu-tujuan, seret-lepas, keyboard, titik tujuan sah, petak terpilih, langkah terakhir, skak, pilihan promosi bergambar, papan dibalik saat memegang hitam, catatan langkah khusus di status.
  - Kalibrasi: workflow manual `calibrate.yml` (Stockfish 19 diunduh saat berjalan), 16 partai per pasangan, 100 ms/langkah. Hasil di `data/calibration/catur.json`:
    - level 1: 0/64 melawan 1320–1900 → di bawah 1320 (taksiran ekstrapolasi ≈888, tidak ditampilkan di UI);
    - level 2: ≈1283 (95%: 1166–1401);
    - level 3: ≈1459 (1357–1562);
    - level 4: ≈1604 (1504–1703).
    Catatan: skala `UCI_Elo` Stockfish, 100 ms/langkah, bukan rating FIDE.
- Temuan tes jendela asli: klik pertama setelah seret tertelan. Di jendela asli dengan reduced motion, bot membalas tanpa jeda dan klik berikutnya datang <150 ms setelah seret dilepas, masih di dalam jendela peredam klik penutup seret. Kini hanya klik yang langsung menyusul pelepasan seret yang diabaikan (tanda, bukan jendela waktu), dan penangkapan penunjuk baru dimulai saat seret aktif (lewat 4 px).
- Tes: 104 tes Rust (catur 22 aturan termasuk perft/kiwipete, bot catur 6 termasuk uji kekuatan antar-level dan rating), 34 tes UI. Tes jendela asli (CI Windows, run 36371740188) lulus: keselarasan papan, kursor keyboard dan hover ke 64 sel, seret e2→e4 melintasi e3/d3/d4 tanpa sel bergeser (bidak bayangan + sasaran seret tampil), empat langkah klik-pilih-lalu-tujuan (petak terpilih + titik tujuan), menyerah, hasil + verify cocok + replay tersimpan, tema P3/P4 tetap selaras.
- Belum/sisa: pilihan promosi, rokade, dan en passant lewat UI belum dilalui tes jendela asli (aturannya dites di Rust; tampilannya belum dicek di jendela asli). Mohon dicek klien.
- Langkah berikutnya: M3 di sesi berikutnya.

### 2026-09-28 — M1b: papan ulang dan tes jendela asli
- Konteks: uji klien M1 di jendela Tauri asli: baris papan tempat kursor berada bergeser dan garis vertikal putus. SPEC Revisi 8 (+ aturan lapisan kursor di §4).
- Dikerjakan (D-038–D-040):
  - `GridBoard` bersama: grid CSS, sel 40×40 tetap, garis 1px, koordinat monospace; kursor keyboard, hover, sorotan tutorial, penanda langkah sah, dan penanda langkah terakhir semuanya lapisan absolut.
  - Sprite piksel SVG buatan sendiri (`PixelSprite`): putih = bidak penuh, hitam = cincin, titik langkah sah. Status memakai sprite yang sama.
  - Tes jendela asli (`ui/e2e`, klien WebDriver tanpa paket tambahan) lewat tauri-driver di CI Windows: keselarasan kolom/baris/jarak, kursor keyboard ke 64 sel, hover ke 64 sel, satu langkah + balasan bot, tema P3 dan P4; tangkapan layar diunggah sebagai artefak `e2e-screenshots`.
  - Agar tes bisa menempel ke WebView2: aplikasi mengikuti `WEBVIEW2_USER_DATA_FOLDER`, `KYUSIN_DATA_DIR`, dan menggabungkan `WEBVIEW2_ADDITIONAL_BROWSER_ARGUMENTS` ke argumen WebView2; tes memakai folder data sementara.
  - Sapaan `evening`, `weekend`, `streak`, `generic_5`–`generic_7`; hari berturut-turut dicatat per tanggal lokal. Bentuk tunggal/jamak bahasa Inggris untuk semua teks berangka (`{n|one|other}`).
- Temuan lewat tangkapan layar tes jendela asli: kelas lapisan kursor bentrok dengan kursor blok konsol (balok terang di sel); diganti `cursor-ring`, dan tes kini memeriksa kursor tergambar sebagai garis tepi.
- Tes: 75 tes Rust, 30 tes UI, tes jendela asli di CI Windows lulus.
- Catatan M4: `chips_low` memakai saldo sebelum tunjangan harian; tunjangan disebut sesudahnya (D-039).
- Langkah berikutnya: M2 (catur) memakai `GridBoard`.

### 2026-09-28 — M1: kontrak, provably fair, replay, Reversi
- Sebelum M1: perbaikan bug hasil uji M0c (D-033, navigasi panah macet di item nonaktif) dan daftar teks Sinematik + 14 sapaan untuk ditinjau klien.
- Dikerjakan (D-034–D-037):
  - Kontrak final `TurnGame`/`Session`/`Player`, `GameRng` (ChaCha20), provably fair commit-reveal dengan aturan kegagalan, `Match`, `Replay` + `verify`.
  - Reversi: tes aturan ditulis dan di-commit lebih dulu (commit c73ee05), lalu mesin bitboard; tutorial dua bahasa, manifest, `man`.
  - Bot Reversi tiga level; penyimpanan replay SQLite; backend pertandingan (seed OS, komitmen di awal, verify otomatis, replay tersimpan).
  - UI: papan Reversi (klik dan keyboard), layar pertandingan dengan panel provably fair, penampil replay, pilihan lawan/posisi, seed pemain di pengaturan, konsol `play`/`verify`, tutorial Reversi dengan papan dan sorotan sel.
  - Glyph UI dibatasi ke font yang dibundel (D-035), dengan tes CI.
- Tes: 75 tes Rust (inti 31 termasuk fair dan RNG; Reversi 13 termasuk perft 1–6; replay/verify 4 termasuk deteksi manipulasi di tiap tahap; bot 6 termasuk uji kekuatan; store 2; tutorial, runner, fixture, jembatan Tauri) dan 23 tes UI. `scripts/check.sh` lulus.
- Dicek di peramban (backend tiruan): papan selaras sampai piksel, satu pertandingan penuh lewat klik sel sampai selesai (panel seed terbuka, verify tampil, replay tersimpan), penampil replay maju-mundur, kontrol keyboard papan (panah, Enter di petak tidak sah diabaikan, Spasi meletakkan bidak).
- Belum/sisa: pertandingan dan tutorial Reversi di jendela Tauri asli (mesin Rust sungguhan, bot sungguhan, SQLite) belum dicek visual; mohon dicek klien. Rating Glicko-2 menyusul di M3.
- Langkah berikutnya: M2 (catur) di sesi berikutnya.

### 2026-09-27 — M0c: tiga mode urutan boot
- Dikerjakan (D-031, D-032):
  - Pengaturan "Urutan boot": Verbose, Sinematik (bawaan), Sapaan, Mati.
  - Verbose: baris ala booting Linux dengan stempel waktu dan status sungguhan (pengaturan, tema, efek, reduced motion, bahasa, backend, lokasi data dari Tauri, sesi sebelumnya, registry dan tiap cartridge, font yang termuat).
  - Sinematik: diketik perlahan dengan jeda, diakhiri nama KyuSin.
  - Sapaan: 14 sapaan bersyarat berbasis jam dan jeda sejak sesi terakhir; syarat profil, game terakhir, dan chip sudah disiapkan dan aktif sendiri begitu backend mengirim datanya (M3/M4). Tidak mengulang sapaan sesi sebelumnya; selalu ada cadangan.
  - Semua mode bisa dilewati; tombol/klik yang melewati tidak ikut memilih menu. Saat reduced motion teks tampil langsung. Semua teks dua bahasa.
  - Backend: `app_info` mengirim `data_dir`.
- Tes: 16 tes UI (tambahan: pemilih sapaan, parameter sapaan sama di dua bahasa, kunci boot ada), 35 tes Rust. `scripts/check.sh` lulus.
- Dicek visual (browser + tiruan, reduced motion): Verbose, Sinematik, Sapaan (12 hari tidak datang → sapaan lama tidak datang; dibuka lagi → "baru saja pergi"), Mati, layar pengaturan. Ditemukan dan diperbaiki: Enter/klik untuk melewati boot ikut mengaktifkan tombol menu pertama.
- Belum/sisa: animasi ketik (tanpa reduced motion) belum terlihat di pane browser karena pane meminta reduced motion; mohon dicek klien di jendela Tauri.
- Langkah berikutnya: M1 di sesi berikutnya.

### 2026-09-27 — M0b: hasil uji klien atas M0
- Dikerjakan:
  - Efek CRT sebagai bumbu samar (D-025): scanline, glow, lengkungan menyala secara bawaan; kekuatannya diatur slider intensitas `[-] ■■■□□□□□□□ [+]` 0–100% (bawaan 30%). Flicker tetap mati.
  - Reduced motion (D-026): hanya flicker dan animasi yang berhenti; efek statis mengikuti pengaturan. Urutan boot tetap tampil, teksnya langsung tanpa animasi ketik. Kotak centang menunjukkan keadaan sebenarnya; flicker yang dipaksa mati tampil nonaktif dengan alasannya.
  - Dua bahasa (D-027, D-029): semua teks UI lewat `ui/src/lib/i18n/{id,en}.json`; teks Rust (man, kategori, kesalahan, runner tutorial) lewat `crates/core/i18n.toml`; manifest dan tutorial fixture berupa `{ id, en }`; tampilan fixture lewat katalognya. Backend mengirim teks dalam dua bahasa, jadi ganti bahasa langsung berlaku (termasuk di tengah tutorial). Bahasa bawaan mengikuti sistem. Kata perintah tetap Inggris; perintah baru `lang <id|en>`.
  - Kontrak sementara: `to_text(&View, Lang)` (D-028).
- Tes: 35 tes Rust (tambahan: katalog dua bahasa, tutorial tanpa bahasa Inggris atau berteks kosong gagal, `man` tanpa kunci mentah, DTO dua bahasa) dan 9 tes UI (tambahan: paritas kunci id/en, kunci yang dipakai kode ada, markup tanpa teks mentah beserta uji-diri pemeriksanya). `scripts/check.sh` lulus.
- Dicek visual (browser + backend tiruan, SPEC §11): boot saat reduced motion, pengaturan dalam Bahasa Indonesia (flicker nonaktif dengan alasan, slider mengubah `--fx` dan tersimpan), tutorial dalam Bahasa Indonesia lalu berganti ke Inggris di tengah langkah beserta petunjuknya.
- Belum/sisa: pengecekan di jendela Tauri asli dilakukan klien (SPEC §11): `cd src-tauri && npx --prefix ../ui tauri dev --features fixture`.
- Langkah berikutnya: M1 di sesi berikutnya.

### 2026-09-27 — M0: kerangka
- Dikerjakan:
  - Workspace Rust (`crates/core`, `games`, `bots`, `net`, `store`, `src-tauri`) dan UI SvelteKit 2 + Svelte 5 (SPA).
  - `kyusin-core`: `ActionSpec` (aksi tetap / templat berparameter), manifest + validasi aturan SPEC, registry, kontrak `TurnGame` sementara + `Session` berbasis perintah teks, runner tutorial, halaman `man` dari manifest + tutorial.
  - `kyusin-games`: registry bawaan (kosong di M0) dan game fixture "Batang" khusus tes (D-005, D-015).
  - UI: tiga tema fosfor, efek CRT (scanline, glow, lengkungan, flicker) yang bisa dimatikan satu per satu dan mati otomatis saat reduced motion, urutan boot yang bisa dilewati/dimatikan, bingkai box-drawing di grid karakter, menu/`man`/tutorial/pengaturan/bantuan yang bisa dipakai penuh dengan mouse dan keyboard, konsol `> _` opsional (`:` atau `` ` ``) dengan autocomplete dan riwayat. Font VT323 dan IBM Plex Mono dibundel lokal.
  - CI: cargo-deny (lisensi), pemeriksa lisensi npm dari isi bundle, fmt, clippy, tes crate (termasuk tes tutorial), svelte-check, tes UI, build aplikasi debug di Windows dan Linux. `scripts/check.sh` menjalankan hal yang sama secara lokal.
- Tes: 26 tes Rust (manifest, ActionSpec, fixture, runner tutorial termasuk tutorial rusak, tes tutorial registry, bentuk DTO Tauri) dan 5 tes UI (pengurai, autocomplete, riwayat). Semua lulus lokal; `cargo deny check licenses` lulus.
- Dicek visual: di peramban (backend tiruan) seluruh alur menu → `man` → tutorial (aksi salah memunculkan petunjuk, tombol benar disorot) → konsol → tema/pengaturan. Di aplikasi Tauri asli: katalog dari registry dan halaman `man` asli dari Rust.
- Belum/sisa:
  - Q-003: konfirmasi klien soal MPL-2.0 (D-021).
  - Alur tutorial di jendela Tauri asli belum diklik manual (logika runner dan bentuk JSON-nya teruji); mohon dicoba klien lewat `--features fixture`.
  - Suara (SPEC §4) belum ada; tidak termasuk isi M0.
- Langkah berikutnya: M1 — kontrak final `TurnGame`/`Player`, RNG provably fair + `verify` (seed gabungan D-001/D-013), replay, dibuktikan dengan Reversi.

<!--
### YYYY-MM-DD — Mx: judul
- Dikerjakan:
- Tes:
- Belum/sisa:
- Langkah berikutnya:
-->
