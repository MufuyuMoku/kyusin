//! Ekonomi chip profil (SPEC §6.7; D-056, D-058): saldo awal 10.000,
//! tunjangan harian, dan ringkasan menang/kalah terhadap bandar per game
//! casino. Chip tidak bisa dibeli, dicairkan, atau dipindahkan.

use rusqlite::{Connection, OptionalExtension, params};

use crate::{Store, StoreError, Suspended, put_suspended};

pub const STARTING_CHIPS: i64 = 10_000;
/// Tunjangan harian: saldo di bawah ambang ini diisi menjadi [`ALLOWANCE_TO`].
pub const ALLOWANCE_BELOW: i64 = 1_000;
pub const ALLOWANCE_TO: i64 = 2_000;

/// Hasil pemeriksaan tunjangan harian.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Allowance {
    /// Saldo sebelum tunjangan (dipakai sapaan `chips_low`, D-039).
    pub before: i64,
    pub after: i64,
    pub granted: bool,
}

/// Ringkasan sepanjang waktu satu game casino.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CasinoStats {
    pub game: String,
    pub rounds: i64,
    /// Total chip yang dipertaruhkan (termasuk double, split, insurance).
    pub wagered: i64,
    /// Hasil bersih terhadap bandar (positif = pemain untung).
    pub net: i64,
    pub last_played: i64,
}

/// Satu titik simpan pertandingan casino (D-059), ditulis setelah setiap
/// aksi dalam **satu transaksi**: perubahan saldo (taruhan dipotong saat
/// dipasang, pengembalian + hasil saat ronde selesai), ringkasan ronde yang
/// baru selesai, dan keadaan pertandingan untuk dilanjutkan. Aplikasi yang
/// dimatikan paksa kapan pun meninggalkan saldo dan pertandingan yang cocok
/// satu sama lain.
#[derive(Debug, Clone, Copy)]
pub struct CasinoCheckpoint<'a> {
    pub game: &'a str,
    /// Perubahan chip yang sedang dipertaruhkan di meja game ini sejak titik
    /// simpan sebelumnya (positif = taruhan baru dipasang).
    pub stake: i64,
    /// Hasil bersih ronde yang baru selesai (0 bila ronde masih berjalan).
    pub net: i64,
    /// Ronde yang baru selesai dan jumlah yang dipertaruhkan di ronde itu.
    pub rounds: i64,
    pub wagered: i64,
    /// Keadaan pertandingan (seed + langkah) untuk dilanjutkan.
    pub suspended: &'a Suspended,
    pub now: i64,
}

/// Migrasi ke skema 5: chip yang sedang dipertaruhkan (D-059). Kolom
/// `chips` adalah saldo yang bisa dipakai; taruhan sudah dipotong darinya
/// saat dipasang dan dicatat di `staked` sampai rondenya selesai.
pub(crate) fn migrate_v5(conn: &Connection) -> Result<(), StoreError> {
    conn.execute_batch(
        "BEGIN;
         ALTER TABLE profile ADD COLUMN staked INTEGER NOT NULL DEFAULT 0;
         PRAGMA user_version = 5;
         COMMIT;",
    )?;
    Ok(())
}

/// Migrasi ke skema 4: saldo chip di profil dan tabel ringkasan casino.
pub(crate) fn migrate_v4(conn: &Connection) -> Result<(), StoreError> {
    conn.execute_batch(&format!(
        "BEGIN;
         ALTER TABLE profile ADD COLUMN chips INTEGER NOT NULL DEFAULT {STARTING_CHIPS};
         ALTER TABLE profile ADD COLUMN allowance_date TEXT;
         CREATE TABLE casino (
             game         TEXT    PRIMARY KEY,
             rounds       INTEGER NOT NULL,
             wagered      INTEGER NOT NULL,
             net          INTEGER NOT NULL,
             last_played  INTEGER NOT NULL
         );
         PRAGMA user_version = 4;
         COMMIT;"
    ))?;
    Ok(())
}

impl Store {
    /// Saldo chip profil.
    pub fn chips(&self, now: i64) -> Result<i64, StoreError> {
        self.profile(now)?;
        Ok(self
            .conn
            .query_row("SELECT chips FROM profile WHERE id = 1", [], |r| r.get(0))?)
    }

    /// Chip yang sedang dipertaruhkan di meja (ronde yang belum selesai,
    /// termasuk pertandingan tertunda).
    pub fn staked(&self, now: i64) -> Result<i64, StoreError> {
        self.profile(now)?;
        Ok(self
            .conn
            .query_row("SELECT staked FROM profile WHERE id = 1", [], |r| r.get(0))?)
    }

