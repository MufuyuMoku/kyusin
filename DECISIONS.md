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

### D-042 — Catur: aturan, notasi, dan tambahan kontrak `canonical`
- Tanggal / milestone: 2026-09-28 / M2
- Diputuskan oleh: developer (dalam batas SPEC §3, §5.2, §6.1)
- Keputusan:
  - Generator langkah cozy-chess (MIT). Rokade dari sandi cozy-chess ("raja ke petak benteng") diterjemahkan ke bentuk standar (raja e1→g1, SAN `O-O`).
  - Notasi kanonik = SAN (`e4`, `Nbd2`, `exd6`, `e8=Q`, `O-O`, dengan `+`/`#`) di `legal_actions`, riwayat, replay, dan PGN. Notasi koordinat (`e2e4`, `e7e8q`, `e1g1`), `0-0`, dan promosi tanpa `=` diterima sebagai alias.
  - Kontrak mendapat satu metode opsional `TurnGame::canonical` (bawaan: tidak ada alias). `Session::act` mencoba alias bila perintah tidak cocok langsung; `Match` menyimpan bentuk kanonik di replay; runner tutorial membandingkan bentuk kanonik. Kompatibel dengan game lama.
  - Remis otomatis (penyederhanaan dari aturan "klaim" FIDE): pat, posisi sama tiga kali, 50 langkah (halfmove ≥ 100), bahan tidak cukup (tanpa pion/benteng/menteri dan ≤1 bidak ringan, atau semua gajah di warna petak yang sama). `resign` untuk menyerah. `timeout` (hanya bila ada jam): kalah, kecuali lawan tidak mungkin mat lagi (tinggal raja, atau raja + satu bidak ringan melawan raja sendirian) = remis.
  - Skor `GameResult` catur = poin × 2 (menang 2, remis 1, kalah 0), untuk Glicko-2 di M3.
  - Posisi awal kustom lewat konfigurasi `fen` (dipakai tes dan impor PGN).
- Rujukan: SPEC §5.2, §6.1, D-034.

### D-043 — Sprite dan input visual catur
- Tanggal / milestone: 2026-09-28 / M2
- Diputuskan oleh: developer (dalam batas D-041)
- Keputusan:
  - Enam bidak 12×12 digambar tangan (dilipatkan 2 → 24px di sel 40×40). Bidak putih diisi penuh; bidak hitam = garis tepi yang dibangkitkan otomatis dari bentuk penuhnya (piksel yang bertetangga dengan kosong), jadi selalu konsisten. Tes UI memastikan keenam bentuk berbeda (penuh maupun garis tepi, beda ≥ 8 piksel antarbidak) dan versi garis tepi jelas lebih kosong.
  - `GridBoard` diperluas: sasaran (titik) terpisah dari petak yang bisa dipilih, petak terpilih (garis ganda), dua penanda langkah terakhir, penanda skak di raja, sasaran seret. Semua lapisan absolut. Bidak yang diseret digambar di lapisan `position: fixed` terpisah; bidak asalnya hanya diredupkan.
  - Klik bidak → titik di tujuan sah → klik tujuan; seret bidak → lepas di tujuan; keyboard: panah + Enter/Spasi dengan urutan yang sama. Promosi memunculkan empat pilihan bergambar (menteri, benteng, gajah, kuda) + batal. Papan dibalik bila pemain memegang hitam.
  - Status menjelaskan langkah khusus terakhir: rokade pendek/panjang (raja dan benteng pindah), en passant (pion yang ditangkap disebut petaknya), promosi (bidak barunya disebut). SAN di daftar langkah tetap standar.
- Rujukan: SPEC §4, D-038, D-041.

### D-044 — Mesin catur dan level
- Tanggal / milestone: 2026-09-28 / M2
- Diputuskan oleh: developer
- Keputusan: negamax alpha-beta + iterative deepening, quiescence untuk tangkapan/promosi (maks. 8 ply), urutan MVV-LVA, evaluasi bahan + tabel posisi (tabel raja akhir permainan terpisah). Empat level dibatasi kedalaman dan jumlah node (bukan waktu), dengan gangguan acak pada langkah akar dari RNG turunan seed ronde: level 1 kedalaman 1 tanpa quiescence (±150 cp), level 2 kedalaman 2 (±40), level 3 kedalaman 3 (±10), level 4 kedalaman 4 (tanpa gangguan, maks. 600 ribu node). Uji kekuatan di CI: level 2 vs 1 = 8/8, level 3 vs 2 = 7,5/8, level 4 vs 3 = 5,5/6. `kyusin-bots` dan cozy-chess dikompilasi teroptimasi juga di profil dev/test.
- Rujukan: SPEC §6.1, §7.2, §8.

### D-045 — Jam catur dihitung host
- Tanggal / milestone: 2026-09-28 / M2
- Diputuskan oleh: developer
- Keputusan: game tidak memakai jam dinding (SPEC §5.2). Jam dihitung host dengan jam monoton: waktu pemain yang melangkah dikurangi, lalu ditambah tambahan per langkah. Waktu habis → host mengajukan `timeout` atas nama pemain itu (juga saat pemain mencoba melangkah setelah waktunya habis), jadi tercatat di replay dan lolos verify. UI hanya berdetak dari potret terakhir dan meminta host memeriksa saat mencapai nol. Pilihan: tanpa jam, 5+0, 10+5, 15+10. Konfigurasi `jam` ikut di replay dan menjadi tag `TimeControl` PGN.
- Rujukan: SPEC §6.1.

