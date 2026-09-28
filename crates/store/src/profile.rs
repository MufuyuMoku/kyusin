//! Profil, rating lokal, dan riwayat hasil (SPEC §8, §9 M3; D-050).
//!
//! Satu profil per instalasi, namanya bisa diganti. Rating Glicko-2 per
//! game kompetitif. Setiap pertandingan yang selesai dicatat di riwayat;
//! pertandingan yang ditunda baru tercatat setelah selesai.

use kyusin_core::SeatKind;
use kyusin_core::rating::{Outcome, Rating};
use kyusin_core::replay::Replay;
use rusqlite::{Connection, OptionalExtension, params};

use crate::{Store, StoreError};

/// Nama profil paling panjang (karakter).
pub const NAME_MAX: usize = 24;

#[derive(Debug, Clone, PartialEq)]
pub struct Profile {
    /// `None` sampai pemain memberi nama.
    pub name: Option<String>,
    pub created_at: i64,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct StoredRating {
    pub rating: Rating,
    /// Jumlah pertandingan yang dihitung ke rating.
    pub games: u32,
    /// Rating tertinggi yang pernah dicapai.
    pub best: f64,
    pub updated_at: i64,
}

/// Satu hasil pertandingan untuk riwayat.
#[derive(Debug, Clone, PartialEq)]
pub struct GameRecord {
    pub game: String,
    pub finished_at: i64,
    pub replay_id: Option<i64>,
    /// Level bot lawan, bila lawannya bot.
    pub opponent_level: Option<u8>,
    pub outcome: Outcome,
    /// Rating sebelum dan sesudah, bila pertandingan ini dihitung.
    pub rating: Option<(Rating, Rating)>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct HistoryRow {
    pub id: i64,
    pub record: GameRecord,
}

/// Ringkasan per game untuk halaman statistik.
#[derive(Debug, Clone, PartialEq)]
pub struct GameStats {
    pub game: String,
    pub played: u32,
    pub wins: u32,
    pub draws: u32,
    pub losses: u32,
    pub last_played: i64,
    pub rating: Option<StoredRating>,
}

fn outcome_text(o: Outcome) -> &'static str {
    match o {
        Outcome::Win => "win",
        Outcome::Draw => "draw",
        Outcome::Loss => "loss",
    }
}

fn outcome_of(text: &str) -> Outcome {
    match text {
        "win" => Outcome::Win,
        "draw" => Outcome::Draw,
        _ => Outcome::Loss,
    }
}

/// Hasil bagi kursi manusia pertama dari sebuah replay yang selesai.
pub fn human_outcome(replay: &Replay) -> Option<(Outcome, Option<u8>)> {
    let result = replay.result.as_ref()?;
    let human = replay.seats.iter().position(|s| *s == SeatKind::Human)? as u8;
    let level = replay.seats.iter().find_map(|s| match s {
        SeatKind::Bot { level } => Some(*level),
        _ => None,
    });
    let outcome = if result.winners.is_empty() {
        Outcome::Draw
    } else if result.winners.contains(&human) {
        Outcome::Win
    } else {
        Outcome::Loss
    };
    Some((outcome, level))
}

/// Migrasi ke skema 3: tabel profil, rating, dan riwayat. Replay yang
/// sudah selesai sebelum M3 dimasukkan ke riwayat tanpa rating.
pub(crate) fn migrate_v3(conn: &Connection) -> Result<(), StoreError> {
    conn.execute_batch(
        "BEGIN;
         CREATE TABLE profile (
             id          INTEGER PRIMARY KEY CHECK (id = 1),
             name        TEXT,
             created_at  INTEGER NOT NULL
         );
         CREATE TABLE ratings (
             game        TEXT    PRIMARY KEY,
             rating      REAL    NOT NULL,
             rd          REAL    NOT NULL,
             vol         REAL    NOT NULL,
             games       INTEGER NOT NULL,
             best        REAL    NOT NULL,
             updated_at  INTEGER NOT NULL
         );
         CREATE TABLE results (
             id              INTEGER PRIMARY KEY,
             game            TEXT    NOT NULL,
             finished_at     INTEGER NOT NULL,
             replay_id       INTEGER,
             opponent_level  INTEGER,
             outcome         TEXT    NOT NULL,
             rated           INTEGER NOT NULL,
             rating_before   REAL,
             rd_before       REAL,
             rating_after    REAL,
             rd_after        REAL
         );
         CREATE INDEX results_game ON results (game, finished_at DESC);",
    )?;
    let backfill = || -> Result<(), StoreError> {
        let mut stmt = conn.prepare(
            "SELECT id, started_at, data FROM replays WHERE finished = 1 ORDER BY started_at, id",
        )?;
        let rows: Vec<(i64, i64, String)> = stmt
            .query_map([], |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?)))?
            .collect::<Result<_, _>>()?;
        for (id, started, data) in rows {
            let replay: Replay = serde_json::from_str(&data)?;
            if let Some((outcome, level)) = human_outcome(&replay) {
                conn.execute(
                    "INSERT INTO results (game, finished_at, replay_id, opponent_level, outcome, rated)
                     VALUES (?1, ?2, ?3, ?4, ?5, 0)",
                    params![replay.game, started, id, level, outcome_text(outcome)],
                )?;
            }
        }
        Ok(())
    };
    match backfill() {
        Ok(()) => {
            conn.execute_batch("PRAGMA user_version = 3; COMMIT;")?;
            Ok(())
        }
        Err(e) => {
            let _ = conn.execute_batch("ROLLBACK;");
            Err(e)
        }
    }
}

