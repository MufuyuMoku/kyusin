//! Video Poker (SPEC §6.5; M6b, D-071): Jacks or Better 9/6, Deuces Wild
//! "Not So Ugly Ducks", dan Joker Poker Kings or Better 18/7/5 (53 kartu).
//!
//! Satu taruhan dibayar seperti 5 koin (Royal Flush selalu 800×). Satu
//! pertandingan = satu tangan: `bet <jumlah>` membagi lima kartu dari dek
//! yang dikocok dengan seed ronde, `draw [posisi…]` menahan kartu di posisi
//! itu (1–5) dan mengganti sisanya dengan kartu berikutnya di dek. RTP =
//! strategi optimal yang dihitung tepat ([`optimal::Solver`], enumerasi di
//! GitHub Actions).

pub mod optimal;

use std::fmt;
use std::marker::PhantomData;
use std::str::FromStr;
use std::sync::OnceLock;

use kyusin_core::game::{GameResult, create_session};
use kyusin_core::i18n::{Catalog, Lang};
use kyusin_core::{
    ActionSpec, Cartridge, GameError, GameRng, PlayerId, RegistryError, Seed, TurnGame,
};
use serde::{Deserialize, Serialize};

use crate::cards::Card;
use crate::meja::{self, MejaView, STANDARD, Umum, WagerRtp};

/// Kartu video poker: indeks 0–51 sama dengan [`Card::deck`] (sekop A–K,
/// hati, wajik, keriting), 52 = joker (`JK`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct VpCard(pub u8);

pub const JOKER: VpCard = VpCard(52);

impl VpCard {
    pub fn is_joker(self) -> bool {
        self.0 == 52
    }

    /// Peringkat 0 = A, 1 = 2, …, 12 = K.
    pub fn rank(self) -> u8 {
        self.0 % 13
    }

    pub fn suit(self) -> u8 {
        self.0 / 13
    }

    pub fn from_card(c: Card) -> VpCard {
        VpCard(c.suit as u8 * 13 + c.rank as u8)
    }
}

impl fmt::Display for VpCard {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.is_joker() {
            f.write_str("JK")
        } else {
            write!(f, "{}", Card::deck()[self.0 as usize])
        }
    }
}

impl Serialize for VpCard {
    fn serialize<S: serde::Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
        s.collect_str(self)
    }
}

impl FromStr for VpCard {
    type Err = GameError;

    fn from_str(s: &str) -> Result<Self, GameError> {
        if s == "JK" {
            return Ok(JOKER);
        }
        s.parse::<Card>()
            .map(VpCard::from_card)
            .map_err(|_| GameError::Config(format!("kartu `{s}`")))
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
pub enum Hand {
    RoyalFlush,
    FourDeuces,
    WildRoyal,
    FiveOfAKind,
    StraightFlush,
    FourOfAKind,
    FullHouse,
    Flush,
    Straight,
    ThreeOfAKind,
    TwoPair,
    HighPair,
}

impl Hand {
    pub fn key(self) -> &'static str {
        match self {
            Hand::RoyalFlush => "royal_flush",
            Hand::FourDeuces => "four_deuces",
            Hand::WildRoyal => "wild_royal",
            Hand::FiveOfAKind => "five_of_a_kind",
            Hand::StraightFlush => "straight_flush",
            Hand::FourOfAKind => "four_of_a_kind",
            Hand::FullHouse => "full_house",
            Hand::Flush => "flush",
            Hand::Straight => "straight",
            Hand::ThreeOfAKind => "three_of_a_kind",
            Hand::TwoPair => "two_pair",
            Hand::HighPair => "high_pair",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize)]
pub enum Variant {
    JacksOrBetter,
    DeucesWild,
    JokerPoker,
}

/// RTP strategi optimal, enumerasi tepat semua tangan awal di GitHub Actions
/// (`rtp_exact`; D-072): 1.653.526.326.983 / 1.661.102.543.100,
/// 552.196.412.883 / 553.700.847.700, 337.607.612.499 / 341.234.243.350.
pub const RTP_JACKS: &[WagerRtp] = &[WagerRtp {
    wager: "bet",
    percent: 99.5439,
    manifest: true,
}];
pub const RTP_DEUCES: &[WagerRtp] = &[WagerRtp {
    wager: "bet",
    percent: 99.7283,
    manifest: true,
}];
pub const RTP_JOKER: &[WagerRtp] = &[WagerRtp {
    wager: "bet",
    percent: 98.9372,
    manifest: true,
}];

impl Variant {
    pub const ALL: [Variant; 3] = [
        Variant::JacksOrBetter,
        Variant::DeucesWild,
        Variant::JokerPoker,
    ];

    pub fn id(self) -> &'static str {
        match self {
            Variant::JacksOrBetter => "jacks-or-better",
            Variant::DeucesWild => "deuces-wild",
            Variant::JokerPoker => "joker-poker",
        }
    }

    pub fn rtp(self) -> &'static [WagerRtp] {
        match self {
            Variant::JacksOrBetter => RTP_JACKS,
            Variant::DeucesWild => RTP_DEUCES,
            Variant::JokerPoker => RTP_JOKER,
        }
    }

