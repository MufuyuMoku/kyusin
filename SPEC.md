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
3. **Acak yang bisa dibuktikan (provably fair).** Setiap kocokan kartu, lemparan dadu, dan putaran slot berasal dari seed gabungan yang disumbang semua peserta. Komitmen (hash) setiap sumbangan diumumkan sebelum ronde, dan semua seed dibuka setelah ronde, jadi siapa pun bisa memverifikasi. Tidak ada pihak, termasuk host, yang bisa memilih atau mengatur hasil. Batasan yang diakui terbuka: di game kartu tertutup, aplikasi host secara teknis mengetahui urutan dek selama ronde (lihat §5.4).
4. **Game lokal Indonesia berdampingan dengan game dunia.** Gaple, Cangkulan, Remi, Capsa Susun, dan Domino QiuQiu ada di samping catur dan poker.
5. **Replay semua pertandingan.** Karena setiap pertandingan adalah seed + urutan perintah, semuanya bisa diputar ulang persis.
6. **Tutorial interaktif di setiap game**, dijamin oleh tes otomatis (lihat §7).
7. **Mudah ditambah game baru** lewat sistem *cartridge* (lihat §5).

## 3. Stack dan aturan dependensi

- **Tauri 2 + Rust + SvelteKit 2 + Svelte 5** (adapter-static, SPA), versi stabil terbaru. Onsa (proyek klien sebelumnya dengan stack yang sama) adalah repo publik di akun MufuyuMoku; temukan lewat `gh repo list MufuyuMoku`. Boleh dibaca sebagai rujukan konvensi (struktur proyek, CI installer Windows), tidak wajib disalin.
- Penyimpanan: SQLite lewat `rusqlite` (fitur `bundled`).
- RNG: `rand_chacha` (ChaCha20) dengan seed eksplisit; hash seed SHA-256.
- Jaringan LAN: `tokio-tungstenite` (WebSocket) + `mdns-sd` (penemuan host).
- Generator langkah catur: `cozy-chess` (MIT) atau buatan sendiri. **Bukan** `shakmaty` (GPL).
- Font dibundel lokal (aplikasi offline): VT323 untuk judul/tampilan besar, IBM Plex Mono untuk isi (keduanya OFL).

**Aturan lisensi (wajib):** repo sengaja tanpa lisensi karena klien mungkin menjualnya atau menutup kodenya nanti. Hanya dependensi berlisensi MIT, Apache-2.0, BSD, zlib, ISC, OFL (font), atau yang setara.

- **Dilarang GPL/LGPL/AGPL** untuk semua kode yang dikompilasi ke dalam atau dibundel bersama KyuSin: crate Rust dan paket npm yang masuk build produksi.
- **MPL-2.0 diizinkan** selama berkasnya tidak diubah (copyleft-nya per berkas, tidak menjangkau kode KyuSin). Bila suatu saat berkas MPL perlu diubah, berhenti dan tanya klien.
- **Pengecualian:** pustaka sistem operasi yang ditautkan secara dinamis dan **tidak ikut dibundel**, yaitu WebView2 di Windows serta webkit2gtk/GTK di Linux.
- Karena itu paket Linux berupa **.deb** yang memakai pustaka sistem. Tidak ada AppImage (AppImage membundel pustaka tersebut).
- CI: `cargo-deny` untuk crate Rust, dan pemeriksa lisensi untuk dependensi produksi npm (alat berlisensi permisif, pilihan developer).
- Stockfish boleh dipakai **hanya sebagai alat kalibrasi di mesin developer**, tidak masuk repo dan tidak ikut dikirim.

## 4. Gaya visual: terminal retro

Arah: layar CRT fosfor tahun 80-an. Seluruh aplikasi terasa seperti satu sesi terminal.

- **Tema bawaan = jenis fosfor:**
  - Hijau P1: teks `#41FF00`, redup `#1F7A00`, latar `#050A05`.
  - Amber P3: teks `#FFB000`, redup `#7A5400`, latar `#0A0703`.
  - Putih P4: teks `#E6EEFF`, redup `#6B7385`, latar `#06070A`.
- **Efek CRT adalah bumbu, bukan fokus.** Tema fosfor adalah dasarnya; efek CRT hanya lapisan samar di atasnya.
  - Scanline, glow, dan lengkungan layar **menyala secara bawaan dengan intensitas samar**: terlihat bila diperhatikan, tidak pernah mengganggu keterbacaan teks.
  - Selain toggle per efek, ada satu **slider intensitas** (0–100%) dengan bawaan rendah.
  - Flicker mati secara bawaan.
  - *Reduced motion* hanya mematikan efek yang **bergerak** (flicker, scanline bergulir, animasi ketik di urutan boot). Efek statis (scanline diam, glow, lengkungan) tetap mengikuti pengaturan pemain.
  - Checkbox selalu mencerminkan keadaan sebenarnya. Efek yang dipaksa mati oleh *reduced motion* tampil nonaktif dengan keterangan alasannya, bukan tampil tercentang.
