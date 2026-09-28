# DECISIONS — KyuSin

Catatan keputusan dan perubahan arah. Perubahan arah hanya lewat klien (SPEC, pembuka). Entri tidak dihapus; keputusan yang diganti ditandai `digantikan oleh D-xxx`.

## Keputusan

D-001 s.d. D-011 adalah jawaban klien lewat SPEC Revisi 2 (27 Sep 2026) atas temuan sesi persiapan. "Temuan n" merujuk ke laporan sesi persiapan.

### D-001 — Seed gabungan commit-reveal
- Tanggal / milestone: 2026-09-27 / pra-M0
- Diputuskan oleh: klien (SPEC Rev. 2)
- Konteks: Temuan 1. Seed dari host saja membuat host bisa mengintip dan memilih seed.
- Keputusan: setiap peserta menyumbang seed 32 byte lewat commit-reveal; seed ronde = `SHA-256(seed_host ‖ seed lain urut id)`; semua seed dibuka setelah ronde dan diverifikasi otomatis. Batasan yang diterima: aplikasi host yang dimodifikasi bisa melihat kartu tertutup; dijelaskan di tutorial/`man` mode LAN. Singleplayer: aplikasi = host, pemain = peserta (seed boleh diisi manual).
- Rujukan: SPEC §2.3, §5.4, §10.

### D-002 — `ActionSpec`, `pending_players`, fase serentak
- Tanggal / milestone: 2026-09-27 / pra-M0
- Diputuskan oleh: klien (SPEC Rev. 2)
- Konteks: Temuan 2. Daftar perintah tetap tidak bisa mewakili taruhan berjumlah bebas, taruhan majemuk, atau aksi serentak.
- Keputusan: `legal_actions` mengembalikan `ActionSpec` (aksi tetap atau templat berparameter dengan min/maks/kelipatan); `apply(player, action)`; taruhan majemuk = beberapa `place …` lalu `done`; fase serentak lewat `pending_players()`, aksi tersembunyi sampai fase selesai.
- Rujukan: SPEC §5.2, §10.

### D-003 — `View: Serialize` untuk `state.data`
- Tanggal / milestone: 2026-09-27 / pra-M0
- Diputuskan oleh: klien (SPEC Rev. 2)
- Konteks: Temuan 3.
- Keputusan: `view_for(player) -> View` dengan `View: Serialize` menjadi `state.data`; `to_text(&View)` menjadi `state.text`.
- Rujukan: SPEC §5.2, §10.

### D-004 — Arcade dibagi menurut mekanik
- Tanggal / milestone: 2026-09-27 / pra-M0
- Diputuskan oleh: klien (SPEC Rev. 2)
- Konteks: Temuan 4.
- Keputusan: `TurnGame`: slot 3-reel, slot 5-reel, Plinko, Mines, Dice over/under, Tower, Limbo (`lan: false`, `agen: true`). `TickGame`: Crash, Pachinko, Pachislot, Coin pusher, Tembak ikan (tanpa LAN, tanpa agen). Hi-Lo tetap di §6.5.
- Rujukan: SPEC §6.6.

### D-005 — Fixture di M0
- Tanggal / milestone: 2026-09-27 / pra-M0
- Diputuskan oleh: klien (SPEC Rev. 2)
- Konteks: Temuan 5. Registry dan runner tutorial di M0 sementara kontrak baru ada di M1.
- Keputusan: M0 memakai game fixture minimal khusus tes (bukan game katalog); kontrak final dibuat di M1 dan fixture disesuaikan.
- Rujukan: SPEC §9 (M0).

### D-006 — Lisensi: pustaka sistem, `.deb`, npm
- Tanggal / milestone: 2026-09-27 / pra-M0
- Diputuskan oleh: klien (SPEC Rev. 2)
- Konteks: Temuan 6. webkit2gtk/GTK berlisensi LGPL; `cargo-deny` tidak mencakup npm.
- Keputusan: larangan GPL/LGPL/AGPL berlaku untuk semua yang dikompilasi/dibundel. Pengecualian: WebView2 (Windows) dan webkit2gtk/GTK (Linux) yang ditautkan dinamis dan tidak dibundel. Paket Linux hanya `.deb`, tanpa AppImage. CI memakai `cargo-deny` + pemeriksa lisensi npm produksi (alat berlisensi permisif, pilihan developer).
- Rujukan: SPEC §3, §9 (M0, M11).

### D-007 — Game antar-pemain tanpa rake
- Tanggal / milestone: 2026-09-27 / pra-M0
- Diputuskan oleh: klien (SPEC Rev. 2)
- Konteks: Temuan 7.
- Keputusan: Texas Hold'em, Omaha, Capsa Susun, Domino QiuQiu, Teen Patti tanpa rake, `rtp: null`, ditampilkan "antar-pemain, tanpa house edge"; lawan singleplayer = bot. DoD-nya: tes peringkat tangan, side pot, dan property test kekekalan chip.
- Rujukan: SPEC §6.3, §7.1.

