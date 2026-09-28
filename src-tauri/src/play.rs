//! Bermain melawan bot, provably fair, replay (SPEC §2.3, §2.5, §5.4).
//!
//! Singleplayer: aplikasi berperan sebagai host dan pemain lokal sebagai
//! peserta. Komitmen keduanya diumumkan sebelum langkah pertama; seed baru
//! dibuka setelah permainan selesai, lalu `verify` dijalankan otomatis dan
//! hasilnya ditampilkan. Setiap pertandingan disimpan sebagai replay,
//! termasuk yang ditinggalkan di tengah jalan.

use std::collections::BTreeMap;
use std::time::{Instant, SystemTime, UNIX_EPOCH};

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

/// Jam catur, dihitung host dengan jam monoton (game tidak memakai jam
/// dinding). Saat waktu seorang pemain habis, host mengajukan `timeout`
/// atas namanya, jadi tercatat di replay (D-045).
struct HostClock {
    remaining: [i64; 2],
    increment: i64,
    running: Option<u8>,
    since: Instant,
}

impl HostClock {
    fn charge(&mut self) {
        if let Some(s) = self.running {
            self.remaining[s as usize] -= self.since.elapsed().as_millis() as i64;
        }
        self.since = Instant::now();
    }

    /// Setelah `mover` melangkah: tambahan waktu, lalu jam `next` berjalan.
    fn after_move(&mut self, mover: u8, next: Option<u8>) {
        self.charge();
        self.remaining[mover as usize] += self.increment;
        self.running = next;
    }

    fn snapshot(&self) -> ClockDto {
        let mut remaining = self.remaining;
        if let Some(s) = self.running {
            remaining[s as usize] -= self.since.elapsed().as_millis() as i64;
        }
        ClockDto {
            remaining_ms: remaining,
            running: self.running,
            increment_ms: self.increment,
        }
    }
}

#[derive(Serialize)]
pub(crate) struct ClockDto {
    remaining_ms: [i64; 2],
    running: Option<u8>,
    increment_ms: i64,
}

pub(crate) struct Running {
    game: String,
    m: Match,
    human: u8,
    started_at: i64,
    replay_id: Option<i64>,
    verify: Option<VerifyReport>,
    save_error: Option<String>,
    clock: Option<HostClock>,
}

impl Running {
    fn pending(&self) -> Option<u8> {
        self.m.session().pending_players().first().copied()
    }

    /// Jam untuk pemain yang sedang melangkah habis? Bila ya, ajukan
    /// `timeout` atas namanya.
    fn flag_if_expired(&mut self) -> bool {
        let Some(clock) = self.clock.as_mut() else {
            return false;
        };
        clock.charge();
        let Some(seat) = clock.running else {
            return false;
        };
        if clock.remaining[seat as usize] > 0 {
            return false;
        }
        clock.remaining[seat as usize] = 0;
        clock.running = None;
        self.m.act(seat, "timeout").is_ok()
    }

