//! SQLite lewat `rusqlite` (fitur `bundled`), SPEC §3, §5.1.
//!
//! M1: replay setiap pertandingan (SPEC §2.5). Profil, chip, dan riwayat
//! menyusul di M3/M4 sebagai migrasi berikutnya.

use std::path::Path;

use kyusin_core::replay::Replay;
use kyusin_core::{GameResult, SeatKind};
use rusqlite::{Connection, OptionalExtension, params};

/// Versi skema; dinaikkan tiap migrasi.
const SCHEMA_VERSION: i32 = 1;

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
        let s = Store::open(&path).unwrap();
        assert_eq!(s.list_replays(None, 10).unwrap().len(), 1);
        drop(s);
        std::fs::remove_dir_all(&dir).unwrap();
    }
}