### D-008 — Cakupan DoD dan toleransi RTP
- Tanggal / milestone: 2026-09-27 / pra-M0
- Diputuskan oleh: klien (SPEC Rev. 2)
- Konteks: Temuan 8.
- Keputusan: manifest punya `lawan` dan `kompetitif`. DoD poin 2 tidak berlaku untuk game solo. RTP casino ber-bandar dihitung analitis/enumerasi bila bisa; game berstrategi memakai strategi dasar/optimal yang didokumentasikan. Simulasi ≥100.000 ronde di CI tiap push, ≥10.000.000 ronde di workflow terjadwal/manual; toleransi 4·σ/√n. Tutorial `TickGame` memakai pemicu event + rekaman input ber-seed tetap. Daftar game kompetitif di §8; pertandingan dengan agen tidak dihitung ke rating.
- Rujukan: SPEC §5.2, §7, §8.

### D-009 — Angka ekonomi chip dan bandar LAN
- Tanggal / milestone: 2026-09-27 / pra-M0
- Diputuskan oleh: klien (SPEC Rev. 2)
- Konteks: Temuan 9.
- Keputusan: saldo awal 10.000; tunjangan sekali per hari kalender lokal (00:00 waktu sistem): bila saldo < 1.000 diisi menjadi 2.000. Meja LAN: buy-in ditetapkan host (bawaan 10.000, sama untuk semua), rebuy bawaan aktif dan bisa dimatikan. Bandar LAN = bandar sistem di aplikasi host dengan chip tak terbatas; host bermain sebagai pemain biasa.
- Rujukan: SPEC §6.7.

### D-010 — Stack dan rujukan Onsa
- Tanggal / milestone: 2026-09-27 / pra-M0
- Diputuskan oleh: klien (SPEC Rev. 2)
- Konteks: Temuan 10 (Onsa tidak diketahui).
- Keputusan: Tauri 2 + SvelteKit 2 + Svelte 5, versi stabil terbaru. Onsa adalah repo publik MufuyuMoku; boleh dibaca sebagai rujukan konvensi, tidak wajib disalin.
- Rujukan: SPEC §3.

### D-011 — Aktivasi endpoint dan `hint`
- Tanggal / milestone: 2026-09-27 / pra-M0
- Diputuskan oleh: klien (SPEC Rev. 2)
- Konteks: Temuan 10.
- Keputusan: server LAN aktif hanya saat menjadi host. Endpoint agen aktif hanya saat "Mode agen" dinyalakan, hanya mengikat `127.0.0.1`, dan memerlukan token dari pengaturan. `hint` hanya untuk agen.
- Rujukan: SPEC §10.

### D-012 — Aturan Dam: Inggris (English draughts)
- Tanggal / milestone: 2026-09-27 / pra-M0
- Diputuskan oleh: developer (diizinkan SPEC §6.1)
- Konteks: SPEC §6.1 meminta memilih aturan internasional atau Inggris.
- Keputusan: **aturan Inggris** — papan 8×8, 12 bidak per pemain, bidak biasa maju dan memakan hanya ke depan secara diagonal, raja bergerak satu petak (bukan raja terbang), makan wajib tetapi bebas memilih rangkaian makan (tidak wajib yang terbanyak), bidak yang menjadi raja di tengah rangkaian makan mengakhiri giliran. Hitam jalan duluan.
- Alasan:
  - Papan 8×8 sama dengan catur, jadi render grid karakter, kontrol seret, dan notasi koordinat bisa dipakai bersama; lebih pas di layar terminal daripada 10×10.
  - Aturan lebih sederhana (tanpa raja terbang dan tanpa kewajiban makan maksimum), sehingga mesin aturan, tes, dan tutorial lebih kecil dan lebih mudah diverifikasi.
  - Ruang pencarian lebih kecil, jadi tiga level bot alpha-beta yang jelas berbeda lebih mudah dicapai.
  - Aturan standar yang terdokumentasi baik (dipakai di kompetisi resmi Inggris/AS), sehingga tidak perlu mengarang varian.
- Alternatif yang ditolak: aturan internasional 10×10 (lebih berat untuk mesin dan tampilan, dengan kasus khusus makan maksimum dan raja terbang).
- Rujukan: SPEC §6.1.

<!--
### D-001 — judul singkat
- Tanggal / milestone:
- Diputuskan oleh: klien | developer (dalam batas SPEC)
- Konteks:
- Keputusan:
- Alternatif yang ditolak:
- Dampak:
-->

### D-013 — Aturan kegagalan provably fair (menutup Q-001)
- Tanggal / milestone: 2026-09-27 / pra-M0
- Diputuskan oleh: klien (SPEC Rev. 3)
- Konteks: Q-001. Host bisa mengaku seorang peserta gagal mengirim seed, sehingga seed ronde berubah dan host bisa memilih hasil.
- Keputusan: jaminannya dirumuskan ulang menjadi "tidak ada yang bisa memilih hasil; paling jauh host curang hanya bisa membatalkan ronde, dan pembatalan selalu terlihat". Gagal komitmen: peserta dikeluarkan dari ronde, dan ronde berjalan tanpa dia. Gagal pembukaan seed: ronde dibatalkan untuk semua pemain, taruhan dikembalikan, ronde berikutnya memakai komitmen baru, dan peserta yang dinyatakan gagal dikeluarkan dari meja sampai bergabung ulang. Setiap pembatalan tampil di log meja, riwayat, dan `verify` (nama peserta dan penghitung pembatalan per sesi); tiap klien mencatat waktu dia mengirim seed.
- Rujukan: SPEC §5.4.

