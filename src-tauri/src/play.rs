//! Bermain melawan bot, provably fair, replay (SPEC §2.3, §2.5, §5.4).
//!
//! Singleplayer: aplikasi berperan sebagai host dan pemain lokal sebagai
//! peserta. Komitmen keduanya diumumkan sebelum langkah pertama; seed baru
//! dibuka setelah permainan selesai, lalu `verify` dijalankan otomatis dan
//! hasilnya ditampilkan. Setiap pertandingan disimpan sebagai replay,
//! termasuk yang ditinggalkan di tengah jalan.

use std::collections::BTreeMap;
use std::time::{SystemTime, UNIX_EPOCH};

use kyusin_core::fair::{FairRecord, FairRound, commitment};
use kyusin_core::hash::unhex32;
use kyusin_core::i18n::core;
use kyusin_core::replay::{Frame, Move, Replay, VerifyReport};
use kyusin_core::{GameResult, Human, Localized, Match, Player, SeatKind, Seed};
use kyusin_store::ReplaySummary;
use serde::Serialize;
use tauri::State;

use crate::{ActionDto, AppState, action_dtos, unknown_game};

const HOST: &str = "host";
const PLAYER: &str = "player";

pub(crate) struct Running {
    game: String,
    m: Match,
    human: u8,
    started_at: i64,
    replay_id: Option<i64>,
    verify: Option<VerifyReport>,
    save_error: Option<String>,
}

#[derive(Serialize)]
pub(crate) struct MatchDto {
    game: String,
    /// Kursi pemain lokal.
    seat: u8,
    seats: Vec<SeatKind>,
    your_turn: bool,
    /// Giliran bot: UI memanggil `match_step` (dengan jeda kecil).
    bot_turn: bool,
    over: bool,
    view_data: serde_json::Value,
    view_text: Localized,
    actions: Vec<ActionDto>,
    moves: Vec<Move>,
    /// Komitmen (hex SHA-256) yang diumumkan sebelum ronde.
    commitments: BTreeMap<String, String>,
    /// Seed yang dibuka dan seed ronde; hanya setelah selesai.
    reveal: Option<FairRecord>,
    result: Option<GameResult>,
    verify: Option<VerifyReport>,
    replay_id: Option<i64>,
    save_error: Option<String>,
}

fn now_ms() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_millis() as i64)
        .unwrap_or(0)
}

fn random_seed() -> Result<Seed, Localized> {
    let mut seed = [0u8; 32];
    getrandom::fill(&mut seed)
        .map_err(|e| core().localized("error.random", &[("detail", &e.to_string())]))?;
    Ok(seed)
}

fn no_match() -> Localized {
    core().localized("error.no_match", &[])
}

/// Commit-reveal singleplayer: host = aplikasi, peserta = pemain lokal.
fn fair_record(player_seed: Option<&str>) -> Result<FairRecord, Localized> {
    let player_seed = match player_seed.map(str::trim).filter(|s| !s.is_empty()) {
        Some(text) => unhex32(text).map_err(|_| core().localized("error.player_seed", &[]))?,
        None => random_seed()?,
    };
    let mut round = FairRound::new(HOST, random_seed()?, vec![PLAYER.into()]);
    let fail = |e: kyusin_core::fair::FairError| {
        core().localized("error.replay_fair", &[("detail", &e.to_string())])
    };
    round
        .commit(PLAYER, commitment(&player_seed))
        .map_err(fail)?;
    round.close_commits().map_err(fail)?;
    round.reveal(PLAYER, player_seed).map_err(fail)?;
    round.close_reveals().map_err(fail)?;
    round.record().ok_or_else(no_match)
}