    fn moved(&mut self, mover: u8) {
        let next = self.pending();
        if let Some(clock) = self.clock.as_mut() {
            clock.after_move(mover, next);
        }
    }
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
    clock: Option<ClockDto>,
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
        clock: r.clock.as_ref().map(HostClock::snapshot),
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

/// Jam catur yang dipilih pemain: menit per pemain + tambahan per langkah.
#[derive(serde::Deserialize)]
pub(crate) struct ClockSpec {
    minutes: u32,
    increment: u32,
}

#[tauri::command]
pub(crate) fn match_start(
    id: String,
    level: u8,
    seat: u8,
    player_seed: Option<String>,
    clock: Option<ClockSpec>,
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
    let config = match &clock {
        Some(c) => {
            serde_json::json!({ "jam": { "menit": c.minutes, "tambahan_detik": c.increment } })
        }
        None => serde_json::Value::Null,
    };
    let m = Match::new(cartridge, config, fair, players).map_err(|e| e.message())?;
    let first = m.session().pending_players().first().copied();
    let running = Running {
        game: id,
        m,
        human: seat,
        started_at: now_ms(),
        replay_id: None,
        verify: None,
        save_error: None,
        clock: clock.map(|c| HostClock {
            remaining: [i64::from(c.minutes) * 60_000; 2],
            increment: i64::from(c.increment) * 1000,
            running: first,
            since: Instant::now(),
        }),
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
    // Waktu habis sebelum langkah ini: yang berlaku adalah timeout.
    if !r.flag_if_expired() {
        r.m.act(r.human, &command).map_err(|e| e.message())?;
        let human = r.human;
        r.moved(human);
    }
    finish(&state, r);
    Ok(dto(r))
}

/// Dipanggil UI saat jam pemain di layar mencapai nol; host memeriksa
/// dengan jamnya sendiri.
#[tauri::command]
pub(crate) fn match_flag(state: State<'_, AppState>) -> Result<MatchDto, Localized> {
    let mut guard = state.play.lock().unwrap();
    let r = guard.as_mut().ok_or_else(no_match)?;
    r.flag_if_expired();
    finish(&state, r);
    Ok(dto(r))
}

/// Satu langkah bot, bila giliran bot.
#[tauri::command]
pub(crate) fn match_step(state: State<'_, AppState>) -> Result<MatchDto, Localized> {
    let mut guard = state.play.lock().unwrap();
    let r = guard.as_mut().ok_or_else(no_match)?;
    if !r.flag_if_expired()
        && let Some(mv) = r.m.step_auto().map_err(|e| e.message())?
    {
        r.moved(mv.seat);
    }
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
    /// `null` untuk partai impor PGN (tidak tersimpan).
    id: Option<i64>,
    game: String,
    seat: u8,
    seats: Vec<SeatKind>,
    frames: Vec<Frame>,
    /// Catatan provably fair; tidak ada untuk impor PGN.
    fair: Option<FairRecord>,
    result: Option<GameResult>,
    verify: Option<VerifyReport>,
    /// Tag PGN (impor).
    tags: Vec<(String, String)>,
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
        id: Some(id),
        game: replay.game.clone(),
        seat,
        seats: replay.seats.clone(),
        frames,
        verify: Some(replay.verify(cartridge)),
        fair: Some(replay.fair),
        result: replay.result,
        tags: Vec::new(),
    })
}

fn load(state: &AppState, id: i64) -> Result<(Replay, i64), Localized> {
    let store = state.store.as_ref().map_err(store_error)?;
    let guard = store.lock().unwrap();
    let replay = guard
        .load_replay(id)
        .map_err(store_error)?
        .ok_or_else(|| core().localized("error.replay_missing", &[("id", &id.to_string())]))?;
    let started = guard
        .list_replays(Some(&replay.game), 10_000)
        .map_err(store_error)?
        .into_iter()
        .find(|s| s.id == id)
        .map(|s| s.started_at)
        .unwrap_or(0);
    Ok((replay, started))
}

