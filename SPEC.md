# SPEC — KyuSin (Project Sinners)

Dokumen ini adalah sumber kebenaran proyek. Kalau ada yang bertentangan dengan dokumen ini, dokumen ini yang menang. Perubahan arah hanya lewat klien; catat di `DECISIONS.md`.

**Peran:** klien (pemilik produk), konsultan (Claude di chat), developer (Claude Code).

**Nama produk:** **KyuSin**. Tulis persis dengan kapitalisasi ini di UI, judul jendela, installer, dan README.
**Label proyek:** **Project Sinners**. Dipakai sebagai label proyek (README, halaman portofolio, layar *boot*), bukan nama aplikasi.
**Identifier teknis:** repo dan crate/paket memakai `kyusin` (huruf kecil). Identifier Tauri: `com.mufuyumoku.kyusin`.

Jangan mengarang kepanjangan, arti, atau nama alternatif untuk KyuSin maupun Sinners.

---

## 1. Ringkasan produk

Aplikasi desktop berisi banyak game ringan (papan, kartu, casino, arcade) dalam satu wadah bergaya **terminal retro**. Semuanya berjalan lokal, tanpa akun, tanpa internet, tanpa iklan, dan tanpa pembelian.

Tiga poin akhir, berurutan:

1. **Singleplayer:** semua game bisa dimainkan melawan bot/bandar, dengan tutorial, rating, dan chip.
2. **Multiplayer tanpa internet:** main bersama lewat jaringan lokal (LAN/Wi-Fi yang sama).
3. **Bermain bersama Project Nor-4:** agen AI lokal bisa masuk sebagai pemain lewat protokol yang sama dengan pemain LAN.

Target platform: **Windows dan Linux**.

## 2. Nilai jual (harus terasa di produk, bukan hanya di README)

1. **Main secara visual, perintah teks sebagai opsi.** Cara utama bermain adalah visual: klik, seret, dan kontrol langsung (menyeret bidak, mengklik kartu, menarik tuas slot, membidik di tembak ikan). Mode perintah teks (`e2e4`, `hit`, `spin`, `bet 50`) tersedia bagi yang suka, tapi tidak pernah diwajibkan. Di balik layar, setiap aksi visual diterjemahkan menjadi aksi yang sama dengan perintah teks. Aksi inilah yang dikirim teman di LAN dan Nor-4, jadi ini tetap fondasi poin akhir 2 dan 3.
2. **Casino yang jujur.** Aturan dan pembayaran memakai angka casino sungguhan. Setiap game casino menampilkan RTP/house edge sebenarnya di tutorial dan layar info. Halaman statistik menunjukkan total menang/kalah terhadap bandar sepanjang waktu. Chip tidak pernah bisa dibeli, dicairkan, atau dipindahkan antar pemain.
3. **Acak yang bisa dibuktikan (provably fair).** Setiap kocokan kartu, lemparan dadu, dan putaran slot berasal dari seed. Hash seed ditampilkan *sebelum* ronde, seed dibuka *setelah* ronde, dan pemain bisa memverifikasinya. Di LAN, ini juga mencegah host mengatur kartu diam-diam.
4. **Game lokal Indonesia berdampingan dengan game dunia.** Gaple, Cangkulan, Remi, Capsa Susun, dan Domino QiuQiu ada di samping catur dan poker.
5. **Replay semua pertandingan.** Karena setiap pertandingan adalah seed + urutan perintah, semuanya bisa diputar ulang persis.
6. **Tutorial interaktif di setiap game**, dijamin oleh tes otomatis (lihat §7).
7. **Mudah ditambah game baru** lewat sistem *cartridge* (lihat §5).

## 3. Stack dan aturan dependensi

- **Tauri 2 + Rust + SvelteKit** (adapter-static, SPA), sama dengan Onsa.
- Penyimpanan: SQLite lewat `rusqlite` (fitur `bundled`).
- RNG: `rand_chacha` (ChaCha20) dengan seed eksplisit; hash seed SHA-256.
- Jaringan LAN: `tokio-tungstenite` (WebSocket) + `mdns-sd` (penemuan host).
- Generator langkah catur: `cozy-chess` (MIT) atau buatan sendiri. **Bukan** `shakmaty` (GPL).
- Font dibundel lokal (aplikasi offline): VT323 untuk judul/tampilan besar, IBM Plex Mono untuk isi (keduanya OFL).