### D-046 — PGN dan kalibrasi
- Tanggal / milestone: 2026-09-28 / M2
- Diputuskan oleh: developer (dalam batas D-041)
- Keputusan:
  - Ekspor PGN dari replay catur tersimpan (tag tujuh baku, FEN/SetUp bila perlu, TimeControl bila ada jam, nama kursi dalam bahasa aktif), ditampilkan dan bisa disalin. Impor PGN (toleran terhadap komentar, variasi, NAG, nomor langkah) membuka partai di penampil replay tanpa disimpan; setiap langkah diperiksa terhadap mesin aturan dan galat menyebut langkah yang salah. Partai impor tidak punya catatan provably fair dan tidak di-verify.
  - Kalibrasi: `crates/bots/examples/calibrate_catur.rs` bicara UCI dengan Stockfish (`UCI_LimitStrength` + `UCI_Elo`), memainkan setiap level dari delapan pembukaan pendek dengan warna bergantian, lalu menaksir rating tiap level dengan kemungkinan maksimum model Elo logistik plus selang 95%. Bila semua kalah atau semua menang, ditambah satu remis semu per lawan dan ditandai ekstrapolasi. Workflow manual `.github/workflows/calibrate.yml` mengunduh rilis Stockfish resmi ke folder sementara runner. Hasil disimpan di `data/calibration/catur.json`, ditanam ke `kyusin-bots`, dan ditampilkan di pilihan level ("≈rating").
- Rujukan: SPEC §6.1, §8, D-041.

### D-047 — Hasil kalibrasi pertama dan rating ekstrapolasi
- Tanggal / milestone: 2026-09-28 / M2
- Diputuskan oleh: developer
- Keputusan: kalibrasi pertama (Stockfish 19, `UCI_Elo` 1320/1500/1700/1900, 16 partai per pasangan, 100 ms/langkah) disimpan apa adanya di `data/calibration/catur.json`. Level 1 kalah semua (0/64) melawan tingkat terendah Stockfish, jadi taksirannya murni ekstrapolasi. Rating ekstrapolasi tidak ditampilkan di pilihan level; yang ditampilkan hanya level 2–4 (≈1283, ≈1459, ≈1604). Angka ini skala `UCI_Elo` Stockfish, bukan rating FIDE. Kalibrasi ulang cukup dengan menjalankan workflow lagi dan meng-commit berkasnya.
- Rujukan: D-041, D-046.

### D-048 — Revisi 9: menu jeda, penundaan, aturan kursor; M2b
- Tanggal / milestone: 2026-09-28 / M2 (perbaikan hasil uji) → M3
- Diputuskan oleh: klien (SPEC Rev. 9, hasil uji M2)
- Konteks: dua bug di jendela asli. (a) `Esc` di tengah permainan langsung keluar tanpa peringatan dan pertandingan hilang. (b) Kursor keyboard kembali ke a2 atau ke petak pilihan lama setelah pemain melangkah (pilih A, pindah pilih B, melangkah → kursor di titik lama).
- Keputusan (§4):
  - `Esc` atau tombol kembali saat pertandingan berjalan membuka menu jeda dengan fokus awal di Lanjutkan, plus Tunda & keluar dan Menyerah. Pertandingan yang ditunda tersimpan dan bisa dilanjutkan dari layar game; jam catur berhenti selama ditunda. Menutup jendela saat pertandingan berjalan otomatis menunda. Di LAN tidak ada jeda, hanya konfirmasi keluar.
  - Kursor keyboard mengikuti interaksi terakhir: klik/seret memindahkan kursor ke petak itu; setelah pemain melangkah kursor di petak tujuan; langkah lawan tidak memindahkan kursor; kursor disembunyikan saat memakai mouse dan muncul lagi di posisi terakhir saat panah ditekan; posisi awal wajar per game (catur e2/e7, Reversi tengah); tidak ada pilihan lama yang tertinggal.
  - §9: milestone M2b (level catur 5–6, target ≥2000 pada skala kalibrasi yang sama), dikerjakan setelah M3.
  - Aturan lapisan kursor dari Rev. 8 dipindah ke butir papan di §4 (isi sama).
- Rujukan: SPEC §4, §9, §12 Rev. 9.

### D-049 — Email commit memakai alamat noreply GitHub
- Tanggal / milestone: 2026-09-28 / perbaikan hasil uji M2
- Diputuskan oleh: klien (dipilih saat push ditolak)
- Konteks: GitHub menolak push ("push declined due to email privacy restrictions") karena commit memakai email pribadi.
- Keputusan: `user.email` lokal repo menjadi `264320223+MufuyuMoku@users.noreply.github.com` (nama tetap MufuyuMoku; konfigurasi global tidak diubah). Lima commit lokal yang belum ter-push ditulis ulang authornya; commit yang sudah ada di GitHub tidak diubah.

### D-050 — Keputusan klien untuk M3 (profil, rating, riwayat, statistik)
- Tanggal / milestone: 2026-09-28 / M3
- Diputuskan oleh: klien
- Keputusan:
  - Satu profil per instalasi, namanya bisa diganti. Tidak ada profil ganda.
  - Rating bot untuk game tanpa kalibrasi eksternal ditetapkan dari hasil antar-bot, level 1 dipatok 1000. Catur tetap memakai data kalibrasi Stockfish. Semua disebut "rating lokal".
  - Sapaan `name` dan `last_game` aktif begitu datanya ada.
  - Pertandingan yang ditunda tidak dihitung ke rating sampai selesai.
  - Tes jendela asli untuk layar profil dan statistik sesuai §11.
- Rujukan: SPEC §8, §9 M3, §11.

### D-051 — Penerapan Rev. 9: jeda, penundaan, kursor
- Tanggal / milestone: 2026-09-28 / perbaikan hasil uji M2
- Diputuskan oleh: developer (dalam batas D-048)
- Keputusan:
  - Reversi mendapat aksi `resign` (hanya pemain yang sedang melangkah, sama seperti catur); tes aturannya di-commit lebih dulu. Menyerah dari menu jeda saat giliran bot: bot melangkah dulu sampai giliran pemain, baru menyerah.
  - Selama menu jeda terbuka, jam dan bot juga berhenti (bukan hanya saat ditunda). `Esc` di menu jeda = Lanjutkan. Menyerah dari menu jeda meminta konfirmasi.
  - Penundaan disimpan di SQLite (satu per game): replay sejauh ini + kursi pemain + sisa jam. Melanjutkan = membangun ulang sesi dari seed ronde lalu memutar ulang langkah lewat aturan (`Match::restore`), jadi keadaannya sama persis dan replay akhirnya tetap lolos verify. Bot dibuat ulang dari seed ronde; keputusan bot setelah dilanjutkan bisa berbeda dari yang akan terjadi tanpa jeda (bot tidak termasuk verify).
  - Pertandingan yang belum selesai tidak pernah dibuang: keluar lewat jalur apa pun (menu, `back`, menutup jendela, memulai pertandingan lain) menundanya. Memulai pertandingan baru di game yang punya pertandingan tertunda meminta konfirmasi, dan yang tertunda dihitung menyerah (tercatat sebagai kalah), supaya penundaan tidak bisa dipakai menghindari kekalahan.
  - Kursor: posisi awal dibaca sekali saat papan dipasang (catur e2/e7, Reversi d4); papan dipasang ulang per pertandingan. Tombol panah/Enter pertama setelah memakai mouse hanya memunculkan kursor di posisi terakhir, tanpa memindahkan atau memilih.
