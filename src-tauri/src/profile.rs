//! Profil, rating lokal, riwayat, dan statistik (SPEC §8, §9 M3; D-050).
//!
//! Satu profil per instalasi. Setiap pertandingan yang selesai dicatat;
//! melawan bot yang punya data kalibrasi di game kompetitif, rating Glicko-2
//! pemain diperbarui. Pertandingan yang ditunda baru dihitung setelah
//! selesai.

use kyusin_core::Localized;
use kyusin_core::rating::{Rating, TAU};
use kyusin_core::replay::Replay;
use kyusin_store::{GameRecord, profile::human_outcome};
use serde::Serialize;
use tauri::State;

use crate::AppState;

fn store_error(e: impl ToString) -> Localized {
    kyusin_core::i18n::core().localized("error.store", &[("detail", &e.to_string())])
}

pub(crate) fn now_ms() -> i64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_millis() as i64)
        .unwrap_or(0)
}

/// Perubahan rating dari satu pertandingan.
#[derive(Debug, Clone, Copy, PartialEq, Serialize)]
pub(crate) struct RatingChange {
    pub(crate) before: f64,
    pub(crate) after: f64,
    pub(crate) rd: f64,
}

/// Mencatat hasil pertandingan yang selesai ke riwayat, dan memperbarui
/// rating bila game-nya kompetitif dan lawannya bot yang terkalibrasi.
pub(crate) fn record(
    state: &AppState,
    replay: &Replay,
    replay_id: Option<i64>,
    finished_at: i64,
) -> Option<RatingChange> {
    let (outcome, level) = human_outcome(replay)?;
    let competitive = state.registry.get(&replay.game)?.manifest.competitive;
    let store = state.store.as_ref().ok()?;
    let mut s = store.lock().unwrap();
    let rating = match (competitive, level) {
        (true, Some(l)) => kyusin_bots::calibration::bot_rating(&replay.game, l).map(|opp| {
            let before = s
                .rating(&replay.game)
                .ok()
                .flatten()
                .map(|r| r.rating)
                .unwrap_or_else(Rating::new_player);
            (before, before.update(&[(opp, outcome)], TAU))
        }),
        _ => None,
    };
    s.record_result(&GameRecord {
        game: replay.game.clone(),
        finished_at,
        replay_id,
        opponent_level: level,
        outcome,
        rating,
    })
    .ok()?;
    rating.map(|(b, a)| RatingChange {
        before: b.rating,
        after: a.rating,
        rd: a.rd,
    })
}

#[derive(Serialize)]
pub(crate) struct ProfileDto {
    name: Option<String>,
    created_at: i64,
    /// Game dari pertandingan terakhir yang selesai.
    last_game: Option<String>,
    name_max: usize,
}

pub(crate) fn profile_dto(state: &AppState) -> Result<ProfileDto, Localized> {
    let store = state.store.as_ref().map_err(store_error)?;
    let s = store.lock().unwrap();
    let p = s.profile(now_ms()).map_err(store_error)?;
    Ok(ProfileDto {
        name: p.name,
        created_at: p.created_at,
        last_game: s.last_game().map_err(store_error)?,
        name_max: kyusin_store::NAME_MAX,
    })
}

#[tauri::command]
pub(crate) fn profile_get(state: State<'_, AppState>) -> Result<ProfileDto, Localized> {
    profile_dto(&state)
}

#[tauri::command]
pub(crate) fn profile_set_name(
    name: String,
    state: State<'_, AppState>,
) -> Result<ProfileDto, Localized> {
    {
        let store = state.store.as_ref().map_err(store_error)?;
        store
            .lock()
            .unwrap()
            .set_profile_name(&name, now_ms())
            .map_err(store_error)?;
    }
    profile_dto(&state)
}

#[derive(Serialize)]
pub(crate) struct RatingDto {
    rating: f64,
    rd: f64,
    games: u32,
    best: f64,
}

#[derive(Serialize)]
pub(crate) struct GameStatsDto {
    game: String,
    played: u32,
    wins: u32,
    draws: u32,
    losses: u32,
    last_played: i64,
    rating: Option<RatingDto>,
}

#[derive(Serialize)]
pub(crate) struct HistoryDto {
    id: i64,
    game: String,
    finished_at: i64,
    replay_id: Option<i64>,
    opponent_level: Option<u8>,
    outcome: kyusin_core::rating::Outcome,
    rating: Option<RatingChange>,
}

#[derive(Serialize)]
pub(crate) struct StatsDto {
    games: Vec<GameStatsDto>,
    history: Vec<HistoryDto>,
}

/// Jumlah baris riwayat di halaman statistik.
const HISTORY_ROWS: usize = 30;

pub(crate) fn stats_dto(state: &AppState, game: Option<&str>) -> Result<StatsDto, Localized> {
    let store = state.store.as_ref().map_err(store_error)?;
    let s = store.lock().unwrap();
    let games = s
        .game_stats()
        .map_err(store_error)?
        .into_iter()
        .map(|g| GameStatsDto {
            game: g.game,
            played: g.played,
            wins: g.wins,
            draws: g.draws,
            losses: g.losses,
            last_played: g.last_played,
            rating: g.rating.map(|r| RatingDto {
                rating: r.rating.rating,
                rd: r.rating.rd,
                games: r.games,
                best: r.best,
            }),
        })
        .collect();
    let history = s
        .history(game, HISTORY_ROWS)
        .map_err(store_error)?
        .into_iter()
        .map(|h| HistoryDto {
            id: h.id,
            game: h.record.game,
            finished_at: h.record.finished_at,
            replay_id: h.record.replay_id,
            opponent_level: h.record.opponent_level,
            outcome: h.record.outcome,
            rating: h.record.rating.map(|(b, a)| RatingChange {
                before: b.rating,
                after: a.rating,
                rd: a.rd,
            }),
        })
        .collect();
    Ok(StatsDto { games, history })
}

#[tauri::command]
pub(crate) fn stats(
    game: Option<String>,
    state: State<'_, AppState>,
) -> Result<StatsDto, Localized> {
    stats_dto(&state, game.as_deref())
}
