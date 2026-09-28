//! SQLite lewat `rusqlite` (fitur `bundled`), SPEC §3, §5.1.
//!
//! M1: replay setiap pertandingan (SPEC §2.5).
//! Rev. 9: pertandingan yang ditunda (SPEC §4), satu per game.
//! M3: profil, rating lokal, dan riwayat hasil ([`profile`]).
//! Chip menyusul di M4 sebagai migrasi berikutnya.

use std::path::Path;

pub mod profile;

pub use profile::{GameRecord, GameStats, HistoryRow, NAME_MAX, Profile, StoredRating};

use kyusin_core::replay::Replay;
use kyusin_core::{GameResult, SeatKind};
use rusqlite::{Connection, OptionalExtension, params};

/// Versi skema; dinaikkan tiap migrasi.
const SCHEMA_VERSION: i32 = 3;

pub struct Store {
    conn: Connection,
}

#[derive(Debug)]
pub enum StoreError {
    Sql(rusqlite::Error),
    Json(serde_json::Error),
    /// Basis data dibuat versi aplikasi yang lebih baru.
    TooNew(i32),
}

impl std::fmt::Display for StoreError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            StoreError::Sql(e) => write!(f, "sqlite: {e}"),
            StoreError::Json(e) => write!(f, "json: {e}"),
            StoreError::TooNew(v) => write!(f, "skema versi {v} lebih baru dari aplikasi"),
        }
    }
}

impl std::error::Error for StoreError {}

impl From<rusqlite::Error> for StoreError {
    fn from(e: rusqlite::Error) -> Self {
        StoreError::Sql(e)
    }
}

impl From<serde_json::Error> for StoreError {
    fn from(e: serde_json::Error) -> Self {
        StoreError::Json(e)
    }
}

/// Ringkasan replay untuk daftar.
#[derive(Debug, Clone, PartialEq)]
pub struct ReplaySummary {
    pub id: i64,
    pub game: String,
    /// Waktu mulai (epoch ms), dari jam dinding aplikasi (bukan dari game).
    pub started_at: i64,
    pub finished: bool,
    pub moves: usize,
    pub seats: Vec<SeatKind>,
    pub result: Option<GameResult>,
}

/// Pertandingan yang ditunda. Isi `data` ditentukan aplikasi (keadaan
/// pertandingan + jam); store hanya menyimpannya.
#[derive(Debug, Clone, PartialEq)]
pub struct Suspended {
    pub game: String,
    pub started_at: i64,
    pub suspended_at: i64,
    pub data: serde_json::Value,
}

impl Store {
    pub fn open(path: &Path) -> Result<Store, StoreError> {
        Self::init(Connection::open(path)?)
    }

    pub fn open_in_memory() -> Result<Store, StoreError> {
        Self::init(Connection::open_in_memory()?)
    }

    fn init(conn: Connection) -> Result<Store, StoreError> {
        let version: i32 = conn.query_row("PRAGMA user_version", [], |r| r.get(0))?;
        if version > SCHEMA_VERSION {
            return Err(StoreError::TooNew(version));
        }
        if version < 1 {
            conn.execute_batch(
                "BEGIN;
                 CREATE TABLE replays (
                     id          INTEGER PRIMARY KEY,
                     game        TEXT    NOT NULL,
                     started_at  INTEGER NOT NULL,
                     finished    INTEGER NOT NULL,
                     moves       INTEGER NOT NULL,
                     data        TEXT    NOT NULL
                 );
                 CREATE INDEX replays_game ON replays (game, started_at DESC);
                 PRAGMA user_version = 1;
                 COMMIT;",
            )?;
        }
        if version < 2 {
            conn.execute_batch(
                "BEGIN;
                 CREATE TABLE suspended (
                     game          TEXT    PRIMARY KEY,
                     started_at    INTEGER NOT NULL,
                     suspended_at  INTEGER NOT NULL,
                     data          TEXT    NOT NULL
                 );
                 PRAGMA user_version = 2;
                 COMMIT;",
            )?;
        }
        if version < 3 {
            profile::migrate_v3(&conn)?;
        }
        Ok(Store { conn })
    }

