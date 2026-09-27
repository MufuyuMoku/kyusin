# Tutorial

Satu berkas per game: `tutorials/<id>.toml` (SPEC §7.6). Tes
`crates/games/tests/tutorials.rs` memutar setiap tutorial terhadap mesin aturan
asli dan gagal bila ada game terdaftar tanpa tutorial atau tutorial tanpa game.

Contoh lengkap: `crates/games/src/fixture/tutorial.toml`.

```toml
game = "<id>"
judul = "…"
seed = "<hex 64>"      # opsional; bawaan semua nol
pemain = 0             # opsional; kursi pemain yang belajar
[config]               # opsional; konfigurasi game

[man]                  # bagian `man <id>` yang tidak ada di manifest
aturan = """…"""
kontrol = """…"""

[[langkah]]
teks = "…"             # penjelasan
sebelum = ["…"]        # opsional; perintah lawan sebelum langkah ini
aksi = "…"             # opsional; aksi yang diharapkan (kosong = bacaan)
sorot = ["aksi:…"]     # wajib bila ada `aksi`; kontrol visual yang disorot
petunjuk = "…"         # wajib bila ada `aksi`; ditampilkan bila salah
```