- **Tata letak:** grid karakter monospace untuk teks, menu, dan bingkai panel (box-drawing boleh di sini, karena satu blok teks utuh).
- **Papan dan meja game digambar sebagai grid CSS atau SVG bergaya terminal, bukan teks box-drawing.** Sel berukuran tetap, garis 1px warna fosfor, koordinat dalam font monospace. Alasannya: papan dari baris teks rapuh terhadap font cadangan, `line-height`, perapian spasi, dan perbedaan mesin render (terbukti di M1). Kursor keyboard, sorotan, hover, dan penanda langkah legal digambar sebagai lapisan di atas sel (misalnya garis tepi atau overlay) dan **tidak boleh mengubah ukuran atau posisi sel mana pun**.
- **Bidak, kartu, dan simbol game berupa sprite piksel SVG buatan sendiri** (grid piksel kecil, misalnya 12×12, diwarnai token tema), bukan glyph font. Tampilannya seperti grafis komputer tahun 80-an dan tidak bergantung pada font. Bidak "terang" diisi penuh; bidak "gelap" berupa garis tepi atau warna redup, dan keduanya harus jelas terbedakan dari petak kosong di ketiga tema fosfor.
- Game dengan animasi kontinu (slot, Plinko, Crash, pachinko, coin pusher, tembak ikan) digambar di canvas dengan gaya yang sama (sprite piksel, warna fosfor). Ini soal cara render, bukan jenis kontrak; jenis kontrak mengikuti §6.6.
- **Input visual adalah cara utama.** Setiap game harus bisa dimainkan penuh dengan mouse/sentuh dan kontrol visual yang wajar untuk jenis game-nya (seret bidak, klik kartu, tombol taruhan, tuas slot, bidik-dan-tembak). Pemain tidak pernah dipaksa mengetik.
- **Mode perintah (opsional).** Konsol `> _` di bawah layar, mati secara bawaan. Dibuka dengan tombol `:` atau `` ` `` dan bisa diatur agar selalu tampil. Memiliki autocomplete dan riwayat (panah atas/bawah). Untuk game real-time, mode perintah hanya mengatur taruhan dan menu, bukan kontrol gerak.
- **Keluar di tengah permainan tidak pernah terjadi tanpa sengaja.** `Esc` (atau tombol kembali) saat pertandingan berjalan membuka menu jeda dengan fokus awal di **Lanjutkan**, plus **Tunda & keluar** dan **Menyerah**. Pertandingan yang ditunda tersimpan dan bisa dilanjutkan dari layar game; jam catur berhenti selama ditunda (singleplayer). Menutup jendela aplikasi saat pertandingan berjalan otomatis menunda pertandingan. Di LAN tidak ada jeda: `Esc` hanya membuka konfirmasi keluar.
- **Kursor keyboard di papan mengikuti interaksi terakhir.** Klik atau seret memindahkan kursor ke petak itu. Setelah pemain melangkah, kursor berada di petak tujuan langkah tersebut. Langkah lawan tidak memindahkan kursor. Kursor disembunyikan saat pemain memakai mouse dan muncul kembali di posisi terakhirnya begitu tombol panah ditekan. Posisi awal adalah petak yang wajar untuk game-nya (catur: e2, atau e7 bila bermain hitam; Reversi: petak tengah), bukan pojok papan. Tidak ada posisi atau pilihan lama yang tertinggal setelah pemain memilih bidak lain.
- **Muat tanpa gulir.** Meja dan papan muat di ukuran jendela bawaan tanpa gulir. Kontrol aksi pemain (taruhan, fold/call/raise, hit/stand, dan sejenisnya) selalu terlihat tanpa menggulir halaman. Bila meja terlalu tinggi, tata letaknya yang dipadatkan (kursi lawan diringkas, bilah aksi menempel di bawah), bukan tesnya yang dilonggarkan. Tes jendela asli memeriksa ini di ukuran jendela bawaan.
- **Keselarasan gaya.** Kontrol visual tetap bergaya terminal: tombol berbentuk `[ HIT ]`, sorotan berupa blok terbalik, kursor berupa blok berkedip.
- **Satu momen khas: urutan *boot*** saat aplikasi dibuka, sebelum menu muncul. Ada tiga mode plus "mati", dipilih di pengaturan:
  - **Verbose:** banyak baris cepat ala booting OS Linux (stempel waktu `[    0.412031]`, status `[  OK  ]`). Baris-barisnya **mencerminkan proses startup yang sungguhan** sejauh mungkin: jumlah dan id cartridge yang benar-benar dimuat registry, tema, bahasa, font, lokasi data. Baris hiasan boleh ditambahkan, tapi tidak boleh mengaku melakukan sesuatu yang tidak terjadi (misalnya "menghubungkan ke server").
  - **Sinematik (bawaan):** beberapa baris saja, ditik perlahan dengan jeda dramatis, seperti terminal di film. Diakhiri logo/nama KyuSin.
  - **Sapaan:** menembus dinding keempat; aplikasi menyapa pemain secara langsung dengan nada santai dan sedikit usil (misalnya menyadari sudah larut malam, atau bahwa pemain sudah lama tidak datang). Sapaan dipilih dari kumpulan teks bersyarat berdasarkan data lokal: jam sistem, jeda sejak sesi terakhir, dan (setelah datanya ada di M3/M4) nama profil, game terakhir, serta saldo chip. Syarat yang datanya belum ada dilewati otomatis. Tidak memakai AI dan tidak ada data yang keluar dari perangkat. Sapaan tidak mengulang teks yang sama dua sesi berturut-turut. Selalu ada sapaan umum sebagai cadangan.
  - **Mati:** langsung ke menu.
  - Semua mode bisa dilewati dengan tombol apa saja atau klik. Saat *reduced motion*, teks tampil langsung tanpa animasi ketik. Semua teks boot tersedia dalam dua bahasa. Selain itu, animasi hanya sebagai respons aksi pemain: kartu dibagikan, dadu dilempar, reel berputar.
- **Aksesibilitas keyboard:** fokus selalu terlihat, dan menu serta game giliran bisa dimainkan dengan keyboard (tanpa harus mengetik perintah).
- **Suara:** bunyi beep/chiptune sederhana, bisa dimatikan.
- **Bahasa: Indonesia dan Inggris.**
  - Pilihan di pengaturan. Bawaan mengikuti bahasa sistem: Indonesia bila sistem berbahasa Indonesia, selain itu Inggris.
  - Semua teks UI, tutorial, halaman `man`, dan pesan kesalahan tersedia dalam dua bahasa. Tidak ada teks UI yang ditulis langsung di kode; semuanya lewat berkas terjemahan.
  - **Kata perintah tetap satu bahasa (Inggris)** di kedua bahasa (`hit`, `stand`, `bet`, `spin`, `help`, `man`), supaya protokol LAN/agen tetap stabil. Deskripsi perintah diterjemahkan.
  - CI gagal bila ada kunci terjemahan yang hilang di salah satu bahasa, termasuk di berkas tutorial.
  - Protokol (§10): klien menyebut bahasa `state.text` yang diinginkan saat terhubung (`id` atau `en`), bawaan `en`.

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
  - `pending_players()`: pemain yang sedang ditunggu aksinya. Bisa lebih dari satu pada **fase serentak** (Capsa Susun menyusun kartu, taruhan multipemain di Roulette/Craps/Sic Bo).
  - `legal_actions(player) -> Vec<ActionSpec>`. Setiap `ActionSpec` berupa **aksi tetap** (`hit`, `stand`) atau **templat berparameter** dengan batas (`bet <jumlah>` dengan min/maks/kelipatan; `place <jenis_taruhan> <jumlah>`), lengkap dengan bentuk perintah teksnya.
  - `apply(player, action)`: ditolak bila tidak cocok dengan salah satu `ActionSpec` pemain itu.
  - **Taruhan majemuk:** pemain mengirim beberapa `place …` berurutan lalu `done`. **Fase serentak** berakhir ketika semua pemain di `pending_players` sudah mengirim aksi penutupnya. Selama fase serentak, aksi seorang pemain tidak terlihat pemain lain sampai fase selesai.
  - `view_for(player) -> View`: tampilan tersaring; kartu lawan tidak bocor. `View` mengimplementasikan `Serialize` dan menjadi `state.data` di protokol (§10).
  - `to_text(&View, Lang)`: bentuk teks yang dilihat manusia dalam bahasa yang diminta (`id`/`en`), menjadi `state.text`.
  - `is_over()` / `result()`
  - `parse_command(str) -> Action`
  - `format_action(Action) -> str`
- **Real-time (`TickGame`):**
  - `step(inputs, dt_tetap)` dengan timestep tetap dan deterministik
  - `snapshot()`
  - Input direkam per tick, jadi replay tetap persis.

**Manifest cartridge** (per game): id, nama tampilan, kategori, jumlah pemain min/maks, jenis (giliran/real-time), `lawan` (bandar / bot / tidak ada), `kompetitif` (dapat rating dan wajib 3 level bot, lihat §8), `lan`, `agen`, `rtp` (angka untuk semua game casino yang melawan rumah, baik ber-bandar maupun solo seperti slot, Keno, Bingo, dan kartu gosok; `null` untuk game antar-pemain dan non-casino), path tutorial, daftar perintah + deskripsi singkatnya.

Registry membaca semua manifest. Menu, `help`, `man`, dan autocomplete **dibangkitkan dari manifest**, tidak ditulis tangan. Menambah game baru = menambah satu modul + manifest + tutorial, tanpa menyentuh kode menu.

### 5.3 Pemain

Abstraksi `Player` dengan implementasi:
- **Human:** UI lokal.
- **Bot:** per game, dengan level kesulitan.
- **Remote:** WebSocket.

Pemain LAN dan Nor-4 sama-sama **Remote**.

### 5.4 Provably fair

Skema sumbangan seed (commit-reveal) untuk **setiap ronde**:

1. Setiap peserta (host, setiap pemain LAN, setiap agen) membuat seed acak 32 byte dan mengirim `SHA-256(seed_i)` ke host. Host menyiarkan daftar lengkap komitmen, termasuk komitmennya sendiri, ke semua peserta.
2. Setelah semua komitmen terkumpul, setiap peserta non-host mengirim `seed_i` **hanya ke host**. Host memeriksa kecocokannya dengan komitmen.
3. Seed ronde = `SHA-256(seed_host ‖ seed_a ‖ seed_b ‖ …)`, dengan sumbangan non-host diurutkan menurut id peserta. Seed ini menjadi masukan ChaCha20.
4. Setelah ronde selesai, host menyiarkan **semua** seed. Setiap klien otomatis menjalankan `verify`: memeriksa komitmen, menghitung ulang seed ronde, memutar ulang ronde, dan membandingkan hasilnya. Hasil verifikasi ditampilkan.

**Sifat yang dijamin:**
- Tidak ada yang bisa **memilih** hasil. Seed host terkunci oleh komitmennya sebelum host melihat seed siapa pun, dan seed peserta lain terkunci sebelum mereka melihat apa pun. Paling jauh, host yang curang hanya bisa **membatalkan** ronde, dan setiap pembatalan selalu terlihat oleh semua pemain (lihat aturan kegagalan di bawah).
- Peserta non-host tidak bisa mengetahui dek selama ronde, karena seed host baru dibuka setelah ronde. Menyadap seed peserta lain di jaringan tidak berguna, jadi koneksi LAN tidak perlu dienkripsi untuk keperluan ini.

**Batasan yang diterima:** host yang menghitung dek, jadi aplikasi host yang dimodifikasi bisa melihat kartu tertutup. Batasan ini dijelaskan terus terang di tutorial dan `man` untuk mode LAN. Solusi kriptografis (mental poker) di luar cakupan.

**Detail lain:**
- **Aturan kegagalan:**
  - Gagal di **tahap komitmen** (sebelum ada seed yang dibuka): peserta itu dikeluarkan dari ronde, dan ronde berjalan tanpa dia.
  - Gagal di **tahap pembukaan seed**: ronde **dibatalkan untuk semua pemain**, semua taruhan dikembalikan, dan ronde berikutnya memakai komitmen baru. Peserta yang dinyatakan gagal dikeluarkan dari meja sampai dia bergabung ulang.
  - Setiap pembatalan tampil di log meja, di riwayat, dan di `verify`, lengkap dengan nama peserta yang dinyatakan gagal dan penghitung pembatalan per sesi. Setiap klien juga mencatat waktu dia sendiri mengirim seed, sehingga peserta yang dituduh gagal padahal sudah mengirim bisa melihat ketidakcocokannya.
- Untuk agen, sumbangan seed ditangani pustaka klien protokol secara otomatis, jadi tidak bergantung pada kecepatan model.
- **Singleplayer:** aplikasi berperan sebagai host, dan pemain lokal adalah peserta. Pemain boleh mengisi seed sendiri di pengaturan; bila kosong, dibuat otomatis.
- Replay menyimpan semua seed ronde.

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

**Game antar-pemain (tanpa bandar):** Texas Hold'em, Omaha, Capsa Susun, Domino QiuQiu, Teen Patti. Tidak ada rake. `rtp: null`, ditampilkan sebagai "antar-pemain, tanpa house edge". Lawan di singleplayer adalah bot.

### 6.4 Casino: dadu, roda, ubin
Roulette (Eropa dan Amerika), Craps, Sic Bo, Big Six / Money Wheel, Fan-Tan, Pai Gow (ubin), Chuck-a-luck.

### 6.5 Casino: lotere dan instan
Keno, Bingo, kartu gosok, Hi-Lo, Video Poker (Jacks or Better, Deuces Wild, Joker Poker).

### 6.6 Casino: arcade dopamin
Pembagian ditentukan oleh mekanik, bukan kategori:

- **Giliran (`TurnGame`)**, hasil ditentukan RNG saat aksi dan animasi hanya kosmetik: slot 3-reel klasik, slot video 5-reel, Plinko, Mines, Dice over/under, Tower, Limbo. Tabel pembayaran dan RTP ditampilkan.
- **Real-time (`TickGame`)**, waktu atau keterampilan pemain memengaruhi hasil: Crash (pengali naik), Pachinko, Pachislot, Coin pusher, Tembak ikan (fish shooter).

Game giliran di kelompok ini singleplayer (`lan: false`) tetapi **boleh dimainkan agen** (`agen: true`). Game real-time singleplayer saja, tanpa LAN dan tanpa agen. Hi-Lo tetap di §6.5.

### 6.7 Ekonomi chip
- Chip profil untuk singleplayer. **Saldo awal: 10.000.**
- **Tunjangan harian:** sekali per hari kalender lokal (mulai pukul 00:00 waktu sistem). Bila saldo di bawah 1.000, saldo diisi menjadi 2.000.
- Tidak ada pembelian, tidak ada pencairan, tidak ada transfer.
- **Meja LAN** memakai chip meja terpisah, bukan chip profil. Buy-in ditetapkan host saat membuat meja (bawaan 10.000 per pemain, sama untuk semua). Rebuy diizinkan secara bawaan dan bisa dimatikan host.
- **Bandar di meja casino LAN:** bandar sistem yang berjalan di aplikasi host, dengan chip tak terbatas. Host ikut bermain sebagai pemain biasa.
- Angka-angka di atas adalah angka awal; klien boleh mengubahnya kapan saja lewat `DECISIONS.md`.

## 7. Definition of Done per game

Sebuah game belum boleh ditandai selesai sebelum semua poin ini terpenuhi:

1. Mesin aturan + tes unit untuk aturan dan pembayaran, ditambah:
   - **Casino melawan rumah (`rtp` berupa angka):** RTP di manifest dihitung secara analitis atau enumerasi bila memungkinkan (slot: enumerasi seluruh kombinasi reel; roulette, sic bo, dan sejenisnya: tabel peluang). Game yang bergantung strategi (Blackjack, Video Poker) memakai RTP untuk strategi dasar/optimal yang didokumentasikan, disimulasikan dengan bot yang memainkan strategi itu. Simulasi memverifikasi RTP: **≥100.000 ronde di CI setiap push**, dan **≥10.000.000 ronde di workflow terjadwal/manual**. Toleransi = 4 × σ/√n (σ = simpangan baku pembayaran per ronde dari simulasi itu sendiri), bukan angka tetap.
   - **Game antar-pemain:** tes peringkat tangan, pembagian pot termasuk side pot, dan *property test* kekekalan chip (total chip meja tidak pernah berubah).
2. Lawan bila game-nya punya lawan (`lawan` di manifest): bandar untuk casino ber-bandar, bot untuk game ber-lawan. Game dengan `kompetitif: true` wajib minimal 3 tingkat kesulitan. Poin ini **tidak berlaku** untuk game solo (Klondike, Keno, Bingo, kartu gosok, slot, dan arcade dopamin).
3. Kontrol visual lengkap: seluruh game bisa dimainkan tanpa mengetik.
4. Perintah teks lengkap (`parse_command`/`format_action`) + tes bolak-balik. Tetap wajib, karena ini protokol LAN/agen dan dipakai mode perintah.
5. Tampilan terminal retro sesuai §4.
6. **Tutorial interaktif** di `tutorials/<id>.toml`, dengan semua teks dalam bahasa Indonesia dan Inggris: langkah berisi keadaan awal, teks penjelasan, aksi yang diharapkan, dan petunjuk bila salah. Tutorial memandu lewat kontrol visual (menyorot bidak/kartu/tombol yang harus disentuh); perintah teks padanannya ditampilkan kecil sebagai info. Plus halaman `man <id>` (aturan lengkap, kontrol, perintah, RTP bila casino).
7. **Tes tutorial:** CI memutar setiap tutorial terhadap mesin aturan asli dan gagal bila ada langkah yang tidak valid. CI juga gagal bila ada game terdaftar tanpa tutorial. Untuk `TickGame`, langkah tutorial memakai pemicu event ("tunggu event X") dalam skenario ber-seed tetap; tes CI memutar rekaman input yang disimpan bersama tutorial dan memastikan setiap langkah tercapai.
8. Replay berfungsi.
9. Manifest lengkap.

## 8. Rating dan kesulitan

- **Game kompetitif** (`kompetitif: true`: dapat rating dan wajib 3 level bot): Catur, Reversi, Dam, Gaple, Cangkulan, Remi, kartu ala Uno, Hearts, Texas Hold'em, Omaha, Capsa Susun, Domino QiuQiu, Teen Patti. Game melawan bandar dan game solo tidak punya rating.
- **Rating pemain:** Glicko-2 (implementasi sendiri), per game kompetitif, dihitung dari hasil melawan bot yang kekuatannya diketahui dan lawan LAN. UI menyebutnya "rating lokal", bukan Elo resmi.
- **Pertandingan yang melibatkan agen tidak dihitung ke rating siapa pun**, karena kekuatan agen tidak diketahui dan agen boleh memakai `hint`.
- **Tangga level merata:** untuk setiap game kompetitif, selisih rating antara dua level bot berurutan paling besar 400 pada skala rating lokal game itu (level terbawah yang hanya punya batas atas dikecualikan). CI gagal bila data kalibrasi melanggar aturan ini. Bila jaraknya terlalu lebar, tambahkan level di antaranya atau setel ulang kekuatan level yang ada. Sebaliknya, selisih dua level berurutan paling kecil 100, supaya setiap level terasa berbeda; level yang terlalu dekat dengan level di bawahnya digabung atau disetel ulang. CI memeriksa kedua batas.
- **Kalibrasi catur:** skrip dev (di luar build) mengadu tiap level bot melawan Stockfish dengan batasan kekuatan yang diketahui. Hasilnya menjadi perkiraan rating tiap level. Hasil kalibrasi disimpan sebagai data di repo; Stockfish-nya tidak.

## 9. Milestone (berurutan)

Aturan: satu milestone per sesi. Setiap milestone diakhiri dengan pembaruan `PROGRESS.md` dan catatan keputusan di `DECISIONS.md`.

| M | Isi |
|---|-----|
| M0 | Kerangka Tauri + workspace Rust, tema fosfor + efek CRT, navigasi visual + mode perintah opsional (help/autocomplete/riwayat), registry manifest, runner tutorial + tes tutorial di CI, `cargo-deny` + pemeriksa lisensi npm. Registry dan runner diuji dengan **game fixture minimal khusus tes** (bukan game katalog); kontrak final dibuat di M1 dan fixture disesuaikan |
| M0b | Perbaikan hasil uji klien: efek CRT samar menyala secara bawaan + slider intensitas, perilaku *reduced motion* yang baru, checkbox yang jujur (§4); dua bahasa Indonesia/Inggris untuk semua yang sudah ada, termasuk tutorial fixture dan cek kelengkapan terjemahan di CI |
| M0c | Tiga mode urutan boot + mati (§4). Kumpulan sapaan menyiapkan syarat untuk data M3/M4 yang otomatis aktif begitu datanya ada |
| M1 | Kontrak `TurnGame`/`Player`, RNG provably fair + `verify`, replay. Dibuktikan dengan **Reversi** memenuhi §7 |
| M1b | Papan Reversi dirender ulang sesuai §4 (grid CSS/SVG + sprite piksel), komponen papan bersama untuk papan berpetak berikutnya, tes jendela asli dengan `tauri-driver` di CI Windows |
| M2 | Catur: mesin, 3+ level, PGN, jam, tutorial. Skrip kalibrasi |
| M2b | Level catur 5–6 yang lebih kuat (target ≥2000 pada skala kalibrasi yang sama): iterative deepening, tabel transposisi, evaluasi lebih baik, waktu berpikir lebih lama; diukur ulang dengan `calibrate.yml`. Tangga level Reversi dirapikan sesuai aturan tangga level merata (§8). Dikerjakan setelah M3 |
| M3 | Profil, Glicko-2, riwayat, halaman statistik |
| M4 | Mesin kartu bersama + ekonomi chip + **Blackjack** |
| M5a | Casino meja kartu melawan bandar (§6.3): Baccarat, Three Card Poker, Caribbean Stud, Casino Hold'em, Pai Gow Poker, Let It Ride, Casino War, Red Dog, Dragon Tiger, Andar Bahar |
| M5b-1 | Casino meja kartu antar-pemain (§6.3), bagian 1: Texas Hold'em, Omaha, Teen Patti |
| M5b-2 | Casino meja kartu antar-pemain (§6.3), bagian 2: Capsa Susun, Domino QiuQiu, termasuk mesin domino bersama (dipakai juga oleh Gaple di M8) |
| M6 | Casino dadu/roda/ubin + lotere/instan (§6.4, §6.5) |
| M7 | Kontrak `TickGame` + arcade dopamin (§6.6) |
| M8 | Papan & kartu non-casino sisanya (§6.1, §6.2). **→ Poin akhir 1** |
| M9 | LAN: host/join, penemuan otomatis, lobi, chip meja, provably fair lintas jaringan, reconnect. **→ Poin akhir 2** |
| M10 | Protokol agen (§10) + agen dummy uji (acak dan heuristik) yang memainkan setiap game giliran sampai selesai. **→ Poin akhir 3 siap, tanpa menunggu Nor-4** |
| M11 | Installer Windows + paket Linux `.deb` lewat CI, halaman Bantuan, halaman **Lisensi pihak ketiga** yang dibangkitkan otomatis dari dependensi (memenuhi kewajiban atribusi MIT/Apache/BSD dan pemberitahuan sumber MPL-2.0) |
| M12 | Loader cartridge WASM: game baru bisa ditambah tanpa rebuild aplikasi, dengan satu game contoh dipindah ke WASM sebagai bukti |

Milestone casino yang besar (M5, M6, M7) boleh dipecah menjadi sub-milestone (M5a, M5b, …) per kelompok game.

## 10. Protokol pemain remote (LAN dan agen)

WebSocket, pesan JSON. Protokol yang sama untuk teman LAN (alamat jaringan) dan agen (`127.0.0.1`).

- **Server LAN** aktif hanya saat pemain menjadi host.
- **Endpoint agen** aktif hanya saat "Mode agen" dinyalakan di pengaturan, hanya mengikat `127.0.0.1`, dan memerlukan token yang ditampilkan di pengaturan.

**Server → pemain:**
- `state`: tampilan tersaring dalam dua bentuk, `text` (yang dilihat manusia) dan `data` (terstruktur).
- `legal_actions`: daftar `ActionSpec` (aksi tetap atau templat berparameter dengan batas), masing-masing dengan bentuk perintah teksnya.
- `event`: kejadian permainan untuk komentar, misalnya "lawan all-in" atau "skak".
- `seed_commits` / `seed_reveal`: provably fair sesuai §5.4.
- `result`

**Pemain → server:**
- `act`: satu perintah teks yang cocok dengan salah satu `ActionSpec` di `legal_actions`.
- `seed_commit` / `seed_contribution`: sumbangan seed sesuai §5.4.
- `chat`

**Khusus agen:**
- Mode tanpa batas waktu per langkah (model lokal lebih lambat dari manusia).
- Opsi `hint`: meminta saran bot pada level tertentu, supaya agen bisa memilih di antara kandidat sambil tetap berkomentar. `hint` **hanya untuk agen**; pemain LAN tidak punya akses ke `hint`.

Aksi di luar `legal_actions` ditolak dengan pesan kesalahan yang jelas; permainan tidak rusak.

## 11. Aturan kerja Claude Code

- Git lokal dulu. Repo GitHub `MufuyuMoku/kyusin` dibuat di akun **MufuyuMoku**, privat, tanpa berkas lisensi. Git global di mesin klien masih memakai identitas clownface471, jadi atur `user.name`/`user.email` lokal repo ke identitas MufuyuMoku sebelum commit pertama.
- **Dilarang otomasi input di tingkat sistem operasi** (SendKeys, xdotool, dan sejenisnya), karena input bisa masuk ke jendela lain. Uji UI lewat browser dengan backend tiruan dan tes otomatis; pengecekan di jendela Tauri asli dilakukan klien.
- **Pengujian jendela asli:** `tauri-driver` (WebDriver) diizinkan, karena perintahnya hanya masuk ke aplikasi KyuSin, bukan ke sistem operasi. Setiap layar game wajib punya tes jendela asli yang memeriksa keselarasan posisi (misalnya semua sel satu kolom punya koordinat x yang sama) dan menyimpan tangkapan layar sebagai artefak. Pengecekan di browser dengan backend tiruan tidak lagi cukup sebagai bukti visual.
- Tes aturan ditulis sebelum implementasi untuk mesin aturan dan pembayaran casino.
- Jangan menambah fitur di luar SPEC. Usulan dicatat di `DECISIONS.md` bagian "Usulan", tidak langsung dikerjakan.
- **Lokal vs cloud:** M0 dan semua pekerjaan yang butuh dicek secara visual dikerjakan di sesi lokal. Pekerjaan logika murni (mesin aturan, bot, simulasi RTP jutaan ronde, protokol) boleh dikerjakan di sesi cloud setelah repo ada di GitHub. Tes di cloud dijalankan pada crate di `crates/` saja (tanpa crate Tauri), jadi tidak perlu membuka jendela aplikasi.

## 12. Riwayat revisi SPEC

- **Revisi 2 (27 Sep 2026):** menjawab temuan sesi persiapan — skema seed gabungan (§2.3, §5.4); `ActionSpec`, `pending_players`, fase serentak, dan `View: Serialize` (§5.2); field manifest baru; pembagian giliran/real-time di arcade (§6.6); game antar-pemain tanpa rake (§6.3); angka ekonomi chip dan bandar LAN (§6.7); DoD dengan cakupan per jenis game dan toleransi statistik (§7); daftar game kompetitif dan pengecualian rating agen (§8); fixture di M0 dan paket `.deb` (§9); pengecualian lisensi pustaka sistem (§3); aktivasi endpoint dan `hint` khusus agen (§10). Aturan Dam tetap dipilih developer dan dicatat di `DECISIONS.md`.
- **Revisi 3 (27 Sep 2026):** menjawab Q-001 dan Q-002 — aturan kegagalan dua tahap dan jaminan "tidak bisa memilih, hanya bisa membatalkan secara terlihat" (§5.4); §4 memakai istilah "animasi kontinu" sebagai soal render, bukan kontrak; `rtp` berupa angka untuk semua casino melawan rumah termasuk game solo (§5.2, §7).
- **Revisi 4 (27 Sep 2026):** menjawab Q-003 — MPL-2.0 diizinkan tanpa modifikasi (§3); halaman lisensi pihak ketiga di M11 (§9); larangan otomasi input tingkat OS (§11).
- **Revisi 5 (27 Sep 2026):** hasil uji klien atas M0 — efek CRT sebagai bumbu samar yang menyala secara bawaan, slider intensitas, *reduced motion* hanya mematikan efek bergerak, checkbox jujur, penjelasan urutan boot (§4); dua bahasa Indonesia/Inggris dengan kata perintah tetap Inggris (§4, §7, §10); milestone M0b (§9).
- **Revisi 6 (27 Sep 2026):** mengesahkan D-028 — `to_text` menerima parameter bahasa (§5.2).
- **Revisi 7 (27 Sep 2026):** urutan boot punya tiga mode — Verbose, Sinematik (bawaan), Sapaan (menembus dinding keempat) — plus mati (§4); milestone M0c (§9). Intensitas bawaan efek CRT 30% disahkan klien.
- **Revisi 8 (28 Sep 2026):** hasil uji klien atas M1 — papan sebagai grid CSS/SVG, bidak dan simbol sebagai sprite piksel SVG (§4); pengujian jendela asli dengan `tauri-driver` (§11); milestone M1b (§9).
- **Revisi 9 (28 Sep 2026):** hasil uji klien atas M2 — menu jeda dan penundaan pertandingan, aturan kursor keyboard mengikuti interaksi terakhir (§4); milestone M2b untuk level catur yang lebih kuat (§9).
- **Revisi 10 (29 Sep 2026):** hasil uji klien atas perbaikan M2 dan M3 — aturan tangga level merata: selisih rating dua level bot berurutan paling besar 400, diperiksa CI (§8); M2b diperluas dengan perapian tangga level Reversi (§9).
- **Revisi 11 (29 Sep 2026):** hasil uji klien atas M4 — M5 dipecah menjadi M5a (casino meja kartu melawan bandar) dan M5b (casino meja kartu antar-pemain) (§9).
- **Revisi 12 (2 Okt 2026):** hasil uji klien atas M5a — M5b dipecah menjadi M5b-1 (Texas Hold'em, Omaha, Teen Patti) dan M5b-2 (Capsa Susun, Domino QiuQiu, mesin domino bersama) (§9).
- **Revisi 13 (4 Okt 2026):** hasil tinjauan klien atas M5b-1 — selisih rating dua level bot berurutan paling kecil 100, diperiksa CI bersama batas 400 (§8); meja dan papan muat di jendela bawaan tanpa gulir dan kontrol aksi selalu terlihat, diperiksa tes jendela asli (§4). Klien menyebutnya Revisi 12; nomor 12 sudah dipakai untuk pemecahan M5b, jadi dicatat sebagai Revisi 13.
