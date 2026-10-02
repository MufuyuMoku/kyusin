//! Teen Patti (SPEC §6.3; M5b-1): tiga kartu antar-pemain dengan taruhan
//! buta/terlihat, show, dan sideshow (D-063).
//!
//! Satu pertandingan = satu sesi meja seperti poker KyuSin: buy-in dari
//! saldo saat duduk, tumpukan kembali saat berdiri, provably fair per sesi
//! (dek tiap tangan dari seed sesi + nomor tangan), tanpa rake. Boot 10,
//! stake mula-mula = boot, stake paling besar 64 × boot, chaal buta paling
//! banyak 4 kali, pot paling besar 1.024 × boot (lalu semua dibandingkan).

use std::collections::BTreeMap;
use std::sync::OnceLock;

use kyusin_core::game::{GameResult, create_session};
use kyusin_core::i18n::{Catalog, Lang, Localized};
use kyusin_core::rng::derive;
use kyusin_core::{
    ActionSpec, Cartridge, GameError, GameRng, PlayerId, RegistryError, Seed, TurnGame,
};
use serde::{Deserialize, Serialize};

use crate::cards::{Card, Shoe};
use crate::meja::{self, Umum};
use crate::poker::rank_value;
use crate::pot::{Pot, award, pots_all_in};

pub const ID: &str = "teen-patti";
pub const BOOT: i64 = 10;
pub const STAKE_CAP: i64 = 64 * BOOT;
pub const POT_LIMIT: i64 = 1024 * BOOT;
pub const BLIND_LIMIT: u8 = 4;
pub const DEFAULT_SEATS: u8 = 5;
pub const DEFAULT_STACK: i64 = 2000;

pub fn catalog() -> &'static Catalog {
    static CATALOG: OnceLock<Catalog> = OnceLock::new();
    CATALOG.get_or_init(|| {
        Catalog::from_toml(include_str!("i18n.toml")).expect("teen_patti/i18n.toml tidak sah")
    })
}

pub fn cartridge() -> Result<Cartridge, RegistryError> {
    Cartridge::new(
        include_str!("manifest.toml"),
        include_str!("../../../../tutorials/teen-patti.toml"),
        create_session::<TeenPatti>,
    )
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum TpCategory {
    HighCard,
    Pair,
    Color,
    Sequence,
    PureSequence,
    Trail,
}

impl TpCategory {
    const ALL: [TpCategory; 6] = [
        TpCategory::HighCard,
        TpCategory::Pair,
        TpCategory::Color,
        TpCategory::Sequence,
        TpCategory::PureSequence,
        TpCategory::Trail,
    ];

    pub fn key(self) -> &'static str {
        match self {
            TpCategory::HighCard => "high_card",
            TpCategory::Pair => "pair",
            TpCategory::Color => "color",
            TpCategory::Sequence => "sequence",
            TpCategory::PureSequence => "pure_sequence",
            TpCategory::Trail => "trail",
        }
    }
}

/// Nilai tangan Teen Patti; lebih besar = lebih kuat.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct TpValue(pub u32);

impl TpValue {
    pub fn category(self) -> TpCategory {
        TpCategory::ALL[(self.0 >> 12) as usize]
    }
}

/// Nilai tiga kartu: trail, pure sequence, sequence, color, pair, kartu
/// tinggi. A-K-Q sequence tertinggi, lalu A-2-3, lalu K-Q-J … 4-3-2.
pub fn eval(cards: &[Card]) -> TpValue {
    assert_eq!(cards.len(), 3, "tiga kartu");
    let mut r: Vec<u32> = cards
        .iter()
        .map(|c| u32::from(rank_value(c.rank)))
        .collect();
    r.sort_unstable_by(|a, b| b.cmp(a));
    let flush = cards.iter().all(|c| c.suit == cards[0].suit);
    let seq = if r == [14, 13, 12] {
        Some(15)
    } else if r == [14, 3, 2] {
        Some(14)
    } else if r[0] == r[1] + 1 && r[1] == r[2] + 1 {
        Some(r[0])
    } else {
        None
    };
    let make = |c: TpCategory, v: u32| TpValue(((c as u32) << 12) | v);
    if r[0] == r[2] {
        return make(TpCategory::Trail, r[0] << 8);
    }
    if let Some(code) = seq {
        let c = if flush {
            TpCategory::PureSequence
        } else {
            TpCategory::Sequence
        };
        return make(c, code << 8);
    }
    let high = (r[0] << 8) | (r[1] << 4) | r[2];
    if flush {
        return make(TpCategory::Color, high);
    }
    if r[0] == r[1] {
        return make(TpCategory::Pair, (r[0] << 8) | (r[2] << 4));
    }
    if r[1] == r[2] {
        return make(TpCategory::Pair, (r[1] << 8) | (r[0] << 4));
    }
    make(TpCategory::HighCard, high)
}

