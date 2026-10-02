//! Mesin meja poker antar-pemain bersama (SPEC §6.3; M5b-1): Texas Hold'em
//! No-Limit dan Omaha Pot-Limit (D-063).
//!
//! Satu pertandingan = satu sesi meja: tumpukan dibawa dari tangan ke
//! tangan, dek tiap tangan dikocok dari seed sesi dan nomor tangan, jadi
//! provably fair berlaku per sesi seperti per shoe di Blackjack. Blind
//! 10/20, tanpa rake. Game ini tidak tahu saldo profil: host membeli chip
//! meja (buy-in) dari saldo saat duduk dan mengembalikan tumpukan akhir
//! saat berdiri, lewat kontrak chip di [`Umum`] (`taruhan_meja` = buy-in
//! selama duduk, `bersih` = hasil saat berdiri; D-059).
//!
//! Aturan taruhan: small blind kursi setelah dealer, big blind sesudahnya
//! (heads-up: dealer = small blind dan bertindak dulu sebelum flop).
//! `bet <n>` / `raise <n>` = total taruhan di babak itu. Raise minimal
//! sebesar raise terakhir (paling sedikit big blind); all-in yang kurang
//! dari raise penuh tidak membuka lagi hak raise bagi yang sudah bertindak.

use std::marker::PhantomData;
use std::sync::OnceLock;

use kyusin_core::game::GameResult;
use kyusin_core::i18n::{Catalog, Lang, Localized};
use kyusin_core::rng::derive;
use kyusin_core::{ActionSpec, GameError, GameRng, Param, ParamKind, PlayerId, Seed, TurnGame};
use serde::{Deserialize, Serialize};

use crate::cards::{Card, Shoe};
use crate::meja::{self, Umum};
use crate::poker::Value;
use crate::pot::{Pot, award, pots};

pub const SMALL_BLIND: i64 = 10;
pub const BIG_BLIND: i64 = 20;
pub const DEFAULT_SEATS: u8 = 6;
pub const DEFAULT_STACK: i64 = 2000;

pub fn catalog() -> &'static Catalog {
    static CATALOG: OnceLock<Catalog> = OnceLock::new();
    CATALOG.get_or_init(|| {
        Catalog::from_toml(include_str!("i18n.toml")).expect("poker_meja/i18n.toml tidak sah")
    })
}

/// Bagian yang membedakan Texas Hold'em dan Omaha.
pub trait Variant: Send + 'static {
    const ID: &'static str;
    /// Kartu tangan per kursi.
    const HOLE: usize;
    /// Pot-Limit (Omaha) atau No-Limit (Texas).
    const POT_LIMIT: bool;
    /// Tangan terbaik dari kartu tangan dan lima kartu meja.
    fn best(hole: &[Card], board: &[Card]) -> Value;
}