### D-014 — Render canvas terpisah dari kontrak; `rtp` untuk semua casino melawan rumah (menutup Q-002)
- Tanggal / milestone: 2026-09-27 / pra-M0
- Diputuskan oleh: klien (SPEC Rev. 3)
- Konteks: Q-002.
- Keputusan: game beranimasi kontinu (slot, Plinko, Crash, pachinko, coin pusher, tembak ikan) dirender di canvas dengan grid karakter; itu soal render, sedangkan jenis kontrak mengikuti §6.6. `rtp` berupa angka untuk semua casino melawan rumah, baik ber-bandar maupun solo (slot, Keno, Bingo, kartu gosok); `null` untuk antar-pemain dan non-casino. Judul DoD §7.1 menjadi "Casino melawan rumah (`rtp` berupa angka)".
- Rujukan: SPEC §4, §5.2, §6.6, §7.1.

### D-015 — Fixture hanya untuk tes dan fitur build `fixture`
- Tanggal / milestone: 2026-09-27 / M0
- Diputuskan oleh: developer (dalam batas SPEC §9 M0)
- Konteks: registry dan runner diuji dengan game fixture yang bukan game katalog, tetapi menu, `man`, konsol, dan tutorial juga perlu dicek visual di M0.
- Keputusan: fixture (Nim, "Fixture: Batang") ada di `crates/games/src/fixture/`, dengan manifest berkategori `uji` dan tutorial di samping modulnya. Tes memakai `kyusin_games::with_fixture()`. Registry aplikasi (`builtin()`) kosong di M0; fixture hanya muncul bila dibangun dengan fitur `fixture` (`cargo tauri dev --features fixture`). Build rilis tidak memuatnya.
- Rujukan: SPEC §9 (M0), D-005.

### D-016 — Kontrak `TurnGame` sementara di M0 (digantikan oleh D-034)
- Tanggal / milestone: 2026-09-27 / M0
- Diputuskan oleh: developer
- Konteks: runner tutorial butuh antarmuka game sebelum kontrak final M1.
- Keputusan: `kyusin-core::game` berisi `TurnGame` sementara yang sudah memakai nama dan bentuk SPEC §5.2 (`pending_players`, `legal_actions -> Vec<ActionSpec>`, `apply(player, action)`, `view_for -> View: Serialize`, `to_text`, `parse_command`, `format_action`), ditambah `Session` tanpa tipe konkret yang menerima perintah teks. RNG yang disuntikkan, replay, dan hasil akhir final ditetapkan di M1.
- Rujukan: SPEC §5.2, §9.

### D-017 — Format tutorial dan manifest
- Tanggal / milestone: 2026-09-27 / M0
- Diputuskan oleh: developer
- Keputusan: manifest = `manifest.toml` di modul game dengan kunci persis SPEC §5.2 (`nama`, `kategori`, `pemain_min`, `pemain_maks`, `jenis`, `lawan`, `kompetitif`, `lan`, `agen`, `rtp` dalam persen, `tutorial`, `[[perintah]]`). Validasi di registry: aturan `rtp` D-014, kompetitif ⇒ lawan bot, LAN/agen ⇒ giliran, path tutorial `tutorials/<id>.toml`. Tutorial TOML: `seed`, `config`, `pemain`, `[man]` (`aturan`, `kontrol` untuk halaman `man`), dan `[[langkah]]` (`teks`, `sebelum`, `aksi`, `sorot`, `petunjuk`). Keadaan awal = konfigurasi + seed + perintah `sebelum`, jadi tutorial selalu diputar terhadap mesin aturan asli. Langkah beraksi wajib punya `petunjuk` dan `sorot`; target `aksi:<perintah>` harus aksi yang sah. Isi tutorial ditanam saat kompilasi; tes memastikan sama dengan berkas di repo, dan gagal bila ada tutorial tanpa game.
- Rujukan: SPEC §5.2, §7.6, §7.7; format lengkap di `tutorials/README.md`.

### D-018 — Pengaturan tampilan di localStorage sampai M3
- Tanggal / milestone: 2026-09-27 / M0
- Diputuskan oleh: developer
- Konteks: SQLite (`store`) baru berisi profil di M3.
- Keputusan: tema, efek CRT, konsol selalu tampil, dan boot disimpan di localStorage jendela; riwayat konsol juga (maks. 100 baris). Pindah ke profil SQLite di M3.
- Rujukan: SPEC §4, §9.

### D-019 — Perintah global konsol dan perilaku tampilan (efek dan reduced motion digantikan oleh D-025, D-026)
- Tanggal / milestone: 2026-09-27 / M0
- Diputuskan oleh: developer
- Keputusan: perintah global bergaya terminal: `help`, `ls`, `man`, `tutorial`, `menu`, `back`, `settings`, `theme`, `fx`, `clear`. Perintah game dan daftar id datang dari manifest dan `legal_actions`. Efek bawaan: scanline, glow, dan lengkungan menyala; flicker mati. Saat reduced motion: semua efek CRT mati dan urutan boot dilewati. Suara belum ada di M0 (tidak termasuk isi M0).
- Rujukan: SPEC §4.