/// PGN sebuah replay catur (SPEC §6.1). Nama kursi dalam bahasa `lang`.
#[tauri::command]
pub(crate) fn replay_pgn(
    id: i64,
    lang: kyusin_core::Lang,
    state: State<'_, AppState>,
) -> Result<String, Localized> {
    use kyusin_games::catur::{START_FEN, pgn};
    let (replay, started) = load(&state, id)?;
    if replay.game != kyusin_games::catur::ID {
        return Err(core().localized("error.no_pgn", &[]));
    }
    let seat_name = |s: &SeatKind| match s {
        SeatKind::Bot { level } => core().text(lang, "pgn.bot", &[("level", &level.to_string())]),
        _ => core().text(lang, "pgn.player", &[]),
    };
    let fen = replay.config["fen"]
        .as_str()
        .unwrap_or(START_FEN)
        .to_string();
    let sans: Vec<String> = replay
        .moves
        .iter()
        .map(|m| m.command.clone())
        .filter(|c| c != "resign" && c != "timeout")
        .collect();
    let winners = replay.result.as_ref().map(|r| r.winners.clone());
    let jam = &replay.config["jam"];
    let time_control = jam["menit"]
        .as_u64()
        .map(|m| format!("{}+{}", m * 60, jam["tambahan_detik"].as_u64().unwrap_or(0)));
    let date = {
        let days = started / 86_400_000;
        // Tanggal UTC dari hari sejak 1970 (algoritme civil_from_days).
        let z = days + 719_468;
        let era = z.div_euclid(146_097);
        let doe = z - era * 146_097;
        let yoe = (doe - doe / 1460 + doe / 36_524 - doe / 146_096) / 365;
        let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
        let mp = (5 * doy + 2) / 153;
        let d = doy - (153 * mp + 2) / 5 + 1;
        let m = if mp < 10 { mp + 3 } else { mp - 9 };
        let y = yoe + era * 400 + i64::from(m <= 2);
        if started > 0 {
            format!("{y:04}.{m:02}.{d:02}")
        } else {
            "????.??.??".into()
        }
    };
    let tags = pgn::Tags {
        white: seat_name(&replay.seats[0]),
        black: seat_name(&replay.seats[1]),
        date,
    };
    Ok(pgn::export_moves(
        &fen,
        &sans,
        pgn::result_token(winners.as_deref()),
        time_control,
        &tags,
    ))
}

/// Membuka teks PGN di penampil replay (tidak disimpan, tanpa verify).
#[tauri::command]
pub(crate) fn pgn_open(text: String, state: State<'_, AppState>) -> Result<ReplayDto, Localized> {
    use kyusin_games::catur::pgn;
    let game = pgn::import(&text).map_err(|e| core().localized("error.pgn", &[("detail", &e)]))?;
    let cartridge = state
        .registry
        .get(kyusin_games::catur::ID)
        .ok_or_else(|| unknown_game(kyusin_games::catur::ID))?;
    let config = match &game.fen {
        Some(f) => serde_json::json!({ "fen": f }),
        None => serde_json::Value::Null,
    };
    let mut session = (cartridge.create)(&config, [0; 32]).map_err(|e| e.message())?;
    let frame = |s: &dyn kyusin_core::Session, index: usize, last: Option<Move>| Frame {
        index,
        last,
        view_data: s.view_data(0),
        view_text: Localized::build(|lang| s.view_text(0, lang)),
    };
    let mut frames = vec![frame(session.as_ref(), 0, None)];
    for (i, san) in game.moves.iter().enumerate() {
        let seat = session.pending_players()[0];
        session.act(seat, san).map_err(|e| e.message())?;
        frames.push(frame(
            session.as_ref(),
            i + 1,
            Some(Move {
                seat,
                command: san.clone(),
            }),
        ));
    }
    Ok(ReplayDto {
        id: None,
        game: kyusin_games::catur::ID.into(),
        seat: 0,
        seats: vec![SeatKind::Human, SeatKind::Human],
        frames,
        fair: None,
        result: session.result(),
        verify: None,
        tags: game.tags,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn host_clock_charges_the_mover_and_adds_increment() {
        let mut c = HostClock {
            remaining: [60_000, 60_000],
            increment: 2_000,
            running: Some(0),
            since: Instant::now() - std::time::Duration::from_millis(1_500),
        };
        c.after_move(0, Some(1));
        assert!(
            (60_400..=60_600).contains(&c.remaining[0]),
            "{:?}",
            c.remaining
        );
        assert_eq!(c.remaining[1], 60_000);
        assert_eq!(c.running, Some(1));
        c.since = Instant::now() - std::time::Duration::from_millis(700);
        let snap = c.snapshot();
        assert!(snap.remaining_ms[1] <= 59_300);
        c.after_move(1, None);
        assert_eq!(c.running, None);
        let still = c.snapshot().remaining_ms;
        assert_eq!(still, c.remaining, "jam berhenti setelah permainan selesai");
    }

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
