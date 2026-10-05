//! Domino QiuQiu (SPEC §6.3; M5b-2): kartu domino antar-pemain dengan dua
//! putaran taruhan pot-limit. Aturan lokal dipilih klien (D-065).
//!
//! Satu pertandingan = satu sesi meja seperti poker KyuSin (D-063): buy-in
//! dari saldo saat duduk, tumpukan kembali saat berdiri, kartu tiap tangan
//! dikocok dari seed sesi dan nomor tangan. Ante 10 dari setiap kursi;
//! tiga kartu dibagi satu per satu mulai kursi setelah dealer, putaran
//! taruhan, kartu ke-4, putaran taruhan, buka kartu. `bet <n>` / `raise
//! <n>` = total taruhan di putaran itu; bet paling kecil 20, raise paling
//! kecil sebesar raise terakhir, paling besar sebesar pot (pot-limit).
//!
//! Empat kartu dibagi otomatis menjadi dua pasangan terbaik (nilai
//! pasangan = jumlah bulatan mod 10; pasangan tertinggi dulu, lalu yang
//! kedua; bila beberapa pembagian sama nilainya, pasangan tertinggi berisi
//! kartu tertinggi). Kartu spesial mengalahkan semua pasangan: Enam Dewa >
//! Balak > Murni Kecil > Murni Besar. Nilai sama: kartu tunggal tertinggi
//! (balak > non-balak, lalu jumlah bulatan, lalu angka terbesar), lalu
//! kartu kedua, dan seterusnya; masih sama = pot dibagi.

use std::cmp::Ordering;
use std::sync::OnceLock;

use kyusin_core::game::{GameResult, create_session};
use kyusin_core::i18n::{Catalog, Lang, Localized};
use kyusin_core::rng::derive;
use kyusin_core::{
    ActionSpec, Cartridge, GameError, GameRng, Param, ParamKind, PlayerId, RegistryError, Seed,
    TurnGame,
};
use serde::{Deserialize, Serialize};

use crate::domino::{Boneyard, Tile};
use crate::meja::{self, Umum};
use crate::pot::{Pot, award, pots};

pub const ID: &str = "domino-qiuqiu";
pub const ANTE: i64 = 10;
pub const MIN_BET: i64 = 20;
pub const DEFAULT_SEATS: u8 = 6;
pub const MAX_SEATS: u8 = 6;
pub const DEFAULT_STACK: i64 = 2000;

pub fn catalog() -> &'static Catalog {
    static CATALOG: OnceLock<Catalog> = OnceLock::new();
    CATALOG.get_or_init(|| {
        Catalog::from_toml(include_str!("i18n.toml")).expect("domino_qiuqiu/i18n.toml tidak sah")
    })
}

pub fn cartridge() -> Result<Cartridge, RegistryError> {
    Cartridge::new(
        include_str!("manifest.toml"),
        include_str!("../../../../tutorials/domino-qiuqiu.toml"),
        create_session::<QiuQiu>,
    )
}

/// Jenis tangan, dari terendah.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum QqClass {
    Pasangan,
    QiuQiu,
    MurniBesar,
    MurniKecil,
    Balak,
    EnamDewa,
}

impl QqClass {
    pub fn key(self) -> &'static str {
        match self {
            QqClass::Pasangan => "pasangan",
            QqClass::QiuQiu => "qiuqiu",
            QqClass::MurniBesar => "murni_besar",
            QqClass::MurniKecil => "murni_kecil",
            QqClass::Balak => "balak",
            QqClass::EnamDewa => "enam_dewa",
        }
    }
}

/// Urutan satu kartu untuk pemecah seri: balak, jumlah bulatan, angka
/// terbesar.
pub fn tile_rank(t: Tile) -> u8 {
    u8::from(t.is_double()) * 100 + t.pips() * 10 + t.hi
}

/// Nilai satu pasangan.
pub fn pair_value(a: Tile, b: Tile) -> u8 {
    (a.pips() + b.pips()) % 10
}

