//! Pai Gow Poker (SPEC §6.3; M5a): tujuh kartu disusun menjadi tangan
//! belakang lima kartu dan tangan depan dua kartu.
//!
//! Aturan (D-060): dek 53 kartu (52 + joker) dikocok per ronde. Joker =
//! "bug": di lima kartu hanya as atau pelengkap straight/flush/straight
//! flush, di dua kartu as. Lima as tertinggi; A-2-3-4-5 straight tertinggi
//! kedua. Bandar menyusun dengan house way gaya Las Vegas ([`house_way`]).
//! Seri (copy) dimenangkan bandar; kedua tangan menang dibayar 0,95:1.
//! Taruhan kelipatan 20 supaya komisi utuh. Satu sesi = satu ronde.

use std::fmt;
use std::str::FromStr;
use std::sync::OnceLock;

use kyusin_core::game::{GameResult, create_session};
use kyusin_core::i18n::{Catalog, Lang};
use kyusin_core::{
    ActionSpec, Cartridge, GameError, GameRng, Param, ParamKind, PlayerId, RegistryError, Seed,
    TurnGame,
};
use serde::{Deserialize, Serialize};

use crate::cards::{Card, Rank, Suit};
use crate::meja::{self, Limits, MejaView, Umum, WagerRtp};
use crate::poker::{Category, Value, eval5, rank_value};

pub const ID: &str = "pai-gow";
pub const LIMITS: Limits = Limits {
    min: 20,
    max: 2000,
    step: 20,
};

/// RTP dengan house way untuk pemain dan bandar (D-060), dari simulasi
/// sangat besar di workflow `rtp.yml`.
pub const RTP: &[WagerRtp] = &[WagerRtp {
    wager: "bet",
    percent: 97.2700,
    manifest: true,
}];

pub fn catalog() -> &'static Catalog {
    static CATALOG: OnceLock<Catalog> = OnceLock::new();
    CATALOG.get_or_init(|| {
        Catalog::from_toml(include_str!("i18n.toml")).expect("pai_gow/i18n.toml tidak sah")
    })
}

pub fn cartridge() -> Result<Cartridge, RegistryError> {
    Cartridge::new(
        include_str!("manifest.toml"),
        include_str!("../../../../tutorials/pai-gow.toml"),
        create_session::<PaiGow>,
    )
}

/// Kartu Pai Gow: kartu biasa atau joker (`JK`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Tile {
    Card(Card),
    Joker,
}

impl Tile {
    /// Peringkat untuk mengelompokkan: joker dihitung as.
    fn rank(self) -> u8 {
        match self {
            Tile::Card(c) => rank_value(c.rank),
            Tile::Joker => 14,
        }
    }
}

impl fmt::Display for Tile {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Tile::Card(c) => c.fmt(f),
            Tile::Joker => f.write_str("JK"),
        }
    }
}

impl FromStr for Tile {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s == "JK" {
            return Ok(Tile::Joker);
        }
        s.parse::<Card>().map(Tile::Card).map_err(|_| s.to_string())
    }
}

impl Serialize for Tile {
    fn serialize<S: serde::Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
        s.collect_str(self)
    }
}

/// Nilai tangan belakang (lima kartu); lebih besar = lebih kuat.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct BackValue(u64);

const FIVE_ACES: u64 = 9;

impl BackValue {
    fn from(v: Value) -> BackValue {
        let cat = v.category() as u64;
        let mut out = cat << 40;
        if matches!(v.category(), Category::Straight | Category::StraightFlush) {
            // Wheel di antara straight K-high (26) dan A-high (28).
            let high = v.rank(0);
            let code = if high == 5 { 27 } else { u64::from(high) * 2 };
            out |= code << 32;
        } else {
            for i in 0..5 {
                out |= u64::from(v.rank(i)) << (32 - 8 * i);
            }
        }
        BackValue(out)
    }

    pub fn category(self) -> u64 {
        self.0 >> 40
    }

