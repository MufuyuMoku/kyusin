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
| M4 | selesai | 2026-09-29 | Mesin kartu bersama, ekonomi chip (saldo, tunjangan harian, ringkasan bandar), Blackjack (RTP 99,64% strategi dasar, provably fair per shoe), label level deskriptif |
| M5a | selesai | 2026-10-02 | Sepuluh meja melawan bandar (Baccarat, Dragon Tiger, Casino War, Red Dog, Andar Bahar, Three Card Poker, Caribbean Stud, Casino Hold'em, Let It Ride, Pai Gow Poker). Sebelumnya: taruhan casino tercatat saat dipasang, titik simpan setiap aksi (D-059); SPEC Rev. 11 |
| M5b-1 | selesai | 2026-10-02 | Texas Hold'em (No-Limit), Omaha (Pot-Limit), Teen Patti antar-pemain melawan bot; buy-in dari saldo; rating sesi multipemain; SPEC Rev. 12 |
| M5b-2 | selesai | 2026-10-05 | Capsa Susun, Domino QiuQiu antar-pemain melawan bot; mesin domino bersama; aturan lokal pilihan klien (D-065) |
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
| Blackjack | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ya (M4); meja LAN menyusul |
| Baccarat | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ya (M5a) |
| Dragon Tiger | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ya (M5a) |
| Casino War | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ya (M5a) |
| Red Dog | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ya (M5a) |
| Andar Bahar | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ya (M5a) |
| Three Card Poker | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ya (M5a) |
| Caribbean Stud | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ya (M5a) |
| Casino Hold'em | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ya (M5a) |
| Let It Ride | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ya (M5a) |
| Pai Gow Poker | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ya (M5a) |
| Texas Hold'em | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ya (M5b-1); meja LAN menyusul |
| Omaha | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ya (M5b-1); meja LAN menyusul |
| Teen Patti | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ya (M5b-1); meja LAN menyusul |
| Capsa Susun | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ya (M5b-2); meja LAN menyusul |
| Domino QiuQiu | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ya (M5b-2); meja LAN menyusul |

## Log sesi

### 2026-10-05 — M5b-2: Capsa Susun, Domino QiuQiu, mesin domino

- Aturan lokal ditanyakan ke klien lebih dulu (11 pertanyaan; semua rekomendasi dipilih): D-065. Penerapan: D-066.
- Tes aturan di-commit sebelum mesinnya: domino (3), Domino QiuQiu (10: urutan kartu spesial, Murni Kecil/Besar, pembagian otomatis, pemecah seri, ante dan dua putaran, batas pot-limit, fold, sesi dan kontrak chip, kartu dari seed, kekekalan chip 60 sesi acak), Capsa Susun (9: baris tiga vs lima kartu dan salah susun, poin baris/sapu bersih/royalti/seri, kartu istimewa, pembayaran proporsional, saran selalu sah, susun serentak dan sesi, kartu istimewa otomatis, kartu dari seed, kekekalan chip 40 sesi acak).
- Mesin: `domino` (bersama Gaple nanti), `domino_qiuqiu`, `capsa_susun`; tutorial keduanya; tes host (buy-in, main, berdiri, rating).
- Bot 3 level, kalibrasi antar-bot di GitHub Actions: Domino QiuQiu 1000 / 1185 / 1358; Capsa Susun 1000 / 1231 / 1413 (selisih 231 / 182).
- UI: meja Capsa Susun (pilih kartu, pindah ke baris, saran, kirim, susun otomatis), kartu domino piksel di meja antar-pemain, pesan untuk perintah yang ditolak. Kedua meja muat di jendela bawaan tanpa gulir (diukur di peramban dengan backend tiruan sementara, tidak di-commit; dibuktikan tes jendela asli).
- Tes: 318 tes Rust, 42 tes UI. Tes jendela asli (CI Windows): E2E_RESULT
- Belum/sisa: meja LAN (M9); kalibrasi dari sesi heads-up.
- Langkah berikutnya: M6 sesuai SPEC §9.

### 2026-10-04 — Perbaikan M5b-1 dari tinjauan klien (SPEC Rev. 13)

- SPEC Revisi 13 (D-064): selisih level bot berurutan paling kecil 100 (§8); meja/papan muat tanpa gulir dan kontrol aksi selalu terlihat (§4).
- CI: `ladder_violations` memeriksa batas 100 dan 400. Teen Patti level 3 dan 4 digabung dan dikalibrasi ulang di Actions: 1000 / 1174 / 1382.
- Meja antar-pemain dipadatkan (kursi lawan dua kolom, bilah aksi menempel di bawah). Tes jendela asli tanpa kompensasi gulir + pemeriksaan `inView` untuk semua meja dan papan.
- Pekerjaan nanti: bot poker yang lebih kuat (D-064).
- Tes jendela asli (CI Windows, run 37252520108): lulus; semua meja (Blackjack, sepuluh meja M5a, tiga meja M5b-1) dan papan catur/Reversi muat tanpa gulir dengan kontrol aksi terlihat; meja enam kursi tanpa kompensasi gulir. Sekali gagal sebelumnya: klik CHECK tidak mengubah meja dalam 15 detik (tidak terulang; diagnosis ditambahkan ke tes). Satu kegagalan lain dari tes catur lama (langkah klik berupa promosi) diperbaiki.

### 2026-10-02 — M5b-1: Texas Hold'em, Omaha, Teen Patti (SPEC Rev. 12)

- Keputusan klien: D-062 (pemecahan M5b, buy-in, rating sesi, kalibrasi antar-bot, pekerjaan nanti RTP optimal Casino Hold'em). Penerapan: D-063.
- Tes aturan di-commit sebelum mesinnya: pot dan side pot (5 + 1), Texas/Omaha (12: blind dan urutan termasuk heads-up, ukuran raise No-Limit dan Pot-Limit, all-in tak penuh, side pot, Omaha tepat 2+3, sesi, kontrak chip, properti kekekalan chip pada 60 sesi acak), Teen Patti (11: urutan tangan, boot, buta/terlihat, batas stake dan chaal buta, show, sideshow terima/tolak/seri, all-in, batas pot, sesi, properti kekekalan chip).
- Mesin: meja poker bersama (`poker_meja`, varian Texas No-Limit dan Omaha Pot-Limit) dan Teen Patti; satu pertandingan = satu sesi meja (provably fair per sesi); pot tanpa penyamaan taruhan untuk Teen Patti.
- Host: buy-in dari saldo (paling banyak 2.000, paling sedikit 400) lewat titik simpan D-059; mati paksa melanjutkan tangan yang sama; membuang sesi memainkan aksi netral sambil bot bertindak; tidak masuk ringkasan bandar. Rating: satu sesi = satu pertandingan Glicko-2 dengan skor pecahan (bagian bot di bawahmu).
- Bot: Texas/Omaha 3 level, Teen Patti 4 level (kemudian 3, D-064); kalibrasi antar-bot di GitHub Actions (`kalibrasi-antarbot.yml`, 2.000 sesi heads-up 60 tangan per pasangan): Texas 1000/1131/1379, Teen Patti 1000/1174/1369/1411, Omaha 1000 / 1378 / 1600.
- UI: meja antar-pemain bersama (`PokerTable`): baris kursi berslot tetap, kartu meja, penanda giliran sebagai lapisan, kontrol bet/raise (min, ½ pot, pot, maks, ±BB) dan Teen Patti (lihat, chaal, raise, show, sideshow, terima/tolak); tes jendela asli `ui/e2e/pvp.e2e.mjs`.
- Tes: 295 tes Rust, 38 tes UI. Tes jendela asli (CI Windows, run 36976470929): lulus; ketiga meja duduk dengan buy-in 2.000, selaras, kontrol tanpa geser (diukur di koordinat isi halaman karena meja enam kursi digulir), satu tangan dimainkan, berdiri mengembalikan tumpukan ke saldo, verify cocok, rating tampil; tema P3/P4 selaras.
- Pekerjaan nanti (D-062): RTP Casino Hold'em dengan strategi optimal lewat enumerasi tepat di GitHub Actions.
- Belum/sisa: meja LAN (M9); kalibrasi memakai sesi heads-up, kekuatan relatif di meja enam kursi bisa berbeda.
- Langkah berikutnya: M5b-2 (Capsa Susun, Domino QiuQiu): aturan lokal yang bervariasi ditanyakan ke klien lebih dulu.

### 2026-10-02 — Taruhan tercatat saat dipasang (D-059), SPEC Rev. 11, M5a: sepuluh meja melawan bandar

- Permintaan klien sebelum M5: taruhan casino harus tercatat permanen saat dipasang, mati paksa tidak boleh membuat pemain lolos dari kekalahan. Temuan: belum aman (keadaan hanya disimpan saat tunda/tutup normal). Perbaikan D-059: titik simpan setelah setiap aksi untuk semua game; casino menulis saldo + pertandingan dalam satu transaksi SQLite, taruhan dipotong saat dipasang (`profile.staked`, skema 5), titik simpan baru dihapus setelah hasil tersimpan, tunjangan harian menghitung chip di meja. Bukti: tes backend dengan berkas SQLite sungguhan yang "dimatikan paksa" (ronde berjalan dilanjutkan dengan kartu dan taruhan yang sama; membuang pertandingan memainkan ronde sampai selesai; saldo = awal + bersih − di meja di setiap langkah walau dimatikan berkali-kali; partai catur tidak hilang) + tes jendela asli Blackjack (saldo turun saat taruhan dipasang).
- SPEC Revisi 11: M5 dipecah menjadi M5a (melawan bandar) dan M5b (antar-pemain).
- M5a (D-060 varian dan sumber, D-061 penerapan):
  - Evaluator poker bersama (5 kartu, terbaik dari 7, 3 kartu), dites terhadap frekuensi baku seluruh tangan satu dek. Bagian meja bersama (`kyusin_games::meja`): batas taruhan, tempat taruhan, shoe/dek, kontrak chip, `netral`.
  - Tiap game: tes aturan dan pembayaran di-commit sebelum mesinnya, manifest, tutorial dua bahasa + `man` (aturan, strategi, RTP tiap taruhan, provably fair), strategi untuk RTP, model meja di UI.
  - RTP: analitis/enumerasi tepat di tes setiap push untuk Dragon Tiger (96,27 / tie 67,23), Casino War (97,12 / tie 81,35), Andar Bahar (Andar 97,85 / Bahar 97,00), Baccarat (player 98,76 / banker 98,94 / tie 85,64), Red Dog (97,25), Pair Plus (92,72). Enumerasi tepat di GitHub Actions: Three Card Poker Ante/Play 96,6270 (run 36567987097), Let It Ride 96,4943 (run 36946263357). Simulasi besar di GitHub Actions: Caribbean Stud 94,69 (run 36945985808), Casino Hold'em 97,08 (run 36945993998); Pai Gow 97,13 dari 200 juta ronde (run 36959067318; 1 miliar ronde melewati batas waktu job, toleransi 200 juta ronde ±0,022%). Semua taruhan diverifikasi 100.000 ronde di CI setiap push dan 10.000.000 ronde di `rtp.yml` (verifikasi akhir dengan angka final: run 36961962381, ketujuh belas taruhan dan dua enumerasi tepat cocok).
  - UI: satu komponen meja bersama (`MejaTable`), sprite joker, ronde berikutnya mulus untuk meja satu ronde per sesi; tes jendela asli memainkan satu ronde di tiap meja (`ui/e2e/meja.e2e.mjs`).
- Tes: 261 tes Rust, 38 tes UI. Tes jendela asli (CI Windows, run 36959059669, semua job hijau): sepuluh meja selaras (kartu 44×60, geser 18/48 px), hover/kursor tidak menggeser kontrol, taruhan dipotong saat dipasang, satu ronde dimainkan di tiap meja dengan saldo sesuai hasil, verify cocok di meja satu ronde per sesi, ronde berikutnya dengan komitmen baru, tema P3/P4, berhenti Baccarat lewat menu jeda, statistik melawan bandar.
- Belum/sisa: meja LAN (M9). Strategi Casino Hold'em sengaja sederhana (sekitar 0,7 poin di bawah optimal, dinyatakan di `man`).
- Langkah berikutnya: M5b.

### 2026-09-29 — M4: mesin kartu, ekonomi chip, Blackjack
- Hasil uji M2b: sesuai, dengan perubahan label level (D-055): tanpa gelar resmi; 1 Pemula, 2 Menengah, 3 Mahir, 4 Kuat, 5 Ahli, 6 Sangat kuat (EN: Beginner … Very strong), perkiraan rating tetap tampil.
- Keputusan klien M4: D-056. Keputusan developer: D-057 (rincian aturan, strategi dasar, RTP), D-058 (chip di meja casino).
- Tes aturan dan pembayaran ditulis dan di-commit lebih dulu (commit 72a9716): 6 tes mesin kartu, 25 tes Blackjack (3:2, insurance 2:1, peek, S17, double termasuk setelah split, split sampai 4 tangan, as split satu kartu, 21 setelah split bukan blackjack, late surrender, titik potong 75%, berhenti, konservasi chip pada 40 shoe acak).
- Dikerjakan:
  - `kyusin_games::cards`: kartu, notasi `Ah`/`Td`, shoe beberapa dek dikocok RNG yang disuntikkan.
  - Blackjack: satu pertandingan = satu shoe (provably fair per shoe; verify memeriksa semua ronde). Tutorial dua bahasa (hit, stand, double, split, surrender; insurance dan provably fair dijelaskan) + `man` dengan RTP.
  - RTP: strategi dasar terdokumentasi; simulasi 1 miliar ronde 99,6388% → manifest 99,64%. CI setiap push 100 ribu ronde; workflow `rtp.yml` 10 juta ronde (manual + tiap Senin; run 36529816166: 99,6259%, toleransi ±0,1443%, lulus).
  - Chip: store skema 4 (saldo 10.000, tunjangan harian, ringkasan casino); host memeriksa saldo sebelum aksi berbiaya dan mencatat tiap ronde; sapaan `chips_low` dari saldo sebelum tunjangan lalu tunjangan disebut sesudahnya; halaman statistik "Melawan bandar".
  - UI: meja dengan sprite piksel kartu (peringkat, jenis, bingkai, sisi belakang), kotak tangan berukuran tetap dan kartu diposisikan absolut, kontrol chip + BAGI, HIT/STAND/DOUBLE/SPLIT/SURRENDER, INSURANCE/TOLAK, alasan tombol nonaktif di satu baris tetap; menu jeda casino dengan Berhenti.
- Tes: 175 tes Rust, 37 tes UI. Tes jendela asli (CI Windows, run 36560697511, semua job hijau): meja selaras (kotak tangan tetap, kartu 44×60, jarak 18px), 7 kontrol taruhan dan 5 kontrol aksi tanpa pergeseran saat hover/kursor, split tidak menggeser kotak mana pun, hit/stand/double/split dimainkan dengan saldo sesuai hasil ronde (10000 → 10060), tema P3/P4 selaras, berhenti membuka seed dan verify cocok, statistik melawan bandar tampil. Dua perbaikan dari CI: `resetToMenu` kini menunggu muat ulang dan daftar game (balapan), dan kotak tangan kosong berukuran sama dengan tangan terisi.
- Belum/sisa: meja Blackjack LAN banyak pemain (bersama milestone LAN); taruhan ronde yang sedang berjalan saat shoe ditunda belum mengurangi saldo sampai ronde selesai (D-058).
- Langkah berikutnya: M5 (casino meja kartu §6.3).

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