### D-020 — Pemeriksa lisensi npm membaca isi bundle
- Tanggal / milestone: 2026-09-27 / M0
- Diputuskan oleh: developer (alat pilihan developer, SPEC §3)
- Konteks: `npm query .prod` ikut menghitung peer dependency SvelteKit (Vite dan `lightningcss` MPL-2.0), padahal alat build tidak dikirim.
- Keputusan: plugin Vite kecil mencatat paket npm yang benar-benar masuk bundle klien; `scripts/check-npm-licenses.mjs` (tanpa dependensi tambahan) memeriksa lisensi daftar itu terhadap daftar izin. Saat ini: `svelte`, `@sveltejs/kit`, `@tauri-apps/api` (MIT / Apache-2.0 OR MIT).
- Rujukan: SPEC §3, D-006.

### D-021 — MPL-2.0 diizinkan di `deny.toml` (digantikan oleh D-022)
- Tanggal / milestone: 2026-09-27 / M0
- Diputuskan oleh: developer, sementara; lihat Q-003
- Konteks: Tauri 2 membawa crate MPL-2.0 (`cssparser`, `cssparser-macros`, `selectors`, `dtoa-short` lewat `tauri-utils`; `option-ext` lewat `dirs`). SPEC §3 tidak menyebut MPL-2.0 di daftar izin, tetapi juga tidak melarangnya; yang dilarang GPL/LGPL/AGPL.
- Keputusan: MPL-2.0 diizinkan sebagai lisensi "setara". Kewajibannya per berkas (hanya berkas MPL yang diubah yang harus dibuka) dan tidak menjangkau kode KyuSin; kami tidak mengubah crate tersebut. Ini sama dengan kebijakan `deny.toml` Onsa. Tanpa ini, stack wajib SPEC §3 tidak bisa dipakai.
- Rujukan: SPEC §3, Q-003.

### D-022 — MPL-2.0 diizinkan selama berkasnya tidak diubah (menutup Q-003)
- Tanggal / milestone: 2026-09-27 / M0b
- Diputuskan oleh: klien (SPEC Rev. 4)
- Keputusan: MPL-2.0 diizinkan selama berkas MPL tidak diubah; copyleft-nya per berkas dan tidak menjangkau kode KyuSin. Bila suatu saat berkas MPL perlu diubah, berhenti dan tanya klien. `deny.toml` tetap mengizinkan MPL-2.0 dengan catatan ini.
- Rujukan: SPEC §3, D-021, Q-003.

### D-023 — Halaman Lisensi pihak ketiga di M11
- Tanggal / milestone: 2026-09-27 / M0b
- Diputuskan oleh: klien (SPEC Rev. 4)
- Keputusan: M11 menambah halaman "Lisensi pihak ketiga" yang dibangkitkan otomatis dari dependensi, memenuhi atribusi MIT/Apache/BSD dan pemberitahuan sumber MPL-2.0.
- Rujukan: SPEC §9 (M11).

### D-024 — Larangan otomasi input tingkat OS
- Tanggal / milestone: 2026-09-27 / M0b
- Diputuskan oleh: klien (SPEC Rev. 4)
- Konteks: di M0 developer sempat memakai SendKeys ke jendela Tauri; input masuk ke jendela lain.
- Keputusan: dilarang SendKeys, xdotool, dan sejenisnya. UI diuji lewat browser dengan backend tiruan dan tes otomatis; pengecekan di jendela Tauri asli dilakukan klien.
- Rujukan: SPEC §11.

### D-025 — Efek CRT samar menyala secara bawaan + slider intensitas
- Tanggal / milestone: 2026-09-27 / M0b
- Diputuskan oleh: klien (SPEC Rev. 5)
- Keputusan: efek CRT adalah bumbu di atas tema fosfor. Scanline, glow, dan lengkungan menyala secara bawaan dengan intensitas samar yang tidak mengganggu keterbacaan; flicker mati. Ada satu slider intensitas 0–100% dengan bawaan rendah, selain toggle per efek.
- Rujukan: SPEC §4.

### D-026 — Reduced motion hanya mematikan efek bergerak; checkbox jujur
- Tanggal / milestone: 2026-09-27 / M0b
- Diputuskan oleh: klien (SPEC Rev. 5)
- Keputusan: saat reduced motion, hanya efek bergerak yang mati (flicker, scanline bergulir, animasi ketik boot); efek statis mengikuti pengaturan pemain. Urutan boot tetap tampil, teksnya langsung tanpa animasi ketik. Checkbox selalu mencerminkan keadaan sebenarnya; efek yang dipaksa mati tampil nonaktif dengan alasannya.
- Rujukan: SPEC §4.

### D-027 — Dua bahasa: Indonesia dan Inggris
- Tanggal / milestone: 2026-09-27 / M0b
- Diputuskan oleh: klien (SPEC Rev. 5)
- Keputusan: pilihan bahasa di pengaturan; bawaan Indonesia bila sistem berbahasa Indonesia, selain itu Inggris. Semua teks UI, tutorial, `man`, dan pesan kesalahan tersedia dalam dua bahasa lewat berkas terjemahan. Kata perintah tetap Inggris di kedua bahasa; deskripsinya diterjemahkan. CI gagal bila ada kunci terjemahan yang hilang, termasuk di tutorial. Protokol: klien menyebut bahasa `state.text` (`id`/`en`, bawaan `en`) saat terhubung.
- Rujukan: SPEC §4, §7.6, §10.

