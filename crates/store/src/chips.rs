//! Ekonomi chip profil (SPEC §6.7; D-056, D-058): saldo awal 10.000,
//! tunjangan harian, dan ringkasan menang/kalah terhadap bandar per game
//! casino. Chip tidak bisa dibeli, dicairkan, atau dipindahkan.

use rusqlite::{Connection, OptionalExtension, params};

use crate::{Store, StoreError};

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

    /// Tunjangan harian (SPEC §6.7): paling banyak sekali per hari kalender
    /// lokal `date` (`YYYY-MM-DD`, dari jam sistem pemain), dan hanya bila
    /// saldo di bawah 1.000; saldo lalu diisi menjadi 2.000. Hari yang
    /// sudah diberi tunjangan tidak diberi lagi.
    pub fn daily_allowance(&mut self, date: &str, now: i64) -> Result<Allowance, StoreError> {
        self.profile(now)?;
        let tx = self.conn.transaction()?;
        let (before, last): (i64, Option<String>) = tx.query_row(
            "SELECT chips, allowance_date FROM profile WHERE id = 1",
            [],
            |r| Ok((r.get(0)?, r.get(1)?)),
        )?;
        let granted = before < ALLOWANCE_BELOW && last.as_deref() != Some(date);
        let after = if granted { ALLOWANCE_TO } else { before };
        if granted {
            tx.execute(
                "UPDATE profile SET chips = ?1, allowance_date = ?2 WHERE id = 1",
                params![after, date],
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