    fn rank(self, i: usize) -> u8 {
        ((self.0 >> (32 - 8 * i)) & 0xFF) as u8
    }

    pub fn key(self) -> &'static str {
        match self.category() {
            FIVE_ACES => "five_aces",
            8 if self.rank(0) == 28 => "royal_flush",
            c => [
                "high_card",
                "pair",
                "two_pair",
                "trips",
                "straight",
                "flush",
                "full_house",
                "quads",
                "straight_flush",
            ][c as usize],
        }
    }
}

/// Nilai tangan belakang lima kartu (dengan aturan joker dan wheel).
pub fn eval_back(tiles: &[Tile]) -> BackValue {
    assert_eq!(tiles.len(), 5, "tangan belakang lima kartu");
    let cards: Vec<Card> = tiles
        .iter()
        .filter_map(|t| match t {
            Tile::Card(c) => Some(*c),
            Tile::Joker => None,
        })
        .collect();
    if cards.len() == 5 {
        return BackValue::from(eval5(&cards));
    }
    if cards.iter().all(|c| c.rank == Rank::Ace) {
        return BackValue(FIVE_ACES << 40);
    }
    let mut best = BackValue(0);
    let mut hand = cards.clone();
    hand.push(cards[0]);
    for suit in Suit::ALL {
        hand[4] = Card::new(Rank::Ace, suit);
        best = best.max(BackValue::from(eval5(&hand)));
    }
    for c in Card::deck() {
        hand[4] = c;
        let v = eval5(&hand);
        if matches!(
            v.category(),
            Category::Straight | Category::Flush | Category::StraightFlush
        ) {
            best = best.max(BackValue::from(v));
        }
    }
    best
}

/// Nilai tangan depan (dua kartu): pair di atas kartu tinggi; joker = as.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct FrontValue(u32);

impl FrontValue {
    pub fn is_pair(self) -> bool {
        self.0 >> 16 == 1
    }

    fn ranks(self) -> (u8, u8) {
        (((self.0 >> 8) & 0xFF) as u8, (self.0 & 0xFF) as u8)
    }
}

pub fn eval_front(tiles: &[Tile]) -> FrontValue {
    assert_eq!(tiles.len(), 2, "tangan depan dua kartu");
    let (a, b) = (tiles[0].rank(), tiles[1].rank());
    let (hi, lo) = (a.max(b), a.min(b));
    FrontValue((u32::from(hi == lo) << 16) | (u32::from(hi) << 8) | u32::from(lo))
}

/// Tangan belakang tidak boleh lebih rendah dari tangan depan.
pub fn valid(back: &[Tile], front: &[Tile]) -> bool {
    let (b, f) = (eval_back(back), eval_front(front));
    let (fh, fl) = f.ranks();
    match b.category() {
        0 => !f.is_pair() && (b.rank(0), b.rank(1)) >= (fh, fl),
        1 => !f.is_pair() || b.rank(0) >= fh,
        _ => true,
    }
}

fn sorted(tiles: &[Tile]) -> Vec<Tile> {
    let mut v = tiles.to_vec();
    v.sort_by_key(|t| std::cmp::Reverse(t.rank()));
    v
}

/// Membagi tujuh kartu: `front` diambil dari `seven`, sisanya belakang.
fn split(seven: &[Tile], front: &[Tile]) -> (Vec<Tile>, Vec<Tile>) {
    let mut rest = seven.to_vec();
    for t in front {
        let i = rest.iter().position(|x| x == t).expect("kartu ada");
        rest.remove(i);
    }
    (sorted(front), sorted(&rest))
}