fn dto(r: &Running) -> MatchDto {
    let session = r.m.session();
    let pending = session.pending_players();
    let over = session.is_over();
    let your_turn = pending.contains(&r.human);
    let bot_turn = !over
        && pending
            .iter()
            .any(|s| matches!(r.m.seat_kind(*s), Some(SeatKind::Bot { .. })));
    MatchDto {
        game: r.game.clone(),
        seat: r.human,
        seats: (0..session.seats())
            .filter_map(|s| r.m.seat_kind(s))
            .collect(),
        your_turn,
        bot_turn,
        over,
        view_data: session.view_data(r.human),
        view_text: Localized::build(|lang| session.view_text(r.human, lang)),
        actions: if your_turn {
            action_dtos(session.legal_actions(r.human))
        } else {
            Vec::new()
        },
        moves: r.m.moves().to_vec(),
        commitments: r.m.fair().commitments.clone(),
        reveal: over.then(|| r.m.fair().clone()),
        result: session.result(),
        verify: r.verify.clone(),
        replay_id: r.replay_id,
        save_error: r.save_error.clone(),
    }
}

fn save(state: &AppState, replay: &Replay, started_at: i64) -> Result<i64, String> {
    let store = state.store.as_ref().map_err(Clone::clone)?;
    store
        .lock()
        .unwrap()
        .save_replay(replay, started_at)
        .map_err(|e| e.to_string())
}

/// Setelah selesai: simpan replay dan jalankan `verify` otomatis.
fn finish(state: &AppState, r: &mut Running) {
    if !r.m.session().is_over() || r.replay_id.is_some() || r.verify.is_some() {
        return;
    }
    let replay = r.m.replay();
    if let Some(c) = state.registry.get(&r.game) {
        r.verify = Some(replay.verify(c));
    }
    match save(state, &replay, r.started_at) {
        Ok(id) => r.replay_id = Some(id),
        Err(e) => r.save_error = Some(e),
    }
}

/// Pertandingan yang ditinggalkan tetap disimpan sebagai replay (SPEC §2.5).
fn leave(state: &AppState, r: Running) {
    if !r.m.session().is_over() && !r.m.moves().is_empty() {
        let _ = save(state, &r.m.replay(), r.started_at);
    }
}

#[tauri::command]
pub(crate) fn match_start(
    id: String,
    level: u8,
    seat: u8,
    player_seed: Option<String>,
    state: State<'_, AppState>,
) -> Result<MatchDto, Localized> {
    let cartridge = state.registry.get(&id).ok_or_else(|| unknown_game(&id))?;
    let fair = fair_record(player_seed.as_deref())?;
    let round = fair
        .round_seed_bytes()
        .map_err(|e| core().localized("error.replay_fair", &[("detail", &e.to_string())]))?;
    let seats = cartridge.manifest.max_players;
    if seat >= seats {
        return Err(core().localized("error.seat", &[("seat", &seat.to_string())]));
    }
    let mut players: Vec<Box<dyn Player>> = Vec::new();
    for s in 0..seats {
        if s == seat {
            players.push(Box::new(Human));
        } else {
            let bot = kyusin_bots::create(&id, level, &round, s).ok_or_else(|| {
                core().localized("error.no_bot", &[("level", &level.to_string())])
            })?;
            players.push(bot);
        }
    }
    let m =
        Match::new(cartridge, serde_json::Value::Null, fair, players).map_err(|e| e.message())?;
    let running = Running {
        game: id,
        m,
        human: seat,
        started_at: now_ms(),
        replay_id: None,
        verify: None,
        save_error: None,
    };
    let dto = dto(&running);
    if let Some(old) = state.play.lock().unwrap().replace(running) {
        leave(&state, old);
    }
    Ok(dto)
}

#[tauri::command]
pub(crate) fn match_act(
    command: String,
    state: State<'_, AppState>,
) -> Result<MatchDto, Localized> {
    let mut guard = state.play.lock().unwrap();
    let r = guard.as_mut().ok_or_else(no_match)?;
    r.m.act(r.human, &command).map_err(|e| e.message())?;
    finish(&state, r);
    Ok(dto(r))
}