**Aturan lisensi (wajib):** repo sengaja tanpa lisensi karena klien mungkin menjualnya atau menutup kodenya nanti. Hanya dependensi berlisensi MIT, Apache-2.0, BSD, zlib, ISC, OFL (font), atau yang setara. **Dilarang GPL/LGPL/AGPL.** Tambahkan pengecekan `cargo-deny` di CI. Stockfish boleh dipakai **hanya sebagai alat kalibrasi di mesin developer**, tidak masuk repo dan tidak ikut dikirim.

## 4. Gaya visual: terminal retro

Arah: layar CRT fosfor tahun 80-an. Seluruh aplikasi terasa seperti satu sesi terminal.

- **Tema bawaan = jenis fosfor:**
  - Hijau P1: teks `#41FF00`, redup `#1F7A00`, latar `#050A05`.
  - Amber P3: teks `#FFB000`, redup `#7A5400`, latar `#0A0703`.
  - Putih P4: teks `#E6EEFF`, redup `#6B7385`, latar `#06070A`.
- **Efek CRT:** scanline, glow halus, lengkungan layar, flicker. Semuanya bisa dimatikan satu per satu. Flicker mati secara bawaan, dan semua efek mati otomatis bila sistem meminta *reduced motion*.
- **Tata letak:** grid karakter monospace. Papan, kartu, meja, dan roda digambar dengan karakter box-drawing dan blok, bukan gambar raster. Game real-time (slot, pachinko, coin pusher, tembak ikan) digambar di canvas **dengan grid karakter yang sama**, supaya tetap satu gaya.
- **Input visual adalah cara utama.** Setiap game harus bisa dimainkan penuh dengan mouse/sentuh dan kontrol visual yang wajar untuk jenis game-nya (seret bidak, klik kartu, tombol taruhan, tuas slot, bidik-dan-tembak). Pemain tidak pernah dipaksa mengetik.
- **Mode perintah (opsional).** Konsol `> _` di bawah layar, mati secara bawaan. Dibuka dengan tombol `:` atau `` ` `` dan bisa diatur agar selalu tampil. Memiliki autocomplete dan riwayat (panah atas/bawah). Untuk game real-time, mode perintah hanya mengatur taruhan dan menu, bukan kontrol gerak.
- **Keselarasan gaya.** Kontrol visual tetap bergaya terminal: tombol berbentuk `[ HIT ]`, sorotan berupa blok terbalik, kursor berupa blok berkedip.
- **Satu momen khas:** urutan *boot* singkat saat aplikasi dibuka (bisa dilewati dengan tombol apa saja, dan bisa dimatikan di pengaturan). Selain itu, animasi hanya sebagai respons aksi pemain: kartu dibagikan, dadu dilempar, reel berputar.
- **Aksesibilitas keyboard:** fokus selalu terlihat, dan menu serta game giliran bisa dimainkan dengan keyboard (tanpa harus mengetik perintah).
- **Suara:** bunyi beep/chiptune sederhana, bisa dimatikan.

## 5. Arsitektur

### 5.1 Workspace Rust

```
crates/
  core/        kontrak Game, Player, RNG provably fair, replay, rating
  games/       satu modul per game (cartridge bawaan)
  bots/        bot per game + mesin catur
  net/         server/klien LAN + protokol agen
  store/       SQLite: profil, chip, riwayat, replay
