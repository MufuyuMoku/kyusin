//! Bermain melawan bot, provably fair, replay (SPEC §2.3, §2.5, §5.4).
//!
//! Singleplayer: aplikasi berperan sebagai host dan pemain lokal sebagai
//! peserta. Komitmen keduanya diumumkan sebelum langkah pertama; seed baru
//! dibuka setelah permainan selesai, lalu `verify` dijalankan otomatis dan
//! hasilnya ditampilkan. Setiap pertandingan yang selesai disimpan sebagai
//! replay. Pertandingan yang belum selesai tidak pernah hilang: menu jeda,
//! keluar dari layar, atau menutup jendela menundanya (SPEC §4 Rev. 9), dan
//! pertandingan itu bisa dilanjutkan dari layar game.

use std::collections::BTreeMap;
use std::time::{Instant, SystemTime, UNIX_EPOCH};

use kyusin_core::fair::{FairRecord, FairRound, commitment};
use kyusin_core::hash::unhex32;
use kyusin_core::i18n::core;
use kyusin_core::replay::{Frame, Move, Replay, VerifyReport};
use kyusin_core::{GameResult, Human, Localized, Match, Player, SeatKind, Seed};
use kyusin_store::{ReplaySummary, Suspended};
use serde::{Deserialize, Serialize};
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
    /// Kursi yang jamnya dihentikan oleh menu jeda.
    paused: Option<u8>,
}

impl HostClock {
    fn new(remaining: [i64; 2], increment: i64, running: Option<u8>) -> Self {
        HostClock {
            remaining,
            increment,
            running,
            since: Instant::now(),
            paused: None,
        }
    }

    /// Menu jeda dibuka: jam yang berjalan berhenti.
    fn pause(&mut self) {
        self.charge();
        if let Some(s) = self.running.take() {
            self.paused = Some(s);
        }
    }

    fn unpause(&mut self) {
        if let Some(s) = self.paused.take() {
            self.running = Some(s);
            self.since = Instant::now();
        }
    }

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
    paused: bool,
    /// Perubahan rating lokal setelah selesai (M3).
    rating: Option<crate::profile::RatingChange>,
    /// Game casino melawan bandar: bagian hasil yang sudah dicatat ke chip
    /// profil (M4). `None` untuk game lain.
    ledger: Option<crate::casino::Ledger>,
    /// Saldo chip terakhir yang diketahui (game casino).
    chips: Option<i64>,
}

impl Running {
    /// Mencatat ronde casino yang baru selesai ke chip profil.
    fn settle_chips(&mut self, state: &AppState) {
        if let Some(ledger) = self.ledger.as_mut() {
            let view = self.m.session().view_data(self.human);
            crate::casino::settle(state, &self.game, &view, ledger);
            self.chips = crate::casino::chips(state);
        }
    }

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

    fn human_pending(&self) -> bool {
        self.m.session().pending_players().contains(&self.human)
    }

    fn pause(&mut self) {
        self.paused = true;
        if let Some(c) = self.clock.as_mut() {
            c.pause();
        }
    }

    fn unpause(&mut self) {
        self.paused = false;
        if let Some(c) = self.clock.as_mut() {
            c.unpause();
        }
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
    /// Waktu mulai (epoch ms); pengenal pertandingan bagi UI.
    started_at: i64,
    /// Menu jeda terbuka: jam dan bot berhenti.
    paused: bool,
    /// Perubahan rating lokal bila pertandingan ini dihitung.
    rating: Option<crate::profile::RatingChange>,
    /// Saldo chip profil (game casino).
    chips: Option<i64>,
}

/// Isi pertandingan tertunda di store: replay sejauh ini (konfigurasi,
/// kursi, catatan provably fair, langkah) + kursi pemain + sisa jam.
#[derive(Serialize, Deserialize)]
struct SuspendedMatch {
    replay: Replay,
    human: u8,
    clock: Option<SavedClock>,
}

#[derive(Serialize, Deserialize)]
struct SavedClock {
    remaining_ms: [i64; 2],
    increment_ms: i64,
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
        started_at: r.started_at,
        paused: r.paused,
        rating: r.rating,
        chips: r.chips,
    }
}

/// Game casino yang melawan bandar (chip profil dipakai).
fn is_house(state: &AppState, game: &str) -> bool {
    state
        .registry
        .get(game)
        .is_some_and(|c| c.manifest.is_against_house())
}