    pub fn deck(self) -> Vec<VpCard> {
        let n = if self == Variant::JokerPoker { 53 } else { 52 };
        (0..n).map(VpCard).collect()
    }

    /// Tangan yang dibayar, urut dari bayaran tertinggi.
    pub fn hands(self) -> &'static [Hand] {
        use Hand::*;
        match self {
            Variant::JacksOrBetter => &[
                RoyalFlush,
                StraightFlush,
                FourOfAKind,
                FullHouse,
                Flush,
                Straight,
                ThreeOfAKind,
                TwoPair,
                HighPair,
            ],
            Variant::DeucesWild => &[
                RoyalFlush,
                FourDeuces,
                WildRoyal,
                FiveOfAKind,
                StraightFlush,
                FourOfAKind,
                FullHouse,
                Flush,
                Straight,
                ThreeOfAKind,
            ],
            Variant::JokerPoker => &[
                RoyalFlush,
                FiveOfAKind,
                WildRoyal,
                StraightFlush,
                FourOfAKind,
                FullHouse,
                Flush,
                Straight,
                ThreeOfAKind,
                TwoPair,
                HighPair,
            ],
        }
    }

    /// Pengali total per 1 taruhan (0 = tidak dibayar).
    pub fn pays(self, hand: Hand) -> u32 {
        use Hand::*;
        match (self, hand) {
            (_, RoyalFlush) => 800,
            (Variant::JacksOrBetter, StraightFlush) => 50,
            (Variant::JacksOrBetter, FourOfAKind) => 25,
            (Variant::JacksOrBetter, FullHouse) => 9,
            (Variant::JacksOrBetter, Flush) => 6,
            (Variant::JacksOrBetter, Straight) => 4,
            (Variant::JacksOrBetter, ThreeOfAKind) => 3,
            (Variant::JacksOrBetter, TwoPair) => 2,
            (Variant::JacksOrBetter, HighPair) => 1,
            (Variant::DeucesWild, FourDeuces) => 200,
            (Variant::DeucesWild, WildRoyal) => 25,
            (Variant::DeucesWild, FiveOfAKind) => 16,
            (Variant::DeucesWild, StraightFlush) => 10,
            (Variant::DeucesWild, FourOfAKind) => 4,
            (Variant::DeucesWild, FullHouse) => 4,
            (Variant::DeucesWild, Flush) => 3,
            (Variant::DeucesWild, Straight) => 2,
            (Variant::DeucesWild, ThreeOfAKind) => 1,
            (Variant::JokerPoker, FiveOfAKind) => 200,
            (Variant::JokerPoker, WildRoyal) => 100,
            (Variant::JokerPoker, StraightFlush) => 50,
            (Variant::JokerPoker, FourOfAKind) => 18,
            (Variant::JokerPoker, FullHouse) => 7,
            (Variant::JokerPoker, Flush) => 5,
            (Variant::JokerPoker, Straight) => 3,
            (Variant::JokerPoker, ThreeOfAKind) => 2,
            (Variant::JokerPoker, TwoPair) => 1,
            (Variant::JokerPoker, HighPair) => 1,
            _ => 0,
        }
    }

    fn is_wild(self, c: VpCard) -> bool {
        match self {
            Variant::JacksOrBetter => false,
            Variant::DeucesWild => !c.is_joker() && c.rank() == 1,
            Variant::JokerPoker => c.is_joker(),
        }
    }
}

/// Jendela lima peringkat (bit 0 = A): A-5, 2-6, …, 9-K, lalu T-A.
const ROYAL: u16 = 0b1_1110_0000_0001;