/// Konfigurasi meja. Semua opsional; host mengisi `tumpukan` (buy-in
/// manusia dari saldo, bot 2.000) dan `manusia`.
#[derive(Debug, Clone, Default, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Config {
    /// Jumlah kursi (2–6; bawaan 6).
    #[serde(default)]
    pub kursi: Option<u8>,
    /// Tumpukan awal tiap kursi (bawaan 2.000).
    #[serde(default)]
    pub tumpukan: Option<Vec<i64>>,
    /// Kursi manusia: sesi selesai bila semuanya berdiri atau habis
    /// (bawaan kursi 0).
    #[serde(default)]
    pub manusia: Option<Vec<u8>>,
    /// Kursi dealer tangan pertama (bawaan: dari seed).
    #[serde(default)]
    pub dealer: Option<u8>,
    /// Batas jumlah tangan (kalibrasi bot).
    #[serde(default)]
    pub tangan_maks: Option<u32>,
    /// Urutan dek tetap untuk tangan-tangan pertama (tes dan tutorial).
    #[serde(default)]
    pub dek: Option<Vec<Vec<String>>>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Action {
    Fold,
    Check,
    Call,
    Bet(i64),
    Raise(i64),
    Next,
    Leave,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
enum Phase {
    Play,
    Between,
    Over,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
struct Seat {
    start: i64,
    stack: i64,
    /// Taruhan di babak berjalan.
    bet: i64,
    /// Total yang dipasang di tangan berjalan.
    committed: i64,
    hole: Vec<Card>,
    in_hand: bool,
    folded: bool,
    allin: bool,
    acted: bool,
    may_raise: bool,
    /// Habis (tumpukan 0) atau berdiri: tidak ikut tangan berikutnya.
    out: bool,
    left: bool,
    ready: bool,
    shown: bool,
    won: i64,
}

impl Seat {
    fn new(stack: i64) -> Seat {
        Seat {
            start: stack,
            stack,
            bet: 0,
            committed: 0,
            hole: Vec::new(),
            in_hand: false,
            folded: false,
            allin: false,
            acted: false,
            may_raise: true,
            out: stack <= 0,
            left: false,
            ready: false,
            shown: false,
            won: 0,
        }
    }

    fn live(&self) -> bool {
        self.in_hand && !self.folded
    }

    fn can_act(&self) -> bool {
        self.live() && !self.allin
    }

    fn seated(&self) -> bool {
        !self.out && !self.left
    }

    fn put(&mut self, amount: i64) {
        let a = amount.min(self.stack);
        self.stack -= a;
        self.bet += a;
        self.committed += a;
        if self.stack == 0 {
            self.allin = true;
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(bound = "")]
pub struct PokerTable<V: Variant> {
    seed: Seed,
    seats: Vec<Seat>,
    humans: Vec<u8>,
    max_hands: Option<u32>,
    fixed: Vec<Vec<Card>>,
    deck: Shoe,
    board: Vec<Card>,
    /// Kartu meja yang sudah dibuka (0, 3, 4, 5).
    shown: usize,
    phase: Phase,
    hand: u32,
    dealer: u8,
    sb: u8,
    bb: u8,
    to_act: Option<u8>,
    current_bet: i64,
    last_raise: i64,
    log: Vec<(u8, String)>,
    last_pots: Vec<Pot>,
    ended: Option<&'static str>,
    #[serde(skip)]
    _v: PhantomData<V>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SeatView {
    pub tumpukan: i64,
    pub awal: i64,
    pub taruhan: i64,
    pub taruhan_tangan: i64,
    /// `aktif`, `fold`, `allin`, `duduk` (di antara tangan), `habis`,
    /// atau `berdiri`.
    pub status: String,
    /// Kartu sendiri, kartu yang dibuka saat showdown, atau `??`.
    pub kartu: Vec<String>,
    pub tangan: Option<String>,
    pub siap: bool,
    /// Yang diterima kursi ini dari pot tangan terakhir.
    pub menang: i64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct HandResult {
    pub menang: i64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct View {
    #[serde(flatten)]
    pub meja: Umum,
    /// Kursi yang melihat.
    pub kamu: u8,
    pub kursi: Vec<SeatView>,
    pub meja_kartu: Vec<String>,
    pub pot: i64,
    /// Pot tangan terakhir yang sampai showdown.
    pub pots: Vec<Pot>,
    pub dealer: u8,
    pub sb: u8,
    pub bb: u8,
    pub giliran: Option<u8>,
    pub tangan_ke: u32,
    pub blind: [i64; 2],
    /// Untuk kursi yang melihat, bila gilirannya: biaya call, dan batas
    /// bet/raise (total di babak ini).
    pub panggil: i64,
    pub naik_min: Option<i64>,
    pub naik_maks: Option<i64>,
    /// Hasil tangan terakhir per kursi.
    pub hasil: Vec<HandResult>,
    /// Aksi tangan berjalan: (kursi, perintah).
    pub log: Vec<(u8, String)>,
    pub pot_limit: bool,
}

impl meja::MejaView for View {
    fn meja(&self) -> &Umum {
        &self.meja
    }
}

impl<V: Variant> PokerTable<V> {
    fn n(&self) -> u8 {
        self.seats.len() as u8
    }

    /// Kursi berikutnya setelah `from` yang memenuhi `ok`.
    fn next_seat(&self, from: u8, ok: impl Fn(&Seat) -> bool) -> Option<u8> {
        let n = self.n();
        (1..=n)
            .map(|i| (from + i) % n)
            .find(|&s| ok(&self.seats[s as usize]))
    }

    fn pot_total(&self) -> i64 {
        self.seats.iter().map(|s| s.committed).sum()
    }

    fn hand_deck(&self) -> Shoe {
        let i = (self.hand - 1) as usize;
        match self.fixed.get(i) {
            Some(cards) => Shoe::from_cards(cards.clone()),
            None => Shoe::shuffled(
                1,
                &mut GameRng::from_seed(derive(&self.seed, &format!("tangan:{}", self.hand))),
            ),
        }
    }

    fn start_hand(&mut self) -> Result<(), GameError> {
        let players: Vec<u8> = (0..self.n())
            .filter(|&s| self.seats[s as usize].seated() && self.seats[s as usize].stack > 0)
            .collect();
        if players.len() < 2 {
            self.finish_session("meja_bubar");
            return Ok(());
        }
        self.hand += 1;
        if self.hand > 1 {
            self.dealer = self
                .next_seat(self.dealer, |s| s.seated() && s.stack > 0)
                .expect("ada pemain");
        } else if !self.seats[self.dealer as usize].seated() {
            self.dealer = self
                .next_seat(self.dealer, |s| s.seated() && s.stack > 0)
                .expect("ada pemain");
        }
        for s in &mut self.seats {
            s.bet = 0;
            s.committed = 0;
            s.hole.clear();
            s.in_hand = !s.out && !s.left && s.stack > 0;
            s.folded = false;
            s.allin = false;
            s.acted = false;
            s.may_raise = true;
            s.ready = false;
            s.shown = false;
            s.won = 0;
        }
        let in_hand = |s: &Seat| s.in_hand;
        if players.len() == 2 {
            self.sb = self.dealer;
            self.bb = self.next_seat(self.dealer, in_hand).expect("dua pemain");
        } else {
            self.sb = self.next_seat(self.dealer, in_hand).expect("pemain");
            self.bb = self.next_seat(self.sb, in_hand).expect("pemain");
        }
        self.seats[self.sb as usize].put(SMALL_BLIND);
        self.seats[self.bb as usize].put(BIG_BLIND);
        self.current_bet = BIG_BLIND;
        self.last_raise = BIG_BLIND;
        self.log.clear();
        self.last_pots.clear();
        self.deck = self.hand_deck();
        for _ in 0..V::HOLE {
            let mut s = self.dealer;
            for _ in 0..players.len() {
                s = self.next_seat(s, in_hand).expect("pemain");
                let c = meja::draw(&mut self.deck)?;
                self.seats[s as usize].hole.push(c);
            }
        }
        self.board = (0..5)
            .map(|_| meja::draw(&mut self.deck))
            .collect::<Result<_, _>>()?;
        self.shown = 0;
        self.phase = Phase::Play;
        let first = if players.len() == 2 {
            self.dealer
        } else {
            self.bb
        };
        self.to_act = if players.len() == 2 {
            Some(first).filter(|&s| self.seats[s as usize].can_act())
        } else {
            None
        };
        if self.to_act.is_none() {
            self.to_act = self.next_to_act(first);
        }
        self.progress()
    }

    /// Kursi berikutnya setelah `from` yang masih harus bertindak.
    fn next_to_act(&self, from: u8) -> Option<u8> {
        let bet = self.current_bet;
        self.next_seat(from, |s| s.can_act() && (!s.acted || s.bet < bet))
    }

    fn round_complete(&self) -> bool {
        self.seats
            .iter()
            .filter(|s| s.can_act())
            .all(|s| s.acted && s.bet == self.current_bet)
    }

    /// Setelah setiap aksi: tangan selesai karena fold, babak berikutnya,
    /// showdown, atau kursi berikutnya.
    fn progress(&mut self) -> Result<(), GameError> {
        let live: Vec<u8> = (0..self.n())
            .filter(|&s| self.seats[s as usize].live())
            .collect();
        if live.len() == 1 {
            let winner = live[0] as usize;
            let total = self.pot_total();
            self.seats[winner].stack += total;
            self.seats[winner].won = total;
            self.end_hand();
            return Ok(());
        }
        if !self.round_complete() {
            if let Some(next) = self.to_act {
                let s = &self.seats[next as usize];
                if s.can_act() && (!s.acted || s.bet < self.current_bet) {
                    return Ok(());
                }
                self.to_act = self.next_to_act(next);
            }
            if self.to_act.is_some() {
                return Ok(());
            }
        }
        // Babak selesai.
        for s in &mut self.seats {
            s.bet = 0;
            s.acted = false;
            s.may_raise = true;
        }
        self.current_bet = 0;
        self.last_raise = BIG_BLIND;
        let actors = self.seats.iter().filter(|s| s.can_act()).count();
        if self.shown == 5 {
            self.showdown();
            return Ok(());
        }
        self.shown = if self.shown == 0 { 3 } else { self.shown + 1 };
        if actors <= 1 {
            // Semua (kecuali paling banyak satu) all-in: buka sisa meja.
            self.shown = 5;
            self.showdown();
            return Ok(());
        }
        self.to_act = self.next_to_act(self.dealer);
        Ok(())
    }

    fn showdown(&mut self) {
        let board = self.board.clone();
        let values: Vec<Option<Value>> = self
            .seats
            .iter()
            .map(|s| s.live().then(|| V::best(&s.hole, &board)))
            .collect();
        let contrib: Vec<i64> = self.seats.iter().map(|s| s.committed).collect();
        let folded: Vec<bool> = self.seats.iter().map(|s| !s.live()).collect();
        let pots = pots(&contrib, &folded);
        let won = award(&pots, |s| values[s as usize], self.n(), self.dealer);
        for (s, w) in self.seats.iter_mut().zip(won) {
            s.stack += w;
            s.won = w;
            if s.live() {
                s.shown = true;
            }
        }
        self.last_pots = pots;
        self.end_hand();
    }

    fn end_hand(&mut self) {
        self.to_act = None;
        for s in &mut self.seats {
            s.committed = 0;
            s.bet = 0;
            if s.stack == 0 {
                s.out = true;
            }
        }
        self.phase = Phase::Between;
        self.check_session();
    }

    fn check_session(&mut self) {
        let humans_gone = self
            .humans
            .iter()
            .all(|&h| !self.seats.get(h as usize).is_some_and(Seat::seated));
        let with_chips = self.seats.iter().filter(|s| s.seated()).count();
        if humans_gone {
            self.finish_session("berdiri");
        } else if with_chips < 2 {
            self.finish_session("meja_bubar");
        } else if self.max_hands.is_some_and(|m| self.hand >= m) {
            self.finish_session("batas_tangan");
        }
    }

    fn finish_session(&mut self, reason: &'static str) {
        self.phase = Phase::Over;
        self.to_act = None;
        self.ended = Some(reason);
    }

    fn to_call(&self, seat: u8) -> i64 {
        let s = &self.seats[seat as usize];
        (self.current_bet - s.bet).min(s.stack)
    }

    /// Batas bet/raise (total di babak ini) untuk `seat`, bila boleh.
    fn raise_range(&self, seat: u8) -> Option<(i64, i64)> {
        let s = &self.seats[seat as usize];
        let allin = s.bet + s.stack;
        if !s.may_raise || allin <= self.current_bet {
            return None;
        }
        let full = if self.current_bet == 0 {
            BIG_BLIND
        } else {
            self.current_bet + self.last_raise
        };
        let mut max = allin;
        if V::POT_LIMIT {
            let call = self.current_bet - s.bet;
            max = max.min(self.current_bet + self.pot_total() + call);
        }
        let min = full.min(allin);
        Some((min, max.max(min)))
    }

    fn betting(&mut self, seat: u8, total: i64) -> Result<(), GameError> {
        let (min, max) = self
            .raise_range(seat)
            .ok_or_else(|| GameError::Illegal(format!("raise {total}")))?;
        if total < min || total > max {
            return Err(GameError::Illegal(format!("raise {total}")));
        }
        let increment = total - self.current_bet;
        let full = increment >= self.last_raise || self.current_bet == 0 && total >= BIG_BLIND;
        let s = &mut self.seats[seat as usize];
        let add = total - s.bet;
        s.put(add);
        if full {
            self.last_raise = increment.max(BIG_BLIND);
        }
        self.current_bet = total;
        for (i, other) in self.seats.iter_mut().enumerate() {
            if i as u8 == seat || !other.can_act() {
                continue;
            }
            if full {
                other.may_raise = true;
            } else if other.acted {
                other.may_raise = false;
            }
            other.acted = false;
        }
        Ok(())
    }

    fn seat_status(&self, s: &Seat) -> &'static str {
        match self.phase {
            Phase::Play if s.in_hand && s.folded => "fold",
            Phase::Play if s.in_hand && s.allin => "allin",
            Phase::Play if s.in_hand => "aktif",
            _ if s.left => "berdiri",
            _ if s.out => "habis",
            Phase::Between if s.in_hand && s.folded => "fold",
            _ => "duduk",
        }
    }
}

impl<V: Variant> TurnGame for PokerTable<V> {
    type Config = Config;
    type Action = Action;
    type View = View;

    fn new(config: Config, seed: Seed) -> Result<Self, GameError> {
        let n = config.kursi.unwrap_or(DEFAULT_SEATS);
        if !(2..=6).contains(&n) {
            return Err(GameError::Config(format!("kursi {n}")));
        }
        let stacks = config
            .tumpukan
            .clone()
            .unwrap_or_else(|| vec![DEFAULT_STACK; n as usize]);
        if stacks.len() != n as usize || stacks.iter().any(|&s| s < 0) {
            return Err(GameError::Config("tumpukan".into()));
        }
        let humans = config.manusia.clone().unwrap_or_else(|| vec![0]);
        if humans.iter().any(|&h| h >= n) {
            return Err(GameError::Config("manusia".into()));
        }
        let fixed = config
            .dek
            .as_ref()
            .map(|decks| decks.iter().map(|d| meja::parse_cards(d)).collect())
            .transpose()?
            .unwrap_or_default();
        let dealer = match config.dealer {
            Some(d) if d < n => d,
            Some(d) => return Err(GameError::Config(format!("dealer {d}"))),
            None => GameRng::from_seed(derive(&seed, "dealer")).below(u32::from(n)) as u8,
        };
        let mut table = PokerTable {
            seed,
            seats: stacks.into_iter().map(Seat::new).collect(),
            humans,
            max_hands: config.tangan_maks,
            fixed,
            deck: Shoe::from_cards(Vec::new()),
            board: Vec::new(),
            shown: 0,
            phase: Phase::Between,
            hand: 0,
            dealer,
            sb: 0,
            bb: 0,
            to_act: None,
            current_bet: 0,
            last_raise: BIG_BLIND,
            log: Vec::new(),
            last_pots: Vec::new(),
            ended: None,
            _v: PhantomData,
        };
        table.start_hand()?;
        Ok(table)
    }

    fn seats(&self) -> u8 {
        self.n()
    }

    fn pending_players(&self) -> Vec<PlayerId> {
        match self.phase {
            Phase::Play => self.to_act.into_iter().collect(),
            Phase::Between => (0..self.n())
                .filter(|&s| self.seats[s as usize].seated() && !self.seats[s as usize].ready)
                .collect(),
            Phase::Over => Vec::new(),
        }
    }

    fn legal_actions(&self, player: PlayerId) -> Vec<ActionSpec> {
        if !self.pending_players().contains(&player) {
            return Vec::new();
        }
        match self.phase {
            Phase::Play => {
                let mut out = vec![ActionSpec::fixed("fold")];
                if self.to_call(player) == 0 {
                    out.push(ActionSpec::fixed("check"));
                } else {
                    out.push(ActionSpec::fixed("call"));
                }
                if let Some((min, max)) = self.raise_range(player) {
                    let verb = if self.current_bet == 0 {
                        "bet"
                    } else {
                        "raise"
                    };
                    out.push(ActionSpec::template(
                        verb,
                        vec![Param {
                            name: "jumlah".into(),
                            kind: ParamKind::Int { min, max, step: 1 },
                        }],
                    ));
                }
                out
            }
            Phase::Between => vec![ActionSpec::fixed("next"), ActionSpec::fixed("leave")],
            Phase::Over => Vec::new(),
        }
    }

    fn apply(&mut self, player: PlayerId, action: Action) -> Result<(), GameError> {
        if self.phase == Phase::Over {
            return Err(GameError::Over);
        }
        if !self.pending_players().contains(&player) {
            return Err(GameError::NotPending(player));
        }
        let text = self.format_action(&action);
        let illegal = || GameError::Illegal(text.clone());
        match (self.phase, action) {
            (Phase::Play, Action::Fold) => self.seats[player as usize].folded = true,
            (Phase::Play, Action::Check) if self.to_call(player) == 0 => {}
            (Phase::Play, Action::Call) if self.to_call(player) > 0 => {
                let c = self.to_call(player);
                self.seats[player as usize].put(c);
            }
            (Phase::Play, Action::Bet(n)) if self.current_bet == 0 => self.betting(player, n)?,
            (Phase::Play, Action::Raise(n)) if self.current_bet > 0 => self.betting(player, n)?,
            (Phase::Between, Action::Next) => {
                self.seats[player as usize].ready = true;
                if self.pending_players().is_empty() {
                    self.start_hand()?;
                }
                return Ok(());
            }
            (Phase::Between, Action::Leave) => {
                self.seats[player as usize].left = true;
                self.check_session();
                if self.phase == Phase::Between && self.pending_players().is_empty() {
                    self.start_hand()?;
                }
                return Ok(());
            }
            _ => return Err(illegal()),
        }
        self.seats[player as usize].acted = true;
        self.log.push((player, text));
        self.progress()
    }

    fn view_for(&self, player: PlayerId) -> View {
        let me = self.seats.get(player as usize);
        let over = self.phase == Phase::Over;
        let fase = match self.phase {
            Phase::Play => "main",
            Phase::Between => "antara",
            Phase::Over => "selesai",
        };
        let mut meja = Umum::new(
            fase,
            meja::Limits {
                min: BIG_BLIND,
                max: me.map(|s| s.stack + s.bet).unwrap_or(0),
                step: 1,
            },
        );
        let gone = over || me.is_some_and(|s| s.left);
        meja.taruhan_meja = if gone {
            0
        } else {
            me.map(|s| s.start).unwrap_or(0)
        };
        meja.bersih = if gone {
            me.map(|s| s.stack - s.start).unwrap_or(0)
        } else {
            0
        };
        meja.ronde = self.hand;
        meja.selesai = over;
        meja.alasan = self.ended.map(str::to_string);
        if self.pending_players().contains(&player) {
            meja.netral = Some(match self.phase {
                Phase::Play if self.to_call(player) == 0 => "check".into(),
                Phase::Play => "fold".into(),
                _ => "leave".into(),
            });
        }
        let my_turn = self.phase == Phase::Play && self.to_act == Some(player);
        let range = if my_turn {
            self.raise_range(player)
        } else {
            None
        };
        View {
            meja,
            kamu: player,
            kursi: self
                .seats
                .iter()
                .enumerate()
                .map(|(i, s)| SeatView {
                    tumpukan: s.stack,
                    awal: s.start,
                    taruhan: s.bet,
                    taruhan_tangan: s.committed,
                    status: self.seat_status(s).into(),
                    kartu: if !s.in_hand || (s.folded && i as u8 != player) {
                        Vec::new()
                    } else if i as u8 == player || s.shown {
                        s.hole.iter().map(Card::to_string).collect()
                    } else {
                        vec!["??".into(); s.hole.len()]
                    },
                    tangan: (s.shown && self.shown == 5)
                        .then(|| V::best(&s.hole, &self.board).key().to_string()),
                    siap: s.ready,
                    menang: s.won,
                })
                .collect(),
            meja_kartu: self.board[..self.shown.min(self.board.len())]
                .iter()
                .map(Card::to_string)
                .collect(),
            pot: self.pot_total(),
            pots: self.last_pots.clone(),
            dealer: self.dealer,
            sb: self.sb,
            bb: self.bb,
            giliran: self.to_act,
            tangan_ke: self.hand,
            blind: [SMALL_BLIND, BIG_BLIND],
            panggil: if my_turn { self.to_call(player) } else { 0 },
            naik_min: range.map(|r| r.0),
            naik_maks: range.map(|r| r.1),
            hasil: self
                .seats
                .iter()
                .map(|s| HandResult { menang: s.won })
                .collect(),
            log: self.log.clone(),
            pot_limit: V::POT_LIMIT,
        }
    }

    fn to_text(view: &View, lang: Lang) -> String {
        let c = catalog();
        let mut out = String::new();
        out.push_str(&c.text(
            lang,
            "hand",
            &[
                ("n", &view.tangan_ke.to_string()),
                ("board", &view.meja_kartu.join(" ")),
                ("pot", &view.pot.to_string()),
            ],
        ));
        out.push('\n');
        for (i, s) in view.kursi.iter().enumerate() {
            let marker = if view.giliran == Some(i as u8) {
                ">"
            } else {
                " "
            };
            let who = if i as u8 == view.kamu {
                c.text(lang, "you", &[])
            } else {
                c.text(lang, "seat", &[("n", &(i + 1).to_string())])
            };
            let dealer = if view.dealer == i as u8 { " (D)" } else { "" };
            let hand = s
                .tangan
                .as_ref()
                .map(|k| format!(" {}", meja::hand_name(lang, k)))
                .unwrap_or_default();
            out.push_str(&format!(
                "{marker} {who}{dealer}: {} · {} {} · {} {}{hand}\n",
                s.tumpukan,
                c.text(lang, "bet", &[]),
                s.taruhan,
                c.text(lang, &format!("status.{}", s.status), &[]),
                s.kartu.join(" "),
            ));
        }
        out.push_str(&match view.meja.fase.as_str() {
            "main" if view.giliran == Some(view.kamu) => {
                c.text(lang, "your_turn", &[("call", &view.panggil.to_string())])
            }
            "main" => c.text(lang, "waiting", &[]),
            "antara" => c.text(lang, "between", &[]),
            _ => c.text(
                lang,
                &format!("over.{}", view.meja.alasan.as_deref().unwrap_or("berdiri")),
                &[],
            ),
        });
        out
    }

    fn is_over(&self) -> bool {
        self.phase == Phase::Over
    }

    fn result(&self) -> Option<GameResult> {
        if self.phase != Phase::Over {
            return None;
        }
        let scores: Vec<i64> = self.seats.iter().map(|s| s.stack - s.start).collect();
        let best = scores.iter().copied().max().unwrap_or(0);
        let winners: Vec<u8> = if scores.iter().all(|&x| x == best) {
            Vec::new()
        } else {
            (0..self.n())
                .filter(|&s| scores[s as usize] == best)
                .collect()
        };
        let hands = self.hand;
        let human = self.humans.first().copied().unwrap_or(0) as usize;
        let net = scores.get(human).copied().unwrap_or(0);
        Some(GameResult {
            winners,
            scores,
            summary: Localized::build(|lang| {
                catalog().text(
                    lang,
                    "summary",
                    &[("hands", &hands.to_string()), ("net", &format!("{net:+}"))],
                )
            }),
        })
    }

    fn parse_command(&self, command: &str) -> Result<Action, GameError> {
        let c = command.trim();
        let parts: Vec<&str> = c.split(' ').collect();
        let err = || GameError::Parse(c.into());
        match parts.as_slice() {
            ["fold"] => Ok(Action::Fold),
            ["check"] => Ok(Action::Check),
            ["call"] => Ok(Action::Call),
            ["bet", n] => Ok(Action::Bet(meja::amount(n).ok_or_else(err)?)),
            ["raise", n] => Ok(Action::Raise(meja::amount(n).ok_or_else(err)?)),
            ["next"] => Ok(Action::Next),
            ["leave"] => Ok(Action::Leave),
            _ => Err(err()),
        }
    }

    fn format_action(&self, action: &Action) -> String {
        match action {
            Action::Fold => "fold".into(),
            Action::Check => "check".into(),
            Action::Call => "call".into(),
            Action::Bet(n) => format!("bet {n}"),
            Action::Raise(n) => format!("raise {n}"),
            Action::Next => "next".into(),
            Action::Leave => "leave".into(),
        }
    }
}