    /// Tunjangan harian (SPEC §6.7): paling banyak sekali per hari kalender
    /// lokal `date` (`YYYY-MM-DD`, dari jam sistem pemain), dan hanya bila
    /// saldo di bawah 1.000; saldo lalu diisi menjadi 2.000. Hari yang
    /// sudah diberi tunjangan tidak diberi lagi. Saldo di sini termasuk chip
    /// yang sedang dipertaruhkan: taruhan yang belum selesai tidak membuat
    /// pemain tampak miskin.
    pub fn daily_allowance(&mut self, date: &str, now: i64) -> Result<Allowance, StoreError> {
        self.profile(now)?;
        let tx = self.conn.transaction()?;
        let (chips, staked, last): (i64, i64, Option<String>) = tx.query_row(
            "SELECT chips, staked, allowance_date FROM profile WHERE id = 1",
            [],
            |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?)),
        )?;
        let before = chips + staked;
        let granted = before < ALLOWANCE_BELOW && last.as_deref() != Some(date);
        let after = if granted { ALLOWANCE_TO } else { before };
        if granted {
            tx.execute(
                "UPDATE profile SET chips = ?1, allowance_date = ?2 WHERE id = 1",
                params![after - staked, date],
            )?;
        }
        tx.commit()?;
        Ok(Allowance {
            before,
            after,
            granted,
        })
    }

    /// Mencatat ronde casino yang selesai: saldo berubah sebesar `net` dan
    /// ringkasan game ikut diperbarui, dalam satu transaksi. Mengembalikan
    /// saldo baru.
    pub fn record_casino(
        &mut self,
        game: &str,
        rounds: i64,
        wagered: i64,
        net: i64,
        now: i64,
    ) -> Result<i64, StoreError> {
        self.profile(now)?;
        let tx = self.conn.transaction()?;
        tx.execute("UPDATE profile SET chips = chips + ?1 WHERE id = 1", [net])?;
        tx.execute(
            "INSERT INTO casino (game, rounds, wagered, net, last_played)
             VALUES (?1, ?2, ?3, ?4, ?5)
             ON CONFLICT (game) DO UPDATE SET
                 rounds = rounds + ?2, wagered = wagered + ?3, net = net + ?4,
                 last_played = ?5",
            params![game, rounds, wagered, net, now],
        )?;
        let chips: i64 =
            tx.query_row("SELECT chips FROM profile WHERE id = 1", [], |r| r.get(0))?;
        tx.commit()?;
        Ok(chips)
    }

    /// Menulis satu titik simpan casino secara atomik; mengembalikan saldo baru.
    pub fn casino_checkpoint(&mut self, c: &CasinoCheckpoint) -> Result<i64, StoreError> {
        self.profile(c.now)?;
        let tx = self.conn.transaction()?;
        tx.execute(
            "UPDATE profile SET chips = chips + ?1 - ?2, staked = staked + ?2 WHERE id = 1",
            params![c.net, c.stake],
        )?;
        if c.rounds > 0 {
            tx.execute(
                "INSERT INTO casino (game, rounds, wagered, net, last_played)
                 VALUES (?1, ?2, ?3, ?4, ?5)
                 ON CONFLICT (game) DO UPDATE SET
                     rounds = rounds + ?2, wagered = wagered + ?3, net = net + ?4,
                     last_played = ?5",
                params![c.game, c.rounds, c.wagered, c.net, c.now],
            )?;
        }
        put_suspended(&tx, c.suspended)?;
        let chips: i64 =
            tx.query_row("SELECT chips FROM profile WHERE id = 1", [], |r| r.get(0))?;
        tx.commit()?;
        Ok(chips)
    }

    /// Ringkasan per game casino, yang terakhir dimainkan dulu.
    pub fn casino_stats(&self) -> Result<Vec<CasinoStats>, StoreError> {
        let mut stmt = self.conn.prepare(
            "SELECT game, rounds, wagered, net, last_played FROM casino
             ORDER BY last_played DESC",
        )?;
        let rows = stmt.query_map([], |r| {
            Ok(CasinoStats {
                game: r.get(0)?,
                rounds: r.get(1)?,
                wagered: r.get(2)?,
                net: r.get(3)?,
                last_played: r.get(4)?,
            })
        })?;
        Ok(rows.collect::<Result<_, _>>()?)
    }

    /// Ringkasan satu game casino, bila pernah dimainkan.
    pub fn casino_game(&self, game: &str) -> Result<Option<CasinoStats>, StoreError> {
        Ok(self
            .conn
            .query_row(
                "SELECT game, rounds, wagered, net, last_played FROM casino WHERE game = ?1",
                [game],
                |r| {
                    Ok(CasinoStats {
                        game: r.get(0)?,
                        rounds: r.get(1)?,
                        wagered: r.get(2)?,
                        net: r.get(3)?,
                        last_played: r.get(4)?,
                    })
                },
            )
            .optional()?)
    }
}