/// Nilai empat kartu; lebih besar = lebih kuat. Pembagian pasangan hanya
/// untuk tampilan dan tidak ikut dibandingkan.
#[derive(Debug, Clone, Copy)]
pub struct QqValue {
    class: QqClass,
    /// Penentu di dalam jenis: pasangan (tinggi·10 + rendah), Murni Kecil
    /// (−total), Murni Besar (total), lainnya 0.
    primary: i16,
    /// Urutan kartu tunggal, dari tertinggi.
    ranks: [u8; 4],
    pairs: (u8, u8),
    split: [[Tile; 2]; 2],
}

impl QqValue {
    pub fn class(&self) -> QqClass {
        self.class
    }

    pub fn key(&self) -> &'static str {
        self.class.key()
    }

    /// Nilai pasangan tertinggi dan kedua dari pembagian terbaik.
    pub fn pairs(&self) -> (u8, u8) {
        self.pairs
    }

    /// Pembagian terbaik: pasangan tertinggi dulu, kartu tertinggi dulu.
    pub fn split(&self) -> [[Tile; 2]; 2] {
        self.split
    }

    fn order(&self) -> (QqClass, i16, [u8; 4]) {
        (self.class, self.primary, self.ranks)
    }
}

impl PartialEq for QqValue {
    fn eq(&self, other: &Self) -> bool {
        self.order() == other.order()
    }
}

impl Eq for QqValue {}