- Rujukan: SPEC §4 Rev. 9, D-048.

### D-052 — Rating lokal: Glicko-2, lawan bot, riwayat
- Tanggal / milestone: 2026-09-28 / M3
- Diputuskan oleh: developer (dalam batas D-050)
- Keputusan:
  - Glicko-2 implementasi sendiri (diuji terhadap contoh Glickman 2013), τ = 0,5, pemain baru 1500 / RD 350 / volatilitas 0,06. Satu pertandingan selesai = satu periode rating. Tidak ada kenaikan RD karena tidak bermain (belum dibutuhkan; bisa diusulkan kemudian).
  - Rating dan RD lawan bot tetap: taksiran kalibrasi, RD = setengah lebar selang 95% / 1,96, minimal 30. Level catur 1 yang taksirannya ekstrapolasi tetap dipakai untuk menghitung (dengan RD lebarnya), tetapi tidak ditampilkan (D-047).
  - Kalibrasi antar-bot (`calibrate_internal`): level 1 = 1000; level berikutnya ditaksir berurutan dari hasil melawan semua level di bawahnya (kemungkinan maksimum, penaksir yang sama dengan kalibrasi catur), empat langkah pembuka acak, kursi bergantian. Reversi, 1000 partai per pasangan: level 2 ≈1344 (±33), level 3 ≈2133 (±99). Selisih level Reversi memang besar (level 3 menang 998,5/1000 atas level 1).
  - Hanya game kompetitif melawan bot terkalibrasi yang dihitung ke rating; hasil lain (misalnya fixture) tetap masuk riwayat tanpa rating. Tes CI memastikan setiap game kompetitif ber-bot punya data kalibrasi untuk setiap level dengan rating naik per level.
  - Skema basis data 3: `profile` (satu baris), `ratings` (per game, termasuk rating terbaik), `results` (riwayat). Replay yang sudah selesai sebelum M3 dimasukkan ke riwayat tanpa rating (rating tidak dihitung mundur).
  - Nama profil: spasi tepi dibuang, maksimal 24 karakter, kosong = tanpa nama (sapaan `name` tidak aktif). Sapaan `last_game` memakai game dari pertandingan terakhir yang selesai, dengan nama tampilan dalam bahasa aktif.
  - Halaman statistik: tabel per game (rating ±RD, jumlah partai, menang–seri–kalah, rating terbaik) dan riwayat 30 pertandingan terakhir (perubahan rating, tautan ke replay). Ringkasan menang/kalah terhadap bandar (§2.2) menyusul bersama game casino di M4.
- Rujukan: SPEC §8, D-047, D-050.

### D-053 — Revisi 10: tangga level merata; D-048–D-052 disetujui
- Tanggal / milestone: 2026-09-29 / M2b
- Diputuskan oleh: klien (hasil uji perbaikan M2 dan M3)
- Keputusan:
  - D-048 sampai D-052 disetujui.
  - §8: untuk setiap game kompetitif, selisih rating dua level bot berurutan paling besar 400 pada skala rating lokal game itu; level terbawah yang hanya punya batas atas dikecualikan. CI gagal bila data kalibrasi melanggar. Bila terlalu lebar: tambah level di antaranya atau setel ulang level yang ada.
  - §9: M2b = level catur 5–6 (target ≥2000) + perapian tangga level Reversi. Kalibrasi catur tetap lewat `calibrate.yml` di GitHub Actions; kalibrasi Reversi boleh lokal.
- Rujukan: SPEC §8, §9, §12 Rev. 10.

### D-054 — M2b: level catur 5–6 dan tangga Reversi
- Tanggal / milestone: 2026-09-29 / M2b
- Diputuskan oleh: developer (dalam batas D-053)
- Keputusan:
  - Pemeriksaan CI: `calibration::ladder_violations` + tes `every_competitive_game_has_an_even_level_ladder` membaca data kalibrasi setiap game kompetitif ber-bot dan gagal bila selisih dua level berurutan > 400. Level terbawah dikecualikan hanya bila taksirannya ekstrapolasi (semua kalah, sehingga hanya ada batas atas).
  - Reversi: empat level. Level 3 baru = alpha-beta 2 langkah; level 4 = alpha-beta 3 langkah + hitung 8 petak terakhir sampai akhir. Level 3 lama (4 langkah + hitung akhir) tidak dipertahankan: dengan tangga yang lebih rapat ia hanya ≈65 di atas alpha-beta 4 langkah biasa, dan jaraknya ke level di bawahnya terlalu lebar. Kalibrasi lokal antar-bot, 1000 partai per pasangan: 1000 / 1344 / 1680 / 2001 (selisih 344 / 336 / 321). Partai Reversi level 3 dari sebelum M2b tercatat dengan level lama.
  - Catur level 5–6: mesin kedua (`catur_search`) — iterative deepening, PVS, tabel transposisi (dikosongkan tiap langkah), null-move pruning, LMR, reverse futility pruning, perpanjangan skak, killer/history, quiescence dengan delta pruning, evaluasi bertahap buatan sendiri (bahan, tabel posisi, mobilitas, pasangan gajah, struktur pion, benteng, perisai raja). Level 1–4 tidak diubah sehingga data kalibrasinya tetap berlaku.
  - Batas node, bukan waktu (deterministik). Mesin baru jauh lebih efisien: 15 ribu node sudah ≈+300 atas level 4 lama (600 ribu node) dalam adu antar-bot. Karena itu "waktu berpikir lebih lama" tidak dipakai: batas node yang memenuhi tangga ≤ 400 kecil (level 5: 10 ribu, level 6: 40 ribu), jadi bot tetap cepat.
  - Pemilihan: tiga kandidat diukur paralel terhadap Stockfish (UCI_Elo 1500–2500, 24 partai per lawan, 100 ms/langkah, pembukaan dan metode sama dengan kalibrasi M2) lewat input baru `calibrate.yml` (`levels`, `merge`, `nodes`). Kandidat 10 ribu / 40 ribu dipilih; hasil run itu dipakai langsung sebagai data resmi karena parameternya sama persis dengan konstanta di kode (batas node dicatat per level di berkas data). Level 1–4 di berkas data tetap dari kalibrasi M2 (16 partai per lawan, UCI_Elo 1320–1900); level 5–6 memakai 24 partai per lawan pada UCI_Elo 1500–2500 (jumlah per lawan tercatat di tiap baris hasil). Hasil: level 5 ≈1826 (95%: 1748–1904), level 6 ≈2093 (2016–2170); tangga 1283 / 1459 / 1604 / 1826 / 2093.
  - Label UI level 5 "master", level 6 "grandmaster".