### D-028 — `to_text` menerima bahasa (disahkan klien, D-030)
- Tanggal / milestone: 2026-09-27 / M0b
- Diputuskan oleh: developer (kontrak masih sementara sampai M1, D-016)
- Konteks: SPEC §5.2 menulis `to_text(&View)`, sedangkan §10 meminta `state.text` dalam bahasa pilihan klien.
- Keputusan: `to_text(&View, Lang)`; `Session::view_text(player, Lang)`. `View` (data terstruktur) tidak bergantung bahasa.
- Rujukan: SPEC §5.2, §10, D-027.

### D-029 — Letak terjemahan dan pemeriksaannya
- Tanggal / milestone: 2026-09-27 / M0b
- Diputuskan oleh: developer
- Keputusan:
  - UI: `ui/src/lib/i18n/{id,en}.json`, dipakai lewat `t(kunci)`; teks dari backend dipilih dengan `L({ id, en })`. Bahasa bawaan dari `navigator.languages` (Indonesia bila `id*`, selain itu Inggris); pilihan pemain disimpan di pengaturan. Perintah konsol baru `lang <id|en>`.
  - Rust: teks inti di `crates/core/i18n.toml` (`[id]`/`[en]`), teks tampilan per game di katalog game itu (fixture: `crates/games/src/fixture/i18n.toml`). Manifest (`nama`, `ringkas`) dan tutorial (`judul`, `man`, `teks`, `petunjuk`) berupa tabel `{ id, en }` yang keduanya wajib. Backend mengirim semua teks pemain, termasuk pesan kesalahan, dalam dua bahasa, jadi UI bisa berganti bahasa tanpa memanggil ulang.
  - Pesan untuk penulis tutorial dan manifest (hanya muncul di CI) tetap satu bahasa.
  - CI gagal bila: kunci hanya ada di satu bahasa (UI, katalog Rust), kode UI memakai kunci yang tidak ada, markup Svelte berisi teks mentah (dengan uji-diri pemeriksanya), tutorial/manifest kehilangan salah satu bahasa atau berisi teks kosong, atau halaman `man` memuat kunci mentah.
- Rujukan: SPEC §4, §7.6, D-027.

### D-030 — Revisi 6: `to_text(&View, Lang)` disahkan; hasil uji M0b
- Tanggal / milestone: 2026-09-27 / M0c
- Diputuskan oleh: klien (SPEC Rev. 6, Rev. 7)
- Keputusan: D-028 disahkan; SPEC §5.2 kini menulis `to_text(&View, Lang)`. Hasil uji M0b dinyatakan sesuai, dan intensitas bawaan efek CRT 30% (D-025) disahkan.
- Rujukan: SPEC §4, §5.2.

### D-031 — Revisi 7: tiga mode urutan boot
- Tanggal / milestone: 2026-09-27 / M0c
- Diputuskan oleh: klien (SPEC Rev. 7)
- Keputusan: pengaturan boot kini memilih satu dari **Verbose**, **Sinematik** (bawaan), **Sapaan**, atau **Mati** (menggantikan toggle boot D-019).
  - Verbose: banyak baris cepat ala booting Linux (stempel waktu, `[  OK  ]`) yang mencerminkan startup sungguhan; baris hiasan tidak boleh mengaku melakukan hal yang tidak terjadi.
  - Sinematik: beberapa baris diketik perlahan dengan jeda dramatis, diakhiri nama KyuSin.
  - Sapaan: menyapa pemain langsung, nada santai dan sedikit usil, dipilih dari kumpulan teks bersyarat berbasis data lokal (jam sistem, jeda sejak sesi terakhir; nama profil, game terakhir, dan saldo chip setelah ada di M3/M4). Syarat yang datanya belum ada dilewati otomatis. Tanpa AI, tanpa data keluar perangkat, tidak mengulang teks dua sesi berturut-turut, selalu ada cadangan umum.
  - Semua mode bisa dilewati; saat reduced motion teks tampil langsung; semua teks dua bahasa.
- Rujukan: SPEC §4, §9 (M0c).

### D-032 — Rincian penerapan tiga mode boot
- Tanggal / milestone: 2026-09-27 / M0c
- Diputuskan oleh: developer (dalam batas D-031)
- Keputusan:
  - Pengaturan `bootMode` (`verbose`/`cinematic`/`greeting`/`off`, bawaan `cinematic`) menggantikan toggle `boot`; pengaturan lama dengan boot mati dimigrasikan menjadi `off`.
  - Verbose menunggu startup selesai (maks. 3 detik) lalu hanya melaporkan hal yang benar-benar terjadi, diurutkan menurut waktu kejadian (`performance.now()` saat langkah selesai): pembacaan pengaturan, tema, efek dan intensitas, reduced motion, bahasa dan sumbernya, antarmuka, backend (Tauri atau tiruan dev), lokasi data dari Tauri (`app_data_dir`; kosong di peramban), sesi sebelumnya, registry dan tiap cartridge, font yang benar-benar termuat (`document.fonts.load`), lalu siap. Langkah yang gagal tampil `[FAILED]`.
  - Sinematik: label proyek, tiga baris diketik perlahan (jumlah cartridge, tema dan bahasa, siap), diakhiri nama KyuSin.
  - Sapaan: 14 sapaan bersyarat dengan prioritas (sesi pertama, lama tidak datang, baru saja pergi, larut malam, pagi sekali, jam makan siang; chip rendah/tinggi, game terakhir, nama profil untuk M3/M4; empat cadangan umum). Dipilih yang syaratnya terpenuhi dan prioritasnya tertinggi, acak bila setara, tidak sama dengan sapaan sesi sebelumnya. Data M3/M4 dibaca dari `app_info.profile` yang belum dikirim backend, jadi sapaan itu aktif sendiri begitu backend mengirimnya.
  - Catatan sesi (waktu sesi terakhir, sapaan terakhir) disimpan di penyimpanan jendela (`kyusin.session.v1`), seperti pengaturan (D-018).
  - Tombol atau klik yang melewati boot "ditelan" supaya tidak ikut memilih tombol menu yang baru muncul.