fn stored_rating(r: &rusqlite::Row<'_>, at: usize) -> rusqlite::Result<StoredRating> {
    Ok(StoredRating {
        rating: Rating {
            rating: r.get(at)?,
            rd: r.get(at + 1)?,
            vol: r.get(at + 2)?,
        },
        games: r.get(at + 3)?,
        best: r.get(at + 4)?,
        updated_at: r.get(at + 5)?,
    })
}

impl Store {
    /// Profil satu-satunya; dibuat saat pertama diminta.
    pub fn profile(&self, now: i64) -> Result<Profile, StoreError> {
        self.conn.execute(
            "INSERT OR IGNORE INTO profile (id, name, created_at) VALUES (1, NULL, ?1)",
            [now],
        )?;
        Ok(self.conn.query_row(
            "SELECT name, created_at FROM profile WHERE id = 1",
            [],
            |r| {
                Ok(Profile {
                    name: r.get(0)?,
                    created_at: r.get(1)?,
                })
            },
        )?)
    }

    /// Mengganti nama profil. Spasi di tepi dibuang; kosong = tanpa nama;
    /// dipotong ke [`NAME_MAX`] karakter.
    pub fn set_profile_name(&self, name: &str, now: i64) -> Result<Profile, StoreError> {
        self.profile(now)?;
        let name: String = name.trim().chars().take(NAME_MAX).collect();
        let name = (!name.is_empty()).then_some(name);
        self.conn
            .execute("UPDATE profile SET name = ?1 WHERE id = 1", [&name])?;
        self.profile(now)
    }

    pub fn rating(&self, game: &str) -> Result<Option<StoredRating>, StoreError> {
        Ok(self
            .conn
            .query_row(
                "SELECT rating, rd, vol, games, best, updated_at FROM ratings WHERE game = ?1",
                [game],
                |r| stored_rating(r, 0),
            )
            .optional()?)
    }