- Rujukan: SPEC §8, §9 M2b, D-047, D-052, D-053.

### D-055 — Label level deskriptif, tanpa gelar resmi
- Tanggal / milestone: 2026-09-29 / hasil uji M2b
- Diputuskan oleh: klien (developer memilih urutan kata)
- Keputusan: label level tidak memakai gelar resmi (misalnya "master"/"grandmaster") karena rating KyuSin adalah rating lokal, bukan FIDE. Label untuk semua game (dibagi per nomor level): 1 Pemula/Beginner, 2 Menengah/Intermediate, 3 Mahir/Advanced, 4 Kuat/Strong, 5 Ahli/Expert, 6 Sangat kuat/Very strong. "Ahli" pindah dari level 4 ke level 5 supaya tidak ada label ganda dan urutannya tetap naik. Perkiraan rating lokal tetap tampil di samping label. Menggantikan butir label di D-054.
- Rujukan: SPEC §2.2, §8, D-054.

### D-056 — Keputusan klien untuk M4 (mesin kartu, chip, Blackjack)
- Tanggal / milestone: 2026-09-29 / M4
- Diputuskan oleh: klien
- Keputusan:
  - Aturan Blackjack: 6 dek, bandar berdiri di soft 17 (S17), blackjack dibayar 3:2, double di dua kartu mana pun termasuk setelah split, split sampai 4 tangan, as yang di-split hanya dapat satu kartu, ada insurance dan late surrender, kocok ulang setelah sekitar 75% shoe terpakai. RTP di manifest untuk strategi dasar aturan ini, dihitung dan diverifikasi sesuai §7.
  - Provably fair Blackjack per shoe: komitmen seed saat shoe dikocok, seed dibuka saat shoe diganti atau pemain berhenti, `verify` memeriksa semua ronde dalam shoe. Ini penerapan §5.4 untuk game ber-shoe: membuka seed setelah tiap ronde akan membocorkan urutan sisa shoe.
  - Batas taruhan meja: minimal 10, maksimal 2.000 chip.
  - Simbol jenis kartu (♠♥♦♣) dan elemen kartu berupa sprite piksel SVG (§4), bukan glyph font.
  - Sapaan `chips_low` mengikuti D-039: dipilih dari saldo sebelum tunjangan harian, lalu tunjangannya disebut sesudahnya.
  - Halaman statistik: ringkasan menang/kalah terhadap bandar sepanjang waktu, per game casino.
  - Tes jendela asli meja Blackjack (§11): keselarasan, taruhan, hit/stand/double/split, kursor/hover tidak menggeser elemen.
- Rujukan: SPEC §2.2, §2.3, §4, §5.4, §6.3, §6.7, §7, §9 M4, §11, D-039.

### D-057 — Blackjack: rincian aturan, strategi dasar, dan RTP
- Tanggal / milestone: 2026-09-29 / M4
- Diputuskan oleh: developer (dalam batas D-056)
- Keputusan:
  - Rincian aturan di luar D-056: taruhan kelipatan 10 (sehingga 3:2, insurance, dan surrender selalu bilangan bulat); bandar mengintip bila kartu terbuka as atau bernilai 10 (hole card ala AS), jadi late surrender dan double/split hanya kehilangan taruhan awal bila bandar blackjack; split hanya untuk peringkat sama (K-K ya, K-Q tidak); as yang di-split tidak bisa di-split lagi dan tidak bisa di-double; tangan yang mencapai 21 otomatis berdiri; tangan hasil split menerima kartu keduanya saat mulai dimainkan; bila semua tangan bust atau menyerah, bandar tidak mengambil kartu; shoe berakhir di akhir ronde saat kartu terpakai mencapai titik potong 234 dari 312 (75%). Tidak ada "even money" terpisah (setara insurance saat pemain blackjack).
  - Meja satu kursi (`pemain_maks = 1`, `lan = false`); meja LAN banyak pemain melawan bandar sistem (§6.7) menyusul bersama milestone LAN.
  - Strategi dasar yang didokumentasikan (`crates/bots/src/blackjack.rs`): tabel baku multi-dek S17 DAS late surrender tanpa resplit as; insurance selalu ditolak.
  - RTP manifest 99,64%: simulasi 1 miliar ronde strategi dasar menghasilkan 99,6388% (σ 1,1408 per ronde, selang 1σ ±0,0036%). Pendekatan kombinatorial penuh tidak dipakai; §7 mengizinkan simulasi untuk game yang bergantung strategi. RTP = 1 + rata-rata hasil bersih per ronde dibagi taruhan awal.
  - Verifikasi §7: tes `rtp_matches_the_manifest_within_four_sigma` (100 ribu ronde, CI setiap push) dan workflow `rtp.yml` (10 juta ronde, manual dan terjadwal tiap Senin). Keduanya gagal bila selisih dengan manifest > 4σ/√n.
  - Provably fair per shoe (D-056): satu pertandingan = satu shoe, jadi komitmen dibuat saat pertandingan (shoe) dimulai dan seed dibuka saat shoe habis atau pemain berhenti; verify memutar ulang semua ronde. Menunda di tengah shoe menyimpan seed tetap tertutup.