fn straight_windows() -> impl Iterator<Item = u16> {
    (0..9)
        .map(|s| 0b11111u16 << s)
        .chain(std::iter::once(ROYAL))
}

/// Tangan terbaik untuk lima kartu (`None` = tidak dibayar).
pub fn evaluate(variant: Variant, cards: &[VpCard; 5]) -> Option<Hand> {
    let wild = cards.iter().filter(|c| variant.is_wild(**c)).count();
    let others: Vec<VpCard> = cards
        .iter()
        .copied()
        .filter(|c| !variant.is_wild(*c))
        .collect();
    let mut counts = [0u8; 13];
    let mut ranks: u16 = 0;
    for c in &others {
        counts[c.rank() as usize] += 1;
        ranks |= 1 << c.rank();
    }
    let distinct = ranks.count_ones() as usize == others.len();
    let flush = others.windows(2).all(|w| w[0].suit() == w[1].suit());
    let straight = distinct && straight_windows().any(|w| ranks & !w == 0);
    let royal = distinct && ranks & !ROYAL == 0;
    let mut sorted = counts;
    sorted.sort_unstable_by(|a, b| b.cmp(a));
    let (top, second) = (sorted[0] as usize + wild, sorted[1] as usize);
    let pairs = counts.iter().filter(|n| **n == 2).count();
    let high_pair = |c: &[u8; 13]| match variant {
        Variant::JacksOrBetter => [0, 10, 11, 12].iter().any(|r| c[*r] >= 2),
        _ => [0, 12].iter().any(|r| c[*r] + wild as u8 >= 2),
    };
    variant.hands().iter().copied().find(|h| match h {
        Hand::RoyalFlush => wild == 0 && flush && royal,
        Hand::FourDeuces => wild == 4,
        Hand::WildRoyal => wild > 0 && flush && royal,
        Hand::FiveOfAKind => top >= 5,
        Hand::StraightFlush => flush && straight,
        Hand::FourOfAKind => top >= 4,
        Hand::FullHouse => {
            (wild == 0 && sorted[0] == 3 && second == 2) || (wild == 1 && pairs == 2)
        }
        Hand::Flush => flush,
        Hand::Straight => straight,
        Hand::ThreeOfAKind => top >= 3,
        Hand::TwoPair => wild == 0 && pairs == 2,
        Hand::HighPair => high_pair(&counts),
    })
}

/// Pengali total tangan lima kartu.
pub fn payout(variant: Variant, cards: &[VpCard; 5]) -> u32 {
    evaluate(variant, cards).map_or(0, |h| variant.pays(h))
}

pub fn catalog() -> &'static Catalog {
    static CATALOG: OnceLock<Catalog> = OnceLock::new();
    CATALOG.get_or_init(|| {
        Catalog::from_toml(include_str!("i18n.toml")).expect("video_poker/i18n.toml tidak sah")
    })
}

/// Penanda varian untuk tipe sesi.
pub trait Kind: Clone + fmt::Debug + PartialEq + Eq + Send + Sync + 'static {
    const VARIANT: Variant;
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Jacks;
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Deuces;
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Joker;

impl Kind for Jacks {
    const VARIANT: Variant = Variant::JacksOrBetter;
}
impl Kind for Deuces {
    const VARIANT: Variant = Variant::DeucesWild;
}
impl Kind for Joker {
    const VARIANT: Variant = Variant::JokerPoker;
}

pub type JacksOrBetterGame = VideoPoker<Jacks>;
pub type DeucesWildGame = VideoPoker<Deuces>;
pub type JokerPokerGame = VideoPoker<Joker>;

pub fn cartridges() -> Result<Vec<Cartridge>, RegistryError> {
    Ok(vec![
        Cartridge::new(
            include_str!("jacks_or_better.toml"),
            include_str!("../../../../tutorials/jacks-or-better.toml"),
            create_session::<JacksOrBetterGame>,
        )?,
        Cartridge::new(
            include_str!("deuces_wild.toml"),
            include_str!("../../../../tutorials/deuces-wild.toml"),
            create_session::<DeucesWildGame>,
        )?,
        Cartridge::new(
            include_str!("joker_poker.toml"),
            include_str!("../../../../tutorials/joker-poker.toml"),
            create_session::<JokerPokerGame>,
        )?,
    ])
}