/// Satu langkah bot, bila giliran bot.
#[tauri::command]
pub(crate) fn match_step(state: State<'_, AppState>) -> Result<MatchDto, Localized> {
    let mut guard = state.play.lock().unwrap();
    let r = guard.as_mut().ok_or_else(no_match)?;
    r.m.step_auto().map_err(|e| e.message())?;
    finish(&state, r);
    Ok(dto(r))
}

#[tauri::command]
pub(crate) fn match_leave(state: State<'_, AppState>) {
    if let Some(r) = state.play.lock().unwrap().take() {
        leave(&state, r);
    }
}

#[derive(Serialize)]
pub(crate) struct ReplaySummaryDto {
    id: i64,
    game: String,
    started_at: i64,
    finished: bool,
    moves: usize,
    seats: Vec<SeatKind>,
    result: Option<GameResult>,
}

impl From<ReplaySummary> for ReplaySummaryDto {
    fn from(s: ReplaySummary) -> Self {
        ReplaySummaryDto {
            id: s.id,
            game: s.game,
            started_at: s.started_at,
            finished: s.finished,
            moves: s.moves,
            seats: s.seats,
            result: s.result,
        }
    }
}

fn store_error(e: impl ToString) -> Localized {
    core().localized("error.store", &[("detail", &e.to_string())])
}

#[tauri::command]
pub(crate) fn replay_list(
    game: Option<String>,
    state: State<'_, AppState>,
) -> Result<Vec<ReplaySummaryDto>, Localized> {
    let store = state.store.as_ref().map_err(store_error)?;
    let list = store
        .lock()
        .unwrap()
        .list_replays(game.as_deref(), 50)
        .map_err(store_error)?;
    Ok(list.into_iter().map(Into::into).collect())
}

#[derive(Serialize)]
pub(crate) struct ReplayDto {
    id: i64,
    game: String,
    seat: u8,
    seats: Vec<SeatKind>,
    frames: Vec<Frame>,
    fair: FairRecord,
    result: Option<GameResult>,
    verify: VerifyReport,
}

#[tauri::command]
pub(crate) fn replay_open(id: i64, state: State<'_, AppState>) -> Result<ReplayDto, Localized> {
    let store = state.store.as_ref().map_err(store_error)?;
    let replay = store
        .lock()
        .unwrap()
        .load_replay(id)
        .map_err(store_error)?
        .ok_or_else(|| core().localized("error.replay_missing", &[("id", &id.to_string())]))?;
    let cartridge = state
        .registry
        .get(&replay.game)
        .ok_or_else(|| unknown_game(&replay.game))?;
    // Dilihat dari kursi manusia pertama (atau kursi 0).
    let seat = replay
        .seats
        .iter()
        .position(|s| *s == SeatKind::Human)
        .unwrap_or(0) as u8;
    let frames = replay.frames(cartridge, seat).map_err(|e| e.message())?;
    Ok(ReplayDto {
        id,
        game: replay.game.clone(),
        seat,
        seats: replay.seats.clone(),
        frames,
        verify: replay.verify(cartridge),
        fair: replay.fair,
        result: replay.result,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn singleplayer_fair_record_verifies_and_hides_nothing_needed() {
        let rec = fair_record(None).unwrap();
        assert!(rec.verify().is_ok());
        assert_eq!(rec.commitments.len(), 2);
        let chosen = "ab".repeat(32);
        let rec = fair_record(Some(&chosen)).unwrap();
        assert_eq!(rec.seeds[PLAYER], chosen);
        assert!(fair_record(Some("bukan hex")).is_err());
        // Seed host acak: dua ronde dengan seed pemain yang sama tetap berbeda.
        assert_ne!(
            fair_record(Some(&chosen)).unwrap().round_seed,
            fair_record(Some(&chosen)).unwrap().round_seed
        );
    }
}