- Rujukan: SPEC §5.4, §6.3, §7, D-056.

### D-058 — Chip profil di meja casino
- Tanggal / milestone: 2026-09-29 / M4
- Diputuskan oleh: developer (dalam batas SPEC §6.7 dan D-056)
- Keputusan:
  - Game casino tetap murni dan tidak tahu saldo. Host memeriksa saldo sebelum `bet`, `double`, `split`, dan `insure` (biaya dari view; tersedia = saldo − yang sedang dipertaruhkan) dan menolak dengan pesan jelas bila kurang. Setiap ronde yang selesai langsung dicatat ke saldo dan ringkasan casino dalam satu transaksi SQLite, jadi chip tidak hilang bila aplikasi ditutup di tengah shoe. Taruhan ronde yang sedang berjalan baru memengaruhi saldo saat ronde selesai (diganti D-059: taruhan dipotong saat dipasang).
  - Tunjangan harian: diperiksa saat aplikasi dibuka dan saat pertandingan casino dimulai, memakai tanggal lokal dari UI (hanya UI yang tahu zona waktu sistem). Hanya diberikan bila saldo < 1.000 dan belum diberikan hari itu; saldo diisi menjadi 2.000. Hari yang tidak memberi tunjangan (saldo cukup) tidak "terpakai".
  - Sapaan: `chips_low` memakai saldo sebelum tunjangan (dari profil yang dibaca sebelum pemeriksaan tunjangan); bila tunjangan diberikan, satu baris sesudahnya menyebutnya dengan nada yang sama (D-039).
  - Menu jeda di meja casino: Lanjutkan, Tunda & keluar, dan Berhenti (mengakhiri shoe, hanya di antara ronde) menggantikan Menyerah. Memulai shoe baru saat masih ada shoe tertunda meminta konfirmasi; shoe tertunda diselesaikan secara netral (insurance ditolak, semua tangan stand, lalu berhenti) dan hasilnya dicatat.
  - Casino melawan bandar tidak masuk riwayat rating (M3); halaman statistik punya tabel terpisah "Melawan bandar": ronde, total dipertaruhkan (termasuk double/split/insurance), dan hasil bersih per game, plus total.
  - Sprite kartu: peta piksel 22×30 (bingkai, peringkat 5×7, jenis kecil 5×5, jenis besar 9×9), kartu tertutup berarsir; semua jenis diisi penuh dan dibedakan oleh bentuk. Tes UI memastikan 52 kartu, 13 peringkat, dan 4 jenis berbeda. Alasan tombol yang tidak tersedia ditulis di satu baris tetap di bawah tombol (D-033) supaya tata letak tidak bergeser.
- Rujukan: SPEC §2.2, §4, §6.7, D-033, D-039, D-051, D-056.

### D-059 — Taruhan casino tercatat saat dipasang; mati paksa tidak membatalkan ronde
- Tanggal / milestone: 2026-09-29 / sebelum M5a (permintaan klien setelah uji M4)
- Diputuskan oleh: klien (syarat), developer (cara)
- Temuan: di M4 belum aman. Keadaan pertandingan hanya disimpan saat ditunda atau jendela ditutup normal, dan taruhan ronde berjalan baru memengaruhi saldo saat ronde selesai (D-058). Aplikasi yang dimatikan paksa setelah kartu terlihat membuang ronde itu beserta taruhannya: pemain bisa lolos dari kekalahan.
- Keputusan:
  - **Titik simpan setelah setiap aksi**, untuk semua game (bukan hanya casino): pertandingan (konfigurasi, catatan provably fair, langkah, sisa jam) ditulis ke tabel `suspended` setelah setiap perubahan keadaan, dan tampilan baru baru dikirim ke UI setelah titik simpannya tertulis. Pemain tidak pernah melihat hasil yang belum tersimpan.
  - **Casino: saldo dan pertandingan dalam satu transaksi SQLite.** Taruhan (`bet`, `double`, `split`, `insure`) dipotong dari saldo saat dipasang dan dicatat di kolom baru `profile.staked` (skema 5). Saat ronde selesai, taruhan kembali plus hasilnya, dan ringkasan casino diperbarui di transaksi yang sama. Invarian yang dites: saldo = awal + bersih − taruhan di meja, di setiap langkah, termasuk setelah mati paksa.
  - **Dibuka lagi = ronde yang sama.** Pertandingan dilanjutkan dari seed dan langkah yang sama, jadi kartu, taruhan, dan urutan shoe identik. Titik simpan baru dihapus setelah pertandingan selesai dan hasilnya tersimpan (bukan saat dilanjutkan), jadi mati lagi tepat setelah dilanjutkan juga aman. Pertandingan yang selesai tepat sebelum aplikasi mati dicatat sekali saja.
  - **Membuang pertandingan tertunda tidak mengembalikan taruhan.** Ronde yang berjalan dimainkan sampai selesai secara netral (insurance ditolak, semua tangan stand, D-058), dan hasilnya berlaku. Memulai pertandingan baru untuk game yang masih punya pertandingan tertunda ditolak backend (`error.suspended_exists`); UI sudah meminta konfirmasi lebih dulu.
  - **Tunjangan harian menghitung chip di meja:** "saldo" untuk ambang 1.000 = tersedia + dipertaruhkan, dan tunjangan mengisi totalnya menjadi 2.000. Taruhan yang belum selesai tidak membuat pemain tampak miskin. Profil menampilkan chip yang sedang dipertaruhkan.
  - Pertandingan casino yang ditunda versi M4 (sebelum titik simpan ini) masih bisa dilanjutkan: taruhan rondenya dipotong saat dilanjutkan.
  - Game kompetitif ikut terlindungi: mematikan aplikasi di tengah partai catur yang kalah tidak lagi menghapus partainya; saat dibuka, partai itu tertunda dan hanya bisa dilanjutkan atau diselesaikan (dihitung menyerah). Sisa jam yang tersimpan adalah sisa saat langkah terakhir; waktu berpikir sejak langkah terakhir sampai mati paksa tidak tercatat.