src-tauri/     jembatan Tauri
ui/            SvelteKit
tutorials/     berkas tutorial per game
```

### 5.2 Kontrak game (cartridge)

Setiap game adalah mesin keadaan murni: tanpa IO, tanpa jam dinding, tanpa RNG global. Semua acak lewat RNG yang disuntikkan. Kontrak dirancang agar kelak bisa dimuat sebagai WASM (§9, M12).

Dua jenis:

- **Giliran (`TurnGame`):**
  - `new(config, seed)`
  - `legal_actions(player)`
  - `apply(action)`
  - `view_for(player)` (tampilan yang disaring; kartu lawan tidak bocor)
  - `to_text(view)`
  - `is_over()` / `result()`
  - `parse_command(str) -> Action`
  - `format_action(Action) -> str`
- **Real-time (`TickGame`):**
  - `step(inputs, dt_tetap)` dengan timestep tetap dan deterministik
  - `snapshot()`
  - Input direkam per tick, jadi replay tetap persis.

**Manifest cartridge** (per game): id, nama tampilan, kategori, jumlah pemain min/maks, jenis (giliran/real-time), dukungan LAN, dukungan agen, RTP (untuk casino), path tutorial, daftar perintah + deskripsi singkatnya.

Registry membaca semua manifest. Menu, `help`, `man`, dan autocomplete **dibangkitkan dari manifest**, tidak ditulis tangan. Menambah game baru = menambah satu modul + manifest + tutorial, tanpa menyentuh kode menu.

### 5.3 Pemain

Abstraksi `Player` dengan implementasi:
- **Human:** UI lokal.
- **Bot:** per game, dengan level kesulitan.
- **Remote:** WebSocket.

Pemain LAN dan Nor-4 sama-sama **Remote**.

### 5.4 Provably fair

Sebelum ronde, host membuat seed, menampilkan/mengirim `SHA-256(seed)`, lalu memainkan ronde. Setelah ronde, seed dibuka. Perintah `verify` menghitung ulang semua acak ronde tersebut dari seed dan membandingkan hasilnya.

## 6. Katalog game

Status tiap game mengikuti Definition of Done di §7. Urutan pengerjaan ada di §9.

### 6.1 Papan
- **Catur:** mesin alpha-beta sendiri dengan level kesulitan terkalibrasi, notasi SAN dan koordinat, impor/ekspor PGN, jam opsional.
- **Reversi**
- **Dam (checkers):** aturan internasional atau Inggris, pilih satu dan catat di DECISIONS.

### 6.2 Kartu dan domino (non-casino)
- Gaple
- Cangkulan
- Remi
- Kartu ala Uno (nama generik; **jangan** memakai nama atau tampilan "Uno")
- Solitaire Klondike
- Hearts

### 6.3 Casino: meja kartu
Blackjack, Baccarat (Punto Banco), Texas Hold'em, Omaha, Three Card Poker, Caribbean Stud, Casino Hold'em, Pai Gow Poker, Let It Ride, Casino War, Red Dog, Dragon Tiger, Capsa Susun, Domino QiuQiu, Teen Patti, Andar Bahar.

### 6.4 Casino: dadu, roda, ubin
Roulette (Eropa dan Amerika), Craps, Sic Bo, Big Six / Money Wheel, Fan-Tan, Pai Gow (ubin), Chuck-a-luck.

### 6.5 Casino: lotere dan instan
Keno, Bingo, kartu gosok, Hi-Lo, Video Poker (Jacks or Better, Deuces Wild, Joker Poker).

### 6.6 Casino: arcade dopamin
- Slot 3-reel klasik dan slot video 5-reel. Tabel pembayaran dan RTP ditampilkan.
- Pachinko dan Pachislot.
- Plinko.
- Coin pusher.
- Tembak ikan (fish shooter).
- Crash (pengali naik).
- Mines.
- Dice over/under.
- Tower / Limbo.

Game real-time di kelompok ini **singleplayer saja**. LAN dan agen hanya untuk game giliran.

### 6.7 Ekonomi chip
- Chip profil untuk singleplayer.
- Saldo awal tetap.
- **Tunjangan harian** bila saldo di bawah ambang.
- Tidak ada pembelian, tidak ada pencairan, tidak ada transfer.
- Meja LAN memakai **chip meja** terpisah (buy-in baru tiap sesi), bukan chip profil.

## 7. Definition of Done per game

Sebuah game belum boleh ditandai selesai sebelum semua poin ini terpenuhi:

1. Mesin aturan + tes unit untuk aturan dan pembayaran. Untuk casino: tes simulasi jutaan ronde bahwa RTP terukur sesuai RTP di manifest (dalam toleransi yang dicatat).
2. Bot/bandar yang bisa dimainkan. Untuk game kompetitif: minimal 3 tingkat kesulitan.
3. Kontrol visual lengkap: seluruh game bisa dimainkan tanpa mengetik.
4. Perintah teks lengkap (`parse_command`/`format_action`) + tes bolak-balik. Tetap wajib, karena ini protokol LAN/agen dan dipakai mode perintah.
5. Tampilan terminal retro sesuai §4.
6. **Tutorial interaktif** di `tutorials/<id>.toml`: langkah berisi keadaan awal, teks penjelasan, aksi yang diharapkan, dan petunjuk bila salah. Tutorial memandu lewat kontrol visual (menyorot bidak/kartu/tombol yang harus disentuh); perintah teks padanannya ditampilkan kecil sebagai info. Plus halaman `man <id>` (aturan lengkap, kontrol, perintah, RTP bila casino).
7. **Tes tutorial:** CI memutar setiap tutorial terhadap mesin aturan asli dan gagal bila ada langkah yang tidak valid. CI juga gagal bila ada game terdaftar tanpa tutorial.
8. Replay berfungsi.
9. Manifest lengkap.

## 8. Rating dan kesulitan

- **Rating pemain:** Glicko-2 (implementasi sendiri), per game kompetitif, dihitung dari hasil melawan bot yang kekuatannya diketahui dan lawan LAN. UI menyebutnya "rating lokal", bukan Elo resmi.
- **Kalibrasi catur:** skrip dev (di luar build) mengadu tiap level bot melawan Stockfish dengan batasan kekuatan yang diketahui. Hasilnya menjadi perkiraan rating tiap level. Hasil kalibrasi disimpan sebagai data di repo; Stockfish-nya tidak.

## 9. Milestone (berurutan)

Aturan: satu milestone per sesi. Setiap milestone diakhiri dengan pembaruan `PROGRESS.md` dan catatan keputusan di `DECISIONS.md`.

| M | Isi |
|---|-----|
| M0 | Kerangka Tauri + workspace Rust, tema fosfor + efek CRT, navigasi visual + mode perintah opsional (help/autocomplete/riwayat), registry manifest, runner tutorial + tes tutorial di CI, `cargo-deny` |
| M1 | Kontrak `TurnGame`/`Player`, RNG provably fair + `verify`, replay. Dibuktikan dengan **Reversi** memenuhi §7 |
| M2 | Catur: mesin, 3+ level, PGN, jam, tutorial. Skrip kalibrasi |
| M3 | Profil, Glicko-2, riwayat, halaman statistik |
| M4 | Mesin kartu bersama + ekonomi chip + **Blackjack** |
| M5 | Casino meja kartu (§6.3) |
| M6 | Casino dadu/roda/ubin + lotere/instan (§6.4, §6.5) |
| M7 | Kontrak `TickGame` + arcade dopamin (§6.6) |
| M8 | Papan & kartu non-casino sisanya (§6.1, §6.2). **→ Poin akhir 1** |
| M9 | LAN: host/join, penemuan otomatis, lobi, chip meja, provably fair lintas jaringan, reconnect. **→ Poin akhir 2** |
| M10 | Protokol agen (§10) + agen dummy uji (acak dan heuristik) yang memainkan setiap game giliran sampai selesai. **→ Poin akhir 3 siap, tanpa menunggu Nor-4** |
| M11 | Installer Windows + paket Linux lewat CI, halaman Bantuan |
| M12 | Loader cartridge WASM: game baru bisa ditambah tanpa rebuild aplikasi, dengan satu game contoh dipindah ke WASM sebagai bukti |

Milestone casino yang besar (M5, M6, M7) boleh dipecah menjadi sub-milestone (M5a, M5b, …) per kelompok game.

## 10. Protokol pemain remote (LAN dan agen)

WebSocket, pesan JSON. Endpoint yang sama untuk teman LAN (alamat jaringan) dan agen (`127.0.0.1`).

**Server → pemain:**
- `state`: tampilan tersaring dalam dua bentuk, `text` (yang dilihat manusia) dan `data` (terstruktur).
- `legal_actions`: daftar aksi dalam bentuk perintah teks.
- `event`: kejadian permainan untuk komentar, misalnya "lawan all-in" atau "skak".
- `commit` / `reveal`: provably fair.
- `result`

**Pemain → server:**
- `act`: satu perintah teks yang harus ada di `legal_actions`.
- `chat`

**Khusus agen:**
- Mode tanpa batas waktu per langkah (model lokal lebih lambat dari manusia).
- Opsi `hint`: meminta saran bot pada level tertentu, supaya agen bisa memilih di antara kandidat sambil tetap berkomentar.

Aksi di luar `legal_actions` ditolak dengan pesan kesalahan yang jelas; permainan tidak rusak.

## 11. Aturan kerja Claude Code

- Git lokal dulu. Repo GitHub `MufuyuMoku/kyusin` dibuat di akun **MufuyuMoku**, privat, tanpa berkas lisensi. Git global di mesin klien masih memakai identitas clownface471, jadi atur `user.name`/`user.email` lokal repo ke identitas MufuyuMoku sebelum commit pertama.
- Tes aturan ditulis sebelum implementasi untuk mesin aturan dan pembayaran casino.
- Jangan menambah fitur di luar SPEC. Usulan dicatat di `DECISIONS.md` bagian "Usulan", tidak langsung dikerjakan.
- **Lokal vs cloud:** M0 dan semua pekerjaan yang butuh dicek secara visual dikerjakan di sesi lokal. Pekerjaan logika murni (mesin aturan, bot, simulasi RTP jutaan ronde, protokol) boleh dikerjakan di sesi cloud setelah repo ada di GitHub. Tes di cloud dijalankan pada crate di `crates/` saja (tanpa crate Tauri), jadi tidak perlu membuka jendela aplikasi.