- Rujukan: SPEC §4, D-031.

### D-033 — Kontrol nonaktif tetap bisa difokus; navigasi bersama
- Tanggal / milestone: 2026-09-27 / perbaikan M0c (sebelum M1)
- Diputuskan oleh: klien (hasil uji M0c) dan developer (penerapan)
- Konteks: dengan efek animasi Windows mati, flicker tampil nonaktif lewat atribut `disabled` asli. Elemen seperti itu menolak `focus()`, sehingga navigasi panah di pengaturan macet di "Lengkungan layar". Tombol intensitas `[-]`/`[+]` di 0%/100% punya masalah yang sama.
- Keputusan:
  - Semua kontrol yang bisa dipilih memakai komponen bersama `NavButton`. Nonaktif = `aria-disabled="true"` (bukan `disabled`): tetap bisa difokus, alasannya tampil dan dibacakan lewat `aria-describedby`, klik/Enter/Spasi tidak berefek. Saat fokus, item nonaktif ditandai garis putus-putus, bukan blok terbalik.
  - Navigasi panah memakai `lib/nav.ts` (`moveFocus`, melingkar) untuk semua layar, dan melewati item nonaktif seperti item lain.
  - Tes regresi (`ui/src/lib/nav.test.ts`): model fokus yang meniru peramban (`disabled` asli menolak fokus) menjelajahi daftar 1–7 item turun, naik, dan turun-lalu-naik untuk setiap kombinasi item nonaktif; model yang sama membuktikan bug lama muncul dengan `disabled` asli. Tes sumber memastikan tidak ada `<button>` mentah, `data-nav` di luar `NavButton`, atau atribut `disabled` asli di markup.
- Rujukan: SPEC §4 (aksesibilitas keyboard), D-026.

### D-034 — Kontrak final M1: TurnGame, Player, RNG, provably fair, replay
- Tanggal / milestone: 2026-09-28 / M1
- Diputuskan oleh: developer (dalam batas SPEC §5)
- Keputusan:
  - `TurnGame` (menggantikan kontrak sementara D-016): `new(config, seed) -> Result` (konfigurasi bisa ditolak), `seats()`, `pending_players`, `legal_actions -> Vec<ActionSpec>`, `apply(player, action)`, `view_for -> View: Serialize`, `to_text(&View, Lang)`, `is_over`, `result`, `parse_command`, `format_action`. Keadaan game wajib `Serialize`; `Session::state_hash` = SHA-256 JSON keadaan, dipakai `verify` dan replay.
  - `GameResult`: kursi pemenang (kosong = seri), skor per kursi, ringkasan dua bahasa. Chip menyusul di M4.
  - RNG: `GameRng` membungkus ChaCha20 dengan seed eksplisit; keadaannya (seed + posisi) bisa diserialisasi. Aliran acak terpisah (misalnya bot per kursi) memakai `derive(seed, label)` = SHA-256(seed ‖ label).
  - Provably fair (`core::fair`): mesin keadaan commit-reveal murni sesuai §5.4 dan D-013 (gagal komitmen = dikeluarkan; gagal pembukaan = ronde dibatalkan, nama peserta dicatat). `FairRecord::verify` memeriksa semua komitmen dan menghitung ulang seed ronde. Batas waktu dan jaringan di M9.
  - `Player`: `Human` (menunggu UI), bot per game (crate `bots`), `Remote` di M9. `Match` menyatukan sesi, pemain, langkah, dan catatan fair, lalu mengekspor `Replay`.
  - `Replay` (format 1): konfigurasi, jenis kursi, catatan fair, urutan perintah, hasil, hash keadaan akhir. `Replay::verify` memeriksa komitmen, seed ronde, legalitas setiap langkah saat diputar ulang, hasil, dan hash; `frames` menghasilkan keadaan per langkah untuk penampil.
- Rujukan: SPEC §2.3, §2.5, §5.2–§5.4.