    /// Mencatat hasil; bila dihitung (`record.rating`), rating game ikut
    /// diperbarui dalam transaksi yang sama.
    pub fn record_result(&mut self, record: &GameRecord) -> Result<i64, StoreError> {
        let tx = self.conn.transaction()?;
        let (before, after) = match record.rating {
            Some((b, a)) => (Some(b), Some(a)),
            None => (None, None),
        };
        tx.execute(
            "INSERT INTO results (game, finished_at, replay_id, opponent_level, outcome, rated,
                                  rating_before, rd_before, rating_after, rd_after)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)",
            params![
                record.game,
                record.finished_at,
                record.replay_id,
                record.opponent_level,
                outcome_text(record.outcome),
                record.rating.is_some() as i32,
                before.map(|r| r.rating),
                before.map(|r| r.rd),
                after.map(|r| r.rating),
                after.map(|r| r.rd),
            ],
        )?;
        let id = tx.last_insert_rowid();
        if let Some(a) = after {
            tx.execute(
                "INSERT INTO ratings (game, rating, rd, vol, games, best, updated_at)
                 VALUES (?1, ?2, ?3, ?4, 1, ?2, ?5)
                 ON CONFLICT (game) DO UPDATE SET
                     rating = ?2, rd = ?3, vol = ?4, games = games + 1,
                     best = MAX(best, ?2), updated_at = ?5",
                params![record.game, a.rating, a.rd, a.vol, record.finished_at],
            )?;
        }
        tx.commit()?;
        Ok(id)
    }

    /// Riwayat terbaru dulu, opsional untuk satu game saja.
    pub fn history(&self, game: Option<&str>, limit: usize) -> Result<Vec<HistoryRow>, StoreError> {
        let mut stmt = self.conn.prepare(
            "SELECT id, game, finished_at, replay_id, opponent_level, outcome,
                    rating_before, rd_before, rating_after, rd_after
             FROM results WHERE (?1 IS NULL OR game = ?1)
             ORDER BY finished_at DESC, id DESC LIMIT ?2",
        )?;
        let rows = stmt.query_map(params![game, limit as i64], |r| {
            let before: Option<(f64, f64)> = match (r.get(6)?, r.get(7)?) {
                (Some(a), Some(b)) => Some((a, b)),
                _ => None,
            };
            let after: Option<(f64, f64)> = match (r.get(8)?, r.get(9)?) {
                (Some(a), Some(b)) => Some((a, b)),
                _ => None,
            };
            let rating = match (before, after) {
                (Some(b), Some(a)) => Some((
                    Rating {
                        rating: b.0,
                        rd: b.1,
                        vol: 0.0,
                    },
                    Rating {
                        rating: a.0,
                        rd: a.1,
                        vol: 0.0,
                    },
                )),
                _ => None,
            };
            Ok(HistoryRow {
                id: r.get(0)?,
                record: GameRecord {
                    game: r.get(1)?,
                    finished_at: r.get(2)?,
                    replay_id: r.get(3)?,
                    opponent_level: r.get(4)?,
                    outcome: outcome_of(&r.get::<_, String>(5)?),
                    rating,
                },
            })
        })?;
        Ok(rows.collect::<Result<_, _>>()?)
    }

    /// Ringkasan per game yang pernah dimainkan, yang terakhir dimainkan dulu.
    pub fn game_stats(&self) -> Result<Vec<GameStats>, StoreError> {
        let mut stmt = self.conn.prepare(
            "SELECT r.game, COUNT(*),
                    SUM(r.outcome = 'win'), SUM(r.outcome = 'draw'), SUM(r.outcome = 'loss'),
                    MAX(r.finished_at),
                    g.rating, g.rd, g.vol, g.games, g.best, g.updated_at
             FROM results r LEFT JOIN ratings g ON g.game = r.game
             GROUP BY r.game ORDER BY MAX(r.finished_at) DESC",
        )?;
        let rows = stmt.query_map([], |r| {
            let rating = match r.get::<_, Option<f64>>(6)? {
                Some(_) => Some(stored_rating(r, 6)?),
                None => None,
            };
            Ok(GameStats {
                game: r.get(0)?,
                played: r.get(1)?,
                wins: r.get(2)?,
                draws: r.get(3)?,
                losses: r.get(4)?,
                last_played: r.get(5)?,
                rating,
            })
        })?;
        Ok(rows.collect::<Result<_, _>>()?)
    }

    /// Game dari pertandingan terakhir yang selesai (untuk sapaan).
    pub fn last_game(&self) -> Result<Option<String>, StoreError> {
        Ok(self
            .conn
            .query_row(
                "SELECT game FROM results ORDER BY finished_at DESC, id DESC LIMIT 1",
                [],
                |r| r.get(0),
            )
            .optional()?)
    }
}