/// Konfigurasi meja; sama dengan meja poker KyuSin.
#[derive(Debug, Clone, Default, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Config {
    /// Jumlah kursi (2–6; bawaan 5).
    #[serde(default)]
    pub kursi: Option<u8>,
    #[serde(default)]
    pub tumpukan: Option<Vec<i64>>,
    #[serde(default)]
    pub manusia: Option<Vec<u8>>,
    #[serde(default)]
    pub dealer: Option<u8>,
    #[serde(default)]
    pub tangan_maks: Option<u32>,
    #[serde(default)]
    pub dek: Option<Vec<Vec<String>>>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Action {
    See,
    Pack,
    Chaal,
    Raise,
    Show,
    Sideshow,
    Accept,
    Deny,
    Next,
    Leave,
}

const COMMANDS: [(Action, &str); 10] = [
    (Action::See, "see"),
    (Action::Pack, "pack"),
    (Action::Chaal, "chaal"),
    (Action::Raise, "raise"),
    (Action::Show, "show"),
    (Action::Sideshow, "sideshow"),
    (Action::Accept, "accept"),
    (Action::Deny, "deny"),
    (Action::Next, "next"),
    (Action::Leave, "leave"),
];

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
    committed: i64,
    hole: Vec<Card>,
    in_hand: bool,
    folded: bool,
    allin: bool,
    seen: bool,
    blind: u8,
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
            committed: 0,
            hole: Vec::new(),
            in_hand: false,
            folded: false,
            allin: false,
            seen: false,
            blind: 0,
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

    fn pay(&mut self, amount: i64) {
        let a = amount.min(self.stack);
        self.stack -= a;
        self.committed += a;
        if self.stack == 0 {
            self.allin = true;
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct TeenPatti {
    seed: Seed,
    seats: Vec<Seat>,
    humans: Vec<u8>,
    max_hands: Option<u32>,
    fixed: Vec<Vec<Card>>,
    phase: Phase,
    hand: u32,
    dealer: u8,
    to_act: Option<u8>,
    stake: i64,
    /// Sideshow yang menunggu jawaban: (peminta, yang diminta).
    sideshow: Option<(u8, u8)>,
    last_pots: Vec<Pot>,
    hand_end: Option<&'static str>,
    log: Vec<(u8, String)>,
    ended: Option<&'static str>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SeatView {
    pub tumpukan: i64,
    pub awal: i64,
    pub taruhan_tangan: i64,
    /// `aktif`, `pack`, `allin`, `duduk`, `habis`, atau `berdiri`.
    pub status: String,
    pub terlihat: bool,
    pub buta_ke: u8,
    pub kartu: Vec<String>,
    pub tangan: Option<String>,
    pub siap: bool,
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
    pub kamu: u8,
    pub kursi: Vec<SeatView>,
    pub pot: i64,
    pub pots: Vec<Pot>,
    pub stake: i64,
    pub boot: i64,
    pub dealer: u8,
    pub giliran: Option<u8>,
    pub sideshow: Option<[u8; 2]>,
    pub tangan_ke: u32,
    pub hasil: Vec<HandResult>,
    /// Cara tangan terakhir berakhir: `pack`, `show`, `allin`, `batas_pot`.
    pub alasan_tangan: Option<String>,
    pub log: Vec<(u8, String)>,
}

impl meja::MejaView for View {
    fn meja(&self) -> &Umum {
        &self.meja
    }
}

impl TeenPatti {
    fn n(&self) -> u8 {
        self.seats.len() as u8
    }

    fn next_seat(&self, from: u8, ok: impl Fn(&Seat) -> bool) -> Option<u8> {
        let n = self.n();
        (1..=n)
            .map(|i| (from + i) % n)
            .find(|&s| ok(&self.seats[s as usize]))
    }

    fn prev_seat(&self, from: u8, ok: impl Fn(&Seat) -> bool) -> Option<u8> {
        let n = self.n();
        (1..n)
            .map(|i| (from + n - i) % n)
            .find(|&s| ok(&self.seats[s as usize]))
    }

    fn pot_total(&self) -> i64 {
        self.seats.iter().map(|s| s.committed).sum()
    }

    fn live_count(&self) -> usize {
        self.seats.iter().filter(|s| s.live()).count()
    }

    fn chaal_cost(&self, seat: u8) -> i64 {
        if self.seats[seat as usize].seen {
            2 * self.stake
        } else {
            self.stake
        }
    }

    fn blind_ok(&self, seat: u8) -> bool {
        let s = &self.seats[seat as usize];
        s.seen || s.blind < BLIND_LIMIT
    }

    fn can_raise(&self, seat: u8) -> bool {
        self.blind_ok(seat)
            && 2 * self.stake <= STAKE_CAP
            && self.seats[seat as usize].stack > 2 * self.chaal_cost(seat)
    }

    fn can_show(&self, seat: u8) -> bool {
        self.live_count() == 2 && self.blind_ok(seat)
    }

    /// Pemain aktif sebelumnya yang bisa diajak sideshow.
    fn sideshow_target(&self, seat: u8) -> Option<u8> {
        let me = &self.seats[seat as usize];
        if !me.seen || self.live_count() < 3 || me.stack < 2 * self.stake {
            return None;
        }
        let prev = self.prev_seat(seat, Seat::live)?;
        let p = &self.seats[prev as usize];
        (p.seen && !p.allin).then_some(prev)
    }

    fn hand_deck(&self) -> Shoe {
        match self.fixed.get((self.hand - 1) as usize) {
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
        let playing = |s: &Seat| s.seated() && s.stack > 0;
        if self.hand > 1 || !playing(&self.seats[self.dealer as usize]) {
            self.dealer = self.next_seat(self.dealer, playing).expect("ada pemain");
        }
        for s in &mut self.seats {
            s.committed = 0;
            s.hole.clear();
            s.in_hand = !s.out && !s.left && s.stack > 0;
            s.folded = false;
            s.allin = false;
            s.seen = false;
            s.blind = 0;
            s.ready = false;
            s.shown = false;
            s.won = 0;
        }
        for &s in &players {
            self.seats[s as usize].pay(BOOT);
        }
        self.stake = BOOT;
        self.sideshow = None;
        self.last_pots.clear();
        self.hand_end = None;
        self.log.clear();
        let mut deck = self.hand_deck();
        for _ in 0..3 {
            let mut s = self.dealer;
            for _ in 0..players.len() {
                s = self.next_seat(s, |x| x.in_hand).expect("pemain");
                let c = meja::draw(&mut deck)?;
                self.seats[s as usize].hole.push(c);
            }
        }
        self.phase = Phase::Play;
        self.advance(self.dealer);
        Ok(())
    }

    /// Setelah setiap aksi: tangan selesai, atau giliran kursi berikutnya
    /// setelah `from`.
    fn advance(&mut self, from: u8) {
        let live = self.live_count();
        if live == 1 {
            let winner = self.seats.iter().position(Seat::live).expect("satu pemain");
            let total = self.pot_total();
            self.seats[winner].stack += total;
            self.seats[winner].won = total;
            self.end_hand("pack");
            return;
        }
        if self.pot_total() >= POT_LIMIT {
            self.showdown("batas_pot");
            return;
        }
        let actors = self.seats.iter().filter(|s| s.can_act()).count();
        if actors <= 1 {
            self.showdown("allin");
            return;
        }
        self.to_act = self.next_seat(from, Seat::can_act);
    }

    fn values(&self) -> Vec<Option<TpValue>> {
        self.seats
            .iter()
            .map(|s| s.live().then(|| eval(&s.hole)))
            .collect()
    }

    fn showdown(&mut self, reason: &'static str) {
        let values = self.values();
        let contrib: Vec<i64> = self.seats.iter().map(|s| s.committed).collect();
        let folded: Vec<bool> = self.seats.iter().map(|s| !s.live()).collect();
        let allin: Vec<bool> = self.seats.iter().map(|s| s.allin).collect();
        let pots = pots_all_in(&contrib, &folded, &allin);
        let won = award(&pots, |s| values[s as usize], self.n(), self.dealer);
        for (s, w) in self.seats.iter_mut().zip(won) {
            s.stack += w;
            s.won = w;
            if s.live() {
                s.shown = true;
            }
        }
        self.last_pots = pots;
        self.end_hand(reason);
    }

    fn end_hand(&mut self, reason: &'static str) {
        self.hand_end = Some(reason);
        self.to_act = None;
        self.sideshow = None;
        for s in &mut self.seats {
            s.committed = 0;
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
        let seated = self.seats.iter().filter(|s| s.seated()).count();
        if humans_gone {
            self.finish_session("berdiri");
        } else if seated < 2 {
            self.finish_session("meja_bubar");
        } else if self.max_hands.is_some_and(|m| self.hand >= m) {
            self.finish_session("batas_tangan");
        }
    }

    fn finish_session(&mut self, reason: &'static str) {
        self.phase = Phase::Over;
        self.to_act = None;
        self.sideshow = None;
        self.ended = Some(reason);
    }

    fn seat_status(&self, s: &Seat) -> &'static str {
        match self.phase {
            Phase::Play if s.in_hand && s.folded => "pack",
            Phase::Play if s.in_hand && s.allin => "allin",
            Phase::Play if s.in_hand => "aktif",
            _ if s.left => "berdiri",
            _ if s.out => "habis",
            Phase::Between if s.in_hand && s.folded => "pack",
            _ => "duduk",
        }
    }

    fn costs(&self, seat: u8) -> BTreeMap<String, i64> {
        let mut out = BTreeMap::new();
        if self.phase != Phase::Play || self.to_act != Some(seat) || self.sideshow.is_some() {
            return out;
        }
        let chaal = self.chaal_cost(seat);
        if self.blind_ok(seat) {
            out.insert("chaal".to_string(), chaal);
        }
        if self.can_raise(seat) {
            out.insert("raise".to_string(), 2 * chaal);
        }
        if self.can_show(seat) {
            out.insert("show".to_string(), chaal);
        }
        if self.sideshow_target(seat).is_some() {
            out.insert("sideshow".to_string(), 2 * self.stake);
        }
        out
    }
}

impl TurnGame for TeenPatti {
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
        let mut table = TeenPatti {
            seed,
            seats: stacks.into_iter().map(Seat::new).collect(),
            humans,
            max_hands: config.tangan_maks,
            fixed,
            phase: Phase::Between,
            hand: 0,
            dealer,
            to_act: None,
            stake: BOOT,
            sideshow: None,
            last_pots: Vec::new(),
            hand_end: None,
            log: Vec::new(),
            ended: None,
        };
        table.start_hand()?;
        Ok(table)
    }

    fn seats(&self) -> u8 {
        self.n()
    }

    fn pending_players(&self) -> Vec<PlayerId> {
        match self.phase {
            Phase::Play => match self.sideshow {
                Some((_, to)) => vec![to],
                None => self.to_act.into_iter().collect(),
            },
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
            Phase::Play if self.sideshow.is_some() => {
                vec![ActionSpec::fixed("accept"), ActionSpec::fixed("deny")]
            }
            Phase::Play => {
                let mut out = Vec::new();
                if !self.seats[player as usize].seen {
                    out.push(ActionSpec::fixed("see"));
                }
                out.push(ActionSpec::fixed("pack"));
                for cmd in self.costs(player).into_keys() {
                    out.push(ActionSpec::fixed(cmd));
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
        let legal = self.legal_actions(player).iter().any(|a| a.usage() == text);
        if !legal {
            return Err(GameError::Illegal(text));
        }
        let p = player as usize;
        match action {
            Action::See => {
                self.seats[p].seen = true;
                self.log.push((player, text));
                return Ok(());
            }
            Action::Pack => {
                self.seats[p].folded = true;
                self.log.push((player, text));
                self.advance(player);
            }
            Action::Chaal | Action::Raise => {
                let mut cost = self.chaal_cost(player);
                if action == Action::Raise {
                    cost *= 2;
                    self.stake *= 2;
                }
                if !self.seats[p].seen {
                    self.seats[p].blind += 1;
                }
                self.seats[p].pay(cost);
                self.log.push((player, text));
                self.advance(player);
            }
            Action::Show => {
                let cost = self.chaal_cost(player);
                self.seats[p].pay(cost);
                self.log.push((player, text));
                self.showdown("show");
            }
            Action::Sideshow => {
                let target = self.sideshow_target(player).expect("sah");
                self.seats[p].pay(2 * self.stake);
                self.log.push((player, text));
                self.sideshow = Some((player, target));
                if self.pot_total() >= POT_LIMIT {
                    self.sideshow = None;
                    self.showdown("batas_pot");
                }
            }
            Action::Accept | Action::Deny => {
                let (from, to) = self.sideshow.take().expect("sideshow");
                if action == Action::Accept {
                    let (a, b) = (
                        eval(&self.seats[from as usize].hole),
                        eval(&self.seats[to as usize].hole),
                    );
                    let loser = if a > b { to } else { from };
                    self.seats[loser as usize].folded = true;
                }
                self.log.push((player, text));
                self.advance(from);
            }
            Action::Next => {
                self.seats[p].ready = true;
                if self.pending_players().is_empty() {
                    self.start_hand()?;
                }
            }
            Action::Leave => {
                self.seats[p].left = true;
                self.check_session();
                if self.phase == Phase::Between && self.pending_players().is_empty() {
                    self.start_hand()?;
                }
            }
        }
        Ok(())
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
                min: BOOT,
                max: STAKE_CAP,
                step: BOOT,
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
        meja.biaya = self.costs(player);
        if self.pending_players().contains(&player) {
            meja.netral = Some(match (self.phase, self.sideshow) {
                (Phase::Play, Some(_)) => "deny".into(),
                (Phase::Play, None) => "pack".into(),
                _ => "leave".into(),
            });
        }
        let in_play = self.phase == Phase::Play;
        View {
            meja,
            kamu: player,
            kursi: self
                .seats
                .iter()
                .enumerate()
                .map(|(i, s)| {
                    let mine = i as u8 == player;
                    SeatView {
                        tumpukan: s.stack,
                        awal: s.start,
                        taruhan_tangan: s.committed,
                        status: self.seat_status(s).into(),
                        terlihat: s.seen,
                        buta_ke: s.blind,
                        kartu: if !s.in_hand || (s.folded && !mine) {
                            Vec::new()
                        } else if s.shown || (mine && (s.seen || !in_play)) {
                            s.hole.iter().map(Card::to_string).collect()
                        } else {
                            vec!["??".into(); s.hole.len()]
                        },
                        tangan: s.shown.then(|| eval(&s.hole).category().key().to_string()),
                        siap: s.ready,
                        menang: s.won,
                    }
                })
                .collect(),
            pot: self.pot_total(),
            pots: self.last_pots.clone(),
            stake: self.stake,
            boot: BOOT,
            dealer: self.dealer,
            giliran: if self.sideshow.is_some() {
                None
            } else {
                self.to_act
            },
            sideshow: self.sideshow.map(|(a, b)| [a, b]),
            tangan_ke: self.hand,
            hasil: self
                .seats
                .iter()
                .map(|s| HandResult { menang: s.won })
                .collect(),
            alasan_tangan: self.hand_end.map(str::to_string),
            log: self.log.clone(),
        }
    }

    fn to_text(view: &View, lang: Lang) -> String {
        let c = catalog();
        let mut out = c.text(
            lang,
            "hand",
            &[
                ("n", &view.tangan_ke.to_string()),
                ("pot", &view.pot.to_string()),
                ("stake", &view.stake.to_string()),
            ],
        );
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
            let blind = if s.terlihat {
                c.text(lang, "seen", &[])
            } else {
                c.text(lang, "blind", &[])
            };
            let hand = s
                .tangan
                .as_ref()
                .map(|k| format!(" {}", c.text(lang, &format!("hand.{k}"), &[])))
                .unwrap_or_default();
            out.push_str(&format!(
                "{marker} {who}{dealer}: {} · {} · {blind} {}{hand}\n",
                s.tumpukan,
                c.text(lang, &format!("status.{}", s.status), &[]),
                s.kartu.join(" "),
            ));
        }
        out.push_str(&match view.meja.fase.as_str() {
            "main" if view.sideshow.is_some_and(|[_, to]| to == view.kamu) => {
                c.text(lang, "sideshow_asked", &[])
            }
            "main" if view.giliran == Some(view.kamu) => c.text(lang, "your_turn", &[]),
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
        COMMANDS
            .iter()
            .find(|(_, t)| *t == c)
            .map(|(a, _)| *a)
            .ok_or_else(|| GameError::Parse(c.into()))
    }

    fn format_action(&self, action: &Action) -> String {
        COMMANDS
            .iter()
            .find(|(a, _)| a == action)
            .map(|(_, t)| t.to_string())
            .unwrap_or_default()
    }
}