### D-035 — Glyph UI hanya dari font yang dibundel (papan: digantikan oleh D-038)
- Tanggal / milestone: 2026-09-28 / M1
- Diputuskan oleh: developer
- Konteks: IBM Plex Mono punya box-drawing dan blok (`█ ░ ▐ ▌`), tetapi tidak punya `● ○ ■ □ ▸ ✗`. Peramban mengambil glyph itu dari font cadangan dengan lebar berbeda, sehingga papan Reversi tidak lurus.
- Keputusan: bidak hitam `█`, putih `░`, langkah sah `·`, langkah terakhir `[█]`; kotak centang `[█]`, pilihan `(•)`, penanda menu `›`, gagal verify `[×]`, bar intensitas `█░`. Tes CI (`ui/src/lib/fonts.test.ts`) membaca tabel `cmap` font yang dibundel dan gagal bila teks UI, terjemahan, tutorial, manifest, atau teks tampilan game memakai karakter di luarnya.
- Rujukan: SPEC §3 (font dibundel), §4 (grid karakter).

### D-036 — Reversi: notasi, bot, dan tampilan (tampilan papan digantikan oleh D-038)
- Tanggal / milestone: 2026-09-28 / M1
- Diputuskan oleh: developer
- Keputusan:
  - Aturan standar Othello: posisi awal d4/e5 putih, d5/e4 hitam; hitam (kursi 0) jalan duluan; pass wajib dan hanya saat tidak ada langkah; selesai bila kedua pihak tidak bisa melangkah. Diuji dengan perft kedalaman 1–6 (4, 12, 56, 244, 1396, 8200).
  - Perintah teks: petak `a1`..`h8` (huruf kecil, baris 1 di atas) dan `pass`. Konfigurasi opsional `posisi` (8 baris `.XO`) dan `giliran` untuk keadaan awal kustom (tutorial dan tes).
  - Bot: level 1 langkah sah acak; level 2 satu langkah ke depan dengan bobot petak; level 3 alpha-beta 4 langkah (bobot + mobilitas) dan hitungan eksak bila petak kosong ≤ 8. Uji kekuatan saat ini: level 2 menang 18/20 atas level 1; level 3 menang 10/10 atas level 1 dan 10/10 atas level 2. Acak bot dari RNG turunan seed ronde.
  - UI: papan satu kontrol ber-role grid (klik sel, atau panah + Enter/Spasi), sorotan tutorial di sel, animasi balik singkat yang mati saat reduced motion.
- Rujukan: SPEC §6.1, §7, §8.

### D-037 — Pertandingan singleplayer, penyimpanan replay, verify otomatis
- Tanggal / milestone: 2026-09-28 / M1
- Diputuskan oleh: developer (dalam batas SPEC §5.4, §2.5)
- Keputusan:
  - Singleplayer: host = aplikasi (seed dari sumber acak OS), peserta = pemain lokal (seed dari pengaturan "Seed pemain" bila diisi hex 64, selain itu acak). Komitmen keduanya tampil sejak langkah pertama; seed dan seed ronde dibuka setelah selesai, lalu `verify` dijalankan otomatis dan setiap pemeriksaannya ditampilkan.
  - Replay disimpan di SQLite (`kyusin-store`, berkas `kyusin.sqlite` di folder data aplikasi) untuk setiap pertandingan, termasuk yang ditinggalkan di tengah jalan. Waktu mulai dari jam dinding aplikasi, bukan dari game.
  - UI: layar game berisi pilihan lawan (level bot) dan posisi, daftar replay terakhir, dan `man`; layar pertandingan dengan panel provably fair dan daftar langkah; penampil replay langkah demi langkah dengan verify. Perintah konsol baru: `play <id> [level]`, `verify`.
  - Boot Verbose menampilkan lokasi basis data dan apakah berhasil dibuka.
- Rujukan: SPEC §2.3, §2.5, §5.4, §6.7 (chip menyusul M4).

### D-038 — Revisi 8: papan grid CSS/SVG, sprite piksel, tes jendela asli
- Tanggal / milestone: 2026-09-28 / M1b
- Diputuskan oleh: klien (SPEC Rev. 8, hasil uji M1)
- Konteks: di jendela Tauri asli, baris papan Reversi tempat kursor berada bergeser ke kiri dan garis vertikal putus di antara baris. Pengecekan di peramban dengan tiruan tidak menangkapnya.
- Keputusan:
  - Papan dan meja game digambar sebagai grid CSS atau SVG bergaya terminal (sel ukuran tetap, garis 1px warna fosfor, koordinat monospace), bukan baris teks box-drawing. Box-drawing tetap boleh untuk teks, menu, dan bingkai panel.
  - Bidak, kartu, dan simbol game berupa sprite piksel SVG buatan sendiri yang diwarnai token tema. Bidak terang diisi penuh; bidak gelap berupa garis tepi atau redup; keduanya jelas berbeda dari petak kosong di ketiga tema.
  - Ditambahkan ke §4 atas permintaan klien: kursor keyboard, sorotan, hover, dan penanda langkah legal hanya berupa lapisan di atas sel dan tidak boleh mengubah ukuran atau posisi sel mana pun.
  - `tauri-driver` (WebDriver ke aplikasi, bukan ke OS) diizinkan. Setiap layar game wajib punya tes jendela asli yang memeriksa keselarasan dan menyimpan tangkapan layar sebagai artefak. Untuk papan: kursor digerakkan ke setiap sel lewat keyboard dan hover, lalu posisi serta ukuran semua sel harus tetap sama persis.
  - Komponen papan bersama untuk papan berpetak berikutnya (catur, dam).