fn ledger_for(state: &AppState, game: &str, m: &Match, human: u8) -> Option<crate::casino::Ledger> {
    is_house(state, game).then(|| crate::casino::Ledger::from_view(&m.session().view_data(human)))
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
    // Riwayat + rating lokal (M3). Pertandingan tertunda sampai di sini
    // hanya setelah benar-benar selesai.
    r.rating = crate::profile::record(state, &replay, r.replay_id, now_ms());
}

fn store_of(state: &AppState) -> Result<&std::sync::Mutex<kyusin_store::Store>, Localized> {
    state.store.as_ref().map_err(store_error)
}

/// Menunda pertandingan yang belum selesai: jam berhenti, keadaannya
/// disimpan (satu per game), dan bisa dilanjutkan dari layar game.
fn suspend(state: &AppState, mut r: Running) -> Result<(), Localized> {
    if r.m.session().is_over() {
        return Ok(());
    }
    if let Some(c) = r.clock.as_mut() {
        c.pause();
    }
    let data = SuspendedMatch {
        replay: r.m.replay(),
        human: r.human,
        clock: r.clock.as_ref().map(|c| SavedClock {
            remaining_ms: c.remaining,
            increment_ms: c.increment,
        }),
    };
    let rec = Suspended {
        game: r.game.clone(),
        started_at: r.started_at,
        suspended_at: now_ms(),
        data: serde_json::to_value(&data).map_err(store_error)?,
    };
    store_of(state)?
        .lock()
        .unwrap()
        .save_suspended(&rec)
        .map_err(store_error)
}

/// Keluar dari pertandingan: yang belum selesai ditunda, bukan dibuang.
fn leave(state: &AppState, r: Running) {
    let _ = suspend(state, r);
}

/// Tutup jendela atau aplikasi berhenti: pertandingan berjalan ditunda.
pub(crate) fn suspend_running(state: &AppState) {
    let taken = state.play.lock().unwrap().take();
    if let Some(r) = taken {
        leave(state, r);
    }
}

fn bot_players(
    game: &str,
    seats: &[SeatKind],
    human: u8,
    round: &Seed,
) -> Result<Vec<Box<dyn Player>>, Localized> {
    let mut players: Vec<Box<dyn Player>> = Vec::new();
    for (s, kind) in seats.iter().enumerate() {
        let s = s as u8;
        let level = match kind {
            SeatKind::Bot { level } if s != human => *level,
            _ => {
                players.push(Box::new(Human));
                continue;
            }
        };
        let bot = kyusin_bots::create(game, level, round, s)
            .ok_or_else(|| core().localized("error.no_bot", &[("level", &level.to_string())]))?;
        players.push(bot);
    }
    Ok(players)
}

/// Membangun ulang pertandingan tertunda sebuah game (dan menghapusnya
/// dari daftar tunda). Jam berjalan lagi dari sisa waktu saat ditunda.
fn restore(state: &AppState, game: &str) -> Result<Running, Localized> {
    let rec = store_of(state)?
        .lock()
        .unwrap()
        .take_suspended(game)
        .map_err(store_error)?
        .ok_or_else(|| core().localized("error.no_suspended", &[]))?;
    let data: SuspendedMatch = serde_json::from_value(rec.data).map_err(store_error)?;
    let cartridge = state.registry.get(game).ok_or_else(|| unknown_game(game))?;
    let round = data
        .replay
        .fair
        .round_seed_bytes()
        .map_err(|e| core().localized("error.replay_fair", &[("detail", &e.to_string())]))?;
    let players = bot_players(game, &data.replay.seats, data.human, &round)?;
    let m = Match::restore(
        cartridge,
        data.replay.config,
        data.replay.fair,
        players,
        data.replay.moves,
    )
    .map_err(|e| e.message())?;
    let pending = m.session().pending_players().first().copied();
    let ledger = ledger_for(state, game, &m, data.human);
    Ok(Running {
        chips: ledger.and_then(|_| crate::casino::chips(state)),
        ledger,
        game: game.to_string(),
        m,
        human: data.human,
        started_at: rec.started_at,
        replay_id: None,
        verify: None,
        save_error: None,
        clock: data
            .clock
            .map(|c| HostClock::new(c.remaining_ms, c.increment_ms, pending)),
        paused: false,
        rating: None,
    })
}

