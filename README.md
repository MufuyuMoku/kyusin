# KyuSin

**Project Sinners**

Aplikasi desktop berisi banyak game ringan (papan, kartu, casino, arcade) dalam
satu wadah bergaya terminal retro. Semuanya berjalan lokal: tanpa akun, tanpa
internet, tanpa iklan, dan tanpa pembelian. Target: Windows dan Linux.

Status: M1 (Reversi melawan bot, provably fair, replay). Rencana dan aturan lengkap ada di [SPEC.md](SPEC.md);
kemajuan di [PROGRESS.md](PROGRESS.md), keputusan di [DECISIONS.md](DECISIONS.md).

## Pengembangan

Butuh Rust stabil, Node 24, dan (Linux) `libwebkit2gtk-4.1-dev`.

```sh
npm --prefix ui ci
cd src-tauri && npx --prefix ../ui tauri dev --features fixture
```

Semua pemeriksaan CI secara lokal: `sh scripts/check.sh`.