impl PartialOrd for QqValue {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for QqValue {
    fn cmp(&self, other: &Self) -> Ordering {
        self.order().cmp(&other.order())
    }
}

fn by_rank(mut pair: [Tile; 2]) -> [Tile; 2] {
    if tile_rank(pair[1]) > tile_rank(pair[0]) {
        pair.swap(0, 1);
    }
    pair
}

/// Pembagian terbaik empat kartu menjadi dua pasangan.
fn best_split(t: &[Tile; 4]) -> ((u8, u8), [[Tile; 2]; 2]) {
    let top = (0..4)
        .max_by_key(|&i| tile_rank(t[i]))
        .expect("empat kartu");
    let mut best: Option<((u8, u8), bool, [[Tile; 2]; 2])> = None;
    for partner in 1..4 {
        let rest: Vec<usize> = (1..4).filter(|&i| i != partner).collect();
        let a = [t[0], t[partner]];
        let b = [t[rest[0]], t[rest[1]]];
        let (va, vb) = (pair_value(a[0], a[1]), pair_value(b[0], b[1]));
        let a_has_top = top == 0 || top == partner;
        // Pasangan bernilai lebih tinggi di depan; seri: yang berisi kartu
        // tertinggi di depan.
        let (ordered, values, top_first) = if va > vb || (va == vb && a_has_top) {
            ([a, b], (va, vb), a_has_top)
        } else {
            ([b, a], (vb, va), !a_has_top)
        };
        let cand = (values, top_first, ordered.map(by_rank));
        if best.as_ref().is_none_or(|b| (cand.0, cand.1) > (b.0, b.1)) {
            best = Some(cand);
        }
    }
    let (pairs, _, split) = best.expect("tiga pembagian");
    (pairs, split)
}

/// Nilai empat kartu QiuQiu.
pub fn eval(tiles: &[Tile]) -> QqValue {
    let t: [Tile; 4] = tiles.try_into().expect("eval butuh empat kartu");
    let mut ranks = t.map(tile_rank);
    ranks.sort_unstable_by(|a, b| b.cmp(a));
    let total: i16 = t.iter().map(|x| i16::from(x.pips())).sum();
    let (pairs, split) = best_split(&t);
    let (class, primary) = if t.iter().all(|x| x.pips() == 6) {
        (QqClass::EnamDewa, 0)
    } else if t.iter().all(|x| x.is_double()) {
        (QqClass::Balak, 0)
    } else if total <= 9 {
        (QqClass::MurniKecil, -total)
    } else if total >= 39 {
        (QqClass::MurniBesar, total)
    } else if pairs == (9, 9) {
        (QqClass::QiuQiu, 0)
    } else {
        (
            QqClass::Pasangan,
            i16::from(pairs.0) * 10 + i16::from(pairs.1),
        )
    };
    QqValue {
        class,
        primary,
        ranks,
        pairs,
        split,
    }
}

/// Konfigurasi meja. Semua opsional; host mengisi `tumpukan` (buy-in
/// manusia dari saldo, bot 2.000) dan `manusia`.
#[derive(Debug, Clone, Default, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Config {
    /// Jumlah kursi (2–6; bawaan 6).
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
    /// Urutan kartu tetap untuk tangan-tangan pertama (tes dan tutorial).
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
    bet: i64,
    committed: i64,
    tiles: Vec<Tile>,
    in_hand: bool,
    folded: bool,
    allin: bool,
    acted: bool,
    may_raise: bool,
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
            tiles: Vec::new(),
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
pub struct QiuQiu {
    seed: Seed,
    seats: Vec<Seat>,
    humans: Vec<u8>,
    max_hands: Option<u32>,
    fixed: Vec<Vec<Tile>>,
    boneyard: Boneyard,
    phase: Phase,
    hand: u32,
    /// Putaran taruhan berjalan (1: tiga kartu, 2: empat kartu).
    round: u8,
    dealer: u8,
    to_act: Option<u8>,
    current_bet: i64,
    last_raise: i64,
    log: Vec<(u8, String)>,
    last_pots: Vec<Pot>,
    ended: Option<&'static str>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SeatView {
    pub tumpukan: i64,
    pub awal: i64,
    pub taruhan: i64,
    pub taruhan_tangan: i64,
    /// `aktif`, `fold`, `allin`, `duduk`, `habis`, atau `berdiri`.
    pub status: String,
    /// Kartu sendiri, kartu yang dibuka saat buka kartu (per pasangan), atau
    /// `??`.
    pub kartu: Vec<String>,
    /// Jenis tangan yang dibuka (`pasangan`, `qiuqiu`, kartu spesial).
    pub tangan: Option<String>,
    /// Nilai dua pasangan yang dibuka.
    pub nilai: Option<[u8; 2]>,
    pub siap: bool,
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
    pub dealer: u8,
    pub giliran: Option<u8>,
    pub tangan_ke: u32,
    pub ante: i64,
    pub putaran: u8,
    pub panggil: i64,
    pub naik_min: Option<i64>,
    pub naik_maks: Option<i64>,
    pub log: Vec<(u8, String)>,
    pub pot_limit: bool,
}

impl meja::MejaView for View {
    fn meja(&self) -> &Umum {
        &self.meja
    }
}

impl QiuQiu {
    fn n(&self) -> u8 {
        self.seats.len() as u8
    }

    fn next_seat(&self, from: u8, ok: impl Fn(&Seat) -> bool) -> Option<u8> {
        let n = self.n();
        (1..=n)
            .map(|i| (from + i) % n)
            .find(|&s| ok(&self.seats[s as usize]))
    }

    fn pot_total(&self) -> i64 {
        self.seats.iter().map(|s| s.committed).sum()
    }

    fn hand_boneyard(&self) -> Boneyard {
        match self.fixed.get((self.hand - 1) as usize) {
            Some(tiles) => Boneyard::from_tiles(tiles.clone()),
            None => Boneyard::shuffled(&mut GameRng::from_seed(derive(
                &self.seed,
                &format!("tangan:{}", self.hand),
            ))),
        }
    }

    /// Satu kartu untuk setiap kursi yang memenuhi `ok`, mulai setelah
    /// dealer.
    fn deal_round(&mut self, ok: fn(&Seat) -> bool) -> Result<(), GameError> {
        let mut s = self.dealer;
        let count = self.seats.iter().filter(|x| ok(x)).count();
        for _ in 0..count {
            s = self.next_seat(s, ok).expect("pemain");
            let t = self
                .boneyard
                .draw()
                .ok_or_else(|| GameError::Config("kartu domino habis".into()))?;
            self.seats[s as usize].tiles.push(t);
        }
        Ok(())
    }