    /// Menyimpan replay; mengembalikan id-nya.
    pub fn save_replay(&self, replay: &Replay, started_at: i64) -> Result<i64, StoreError> {
        self.conn.execute(
            "INSERT INTO replays (game, started_at, finished, moves, data) VALUES (?1, ?2, ?3, ?4, ?5)",
            params![
                replay.game,
                started_at,
                replay.result.is_some() as i32,
                replay.moves.len() as i64,
                serde_json::to_string(replay)?
            ],
        )?;
        Ok(self.conn.last_insert_rowid())
    }

    /// Replay terbaru dulu, opsional untuk satu game saja.
    pub fn list_replays(
        &self,
        game: Option<&str>,
        limit: usize,
    ) -> Result<Vec<ReplaySummary>, StoreError> {
        let mut stmt = self.conn.prepare(
            "SELECT id, started_at, finished, data FROM replays
             WHERE (?1 IS NULL OR game = ?1)
             ORDER BY started_at DESC, id DESC LIMIT ?2",
        )?;
        let rows = stmt.query_map(params![game, limit as i64], |r| {
            Ok((
                r.get::<_, i64>(0)?,
                r.get::<_, i64>(1)?,
                r.get::<_, i32>(2)?,
                r.get::<_, String>(3)?,
            ))
        })?;
        let mut out = Vec::new();
        for row in rows {
            let (id, started_at, finished, data) = row?;
            let replay: Replay = serde_json::from_str(&data)?;
            out.push(ReplaySummary {
                id,
                game: replay.game,
                started_at,
                finished: finished != 0,
                moves: replay.moves.len(),
                seats: replay.seats,
                result: replay.result,
            });
        }
        Ok(out)
    }

    /// Menyimpan (atau mengganti) pertandingan tertunda untuk satu game.
    pub fn save_suspended(&self, s: &Suspended) -> Result<(), StoreError> {
        self.conn.execute(
            "INSERT OR REPLACE INTO suspended (game, started_at, suspended_at, data)
             VALUES (?1, ?2, ?3, ?4)",
            params![
                s.game,
                s.started_at,
                s.suspended_at,
                serde_json::to_string(&s.data)?
            ],
        )?;
        Ok(())
    }

    /// Mengambil dan sekaligus menghapus pertandingan tertunda sebuah game.
    pub fn take_suspended(&self, game: &str) -> Result<Option<Suspended>, StoreError> {
        let found = self.find_suspended(game)?;
        if found.is_some() {
            self.conn
                .execute("DELETE FROM suspended WHERE game = ?1", [game])?;
        }
        Ok(found)
    }

    pub fn find_suspended(&self, game: &str) -> Result<Option<Suspended>, StoreError> {
        Ok(self.list_suspended()?.into_iter().find(|s| s.game == game))
    }

    /// Semua pertandingan tertunda, terbaru dulu.
    pub fn list_suspended(&self) -> Result<Vec<Suspended>, StoreError> {
        let mut stmt = self.conn.prepare(
            "SELECT game, started_at, suspended_at, data FROM suspended
             ORDER BY suspended_at DESC",
        )?;
        let rows = stmt.query_map([], |r| {
            Ok((
                r.get::<_, String>(0)?,
                r.get::<_, i64>(1)?,
                r.get::<_, i64>(2)?,
                r.get::<_, String>(3)?,
            ))
        })?;
        let mut out = Vec::new();
        for row in rows {
            let (game, started_at, suspended_at, data) = row?;
            out.push(Suspended {
                game,
                started_at,
                suspended_at,
                data: serde_json::from_str(&data)?,
            });
        }
        Ok(out)
    }