/// Susunan terbaik yang tangan belakangnya straight atau lebih tinggi,
/// dengan tangan depan setinggi mungkin.
fn best_straight_or_flush(seven: &[Tile]) -> Option<(Vec<Tile>, Vec<Tile>)> {
    let mut best: Option<(FrontValue, Vec<Tile>, Vec<Tile>)> = None;
    for i in 0..7 {
        for j in i + 1..7 {
            let front = [seven[i], seven[j]];
            let (f, b) = split(seven, &front);
            let bv = eval_back(&b);
            if bv.category() < 4 || !valid(&b, &f) {
                continue;
            }
            let fv = eval_front(&f);
            if best.as_ref().is_none_or(|(x, _, _)| fv > *x) {
                best = Some((fv, f, b));
            }
        }
    }
    best.map(|(_, f, b)| (f, b))
}

/// House way gaya Las Vegas (D-060), dipakai bandar dan perintah
/// `houseway`. Mengembalikan (depan, belakang), masing-masing terurut dari
/// peringkat tertinggi.
pub fn house_way(seven: &[Tile]) -> (Vec<Tile>, Vec<Tile>) {
    assert_eq!(seven.len(), 7, "tujuh kartu");
    let s = sorted(seven);
    let mut count = [0u8; 15];
    for t in &s {
        count[t.rank() as usize] += 1;
    }
    let of = |n: u8| -> Vec<u8> {
        (2..=14u8)
            .rev()
            .filter(|r| count[*r as usize] == n)
            .collect()
    };
    let (quads, trips, pairs, singles) = (of(4), of(3), of(2), of(1));
    let tiles_of = |r: u8, n: usize| -> Vec<Tile> {
        s.iter()
            .filter(|t| t.rank() == r)
            .take(n)
            .copied()
            .collect()
    };
    let single = |skip: &[u8], n: usize| -> Vec<Tile> {
        s.iter()
            .filter(|t| count[t.rank() as usize] == 1 && !skip.contains(&t.rank()))
            .take(n)
            .copied()
            .collect()
    };
    let pick = |front: Vec<Tile>| split(seven, &front);

    let result = if count[14] == 5 {
        // Lima as: pair lain di depan bila ada, selain itu pair as.
        match pairs.first() {
            Some(&p) => pick(tiles_of(p, 2)),
            None => pick(tiles_of(14, 2)),
        }
    } else if let Some(&q) = quads.first() {
        if let Some(&p) = trips.first().or(pairs.first()) {
            pick(tiles_of(p, 2))
        } else if q <= 6 || (q <= 10 && singles.contains(&14)) {
            pick(single(&[], 2))
        } else {
            pick(tiles_of(q, 2))
        }
    } else if trips.len() == 2 {
        pick(tiles_of(trips[0], 2))
    } else if let (Some(_), Some(&p)) = (trips.first(), pairs.first()) {
        pick(tiles_of(p, 2))
    } else if let Some(&t) = trips.first() {
        if t == 14 {
            let mut front = tiles_of(14, 1);
            front.extend(single(&[], 1));
            pick(front)
        } else {
            pick(single(&[], 2))
        }
    } else if pairs.len() == 3 {
        pick(tiles_of(pairs[0], 2))
    } else if pairs.len() == 2 {
        let (high, low) = (pairs[0], pairs[1]);
        if high <= 10 && singles.contains(&14) {
            pick(single(&[], 2))
        } else {
            pick(tiles_of(low, 2))
        }
    } else if pairs.len() == 1 {
        match best_straight_or_flush(seven) {
            Some((f, b)) if eval_front(&f).is_pair() || eval_front(&f).ranks().0 == 14 => (f, b),
            _ => pick(single(&[], 2)),
        }
    } else {
        match best_straight_or_flush(seven) {
            Some(fb) => fb,
            None => pick(s[1..3].to_vec()),
        }
    };
    if valid(&result.1, &result.0) {
        return result;
    }
    // Cadangan (seharusnya tidak terjadi): susunan sah dengan depan tertinggi.
    type Setting = (Vec<Tile>, Vec<Tile>);
    let mut best: Option<(FrontValue, Setting)> = None;
    for i in 0..7 {
        for j in i + 1..7 {
            let (f, b) = split(seven, &[seven[i], seven[j]]);
            if valid(&b, &f) {
                let fv = eval_front(&f);
                if best.as_ref().is_none_or(|(x, _)| fv > *x) {
                    best = Some((fv, (f, b)));
                }
            }
        }
    }
    best.expect("selalu ada susunan sah").1
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Action {
    Bet(i64),
    Set(Tile, Tile),
    HouseWay,
    Leave,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
enum Phase {
    Betting,
    Set,
    Over,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
struct Round {
    player: Vec<Tile>,
    dealer: Vec<Tile>,
    bet: i64,
    set: Option<(Vec<Tile>, Vec<Tile>)>,
    dealer_set: Option<(Vec<Tile>, Vec<Tile>)>,
    front_win: Option<bool>,
    back_win: Option<bool>,
    payout: Option<i64>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct PaiGow {
    deck: Vec<Tile>,
    next: usize,
    phase: Phase,
    round: Option<Round>,
    ended: Option<&'static str>,
}

/// Konfigurasi; `kartu` = urutan dek tetap untuk tes dan tutorial (`JK`
/// untuk joker).
#[derive(Debug, Clone, Default, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Config {
    #[serde(default)]
    pub kartu: Option<Vec<String>>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct View {
    #[serde(flatten)]
    pub meja: Umum,
    /// Tujuh kartu pemain, terurut dari tertinggi.
    pub pemain: Vec<String>,
    /// Tujuh kartu bandar; `??` sampai ronde selesai.
    pub bandar: Vec<String>,
    pub depan: Vec<String>,
    pub belakang: Vec<String>,
    pub bandar_depan: Vec<String>,
    pub bandar_belakang: Vec<String>,
    pub tangan_depan: Option<String>,
    pub tangan_belakang: Option<String>,
    pub tangan_bandar_belakang: Option<String>,
    /// `menang` atau `kalah` (seri = kalah) setelah ronde selesai.
    pub hasil_depan: Option<String>,
    pub hasil_belakang: Option<String>,
    pub taruhan: i64,
    pub bayar: Option<i64>,
}

impl MejaView for View {
    fn meja(&self) -> &Umum {
        &self.meja
    }
}

fn front_name(f: &[Tile]) -> String {
    if eval_front(f).is_pair() {
        "pair".into()
    } else {
        "high_card".into()
    }
}

impl PaiGow {
    fn draw(&mut self) -> Result<Tile, GameError> {
        let t = self
            .deck
            .get(self.next)
            .copied()
            .ok_or_else(|| GameError::Illegal("kartu habis".into()))?;
        self.next += 1;
        Ok(t)
    }

    fn deal(&mut self, bet: i64) -> Result<(), GameError> {
        let player = (0..7).map(|_| self.draw()).collect::<Result<Vec<_>, _>>()?;
        let dealer = (0..7).map(|_| self.draw()).collect::<Result<Vec<_>, _>>()?;
        self.round = Some(Round {
            player: sorted(&player),
            dealer: sorted(&dealer),
            bet,
            set: None,
            dealer_set: None,
            front_win: None,
            back_win: None,
            payout: None,
        });
        self.phase = Phase::Set;
        Ok(())
    }

    fn settle(&mut self, front: Vec<Tile>, back: Vec<Tile>) {
        let r = self.round.as_mut().expect("ronde berjalan");
        let (df, db) = house_way(&r.dealer);
        let fw = eval_front(&front) > eval_front(&df);
        let bw = eval_back(&back) > eval_back(&db);
        r.payout = Some(match (fw, bw) {
            (true, true) => r.bet * 19 / 20,
            (false, false) => -r.bet,
            _ => 0,
        });
        r.front_win = Some(fw);
        r.back_win = Some(bw);
        r.set = Some((front, back));
        r.dealer_set = Some((df, db));
        self.phase = Phase::Over;
        self.ended = Some("ronde_selesai");
    }
}

impl TurnGame for PaiGow {
    type Config = Config;
    type Action = Action;
    type View = View;

    fn new(config: Config, seed: Seed) -> Result<Self, GameError> {
        let deck = match &config.kartu {
            Some(list) => list
                .iter()
                .map(|c| {
                    c.parse::<Tile>()
                        .map_err(|_| GameError::Config(format!("kartu `{c}`")))
                })
                .collect::<Result<Vec<_>, _>>()?,
            None => {
                let mut d: Vec<Tile> = Card::deck().into_iter().map(Tile::Card).collect();
                d.push(Tile::Joker);
                GameRng::from_seed(seed).shuffle(&mut d);
                d
            }
        };
        Ok(PaiGow {
            deck,
            next: 0,
            phase: Phase::Betting,
            round: None,
            ended: None,
        })
    }

    fn seats(&self) -> u8 {
        1
    }

    fn pending_players(&self) -> Vec<PlayerId> {
        if self.phase == Phase::Over {
            Vec::new()
        } else {
            vec![0]
        }
    }

    fn legal_actions(&self, player: PlayerId) -> Vec<ActionSpec> {
        if player != 0 {
            return Vec::new();
        }
        match self.phase {
            Phase::Betting => vec![LIMITS.single(), ActionSpec::fixed("leave")],
            Phase::Set => {
                let options: Vec<String> = self
                    .round
                    .as_ref()
                    .map(|r| r.player.iter().map(Tile::to_string).collect())
                    .unwrap_or_default();
                let card = || Param {
                    name: "kartu".into(),
                    kind: ParamKind::Choice {
                        options: options.clone(),
                    },
                };
                vec![
                    ActionSpec::template("set", vec![card(), card()]),
                    ActionSpec::fixed("houseway"),
                ]
            }
            Phase::Over => Vec::new(),
        }
    }

    fn apply(&mut self, player: PlayerId, action: Action) -> Result<(), GameError> {
        if self.phase == Phase::Over {
            return Err(GameError::Over);
        }
        if player != 0 {
            return Err(GameError::NotPending(player));
        }
        let illegal = GameError::Illegal(self.format_action(&action));
        match (self.phase, action) {
            (Phase::Betting, Action::Bet(n)) if LIMITS.accepts(n) => self.deal(n),
            (Phase::Betting, Action::Leave) => {
                self.phase = Phase::Over;
                self.ended = Some("berhenti");
                Ok(())
            }
            (Phase::Set, Action::Set(a, b)) => {
                let seven = self.round.as_ref().expect("ronde").player.clone();
                if a == b || !seven.contains(&a) || !seven.contains(&b) {
                    return Err(illegal);
                }
                let (front, back) = split(&seven, &[a, b]);
                if !valid(&back, &front) {
                    return Err(illegal);
                }
                self.settle(front, back);
                Ok(())
            }
            (Phase::Set, Action::HouseWay) => {
                let seven = self.round.as_ref().expect("ronde").player.clone();
                let (front, back) = house_way(&seven);
                self.settle(front, back);
                Ok(())
            }
            _ => Err(illegal),
        }
    }

    fn view_for(&self, _player: PlayerId) -> View {
        let fase = match self.phase {
            Phase::Betting => "taruhan",
            Phase::Set => "susun",
            Phase::Over => "selesai",
        };
        let mut meja = Umum::new(fase, LIMITS);
        let r = self.round.as_ref();
        let over = self.phase == Phase::Over;
        meja.bersih = r.and_then(|r| r.payout).unwrap_or(0);
        meja.ronde = u32::from(r.is_some());
        meja.taruhan_meja = if self.phase == Phase::Set {
            r.map(|r| r.bet).unwrap_or(0)
        } else {
            0
        };
        meja.dipertaruhkan = r.map(|r| r.bet).unwrap_or(0);
        if self.phase == Phase::Set {
            meja.netral = Some("houseway".into());
        }
        meja.selesai = over;
        meja.alasan = self.ended.map(str::to_string);
        let text = |v: &[Tile]| v.iter().map(Tile::to_string).collect::<Vec<_>>();
        let set = r.and_then(|r| r.set.as_ref());
        let dset = r.and_then(|r| r.dealer_set.as_ref());
        let outcome = |w: Option<bool>| w.map(|w| if w { "menang" } else { "kalah" }.to_string());
        View {
            meja,
            pemain: r.map(|r| text(&r.player)).unwrap_or_default(),
            bandar: r
                .map(|r| {
                    if over {
                        text(&r.dealer)
                    } else {
                        vec!["??".into(); r.dealer.len()]
                    }
                })
                .unwrap_or_default(),
            depan: set.map(|s| text(&s.0)).unwrap_or_default(),
            belakang: set.map(|s| text(&s.1)).unwrap_or_default(),
            bandar_depan: dset.map(|s| text(&s.0)).unwrap_or_default(),
            bandar_belakang: dset.map(|s| text(&s.1)).unwrap_or_default(),
            tangan_depan: set.map(|s| front_name(&s.0)),
            tangan_belakang: set.map(|s| eval_back(&s.1).key().to_string()),
            tangan_bandar_belakang: dset.map(|s| eval_back(&s.1).key().to_string()),
            hasil_depan: outcome(r.and_then(|r| r.front_win)),
            hasil_belakang: outcome(r.and_then(|r| r.back_win)),
            taruhan: r.map(|r| r.bet).unwrap_or(0),
            bayar: r.and_then(|r| r.payout),
        }
    }

    fn to_text(view: &View, lang: Lang) -> String {
        let c = catalog();
        let mut out = String::new();
        if !view.pemain.is_empty() {
            out.push_str(&c.text(lang, "cards", &[("cards", &view.pemain.join(" "))]));
            out.push('\n');
        }
        if !view.depan.is_empty() {
            out.push_str(&c.text(
                lang,
                "hands",
                &[
                    ("front", &view.depan.join(" ")),
                    ("back", &view.belakang.join(" ")),
                    ("dfront", &view.bandar_depan.join(" ")),
                    ("dback", &view.bandar_belakang.join(" ")),
                ],
            ));
            out.push('\n');
            if let Some(p) = view.bayar {
                out.push_str(&format!("  → {p:+}\n"));
            }
        }
        out.push_str(
            &match (view.meja.fase.as_str(), view.meja.alasan.as_deref()) {
                ("susun", _) => c.text(lang, "set", &[]),
                (_, Some(reason)) => c.text(
                    lang,
                    &format!("over.{reason}"),
                    &[("net", &format!("{:+}", view.meja.bersih))],
                ),
                _ => c.text(lang, "place_bet", &[]),
            },
        );
        out
    }

    fn is_over(&self) -> bool {
        self.phase == Phase::Over
    }

    fn result(&self) -> Option<GameResult> {
        (self.phase == Phase::Over).then(|| {
            let net = self.round.as_ref().and_then(|r| r.payout).unwrap_or(0);
            meja::result(catalog(), "summary", net, u32::from(self.round.is_some()))
        })
    }

    fn parse_command(&self, command: &str) -> Result<Action, GameError> {
        let c = command.trim();
        let parts: Vec<&str> = c.split(' ').collect();
        let err = || GameError::Parse(c.into());
        match parts.as_slice() {
            ["bet", n] => Ok(Action::Bet(meja::amount(n).ok_or_else(err)?)),
            ["set", a, b] => Ok(Action::Set(
                a.parse().map_err(|_| err())?,
                b.parse().map_err(|_| err())?,
            )),
            ["houseway"] => Ok(Action::HouseWay),
            ["leave"] => Ok(Action::Leave),
            _ => Err(err()),
        }
    }

    fn format_action(&self, action: &Action) -> String {
        match action {
            Action::Bet(n) => format!("bet {n}"),
            Action::Set(a, b) => format!("set {a} {b}"),
            Action::HouseWay => "houseway".into(),
            Action::Leave => "leave".into(),
        }
    }
}