    fn start_hand(&mut self) -> Result<(), GameError> {
        let players = self
            .seats
            .iter()
            .filter(|s| s.seated() && s.stack > 0)
            .count();
        if players < 2 {
            self.finish_session("meja_bubar");
            return Ok(());
        }
        self.hand += 1;
        if self.hand > 1 || !self.seats[self.dealer as usize].seated() {
            self.dealer = self
                .next_seat(self.dealer, |s| s.seated() && s.stack > 0)
                .expect("ada pemain");
        }
        for s in &mut self.seats {
            s.bet = 0;
            s.committed = 0;
            s.tiles.clear();
            s.in_hand = !s.out && !s.left && s.stack > 0;
            s.folded = false;
            s.allin = false;
            s.acted = false;
            s.may_raise = true;
            s.ready = false;
            s.shown = false;
            s.won = 0;
        }
        for s in self.seats.iter_mut().filter(|s| s.in_hand) {
            s.put(ANTE);
            s.bet = 0;
        }
        self.log.clear();
        self.last_pots.clear();
        self.boneyard = self.hand_boneyard();
        for _ in 0..3 {
            self.deal_round(|s| s.in_hand)?;
        }
        self.phase = Phase::Play;
        self.start_round(1)
    }

    fn start_round(&mut self, round: u8) -> Result<(), GameError> {
        self.round = round;
        for s in &mut self.seats {
            s.bet = 0;
            s.acted = false;
            s.may_raise = true;
        }
        self.current_bet = 0;
        self.last_raise = MIN_BET;
        self.to_act = self.next_to_act(self.dealer);
        self.progress()
    }

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