/// Game casino: ronde berjalan diselesaikan secara netral (insurance
/// ditolak, semua tangan stand), lalu sesi diakhiri dengan `leave` (D-058).
fn close_house(r: &mut Running) -> Result<(), Localized> {
    let mut guard = 0;
    while !r.m.session().is_over() && guard < 64 {
        let legal: Vec<String> =
            r.m.session()
                .legal_actions(r.human)
                .iter()
                .map(|a| a.usage())
                .collect();
        let pick = ["leave", "decline", "stand"]
            .into_iter()
            .find(|c| legal.iter().any(|l| l == c))
            .ok_or_else(no_match)?;
        r.m.act(r.human, pick).map_err(|e| e.message())?;
        guard += 1;
    }
    Ok(())
}

/// Menyerah atas nama pemain lokal. Bila sedang giliran bot, bot
/// melangkah dulu sampai giliran pemain (aturan hanya menerima aksi dari
/// kursi yang ditunggu).
fn resign(r: &mut Running) -> Result<(), Localized> {
    if r.ledger.is_some() {
        return close_house(r);
    }
    while !r.m.session().is_over() && !r.human_pending() {
        match r.m.step_auto().map_err(|e| e.message())? {
            Some(mv) => r.moved(mv.seat),
            None => break,
        }
    }
    if !r.m.session().is_over() {
        r.m.act(r.human, "resign").map_err(|e| e.message())?;
        let human = r.human;
        r.moved(human);
    }
    Ok(())
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
        clock: clock.map(|c| {
            HostClock::new(
                [i64::from(c.minutes) * 60_000; 2],
                i64::from(c.increment) * 1000,
                first,
            )
        }),
        paused: false,
        rating: None,
        ledger: None,
        chips: None,
    };
    let mut running = running;
    running.ledger = ledger_for(&state, &running.game, &running.m, running.human);
    if running.ledger.is_some() {
        running.chips = crate::casino::chips(&state);
    }
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
    r.unpause();
    if r.ledger.is_some() {
        let view = r.m.session().view_data(r.human);
        crate::casino::check(&state, &view, &command)?;
    }
    // Waktu habis sebelum langkah ini: yang berlaku adalah timeout.
    if !r.flag_if_expired() {
        r.m.act(r.human, &command).map_err(|e| e.message())?;
        let human = r.human;
        r.moved(human);
    }
    r.settle_chips(&state);
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
    if !r.paused
        && !r.flag_if_expired()
        && let Some(mv) = r.m.step_auto().map_err(|e| e.message())?
    {
        r.moved(mv.seat);
    }
    r.settle_chips(&state);
    finish(&state, r);
    Ok(dto(r))
}

#[tauri::command]
pub(crate) fn match_leave(state: State<'_, AppState>) {
    suspend_running(&state);
}

/// Menu jeda dibuka (`Esc`): jam dan bot berhenti.
#[tauri::command]
pub(crate) fn match_pause(state: State<'_, AppState>) -> Result<MatchDto, Localized> {
    let mut guard = state.play.lock().unwrap();
    let r = guard.as_mut().ok_or_else(no_match)?;
    if !r.m.session().is_over() {
        r.pause();
    }
    Ok(dto(r))
}

/// Menu jeda ditutup lewat Lanjutkan.
#[tauri::command]
pub(crate) fn match_unpause(state: State<'_, AppState>) -> Result<MatchDto, Localized> {
    let mut guard = state.play.lock().unwrap();
    let r = guard.as_mut().ok_or_else(no_match)?;
    r.unpause();
    Ok(dto(r))
}

/// Tunda & keluar.
#[tauri::command]
pub(crate) fn match_suspend(state: State<'_, AppState>) -> Result<(), Localized> {
    let taken = state.play.lock().unwrap().take();
    match taken {
        Some(r) => suspend(&state, r),
        None => Ok(()),
    }
}

/// Menyerah dari menu jeda (kapan pun, termasuk saat giliran bot).
#[tauri::command]
pub(crate) fn match_resign(state: State<'_, AppState>) -> Result<MatchDto, Localized> {
    let mut guard = state.play.lock().unwrap();
    let r = guard.as_mut().ok_or_else(no_match)?;
    r.unpause();
    resign(r)?;
    r.settle_chips(&state);
    finish(&state, r);
    Ok(dto(r))
}