    pub fn load_replay(&self, id: i64) -> Result<Option<Replay>, StoreError> {
        let data: Option<String> = self
            .conn
            .query_row("SELECT data FROM replays WHERE id = ?1", [id], |r| r.get(0))
            .optional()?;
        Ok(match data {
            Some(d) => Some(serde_json::from_str(&d)?),
            None => None,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use kyusin_core::fair::FairRecord;
    use kyusin_core::rating::{Outcome, Rating};
    use kyusin_core::replay::{Move, REPLAY_FORMAT};

    fn replay(game: &str, moves: usize, finished: bool) -> Replay {
        Replay {
            format: REPLAY_FORMAT,
            game: game.into(),
            config: serde_json::Value::Null,
            seats: vec![SeatKind::Human, SeatKind::Bot { level: 2 }],
            fair: FairRecord {
                host: "host".into(),
                commitments: Default::default(),
                seeds: Default::default(),
                excluded: vec![],
                round_seed: "00".repeat(32),
            },
            moves: (0..moves)
                .map(|i| Move {
                    seat: (i % 2) as u8,
                    command: "d3".into(),
                })
                .collect(),
            result: finished.then(|| GameResult {
                winners: vec![0],
                scores: vec![40, 24],
                summary: kyusin_core::Localized {
                    id: "a".into(),
                    en: "b".into(),
                },
            }),
            state_hash: "ab".repeat(32),
        }
    }

    #[test]
    fn save_list_load() {
        let s = Store::open_in_memory().unwrap();
        let a = s.save_replay(&replay("reversi", 3, true), 1000).unwrap();
        let b = s.save_replay(&replay("reversi", 5, false), 2000).unwrap();
        s.save_replay(&replay("catur", 1, true), 3000).unwrap();

        let all = s.list_replays(None, 10).unwrap();
        assert_eq!(all.len(), 3);
        assert_eq!(all[0].game, "catur");

        let rev = s.list_replays(Some("reversi"), 10).unwrap();
        assert_eq!(rev.iter().map(|r| r.id).collect::<Vec<_>>(), vec![b, a]);
        assert!(!rev[0].finished && rev[1].finished);
        assert_eq!(rev[1].moves, 3);
        assert_eq!(rev[1].seats[1], SeatKind::Bot { level: 2 });

        assert_eq!(
            s.load_replay(a).unwrap().unwrap(),
            replay("reversi", 3, true)
        );
        assert!(s.load_replay(999).unwrap().is_none());
        assert_eq!(s.list_replays(None, 1).unwrap().len(), 1);
    }

    fn rated(before: f64, after: f64) -> Option<(Rating, Rating)> {
        let r = |x| Rating {
            rating: x,
            rd: 200.0,
            vol: 0.06,
        };
        Some((r(before), r(after)))
    }

    #[test]
    fn single_profile_with_renamable_name() {
        let s = Store::open_in_memory().unwrap();
        let p = s.profile(100).unwrap();
        assert_eq!(p.name, None);
        assert_eq!(p.created_at, 100);
        // Tetap satu profil: waktu buat tidak berubah.
        assert_eq!(s.profile(999).unwrap().created_at, 100);
        let p = s.set_profile_name("  Mufuyu  ", 5).unwrap();
        assert_eq!(p.name.as_deref(), Some("Mufuyu"));
        let long = "x".repeat(40);
        assert_eq!(
            s.set_profile_name(&long, 5)
                .unwrap()
                .name
                .unwrap()
                .chars()
                .count(),
            NAME_MAX
        );
        assert_eq!(s.set_profile_name("   ", 5).unwrap().name, None);
        let n: i64 = s
            .conn
            .query_row("SELECT COUNT(*) FROM profile", [], |r| r.get(0))
            .unwrap();
        assert_eq!(n, 1);
    }

    #[test]
    fn results_history_ratings_and_stats() {
        let mut s = Store::open_in_memory().unwrap();
        assert_eq!(s.last_game().unwrap(), None);
        let rec = |game: &str, at: i64, outcome, rating| GameRecord {
            game: game.into(),
            finished_at: at,
            replay_id: Some(at),
            opponent_level: Some(1),
            outcome,
            rating,
        };
        s.record_result(&rec("catur", 10, Outcome::Win, rated(1500.0, 1580.0)))
            .unwrap();
        s.record_result(&rec("catur", 20, Outcome::Loss, rated(1580.0, 1540.0)))
            .unwrap();
        s.record_result(&rec("reversi", 30, Outcome::Draw, None))
            .unwrap();
        s.record_result(&rec("catur", 40, Outcome::Draw, None))
            .unwrap();

        let r = s.rating("catur").unwrap().unwrap();
        assert_eq!(r.rating.rating, 1540.0);
        assert_eq!(r.games, 2, "hasil tanpa rating tidak dihitung");
        assert_eq!(r.best, 1580.0);
        assert_eq!(s.rating("reversi").unwrap(), None);

        let h = s.history(Some("catur"), 10).unwrap();
        assert_eq!(h.len(), 3);
        assert_eq!(h[0].record.finished_at, 40);
        assert_eq!(h[1].record.rating.unwrap().1.rating, 1540.0);
        assert_eq!(s.history(None, 2).unwrap().len(), 2);

        let stats = s.game_stats().unwrap();
        assert_eq!(stats[0].game, "catur");
        assert_eq!(
            (
                stats[0].played,
                stats[0].wins,
                stats[0].draws,
                stats[0].losses
            ),
            (3, 1, 1, 1)
        );
        assert_eq!(stats[0].last_played, 40);
        assert!(stats[0].rating.is_some());
        assert_eq!(stats[1].game, "reversi");
        assert!(stats[1].rating.is_none());
        assert_eq!(s.last_game().unwrap().as_deref(), Some("catur"));
    }

    #[test]
    fn suspended_one_per_game_taken_once() {
        let s = Store::open_in_memory().unwrap();
        let rec = |game: &str, at: i64, n: i64| Suspended {
            game: game.into(),
            started_at: 1,
            suspended_at: at,
            data: serde_json::json!({ "n": n }),
        };
        s.save_suspended(&rec("catur", 10, 1)).unwrap();
        s.save_suspended(&rec("reversi", 20, 2)).unwrap();
        // Game yang sama: yang baru menggantikan yang lama.
        s.save_suspended(&rec("catur", 30, 3)).unwrap();
        let all = s.list_suspended().unwrap();
        assert_eq!(all.len(), 2);
        assert_eq!(all[0], rec("catur", 30, 3));
        assert_eq!(
            s.find_suspended("reversi").unwrap(),
            Some(rec("reversi", 20, 2))
        );
        assert_eq!(
            s.take_suspended("catur").unwrap(),
            Some(rec("catur", 30, 3))
        );
        assert_eq!(s.take_suspended("catur").unwrap(), None);
        assert_eq!(s.list_suspended().unwrap().len(), 1);
    }

    #[test]
    fn reopening_keeps_data_and_schema() {
        let dir = std::env::temp_dir().join(format!("kyusin-store-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join("t.sqlite");
        let _ = std::fs::remove_file(&path);
        {
            let s = Store::open(&path).unwrap();
            s.save_replay(&replay("reversi", 2, true), 5).unwrap();
        }
        // Basis data versi 1 (sebelum Rev. 9) dimigrasikan tanpa kehilangan
        // replay; replay yang selesai masuk riwayat M3 tanpa rating.
        {
            let conn = Connection::open(&path).unwrap();
            conn.execute_batch(
                "DROP TABLE suspended; DROP TABLE profile; DROP TABLE ratings;
                 DROP TABLE results; PRAGMA user_version = 1;",
            )
            .unwrap();
        }
        let s = Store::open(&path).unwrap();
        assert_eq!(s.list_replays(None, 10).unwrap().len(), 1);
        assert!(s.list_suspended().unwrap().is_empty());
        let h = s.history(None, 10).unwrap();
        assert_eq!(h.len(), 1);
        assert_eq!(h[0].record.outcome, Outcome::Win);
        assert_eq!(h[0].record.opponent_level, Some(2));
        assert_eq!(h[0].record.rating, None);
        let v: i32 = s
            .conn
            .query_row("PRAGMA user_version", [], |r| r.get(0))
            .unwrap();
        assert_eq!(v, SCHEMA_VERSION);
        drop(s);
        std::fs::remove_dir_all(&dir).unwrap();
    }
}