    fn progress(&mut self) -> Result<(), GameError> {
        let live: Vec<usize> = (0..self.seats.len())
            .filter(|&s| self.seats[s].live())
            .collect();
        if live.len() == 1 {
            let total = self.pot_total();
            self.seats[live[0]].stack += total;
            self.seats[live[0]].won = total;
            self.end_hand();
            return Ok(());
        }
        let actors = self.seats.iter().filter(|s| s.can_act()).count();
        // Satu pemain tersisa yang bisa bertindak dan sudah menyamai taruhan
        // tertinggi: tidak ada lagi yang perlu diputuskan.
        let settled = actors <= 1
            && self
                .seats
                .iter()
                .filter(|s| s.can_act())
                .all(|s| s.bet >= self.current_bet);
        if !self.round_complete() && !settled {
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
        if self.round == 1 {
            self.deal_round(Seat::live)?;
            return self.start_round(2);
        }
        self.showdown();
        Ok(())
    }

    fn showdown(&mut self) {
        let values: Vec<Option<QqValue>> = self
            .seats
            .iter()
            .map(|s| s.live().then(|| eval(&s.tiles)))
            .collect();
        let contrib: Vec<i64> = self.seats.iter().map(|s| s.committed).collect();
        let folded: Vec<bool> = self.seats.iter().map(|s| !s.live()).collect();
        let pots = pots(&contrib, &folded);
        let won = award(&pots, |s| values[s as usize], self.n(), self.dealer);
        for ((s, w), v) in self.seats.iter_mut().zip(won).zip(&values) {
            s.stack += w;
            s.won = w;
            if let Some(v) = v {
                s.shown = true;
                s.tiles = v.split().iter().flatten().copied().collect();
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

    /// Batas bet/raise (total di putaran ini) untuk `seat`, bila boleh.
    fn raise_range(&self, seat: u8) -> Option<(i64, i64)> {
        let s = &self.seats[seat as usize];
        let allin = s.bet + s.stack;
        if !s.may_raise || allin <= self.current_bet {
            return None;
        }
        let full = if self.current_bet == 0 {
            MIN_BET
        } else {
            self.current_bet + self.last_raise
        };
        let call = self.current_bet - s.bet;
        let max = allin.min(self.current_bet + self.pot_total() + call);
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
        let full = increment >= self.last_raise || self.current_bet == 0 && total >= MIN_BET;
        let s = &mut self.seats[seat as usize];
        let add = total - s.bet;
        s.put(add);
        if full {
            self.last_raise = increment.max(MIN_BET);
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

impl TurnGame for QiuQiu {
    type Config = Config;
    type Action = Action;
    type View = View;

    fn new(config: Config, seed: Seed) -> Result<Self, GameError> {
        let n = config.kursi.unwrap_or(DEFAULT_SEATS);
        if !(2..=MAX_SEATS).contains(&n) {
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
            .map(|decks| {
                decks
                    .iter()
                    .map(|d| {
                        d.iter()
                            .map(|t| {
                                t.parse::<Tile>()
                                    .map_err(|_| GameError::Config(format!("kartu domino `{t}`")))
                            })
                            .collect::<Result<Vec<_>, _>>()
                    })
                    .collect::<Result<Vec<_>, _>>()
            })
            .transpose()?
            .unwrap_or_default();
        let dealer = match config.dealer {
            Some(d) if d < n => d,
            Some(d) => return Err(GameError::Config(format!("dealer {d}"))),
            None => GameRng::from_seed(derive(&seed, "dealer")).below(u32::from(n)) as u8,
        };
        let mut table = QiuQiu {
            seed,
            seats: stacks.into_iter().map(Seat::new).collect(),
            humans,
            max_hands: config.tangan_maks,
            fixed,
            boneyard: Boneyard::from_tiles(Vec::new()),
            phase: Phase::Between,
            hand: 0,
            round: 1,
            dealer,
            to_act: None,
            current_bet: 0,
            last_raise: MIN_BET,
            log: Vec::new(),
            last_pots: Vec::new(),
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
                min: MIN_BET,
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
                .map(|(i, s)| {
                    let value = (s.shown && s.tiles.len() == 4).then(|| eval(&s.tiles));
                    SeatView {
                        tumpukan: s.stack,
                        awal: s.start,
                        taruhan: s.bet,
                        taruhan_tangan: s.committed,
                        status: self.seat_status(s).into(),
                        kartu: if !s.in_hand || (s.folded && i as u8 != player) {
                            Vec::new()
                        } else if i as u8 == player || s.shown {
                            s.tiles.iter().map(Tile::to_string).collect()
                        } else {
                            vec!["??".into(); s.tiles.len()]
                        },
                        tangan: value.map(|v| v.key().to_string()),
                        nilai: value
                            .filter(|v| v.class() <= QqClass::QiuQiu)
                            .map(|v| [v.pairs().0, v.pairs().1]),
                        siap: s.ready,
                        menang: s.won,
                    }
                })
                .collect(),
            pot: self.pot_total(),
            pots: self.last_pots.clone(),
            dealer: self.dealer,
            giliran: self.to_act,
            tangan_ke: self.hand,
            ante: ANTE,
            putaran: self.round,
            panggil: if my_turn { self.to_call(player) } else { 0 },
            naik_min: range.map(|r| r.0),
            naik_maks: range.map(|r| r.1),
            log: self.log.clone(),
            pot_limit: true,
        }
    }

    fn to_text(view: &View, lang: Lang) -> String {
        let c = catalog();
        let mut out = c.text(
            lang,
            "hand",
            &[
                ("n", &view.tangan_ke.to_string()),
                ("round", &view.putaran.to_string()),
                ("pot", &view.pot.to_string()),
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
            let hand = match (&s.tangan, s.nilai) {
                (Some(_), Some([a, b])) => format!(" {a}-{b}"),
                (Some(k), None) => format!(" {}", c.text(lang, &format!("class.{k}"), &[])),
                _ => String::new(),
            };
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