/// Melanjutkan pertandingan tertunda sebuah game.
#[tauri::command]
pub(crate) fn match_resume(id: String, state: State<'_, AppState>) -> Result<MatchDto, Localized> {
    let running = restore(&state, &id)?;
    let dto = dto(&running);
    let old = state.play.lock().unwrap().replace(running);
    if let Some(old) = old {
        leave(&state, old);
    }
    Ok(dto)
}

/// Pertandingan tertunda dibuang untuk memulai yang baru: dihitung
/// menyerah (hasilnya disimpan sebagai replay yang selesai).
#[tauri::command]
pub(crate) fn suspended_forfeit(id: String, state: State<'_, AppState>) -> Result<(), Localized> {
    let mut r = restore(&state, &id)?;
    resign(&mut r)?;
    r.settle_chips(&state);
    finish(&state, &mut r);
    Ok(())
}

#[derive(Serialize)]
pub(crate) struct SuspendedDto {
    game: String,
    started_at: i64,
    suspended_at: i64,
    moves: usize,
    seat: u8,
    seats: Vec<SeatKind>,
    clock: Option<ClockDto>,
}

#[tauri::command]
pub(crate) fn suspended_list(state: State<'_, AppState>) -> Result<Vec<SuspendedDto>, Localized> {
    let list = store_of(&state)?
        .lock()
        .unwrap()
        .list_suspended()
        .map_err(store_error)?;
    Ok(list
        .into_iter()
        .filter_map(|rec| {
            let data: SuspendedMatch = serde_json::from_value(rec.data).ok()?;
            Some(SuspendedDto {
                game: rec.game,
                started_at: rec.started_at,
                suspended_at: rec.suspended_at,
                moves: data.replay.moves.len(),
                seat: data.human,
                seats: data.replay.seats,
                clock: data.clock.map(|c| ClockDto {
                    remaining_ms: c.remaining_ms,
                    running: None,
                    increment_ms: c.increment_ms,
                }),
            })
        })
        .collect())
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
        let mut c = HostClock::new([60_000, 60_000], 2_000, Some(0));
        c.since = Instant::now() - std::time::Duration::from_millis(1_500);
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

    fn state() -> AppState {
        AppState {
            registry: kyusin_games::builtin().unwrap(),
            tutorial: std::sync::Mutex::new(None),
            play: std::sync::Mutex::new(None),
            store: Ok(std::sync::Mutex::new(
                kyusin_store::Store::open_in_memory().unwrap(),
            )),
            db_path: None,
        }
    }

    fn running(state: &AppState, game: &str, clock: bool) -> Running {
        let fair = fair_record(None).unwrap();
        let round = fair.round_seed_bytes().unwrap();
        let seats = [SeatKind::Human, SeatKind::Bot { level: 1 }];
        let players = bot_players(game, &seats, 0, &round).unwrap();
        let config = if clock {
            serde_json::json!({ "jam": { "menit": 5, "tambahan_detik": 0 } })
        } else {
            serde_json::Value::Null
        };
        let m = Match::new(state.registry.get(game).unwrap(), config, fair, players).unwrap();
        Running {
            game: game.into(),
            m,
            human: 0,
            started_at: 42,
            replay_id: None,
            verify: None,
            save_error: None,
            clock: clock.then(|| HostClock::new([300_000; 2], 0, Some(0))),
            paused: false,
            rating: None,
            ledger: None,
            chips: None,
        }
    }

    #[test]
    fn suspend_and_resume_keep_position_moves_and_clock() {
        let st = state();
        let mut r = running(&st, "catur", true);
        r.m.act(0, "e4").unwrap();
        r.moved(0);
        let mv = r.m.step_auto().unwrap().unwrap();
        r.moved(mv.seat);
        let hash = r.m.session().state_hash();
        let moves = r.m.moves().to_vec();
        suspend(&st, r).unwrap();
        // Ditunda: belum masuk riwayat maupun rating.
        {
            let s = store_of(&st).unwrap().lock().unwrap();
            assert!(s.history(None, 10).unwrap().is_empty());
            assert!(s.rating("catur").unwrap().is_none());
        }

        let list = store_of(&st)
            .unwrap()
            .lock()
            .unwrap()
            .list_suspended()
            .unwrap();
        assert_eq!(list.len(), 1);
        let saved: SuspendedMatch = serde_json::from_value(list[0].data.clone()).unwrap();
        let left = saved.clock.as_ref().unwrap().remaining_ms;

        let back = restore(&st, "catur").unwrap();
        assert_eq!(back.m.session().state_hash(), hash);
        assert_eq!(back.m.moves(), moves.as_slice());
        assert_eq!(back.started_at, 42);
        let c = back.clock.as_ref().unwrap();
        assert_eq!(c.remaining, left, "sisa jam sama seperti saat ditunda");
        assert_eq!(c.running, Some(0), "jam pemain yang ditunggu berjalan lagi");
        // Diambil sekali: daftar tunda kosong lagi.
        assert!(restore(&st, "catur").is_err());
    }

    #[test]
    fn leaving_or_closing_suspends_instead_of_dropping() {
        let st = state();
        let mut r = running(&st, "reversi", false);
        r.m.act(0, "d3").unwrap();
        *st.play.lock().unwrap() = Some(r);
        suspend_running(&st);
        assert!(st.play.lock().unwrap().is_none());
        let list = store_of(&st)
            .unwrap()
            .lock()
            .unwrap()
            .list_suspended()
            .unwrap();
        assert_eq!(list[0].game, "reversi");
        // Pertandingan yang sudah selesai tidak ditunda.
        let mut done = running(&st, "reversi", false);
        done.m.act(0, "resign").unwrap();
        suspend(&st, done).unwrap();
        assert_eq!(
            store_of(&st)
                .unwrap()
                .lock()
                .unwrap()
                .list_suspended()
                .unwrap()
                .len(),
            1
        );
    }

    #[test]
    fn resign_waits_for_the_bot_then_counts_as_a_loss() {
        let st = state();
        let mut r = running(&st, "reversi", false);
        r.m.act(0, "d3").unwrap();
        // Giliran bot: menyerah tetap bisa, bot melangkah dulu.
        assert!(!r.human_pending());
        resign(&mut r).unwrap();
        let result = r.m.session().result().unwrap();
        assert_eq!(result.winners, vec![1]);
        assert_eq!(r.m.moves().last().unwrap().command, "resign");
        finish(&st, &mut r);
        assert!(r.replay_id.is_some() && r.verify.as_ref().unwrap().ok);
        // Kalah dari bot level 1 (rating 1000): rating lokal turun dari 1500.
        let change = r.rating.expect("dihitung ke rating");
        assert_eq!(change.before, 1500.0);
        assert!(change.after < 1500.0, "{change:?}");
        let s = store_of(&st).unwrap().lock().unwrap();
        let stored = s.rating("reversi").unwrap().unwrap();
        assert_eq!(stored.games, 1);
        assert_eq!(stored.rating.rating, change.after);
        let h = s.history(None, 10).unwrap();
        assert_eq!(h[0].record.replay_id, r.replay_id);
        assert_eq!(s.last_game().unwrap().as_deref(), Some("reversi"));
    }

    #[test]
    fn non_competitive_games_are_recorded_without_rating() {
        let st = AppState {
            registry: kyusin_games::with_fixture().unwrap(),
            ..state()
        };
        let fair = fair_record(None).unwrap();
        let players: Vec<Box<dyn Player>> = vec![Box::new(Human), Box::new(Human)];
        let mut m = Match::new(
            st.registry.get("fixture").unwrap(),
            serde_json::json!({ "batang": 1 }),
            fair,
            players,
        )
        .unwrap();
        m.act(0, "take 1").unwrap();
        let mut r = Running {
            game: "fixture".into(),
            m,
            human: 0,
            started_at: 1,
            replay_id: None,
            verify: None,
            save_error: None,
            clock: None,
            paused: false,
            rating: None,
            ledger: None,
            chips: None,
        };
        finish(&st, &mut r);
        assert!(r.rating.is_none());
        let s = store_of(&st).unwrap().lock().unwrap();
        assert_eq!(s.history(None, 10).unwrap().len(), 1);
        assert!(s.rating("fixture").unwrap().is_none());
    }

    fn house_running(st: &AppState, shoe: &[&str]) -> Running {
        let fair = fair_record(None).unwrap();
        let config = serde_json::json!({ "shoe": shoe, "potong": 1000 });
        let players: Vec<Box<dyn Player>> = vec![Box::new(Human)];
        let m = Match::new(st.registry.get("blackjack").unwrap(), config, fair, players).unwrap();
        let ledger = ledger_for(st, "blackjack", &m, 0);
        assert!(ledger.is_some(), "blackjack memakai chip profil");
        Running {
            game: "blackjack".into(),
            m,
            human: 0,
            started_at: 7,
            replay_id: None,
            verify: None,
            save_error: None,
            clock: None,
            paused: false,
            rating: None,
            ledger,
            chips: None,
        }
    }

    fn house_act(st: &AppState, r: &mut Running, command: &str) -> Result<(), Localized> {
        let view = r.m.session().view_data(0);
        crate::casino::check(st, &view, command)?;
        r.m.act(0, command).map_err(|e| e.message())?;
        r.settle_chips(st);
        finish(st, r);
        Ok(())
    }

    #[test]
    fn casino_rounds_move_profile_chips_once() {
        let st = state();
        // Ronde 1: blackjack +150. Ronde 2: 20 lawan 17 → +100 (setelah tunda).
        let mut r = house_running(&st, &["Ah", "9c", "Kd", "7s", "Th", "7d", "Tc", "Kh"]);
        house_act(&st, &mut r, "bet 100").unwrap();
        assert_eq!(r.chips, Some(10_150));
        house_act(&st, &mut r, "bet 100").unwrap();
        // Ditunda di tengah ronde lalu dilanjutkan: ronde 1 tidak dicatat ulang.
        suspend(&st, r).unwrap();
        let mut r = restore(&st, "blackjack").unwrap();
        assert_eq!(r.chips, Some(10_150));
        house_act(&st, &mut r, "stand").unwrap();
        assert_eq!(r.chips, Some(10_250));
        let s = store_of(&st).unwrap().lock().unwrap();
        let c = s.casino_game("blackjack").unwrap().unwrap();
        assert_eq!((c.rounds, c.wagered, c.net), (2, 200, 250));
        // Casino tidak masuk riwayat rating.
        assert!(s.history(None, 10).unwrap().is_empty());
    }

    #[test]
    fn casino_actions_need_enough_chips() {
        let st = state();
        store_of(&st)
            .unwrap()
            .lock()
            .unwrap()
            .record_casino("blackjack", 1, 9_950, -9_950, 1)
            .unwrap();
        // Saldo 50.
        let mut r = house_running(&st, &["6h", "9d", "5s", "7c", "Th", "Kd"]);
        assert!(house_act(&st, &mut r, "bet 100").is_err());
        house_act(&st, &mut r, "bet 50").unwrap();
        // Double butuh 50 lagi, padahal 50 sedang dipertaruhkan.
        let err = house_act(&st, &mut r, "double").unwrap_err();
        assert!(err.id.contains("Chip tidak cukup"), "{err:?}");
        house_act(&st, &mut r, "hit").unwrap();
    }

    #[test]
    fn abandoning_a_suspended_casino_match_finishes_it_neutrally() {
        let st = state();
        let mut r = house_running(&st, &["Th", "6d", "6s", "Tc", "9h", "2c"]);
        house_act(&st, &mut r, "bet 100").unwrap();
        suspend(&st, r).unwrap();
        let mut r = restore(&st, "blackjack").unwrap();
        resign(&mut r).unwrap();
        r.settle_chips(&st);
        finish(&st, &mut r);
        assert!(r.m.session().is_over());
        let moves: Vec<&str> = r.m.moves().iter().map(|m| m.command.as_str()).collect();
        assert_eq!(moves, vec!["bet 100", "stand", "leave"]);
        // 16 lawan bandar 16 yang lalu mengambil 9 (bust): menang +100.
        assert_eq!(r.chips, Some(10_100));
        assert!(r.verify.as_ref().unwrap().ok);
    }

    #[test]
    fn paused_clock_does_not_run() {
        let mut c = HostClock::new([60_000, 60_000], 0, Some(1));
        c.since = Instant::now() - std::time::Duration::from_millis(1_000);
        c.pause();
        let at_pause = c.remaining;
        assert!(at_pause[1] <= 59_000);
        assert_eq!(c.snapshot().running, None);
        c.since = Instant::now() - std::time::Duration::from_millis(5_000);
        assert_eq!(
            c.snapshot().remaining_ms,
            at_pause,
            "jam berhenti saat jeda"
        );
        c.charge();
        assert_eq!(c.remaining, at_pause);
        c.unpause();
        assert_eq!(c.running, Some(1));
        assert!(c.snapshot().remaining_ms[1] >= at_pause[1] - 50);
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
