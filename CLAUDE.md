# CLAUDE.md — KyuSin

`SPEC.md` adalah sumber kebenaran. Bila berkas ini, kode, atau catatan lain bertentangan dengan SPEC, SPEC yang menang. Perubahan arah hanya lewat klien dan dicatat di `DECISIONS.md`.

## Penamaan
- Produk: **KyuSin** (kapitalisasi persis). Label proyek: **Project Sinners** (bukan nama aplikasi).
- Identifier teknis: `kyusin` (huruf kecil); Tauri: `com.mufuyumoku.kyusin`.
- Jangan mengarang kepanjangan, arti, atau nama alternatif untuk KyuSin maupun Sinners.

## Stack (SPEC §3)
- Tauri 2 + Rust + SvelteKit (adapter-static, SPA).
- SQLite via `rusqlite` (fitur `bundled`).
- RNG `rand_chacha` (ChaCha20) dengan seed eksplisit; hash seed SHA-256.
- LAN: `tokio-tungstenite` + `mdns-sd`.
- Catur: `cozy-chess` atau buatan sendiri. **Bukan `shakmaty`** (GPL).
- Font dibundel lokal: VT323 (judul) dan IBM Plex Mono (isi).

## Lisensi (wajib)
- Repo sengaja **tanpa berkas lisensi**. Jangan menambahkan LICENSE.
- Dependensi hanya MIT, Apache-2.0, BSD, zlib, ISC, OFL (font), atau setara. **Dilarang GPL/LGPL/AGPL** untuk semua yang dikompilasi/dibundel (crate Rust dan npm produksi).
- Pengecualian: WebView2 (Windows) dan webkit2gtk/GTK (Linux), ditautkan dinamis dan tidak dibundel. Karena itu paket Linux hanya `.deb`, tanpa AppImage.
- CI wajib: `cargo-deny` + pemeriksa lisensi dependensi produksi npm.
- MPL-2.0 diizinkan **selama berkasnya tidak diubah**. Bila berkas MPL perlu diubah, berhenti dan tanya klien (D-022).
- Stockfish hanya alat kalibrasi di mesin developer: tidak masuk repo, tidak ikut dikirim.

## Aturan kerja (SPEC §11, §9)
- Satu milestone per sesi, berurutan sesuai §9. Akhiri setiap milestone dengan memperbarui `PROGRESS.md` dan mencatat keputusan di `DECISIONS.md`.
- Tes aturan ditulis **sebelum** implementasi untuk mesin aturan dan pembayaran casino.
- Jangan menambah fitur di luar SPEC. Usulan masuk `DECISIONS.md` bagian "Usulan", tidak langsung dikerjakan.
- Git: repo `MufuyuMoku/kyusin` (privat). Git global mesin klien memakai identitas clownface471; repo ini memakai `user.name` lokal MufuyuMoku dan `user.email` lokal `264320223+MufuyuMoku@users.noreply.github.com` (GitHub menolak push yang memuat email pribadi; D-049). Jangan mengubah konfigurasi global.
- **Dilarang otomasi input tingkat OS** (SendKeys, xdotool, dsb.). Uji UI lewat browser dengan backend tiruan dan tes otomatis; jendela Tauri asli dicek klien (SPEC §11).
- **Lokal vs cloud:** M0 dan semua pekerjaan yang perlu dicek visual dikerjakan di sesi lokal. Logika murni (mesin aturan, bot, simulasi RTP, protokol) boleh di sesi cloud; tes cloud hanya pada crate di `crates/` (tanpa crate Tauri).

## Bahasa (SPEC §4, D-027)
- Indonesia dan Inggris. Tidak ada teks UI langsung di kode: semua lewat berkas terjemahan (UI: `ui/src/lib/i18n/*.json`; Rust: `crates/core/i18n/*.toml`; manifest/tutorial: tabel `{ id, en }`).
- Kata perintah tetap Inggris (`help`, `take`, `bet`); deskripsinya diterjemahkan.
- CI gagal bila ada kunci terjemahan yang hilang.

## Perintah
- Cek seperti CI: `sh scripts/check.sh` (svelte-check, tes UI, build UI, lisensi npm, fmt, clippy, tes Rust termasuk tes tutorial, cargo-deny).
- Aplikasi dev: `cd src-tauri && npx --prefix ../ui tauri dev` (tambah `--features fixture` untuk menampilkan game fixture, D-015).
- UI saja di peramban: `npm --prefix ui run dev` (backend tiruan, hanya mode dev; bukan bukti visual, SPEC §11).
- Tes jendela asli: `npx tauri build --debug --no-bundle` lalu `npm --prefix ui run e2e` (butuh `tauri-driver` dan msedgedriver versi WebView2; lihat `ui/e2e/run.mjs`).

## Pengingat arsitektur
- Game = mesin keadaan murni: tanpa IO, tanpa jam dinding, tanpa RNG global; acak lewat RNG yang disuntikkan.
- Menu, `help`, `man`, dan autocomplete dibangkitkan dari manifest, bukan ditulis tangan.
- Kontrol visual adalah cara utama bermain; perintah teks opsional tapi tetap wajib ada (protokol LAN/agen).
- Definition of Done per game ada di SPEC §7; jangan menandai game selesai sebelum kesembilan poinnya terpenuhi.
- Game baru = modul di `crates/games/src/<id>/` (+ `manifest.toml`, `i18n.toml`), tutorial `tutorials/<id>.toml`, satu baris di `kyusin_games::builtin()`, bot di `crates/bots` (`levels`/`create`), dan kontrol visual di `ui/src/lib/games/` (`GAME_UI`).
- Tes aturan ditulis dan di-commit sebelum mesin aturannya (contoh: Reversi, commit c73ee05).
- Glyph UI hanya yang ada di IBM Plex Mono (D-035); `fonts.test.ts` memeriksanya.
- Papan/meja game: grid CSS/SVG (`ui/src/lib/games/GridBoard.svelte`) + sprite piksel SVG (`PixelSprite`), bukan teks box-drawing. Kursor/hover/sorotan hanya lapisan absolut (SPEC §4, D-038).
- Setiap layar game wajib punya tes jendela asli di `ui/e2e/` (tauri-driver, CI Windows) yang memeriksa keselarasan dan menyimpan tangkapan layar (SPEC §11).
- Pertandingan yang belum selesai tidak pernah dibuang: jeda/tunda/tutup jendela menundanya (SPEC §4 Rev. 9, D-051); dilanjutkan lewat `Match::restore` (seed ronde + langkah). Game kompetitif baru wajib punya `resign` untuk menu jeda.
- Rating lokal (D-052): Glicko-2 di `kyusin_core::rating`; rating level bot dari `data/calibration/<game>.json` (catur: Stockfish lewat workflow; lainnya: `cargo run --release -p kyusin-bots --example calibrate_internal -- --game <id>`, level 1 = 1000). Game kompetitif ber-bot tanpa berkas kalibrasi, atau dengan selisih dua level berurutan > 400 (SPEC §8 Rev. 10), membuat tes CI gagal. Catur level 1–4 = mesin lama (`catur.rs`), level 5–6 = `catur_search.rs`; mengubah level mana pun berarti kalibrasi ulang.
