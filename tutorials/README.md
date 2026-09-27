# Tutorial

Satu berkas per game: `tutorials/<id>.toml` (SPEC §7.6). Tes
`crates/games/tests/tutorials.rs` memutar setiap tutorial terhadap mesin aturan
asli dan gagal bila ada game terdaftar tanpa tutorial atau tutorial tanpa game.

Contoh lengkap: `crates/games/src/fixture/tutorial.toml`.

Semua teks untuk pemain wajib dalam dua bahasa (`{ id, en }`); berkas yang
kehilangan salah satu bahasa gagal dibaca dan CI gagal (SPEC §4, §7.6). Kata
perintah (`aksi`, `sebelum`, `sorot`) tetap Inggris.

```toml
game = "<id>"
judul = { id = "…", en = "…" }
seed = "<hex 64>"      # opsional; bawaan semua nol
pemain = 0             # opsional; kursi pemain yang belajar
[config]               # opsional; konfigurasi game

[man.aturan]           # bagian `man <id>` yang tidak ada di manifest
id = """…"""
en = """…"""
[man.kontrol]
id = """…"""
en = """…"""

[[langkah]]
teks.id = "…"          # penjelasan
teks.en = "…"
sebelum = ["…"]        # opsional; perintah lawan sebelum langkah ini
aksi = "…"             # opsional; aksi yang diharapkan (kosong = bacaan)
sorot = ["aksi:…"]     # wajib bila ada `aksi`; kontrol visual yang disorot
petunjuk.id = "…"      # wajib bila ada `aksi`; ditampilkan bila salah
petunjuk.en = "…"
```