- Bukti: tes backend dengan berkas SQLite sungguhan yang "dimatikan paksa" (semua objek dibuang tanpa tunda/tutup, lalu berkas dibuka `AppState` baru): `a_killed_app_keeps_the_casino_bet_and_resumes_the_same_round`, `a_killed_app_cannot_escape_a_split_round_even_by_abandoning_it`, `balance_always_equals_start_plus_net_minus_stake` (mati paksa berkali-kali di tengah shoe acak), `a_killed_app_resumes_a_rated_game_instead_of_dropping_it`; tes store untuk transaksi titik simpan dan tunjangan. Tes jendela asli Blackjack memeriksa saldo turun tepat saat taruhan dipasang.
- Batasan yang diterima: titik simpan memuat seed ronde, sama seperti pertandingan tertunda di M4, jadi orang yang membaca berkas SQLite-nya sendiri bisa menghitung kartu tertutup. Ini setara dengan batasan host di SPEC §5.4 (aplikasi host yang dimodifikasi bisa melihat kartu tertutup).
- Rujukan: SPEC §4 (Rev. 9), §5.4, §6.7, D-051, D-058.

### D-060 — M5a: varian aturan dan tabel pembayaran sepuluh meja melawan bandar
- Tanggal / milestone: 2026-10-02 / M5a (SPEC Revisi 11)
- Diputuskan oleh: developer (varian paling umum, sesuai arahan klien); klien untuk dua hal yang tidak punya varian paling umum (Andar Bahar: kartu pertama selalu ke Andar; Pai Gow Poker: house way gaya Las Vegas)
- Sumber: Wikipedia (artikel Baccarat, Caribbean stud poker, Casino hold 'em, Let It Ride, Pai gow poker, Casino War, Red dog, Three Card Poker), pagat.com (Andar Bahar), ringkasan halaman Wizard of Odds lewat mesin pencari (Three Card Poker, Casino War, Dragon Tiger, Andar Bahar; situsnya sendiri tidak bisa dibuka karena sertifikat TLS-nya kedaluwarsa, dan pemeriksaan TLS tidak dilewati), aturan resmi kasino Singapura (GRA, Dragon Tiger) lewat ringkasan pencarian.
- Keputusan per game (semua: batas meja 10–2.000 kelipatan 10 seperti Blackjack, kecuali Baccarat dan Pai Gow 20–2.000 kelipatan 20 supaya komisi 5% selalu utuh; tanpa jackpot progresif dan tanpa taruhan sampingan opsional):
  - **Baccarat (Punto Banco):** 8 dek, shoe selesai saat tersisa 16 kartu; player 1:1, banker 0,95:1, tie 8:1 (Inggris memakai 9:1; 8:1 baku di tempat lain); tabel kartu ketiga baku.
  - **Dragon Tiger:** 8 dek, potong 75%, as terendah, dragon/tiger 1:1 dan kalah setengah bila seri, tie 8:1 (11:1 ada di sebagian kasino; 8:1 yang paling sering disebut).
  - **Casino War:** 6 dek, potong 75%, as tertinggi; seri: surrender (½ ante) atau war (raise = ante, buang 3 kartu); setelah perang kartu pemain ≥ bandar = raise 1:1 dan ante kembali (aturan baku; bonus "seri setelah seri" hanya di sebagian kasino); tie bet 10:1 (bagian tata letak meja).
  - **Red Dog:** 8 dek, potong 75%; spread 1/2/3/4+ = 5/4/2/1, three of a kind 11:1, berurutan seri, raise menggandakan taruhan.
  - **Andar Bahar:** satu dek per ronde; kartu pertama setelah kartu tengah selalu ke Andar (keputusan klien); Andar 0,9:1, Bahar 1:1.
  - **Three Card Poker:** satu dek per ronde; bandar memenuhi syarat Q-high; ante bonus 1-4-5; Pair Plus 1-3-6-30-40 (tabel yang kini paling umum; 1-4-6-30-40 lebih langka). Pair Plus boleh dimainkan tanpa ante. Tanpa 6-card bonus.
  - **Caribbean Stud:** satu dek per ronde; raise 2× ante; bandar memenuhi syarat A-K; tabel raise baku AS 1-2-3-4-5-7-20-50-100; tanpa jackpot progresif.
  - **Casino Hold'em:** satu dek per ronde; call 2× ante; bandar memenuhi syarat pair 4; AnteWin 100-20-10-3-2-1; tanpa AA bonus.
  - **Let It Ride:** satu dek per ronde; tabel baku 1-2-3-5-8-11-50-200-1000; tanpa three-card bonus.
  - **Pai Gow Poker:** dek 53 kartu (joker = bug), wheel straight tertinggi kedua (aturan di luar California), lima as tertinggi, komisi 5%, copy dimenangkan bandar, pemain tidak menjadi bankir. House way gaya Las Vegas (keputusan klien), lengkap: tanpa pair → tertinggi di belakang, dua berikutnya di depan (straight/flush dipakai bila ada, dengan depan setinggi mungkin); satu pair → pair di belakang, dua tertinggi di depan, kecuali straight/flush bisa dimainkan sambil depan tetap pair atau berisi as; two pair → dipisah (pair rendah di depan) kecuali pair tingginya 10 ke bawah dan ada as tunggal (keduanya tetap di belakang, as di depan); tiga pair → pair tertinggi di depan; three of a kind as → pair as di belakang, as + kartu tertinggi di depan; three of a kind lain di belakang, dua tertinggi di depan; dua three of a kind → pair dari yang lebih tinggi di depan; full house → pair tertinggi di depan; four of a kind 2–6 utuh, 7–10 utuh bila ada as tunggal, J–A dipecah, dengan pair lain → pair itu di depan; lima as → pair lain di depan bila ada, selain itu pair as. Joker dihitung as untuk pengelompokan. Susunan yang tidak sah tidak pernah dihasilkan (ada cadangan yang dites).
- **Angka RTP di manifest** (SPEC §5.2 hanya punya satu angka): RTP taruhan utama yang **terendah**, supaya angka utama tidak pernah tampak lebih baik dari kenyataan; semua taruhan lain dicantumkan di `man` dan diverifikasi sama ketatnya. Satuannya taruhan dasar per game (ante; satu tempat untuk Let It Ride, konvensi baku), dan hasil bersih dibandingkan dengan taruhan itu.
- **Strategi untuk RTP:** Casino War selalu war; Red Dog raise pada spread 7+; Three Card Poker Q-6-4; Caribbean Stud strategi sederhana yang diadaptasi dari Wizard of Odds (ditulis lengkap di `man`; RTP terukurnya 94,69%, sedikit di bawah angka yang dikutip untuk strategi aslinya); Let It Ride optimal (dihitung tepat per tangan); Pai Gow house way untuk pemain juga; Casino Hold'em strategi sederhana KyuSin, dipilih dari beberapa aturan sederhana dengan simulasi bilangan acak yang sama (strategi yang saya rancang pertama ternyata lebih buruk dari selalu call dan dibuang), sekitar 0,7 poin di bawah optimal — dinyatakan terus terang di `man`.
- Rujukan: SPEC §5.2, §6.3, §7; D-056, D-057, D-059.

### D-061 — M5a: penerapan
- Tanggal / milestone: 2026-10-02 / M5a
- Diputuskan oleh: developer
- Keputusan:
  - **Provably fair:** game ber-shoe (Baccarat, Dragon Tiger, Casino War, Red Dog) satu shoe per pertandingan seperti Blackjack (D-056). Game yang dikocok ulang tiap ronde (Andar Bahar, Three Card Poker, Caribbean Stud, Casino Hold'em, Let It Ride, Pai Gow) satu ronde per pertandingan, jadi commit-reveal §5.4 berlaku per ronde tanpa pengecualian. Di UI ronde berikutnya mulus: taruhan di meja yang rondenya selesai memulai pertandingan baru dengan komitmen baru.
  - **Kontrak chip bersama** (`kyusin_games::meja::Umum`): `fase`, `ronde`, `bersih`, `taruhan_meja`, `dipertaruhkan`, `biaya`, `pengali`, `netral`. Biaya sebuah perintah = `biaya[perintah]`, atau angka terakhir perintah × `pengali[kata kerja]` (Let It Ride: `bet 100` = 300). Taruhan dipasang dan dipotong dari saldo sebelum kartu dibagi (`bet <tempat> <jumlah>`, `clear`, `deal`).
  - **Membuang pertandingan di tengah ronde:** host memainkan `netral` dari view, pilihan yang tidak menambah taruhan (fold, surrender, call, pull, house way), seperti stand di Blackjack (D-058/D-059).
  - **RTP:** meja tanpa keputusan (Dragon Tiger, Casino War, Andar Bahar, Baccarat, Red Dog, Pair Plus) dihitung analitis/enumerasi tepat di tes setiap push (`crates/bots/tests/meja_exact.rs`). Three Card Poker Ante/Play dan Let It Ride dienumerasi tepat di workflow `rtp.yml` (job `tepat`). Caribbean Stud dan Casino Hold'em dari simulasi 1 miliar ronde di GitHub Actions; Pai Gow dari 200 juta ronde (1 miliar melewati batas waktu job; toleransi 200 juta ronde ±0,022%). Semua taruhan disimulasikan ≥100.000 ronde di CI setiap push dan ≥10.000.000 ronde di `rtp.yml` (job `meja`), dengan toleransi 4σ/√n. Tidak ada simulasi berat di laptop.
  - **Evaluator poker bersama** (`kyusin_games::poker`): lima kartu, terbaik dari 5–7, dan tiga kartu; dites terhadap frekuensi baku seluruh 2.598.960 tangan dan 22.100 tangan tiga kartu. Pai Gow punya evaluatornya sendiri di atasnya (joker, wheel, lima as).
  - **UI:** satu komponen meja (`MejaTable`) dengan model per game; baris kartu bertumpuk 18 px atau terpisah 48 px (Pai Gow, supaya kartu bisa dipilih); tombol tempat taruhan berlabel tetap dan jumlahnya di baris tetap, jadi tidak ada yang bergeser. Sprite joker baru. Tes jendela asli memainkan satu ronde di tiap meja.
- Rujukan: SPEC §4, §5.4, §7, §11; D-056, D-058, D-059, D-060.

### D-062 — Keputusan klien untuk M5b; SPEC Revisi 12
- Tanggal / milestone: 2026-10-02 / M5b-1
- Diputuskan oleh: klien
- Keputusan:
  - Hasil uji M5a sesuai.
  - **Pekerjaan nanti (dicatat, belum dikerjakan):** RTP Casino Hold'em dihitung ulang dengan strategi optimal lewat enumerasi tepat di GitHub Actions, supaya angka yang tampil tidak lebih buruk dari casino sungguhan (sekarang 97,08% untuk strategi sederhana KyuSin, D-060).
  - SPEC Revisi 12: M5b dipecah menjadi M5b-1 (Texas Hold'em, Omaha, Teen Patti) dan M5b-2 (Capsa Susun, Domino QiuQiu, termasuk mesin domino bersama yang nanti dipakai Gaple) (§9, §12).
  - Meja singleplayer melawan bot memakai chip profil lewat buy-in: chip meja dibeli dari saldo saat duduk, sisanya dikembalikan saat berdiri; semua tercatat permanen sesuai D-059.
  - Bot minimal 3 level, tangga paling lebar 400 poin (§8), dikalibrasi antar-bot. Cara menghitung rating Glicko-2 untuk game multipemain berbasis sesi ditentukan developer dan dijelaskan di DECISIONS.
  - Capsa Susun dan Domino QiuQiu: setiap aturan lokal yang variannya berbeda-beda ditanyakan ke klien. Poker dan Teen Patti: varian paling umum seperti di M5a.
  - Simulasi dan kalibrasi berat lewat GitHub Actions.
- Rujukan: SPEC §6.3, §6.7, §7, §8, §9; D-059, D-060.

### D-063 — M5b-1: Texas Hold'em, Omaha, Teen Patti (varian, meja, rating, bot)
- Tanggal / milestone: 2026-10-02 / M5b-1
- Diputuskan oleh: developer (dalam batas D-062)
- Varian (paling umum, D-062 butir 4; sumber: Wikipedia "Teen patti" dan ringkasan pencarian aturan aplikasi Teen Patti populer):
  - **Texas Hold'em No-Limit**, **Omaha Pot-Limit** (PLO; Omaha paling umum dimainkan pot-limit), blind 10/20, sampai 6 kursi, tanpa rake. Raise minimal sebesar raise terakhir (paling sedikit big blind); all-in yang kurang dari raise penuh tidak membuka lagi hak raise bagi yang sudah bertindak; heads-up dealer = small blind. Omaha memakai tepat dua kartu tangan dan tepat tiga kartu meja. Tanpa burn card (tidak memengaruhi keadilan karena dek dari seed). Seri dibagi rata; chip ganjil mulai dari kursi pertama setelah dealer.
  - **Teen Patti**: boot 10, stake mula-mula = boot; buta chaal 1× / raise 2× stake, terlihat 2× / 4×; stake paling besar 64 × boot; paling banyak 4 chaal buta; show hanya saat tinggal dua pemain (biaya = chaal peminta, boleh oleh pemain buta supaya dua pemain buta tidak macet); sideshow antara dua pemain terlihat (peminta dan pemain aktif sebelumnya), paling sedikit tiga pemain tersisa, yang lebih rendah pack, seri = peminta pack; pot mencapai 1.024 × boot = semua dibandingkan. Urutan: trail, pure sequence, sequence, color, pair, kartu tinggi; A-K-Q > A-2-3 > K-Q-J. Tangan sama membagi pot. Sampai 6 kursi.
  - Pot: poker memakai side pot baku (chip yang tidak disamakan kembali); Teen Patti memakai pot tanpa penyamaan taruhan: semua chip masuk satu pot, lapisan hanya di kontribusi pemain all-in (`pot::pots_all_in`).
- **Satu pertandingan = satu sesi meja.** Tumpukan dibawa dari tangan ke tangan; setiap tangan dikocok dari seed sesi dan nomor tangan; seed dibuka saat sesi selesai dan verify memutar ulang semua tangan. Ini penerapan §5.4 untuk meja bersesi, seperti per shoe di Blackjack (D-056): commit-reveal per tangan tidak bisa dipakai karena tumpukan harus berlanjut di pertandingan yang sama. Sesi selesai bila kursi manusia berdiri (di antara tangan) atau habis, bila tinggal satu kursi bertumpukan, atau (kalibrasi) setelah batas tangan. Di antara tangan semua kursi menjawab `next`/`leave` (bot otomatis).
- **Buy-in (D-062 butir 2):** saat duduk, host memindahkan min(saldo, 2.000) dari saldo ke meja (paling sedikit 400 = 20 big blind; kurang dari itu ditolak); bot duduk dengan 2.000. Lewat kontrak chip D-059: `taruhan_meja` = buy-in selama duduk (dipotong di titik simpan pertama), `bersih` = tumpukan akhir − buy-in saat berdiri; mati paksa melanjutkan tangan yang sama; membuang sesi memainkan aksi netral (`check`/`fold`, `pack`/`deny`, `leave`) sambil bot tetap bertindak. Meja antar-pemain tidak masuk ringkasan "melawan bandar"; hasilnya masuk riwayat dan rating.
- **Rating Glicko-2 untuk sesi multipemain (D-062 butir 3):** satu sesi = satu pertandingan melawan level bot yang dipilih (semua bot di meja selevel), dengan skor pecahan = bagian bot yang hasil bersih sesinya di bawah pemain (seri setengah). Glicko-2 menerima skor di antara 0 dan 1 (`Rating::update_scores`). Alasannya: (1) satu periode per sesi, bukan per tangan atau per bot, supaya sesi dengan lima bot tidak dihitung lima kali walau hasilnya saling berkorelasi; (2) peringkat terhadap bot lain lebih stabil daripada menang/kalah total, dan tidak bergantung pada besar chip; (3) setiap sesi yang selesai dihitung (paling sedikit satu tangan selalu dimainkan), jadi berdiri saat kalah tidak menghindarkan rating. Riwayat menampilkan menang (> 0,5), seri (= 0,5), atau kalah (< 0,5).
- **Bot dan kalibrasi:** Texas/Omaha 3 level (longgar-pasif; aturan kategori tangan; equity Monte Carlo 400 kali (Omaha 133) dengan model lawan, agresi sebelum flop, besar taruhan menurut equity). Level keempat dicoba (fitur-fitur level 3 ditambahkan satu per satu) dan hanya sekitar 30 poin di atas level 3 tanpa fitur itu, jadi tidak dijadikan level tersendiri. Teen Patti 4 level (level 2 dengan 35% salah langkah; ambang tetap; nilai harapan show/pack dari persentil tangan; ditambah model lawan). Kalibrasi antar-bot di GitHub Actions (`kalibrasi-antarbot.yml`): level 1 = 1000, sesi heads-up 60 tangan, 2.000 sesi per pasangan, menang = chip lebih banyak di akhir sesi. Hasil: Texas 1000 / 1131 / 1379; Teen Patti 1000 / 1174 / 1369 / 1411; Omaha 1000 / 1378 / 1600. Semua selisih berurutan ≤ 400 (§8). Catatan: skala ini dari sesi heads-up; di meja enam kursi kekuatan relatif bisa berbeda.
- Rujukan: SPEC §5.4, §6.3, §6.7, §7, §8; D-052, D-056, D-058, D-059, D-062.

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
