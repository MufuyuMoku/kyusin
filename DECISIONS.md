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

## Pertanyaan terbuka

### Q-001 — Host dapat mengeluarkan peserta setelah melihat seed-nya
- Bagian SPEC: §5.4 langkah 2 dan "Detail lain" vs "Sifat yang dijamin" dan §2.3
- Pertanyaan: setelah langkah 2, host sudah memegang semua seed dan bisa menghitung hasil ronde. Aplikasi host yang dimodifikasi dapat mengaku seorang peserta "tidak mengirim seed dalam batas waktu", lalu mengeluarkannya. Seed ronde pun berubah, sehingga host bisa memilih di antara beberapa hasil. Ini bertentangan dengan "tidak ada yang bisa menggeser hasil". Usul developer: peserta yang gagal pada tahap komitmen boleh dikeluarkan, tetapi kegagalan pada tahap pembukaan seed membatalkan ronde untuk semua pemain (taruhan dikembalikan) dan dicatat terlihat di `verify`/riwayat. Sisa celah (host bisa membatalkan ronde berulang-ulang, dan pembatalan itu terlihat) dicatat sebagai batasan yang diterima.
- Status: menunggu klien. Tidak memengaruhi M0 (baru dikerjakan di M1/M9).

### Q-002 — Kedudukan slot dan game solo casino dalam manifest
- Bagian SPEC: §4 (slot disebut "game real-time") vs §6.6 (slot = `TurnGame`); §5.2 (`rtp` untuk "casino ber-bandar") vs §7.2 (slot, Keno, Bingo, kartu gosok = game solo tanpa lawan)
- Pertanyaan: tafsiran developer: (a) di §4 slot hanya berarti *dirender di canvas*; kontraknya tetap `TurnGame`, dan mode perintah penuh berlaku. (b) Game solo casino berisi `lawan: tidak ada` tetapi `rtp` berupa angka; `rtp: null` hanya untuk game antar-pemain dan non-casino. Tafsiran (b) menentukan skema manifest di M0.
- Status: menunggu klien.

<!--
### Q-001 — judul singkat
- Bagian SPEC:
- Pertanyaan:
- Status: menunggu klien | terjawab (lihat D-xxx)
-->

## Usulan

Ide di luar SPEC (SPEC §11). Tidak dikerjakan sebelum disetujui klien.

<!--
### U-001 — judul singkat
- Diusulkan:
- Alasan:
- Status: menunggu | diterima (D-xxx) | ditolak
-->