#[derive(Debug, Clone, Default, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Config {
    /// Urutan dek tetap: lima pertama dibagi, sisanya pengganti berurutan
    /// (tes dan tutorial).
    #[serde(default)]
    pub kartu: Option<Vec<String>>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Action {
    Bet(i64),
    /// Posisi yang ditahan (1–5), urut naik.
    Draw(Vec<u8>),
    Leave,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
enum Phase {
    Betting,
    Hold,
    Over,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct VideoPoker<V: Kind> {
    deck: Vec<VpCard>,
    phase: Phase,
    hand: Vec<VpCard>,
    held: Vec<bool>,
    bet: i64,
    payout: Option<i64>,
    ended: Option<&'static str>,
    #[serde(skip)]
    _v: PhantomData<V>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct View {
    #[serde(flatten)]
    pub meja: Umum,
    /// Lima kartu di tangan (setelah dibagi, lalu setelah diganti).
    pub kartu: Vec<String>,
    /// Kartu yang ditahan, setelah `draw`.
    pub tahan: Vec<bool>,
    /// Nilai tangan sekarang (`high_pair`, `flush`, …; `null` = tidak dibayar).
    pub tangan: Option<String>,
    /// Tabel bayar varian ini: [tangan, pengali].
    pub tabel: Vec<(String, u32)>,
    pub taruhan: i64,
    pub bayar: Option<i64>,
}

impl MejaView for View {
    fn meja(&self) -> &Umum {
        &self.meja
    }
}

impl<V: Kind> VideoPoker<V> {
    pub const ID: &'static str = match V::VARIANT {
        Variant::JacksOrBetter => "jacks-or-better",
        Variant::DeucesWild => "deuces-wild",
        Variant::JokerPoker => "joker-poker",
    };

    fn five(&self) -> Option<[VpCard; 5]> {
        self.hand.clone().try_into().ok()
    }
}

impl<V: Kind> TurnGame for VideoPoker<V> {
    type Config = Config;
    type Action = Action;
    type View = View;

    fn new(config: Config, seed: Seed) -> Result<Self, GameError> {
        let variant = V::VARIANT;
        let deck = match config.kartu {
            Some(list) => {
                let cards = list
                    .iter()
                    .map(|s| s.parse::<VpCard>())
                    .collect::<Result<Vec<_>, _>>()?;
                let mut seen = cards.clone();
                seen.sort_unstable();
                seen.dedup();
                if cards.len() < 5
                    || seen.len() != cards.len()
                    || cards
                        .iter()
                        .any(|c| c.is_joker() && variant != Variant::JokerPoker)
                {
                    return Err(GameError::Config(
                        "kartu: paling sedikit 5, berbeda, sah untuk variannya".into(),
                    ));
                }
                cards
            }
            None => {
                let mut d = variant.deck();
                GameRng::from_seed(seed).shuffle(&mut d);
                d
            }
        };
        Ok(VideoPoker {
            deck,
            phase: Phase::Betting,
            hand: Vec::new(),
            held: Vec::new(),
            bet: 0,
            payout: None,
            ended: None,
            _v: PhantomData,
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
            Phase::Betting => vec![STANDARD.single(), ActionSpec::fixed("leave")],
            // Semua 32 pilihan tahan sebagai aksi tetap (posisi urut naik).
            Phase::Hold => (0u8..32)
                .map(|m| {
                    let pos: Vec<u8> = (1..=5).filter(|p| m & (1 << (p - 1)) != 0).collect();
                    ActionSpec::fixed(self.format_action(&Action::Draw(pos)))
                })
                .collect(),
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
            (Phase::Betting, Action::Bet(n)) if STANDARD.accepts(n) => {
                self.hand = self.deck[..5].to_vec();
                self.bet = n;
                self.phase = Phase::Hold;
                Ok(())
            }
            (Phase::Betting, Action::Leave) => {
                self.phase = Phase::Over;
                self.ended = Some("berhenti");
                Ok(())
            }
            (Phase::Hold, Action::Draw(pos)) => {
                let mut held = vec![false; 5];
                for p in &pos {
                    if !(1..=5).contains(p) || held[*p as usize - 1] {
                        return Err(illegal);
                    }
                    held[*p as usize - 1] = true;
                }
                let need = held.iter().filter(|h| !**h).count();
                if self.deck.len() < 5 + need {
                    return Err(GameError::Config("kartu tetap habis".into()));
                }
                let mut next = self.deck[5..].iter();
                for (i, h) in held.iter().enumerate() {
                    if !h {
                        self.hand[i] = *next.next().expect("kartu pengganti");
                    }
                }
                self.held = held;
                let five = self.five().expect("lima kartu");
                let back = self.bet * i64::from(payout(V::VARIANT, &five));
                self.payout = Some(back - self.bet);
                self.phase = Phase::Over;
                self.ended = Some("ronde_selesai");
                Ok(())
            }
            _ => Err(illegal),
        }
    }

    fn view_for(&self, _player: PlayerId) -> View {
        let fase = match self.phase {
            Phase::Betting => "taruhan",
            Phase::Hold => "tahan",
            Phase::Over => "selesai",
        };
        let mut meja = Umum::new(fase, STANDARD);
        meja.bersih = self.payout.unwrap_or(0);
        meja.ronde = u32::from(self.payout.is_some());
        meja.taruhan_meja = if self.phase == Phase::Hold {
            self.bet
        } else {
            0
        };
        meja.dipertaruhkan = self.bet;
        if self.phase == Phase::Hold {
            meja.netral = Some("draw 1 2 3 4 5".into());
        }
        meja.selesai = self.phase == Phase::Over;
        meja.alasan = self.ended.map(str::to_string);
        let variant = V::VARIANT;
        View {
            meja,
            kartu: self.hand.iter().map(VpCard::to_string).collect(),
            tahan: self.held.clone(),
            tangan: self
                .five()
                .and_then(|f| evaluate(variant, &f))
                .map(|h| h.key().to_string()),
            tabel: variant
                .hands()
                .iter()
                .map(|h| (h.key().to_string(), variant.pays(*h)))
                .collect(),
            taruhan: self.bet,
            bayar: self.payout,
        }
    }

    fn to_text(view: &View, lang: Lang) -> String {
        let c = catalog();
        let mut out = String::new();
        if !view.kartu.is_empty() {
            out.push_str(&c.text(lang, "cards", &[("cards", &view.kartu.join(" "))]));
            if let Some(h) = &view.tangan {
                out.push_str(" · ");
                out.push_str(&c.text(lang, &format!("hand.{h}"), &[]));
            }
            out.push('\n');
        }
        out.push_str(
            &match (view.meja.fase.as_str(), view.meja.alasan.as_deref()) {
                ("taruhan", _) => c.text(lang, "place_bet", &[]),
                ("tahan", _) => c.text(lang, "hold", &[]),
                (_, Some(reason)) => c.text(
                    lang,
                    &format!("over.{reason}"),
                    &[("net", &format!("{:+}", view.bayar.unwrap_or(0)))],
                ),
                _ => String::new(),
            },
        );
        out
    }

    fn is_over(&self) -> bool {
        self.phase == Phase::Over
    }

    fn result(&self) -> Option<GameResult> {
        (self.phase == Phase::Over).then(|| {
            meja::result(
                catalog(),
                "summary",
                self.payout.unwrap_or(0),
                u32::from(self.payout.is_some()),
            )
        })
    }

    fn parse_command(&self, command: &str) -> Result<Action, GameError> {
        let c = command.trim();
        let parts: Vec<&str> = c.split(' ').collect();
        let err = || GameError::Parse(c.into());
        match parts.as_slice() {
            ["bet", n] => Ok(Action::Bet(meja::amount(n).ok_or_else(err)?)),
            ["draw", rest @ ..] => {
                let mut pos = rest
                    .iter()
                    .map(|p| p.parse::<u8>().map_err(|_| err()))
                    .collect::<Result<Vec<_>, _>>()?;
                pos.sort_unstable();
                Ok(Action::Draw(pos))
            }
            ["leave"] => Ok(Action::Leave),
            _ => Err(err()),
        }
    }

    fn format_action(&self, action: &Action) -> String {
        match action {
            Action::Bet(n) => format!("bet {n}"),
            Action::Draw(pos) if pos.is_empty() => "draw".into(),
            Action::Draw(pos) => format!(
                "draw {}",
                pos.iter().map(u8::to_string).collect::<Vec<_>>().join(" ")
            ),
            Action::Leave => "leave".into(),
        }
    }
}
