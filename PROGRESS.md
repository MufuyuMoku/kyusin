# PROGRESS — KyuSin

Diperbarui di akhir setiap milestone (SPEC §9). Entri terbaru di atas.

## Status milestone

| M | Status | Sesi/tanggal | Catatan |
|---|--------|--------------|---------|
| M0 | selesai | 2026-09-27 | Kerangka, tema, CRT, navigasi, konsol, registry, runner + tes tutorial, CI lisensi. MPL-2.0 dijawab di Rev. 4 (D-022) |
| M0b | selesai | 2026-09-27 | Efek CRT samar + slider, reduced motion baru, checkbox jujur, dua bahasa + cek terjemahan di CI |
| M1 | belum mulai | | |
| M2 | belum mulai | | |
| M3 | belum mulai | | |
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

## Log sesi

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
