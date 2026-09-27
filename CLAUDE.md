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
- Dependensi hanya MIT, Apache-2.0, BSD, zlib, ISC, OFL (font), atau setara. **Dilarang GPL/LGPL/AGPL.**
- `cargo-deny` wajib di CI.
- Stockfish hanya alat kalibrasi di mesin developer: tidak masuk repo, tidak ikut dikirim.

## Aturan kerja (SPEC §11, §9)
- Satu milestone per sesi, berurutan sesuai §9. Akhiri setiap milestone dengan memperbarui `PROGRESS.md` dan mencatat keputusan di `DECISIONS.md`.
- Tes aturan ditulis **sebelum** implementasi untuk mesin aturan dan pembayaran casino.
- Jangan menambah fitur di luar SPEC. Usulan masuk `DECISIONS.md` bagian "Usulan", tidak langsung dikerjakan.
- Git: repo `MufuyuMoku/kyusin` (privat). Git global mesin klien memakai identitas clownface471; repo ini memakai `user.name`/`user.email` lokal MufuyuMoku. Jangan mengubah konfigurasi global.
- **Lokal vs cloud:** M0 dan semua pekerjaan yang perlu dicek visual dikerjakan di sesi lokal. Logika murni (mesin aturan, bot, simulasi RTP, protokol) boleh di sesi cloud; tes cloud hanya pada crate di `crates/` (tanpa crate Tauri).

## Pengingat arsitektur
- Game = mesin keadaan murni: tanpa IO, tanpa jam dinding, tanpa RNG global; acak lewat RNG yang disuntikkan.
- Menu, `help`, `man`, dan autocomplete dibangkitkan dari manifest, bukan ditulis tangan.
- Kontrol visual adalah cara utama bermain; perintah teks opsional tapi tetap wajib ada (protokol LAN/agen).
- Definition of Done per game ada di SPEC §7; jangan menandai game selesai sebelum kesembilan poinnya terpenuhi.