- Rujukan: SPEC §4, §9 (M1b), §11.

### D-039 — Sapaan tambahan dan bentuk jamak
- Tanggal / milestone: 2026-09-28 / M1b
- Diputuskan oleh: klien
- Keputusan:
  - generic_4 tetap. Sapaan baru (pola syarat sama dengan yang lain): `evening` (prio 1, jam 18:00–21:59), `weekend` (prio 1, Sabtu/Minggu), `streak` (prio 2, sesi di ≥3 hari kalender berturut-turut, parameter `{n}` = jumlah hari), `generic_5`, `generic_6`, `generic_7` (prio 0). Teks persis seperti yang diberikan klien.
  - Bentuk tunggal/jamak bahasa Inggris ditangani untuk semua sapaan berangka (dan teks berangka lain yang ikut terdampak, misalnya "1 minute ago"), memakai aturan jamak bahasa (`Intl.PluralRules`). Bahasa Indonesia tidak berubah bentuk.
  - Catatan untuk M4: `chips_low` dipilih dari saldo **sebelum** tunjangan harian, lalu tunjangannya disebut sesudahnya dengan nada yang sama.
- Rujukan: SPEC §4, D-031, D-032.

### D-040 — Aplikasi mengikuti folder data dari otomasi dan dari variabel uji
- Tanggal / milestone: 2026-09-28 / M1b
- Diputuskan oleh: developer
- Konteks: tes tauri-driver pertama di CI gagal dengan "DevToolsActivePort file doesn't exist". msedgedriver mengeset `WEBVIEW2_USER_DATA_FOLDER` dan menunggu berkas itu di sana, tetapi Tauri memberi WebView2 folder datanya sendiri secara eksplisit, yang menang atas variabel tersebut. Dibuktikan lokal: port DevTools terbuka, berkasnya ditulis di folder aplikasi.
- Keputusan:
  - Jendela utama dibuat di `setup` (bukan otomatis dari konfigurasi); bila `WEBVIEW2_USER_DATA_FOLDER` diset, folder itu dipakai sebagai folder data WebView. Basis data SQLite bisa dialihkan dengan `KYUSIN_DATA_DIR`. Pemakaian biasa tidak berubah.
  - Membiarkan msedgedriver meluncurkan aplikasi tetap gagal di CI (folder profilnya tidak sampai ke WebView2). Karena itu runner tes menjalankan aplikasi sendiri dengan `WEBVIEW2_ADDITIONAL_BROWSER_ARGUMENTS=--remote-debugging-port=9222` dan folder data sementara, lalu membuat sesi lewat tauri-driver yang meneruskan `ms:edgeOptions.debuggerAddress` ke msedgedriver (cara menempel ke WebView2 yang didokumentasikan Microsoft). Di runner CI, argumen yang dikirim wry lewat API menimpa variabel `WEBVIEW2_ADDITIONAL_BROWSER_ARGUMENTS` (baris perintah WebView2 tanpa `--remote-debugging-port`), jadi aplikasi menggabungkan isi variabel itu secara eksplisit dengan argumen bawaan wry. Port dibaca dari berkas `DevToolsActivePort`. Perintah tetap hanya masuk ke jendela KyuSin, dan tes tidak menyentuh pengaturan maupun replay pemain.
- Rujukan: SPEC §11, D-038.

### D-041 — Keputusan klien untuk M2 (catur)
- Tanggal / milestone: 2026-09-28 / M2
- Diputuskan oleh: klien (hasil uji M1b sesuai; tes jendela asli cukup di CI, tanpa msedgedriver lokal)
- Keputusan:
  1. Bidak catur berupa sprite piksel SVG (§4): enam jenis jelas terbedakan di sel 40×40; putih penuh, hitam bergaris tepi (konsisten dengan Reversi); terbaca di ketiga tema.
  2. Input visual: klik-pilih-lalu-klik-tujuan dan seret-lepas, keduanya berfungsi; langkah sah disorot saat bidak dipilih; promosi lewat pilihan visual; rokade, en passant, dan promosi tampil jelas.
  3. Kalibrasi level terhadap Stockfish berjalan sebagai workflow GitHub Actions manual (`workflow_dispatch`) yang mengunduh Stockfish saat berjalan. Stockfish tidak masuk repo dan tidak dijalankan di mesin lokal. Hasil kalibrasi disimpan sebagai data di repo.
  4. Tes jendela asli (§11): keselarasan papan, kursor/hover/seret tidak menggeser sel, dan satu partai pendek melawan bot.
- Rujukan: SPEC §3, §4, §6.1, §8, §9 (M2), §11.

## Pertanyaan terbuka

### Q-001 — Host dapat mengeluarkan peserta setelah melihat seed-nya
- Status: terjawab (lihat D-013).

### Q-002 — Kedudukan slot dan game solo casino dalam manifest
- Status: terjawab (lihat D-014).

### Q-003 — Konfirmasi MPL-2.0 sebagai lisensi "setara"
- Status: terjawab (lihat D-022).

## Usulan

Ide di luar SPEC (SPEC §11). Tidak dikerjakan sebelum disetujui klien.

<!--
### U-001 — judul singkat
- Diusulkan:
- Alasan:
- Status: menunggu | diterima (D-xxx) | ditolak
-->
