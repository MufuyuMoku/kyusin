//! Bingo melawan bandar (SPEC §6.5; M6b, D-071): 75 bola, kartu 5×5 dengan
//! petak bebas di tengah, 1–4 kartu per ronde, 40 bola ditarik.
//!
//! Pola tertinggi yang terpenuhi di tiap kartu dibayar (pengali total):
//! 1 garis 1,5 / 4 sudut 2 / 2 garis 3 / 3 garis 4 / X 6 / bingkai 500 /
//! penuh 5.000. RTP tepat lewat enumerasi semua 2^24 isi kartu
//! ([`exact_rtp`]).
//!
//! Satu pertandingan = satu tarikan. Kartu diturunkan dari seed ronde
//! (`kartu`) dan sudah terlihat sebelum bertaruh; bola dari turunan lain
//! (`bola`), jadi melihat kartu tidak membocorkan bola.

use std::sync::OnceLock;

use kyusin_core::game::{GameResult, create_session};
use kyusin_core::i18n::{Catalog, Lang};
use kyusin_core::rng::derive;
use kyusin_core::{
    ActionSpec, Cartridge, GameError, GameRng, Param, ParamKind, PlayerId, RegistryError, Seed,
    TurnGame,
};
use serde::{Deserialize, Serialize};

use crate::keno::{distinct_in_range, draw_distinct};
use crate::meja::{self, MejaView, STANDARD, Umum, WagerRtp};

pub const ID: &str = "bingo";
pub const BALLS: u8 = 75;
pub const DRAWN: usize = 40;
pub const MAX_CARDS: usize = 4;
/// Petak bebas di tengah kartu.
pub const FREE: usize = 12;

/// RTP tepat per kartu (enumerasi 2^24, 40 bola; D-071).
pub const RTP: &[WagerRtp] = &[WagerRtp {
    wager: "bet",
    percent: 95.4342,
    manifest: true,
}];

pub type Card = [u8; 25];

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum Pattern {
    Line,
    Corners,
    TwoLines,
    ThreeLines,
    X,
    Frame,
    Full,
}

impl Pattern {
    pub const ALL: [Pattern; 7] = [
        Pattern::Line,
        Pattern::Corners,
        Pattern::TwoLines,
        Pattern::ThreeLines,
        Pattern::X,
        Pattern::Frame,
        Pattern::Full,
    ];

    /// Pengali total × 10.
    pub fn pays_tenths(self) -> u32 {
        match self {
            Pattern::Line => 15,
            Pattern::Corners => 20,
            Pattern::TwoLines => 30,
            Pattern::ThreeLines => 40,
            Pattern::X => 60,
            Pattern::Frame => 5000,
            Pattern::Full => 50000,
        }
    }

    pub fn key(self) -> &'static str {
        match self {
            Pattern::Line => "garis",
            Pattern::Corners => "sudut",
            Pattern::TwoLines => "dua_garis",
            Pattern::ThreeLines => "tiga_garis",
            Pattern::X => "x",
            Pattern::Frame => "bingkai",
            Pattern::Full => "penuh",
        }
    }
}

/// Bit tiap petak (petak bebas tidak punya bit; 24 bit).
fn bit(cell: usize) -> u32 {
    match cell {
        FREE => 0,
        c if c < FREE => 1 << c,
        c => 1 << (c - 1),
    }
}

fn mask(cells: impl IntoIterator<Item = usize>) -> u32 {
    cells.into_iter().fold(0, |m, c| m | bit(c))
}

struct Masks {
    lines: Vec<u32>,
    corners: u32,
    x: u32,
    frame: u32,
    full: u32,
}

fn masks() -> &'static Masks {
    static M: OnceLock<Masks> = OnceLock::new();
    M.get_or_init(|| {
        let mut lines = Vec::new();
        for r in 0..5 {
            lines.push(mask((0..5).map(|c| r * 5 + c)));
        }
        for c in 0..5 {
            lines.push(mask((0..5).map(|r| r * 5 + c)));
        }
        let d1 = mask((0..5).map(|i| i * 6));
        let d2 = mask((0..5).map(|i| i * 4 + 4));
        lines.push(d1);
        lines.push(d2);
        Masks {
            lines,
            corners: mask([0, 4, 20, 24]),
            x: d1 | d2,
            frame: mask((0..25).filter(|c| c / 5 == 0 || c / 5 == 4 || c % 5 == 0 || c % 5 == 4)),
            full: (1 << 24) - 1,
        }
    })
}

/// Pola tertinggi untuk petak yang tertandai (`hit`, 24 bit).
fn best_mask(hit: u32) -> Option<Pattern> {
    let m = masks();
    let has = |x: u32| hit & x == x;
    let lines = m.lines.iter().filter(|l| has(**l)).count();
    let found = [
        (has(m.full), Pattern::Full),
        (has(m.frame), Pattern::Frame),
        (has(m.x), Pattern::X),
        (lines >= 3, Pattern::ThreeLines),
        (lines >= 2, Pattern::TwoLines),
        (has(m.corners), Pattern::Corners),
        (lines >= 1, Pattern::Line),
    ];
    found.iter().find(|(ok, _)| *ok).map(|(_, p)| *p)
}

/// Pola tertinggi kartu untuk bola yang sudah ditarik.
pub fn best(card: &Card, drawn: &[u8]) -> Option<Pattern> {
    let hit = mask((0..25).filter(|&c| c != FREE && drawn.contains(&card[c])));
    best_mask(hit)
}

/// Kartu sah: kolom B 1–15, I 16–30, N 31–45, G 46–60, O 61–75, berbeda,
/// petak tengah 0.
pub fn valid_card(card: &Card) -> bool {
    let mut seen = Vec::new();
    for (i, &n) in card.iter().enumerate() {
        if i == FREE {
            if n != 0 {
                return false;
            }
            continue;
        }
        let col = (i % 5) as u8;
        if !(col * 15 + 1..=col * 15 + 15).contains(&n) || seen.contains(&n) {
            return false;
        }
        seen.push(n);
    }
    true
}

/// Kartu acak yang sah dari RNG.
pub fn random_card(rng: &mut GameRng) -> Card {
    let mut card = [0u8; 25];
    for col in 0..5u8 {
        let mut nums: Vec<u8> = (col * 15 + 1..=col * 15 + 15).collect();
        rng.shuffle(&mut nums);
        for row in 0..5 {
            card[row * 5 + col as usize] = nums[row];
        }
    }
    card[FREE] = 0;
    card
}

fn choose(n: u64, k: u64) -> f64 {
    if k > n {
        return 0.0;
    }
    (0..k).fold(1.0, |acc, i| acc * (n - i) as f64 / (i + 1) as f64)
}

/// RTP tepat per kartu (persen): setiap himpunan petak yang tertandai
/// berpeluang C(51, 40 − k) / C(75, 40), k = jumlah petaknya.
pub fn exact_rtp() -> f64 {
    let mut by_size = [0u64; 25];
    for hit in 0u32..(1 << 24) {
        if let Some(p) = best_mask(hit) {
            by_size[hit.count_ones() as usize] += u64::from(p.pays_tenths());
        }
    }
    let other = u64::from(BALLS) - 24;
    let total = choose(u64::from(BALLS), DRAWN as u64);
    let rtp: f64 = by_size
        .iter()
        .enumerate()
        .filter(|(k, _)| *k <= DRAWN)
        .map(|(k, s)| *s as f64 * choose(other, (DRAWN - k) as u64) / total)
        .sum();
    rtp * 10.0
}

pub fn catalog() -> &'static Catalog {
    static CATALOG: OnceLock<Catalog> = OnceLock::new();
    CATALOG.get_or_init(|| {
        Catalog::from_toml(include_str!("i18n.toml")).expect("bingo/i18n.toml tidak sah")
    })
}

pub fn cartridge() -> Result<Cartridge, RegistryError> {
    Cartridge::new(
        include_str!("manifest.toml"),
        include_str!("../../../../tutorials/bingo.toml"),
        create_session::<Bingo>,
    )
}

#[derive(Debug, Clone, Default, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Config {
    /// Kartu tetap (1–4; tes dan tutorial).
    #[serde(default)]
    pub kartu: Option<Vec<Card>>,
    /// 40 bola tetap.
    #[serde(default)]
    pub bola: Option<Vec<u8>>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Action {
    Cards(usize),
    Bet(i64),
    Leave,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Bingo {
    seed: Seed,
    cards: Vec<Card>,
    fixed_balls: Option<Vec<u8>>,
    count: usize,
    balls: Vec<u8>,
    bet: i64,
    payouts: Vec<i64>,
    ended: Option<&'static str>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct View {
    #[serde(flatten)]
    pub meja: Umum,
    /// Semua kartu yang tersedia (pemain memakai `jumlah_kartu` pertama).
    pub kartu: Vec<Card>,
    pub jumlah_kartu: usize,
    /// 40 bola yang ditarik (urutan tarikan), setelah ronde berjalan.
    pub bola: Vec<u8>,
    /// Pola tiap kartu yang dimainkan (`null` = tanpa pola).
    pub pola: Vec<Option<String>>,
    /// Hasil bersih tiap kartu.
    pub bayar_kartu: Vec<i64>,
    /// Taruhan per kartu.
    pub taruhan: i64,
    pub bayar: Option<i64>,
}

impl MejaView for View {
    fn meja(&self) -> &Umum {
        &self.meja
    }
}

impl Bingo {
    fn over(&self) -> bool {
        self.ended.is_some()
    }

    fn played(&self) -> bool {
        !self.balls.is_empty()
    }
}

impl TurnGame for Bingo {
    type Config = Config;
    type Action = Action;
    type View = View;

    fn new(config: Config, seed: Seed) -> Result<Self, GameError> {
        let cards = match config.kartu {
            Some(list) => {
                if list.is_empty() || list.len() > MAX_CARDS || !list.iter().all(valid_card) {
                    return Err(GameError::Config("kartu: 1–4 kartu bingo yang sah".into()));
                }
                list
            }
            None => {
                let mut rng = GameRng::from_seed(derive(&seed, "kartu"));
                (0..MAX_CARDS).map(|_| random_card(&mut rng)).collect()
            }
        };
        if let Some(b) = &config.bola
            && !distinct_in_range(b, BALLS, DRAWN)
        {
            return Err(GameError::Config("bola: 40 angka berbeda 1–75".into()));
        }
        Ok(Bingo {
            seed,
            cards,
            fixed_balls: config.bola,
            count: 1,
            balls: Vec::new(),
            bet: 0,
            payouts: Vec::new(),
            ended: None,
        })
    }

    fn seats(&self) -> u8 {
        1
    }

    fn pending_players(&self) -> Vec<PlayerId> {
        if self.over() { Vec::new() } else { vec![0] }
    }

    fn legal_actions(&self, player: PlayerId) -> Vec<ActionSpec> {
        if player != 0 || self.over() {
            return Vec::new();
        }
        vec![
            ActionSpec::template(
                "cards",
                vec![Param {
                    name: "jumlah_kartu".into(),
                    kind: ParamKind::Int {
                        min: 1,
                        max: self.cards.len() as i64,
                        step: 1,
                    },
                }],
            ),
            STANDARD.single(),
            ActionSpec::fixed("leave"),
        ]
    }

    fn apply(&mut self, player: PlayerId, action: Action) -> Result<(), GameError> {
        if self.over() {
            return Err(GameError::Over);
        }
        if player != 0 {
            return Err(GameError::NotPending(player));
        }
        match action {
            Action::Cards(n) if (1..=self.cards.len()).contains(&n) => self.count = n,
            Action::Bet(n) if STANDARD.accepts(n) => {
                self.balls = self
                    .fixed_balls
                    .clone()
                    .unwrap_or_else(|| draw_distinct(derive(&self.seed, "bola"), BALLS, DRAWN));
                self.bet = n;
                self.payouts = self.cards[..self.count]
                    .iter()
                    .map(|c| {
                        let back = best(c, &self.balls)
                            .map_or(0, |p| n * i64::from(p.pays_tenths()) / 10);
                        back - n
                    })
                    .collect();
                self.ended = Some("ronde_selesai");
            }
            Action::Leave => self.ended = Some("berhenti"),
            _ => return Err(GameError::Illegal(self.format_action(&action))),
        }
        Ok(())
    }

    fn view_for(&self, _player: PlayerId) -> View {
        let fase = if self.over() { "selesai" } else { "taruhan" };
        let mut meja = Umum::new(fase, STANDARD);
        let net: i64 = self.payouts.iter().sum();
        meja.bersih = net;
        meja.ronde = u32::from(self.played());
        meja.dipertaruhkan = self.bet * self.count as i64;
        meja.pengali.insert("bet".into(), self.count as i64);
        meja.selesai = self.over();
        meja.alasan = self.ended.map(str::to_string);
        View {
            meja,
            kartu: self.cards.clone(),
            jumlah_kartu: self.count,
            bola: self.balls.clone(),
            pola: if self.played() {
                self.cards[..self.count]
                    .iter()
                    .map(|c| best(c, &self.balls).map(|p| p.key().to_string()))
                    .collect()
            } else {
                Vec::new()
            },
            bayar_kartu: self.payouts.clone(),
            taruhan: self.bet,
            bayar: self.played().then_some(net),
        }
    }

    fn to_text(view: &View, lang: Lang) -> String {
        let c = catalog();
        let mut out = String::new();
        for (i, card) in view.kartu.iter().take(view.jumlah_kartu).enumerate() {
            out.push_str(&c.text(lang, "card", &[("n", &(i + 1).to_string())]));
            out.push('\n');
            for row in card.chunks(5) {
                let cells: Vec<String> = row
                    .iter()
                    .map(|&n| {
                        if n == 0 {
                            " *".into()
                        } else if view.bola.contains(&n) {
                            format!("[{n:>2}]")
                        } else {
                            format!(" {n:>2} ")
                        }
                    })
                    .collect();
                out.push_str(&cells.join(" "));
                out.push('\n');
            }
            if let Some(Some(p)) = view.pola.get(i) {
                out.push_str(&c.text(lang, &format!("pattern.{p}"), &[]));
                out.push('\n');
            }
        }
        if !view.bola.is_empty() {
            let balls: Vec<String> = view.bola.iter().map(u8::to_string).collect();
            out.push_str(&c.text(lang, "balls", &[("list", &balls.join(" "))]));
            out.push('\n');
        }
        out.push_str(&match view.meja.alasan.as_deref() {
            None => c.text(lang, "place_bet", &[]),
            Some(reason) => c.text(
                lang,
                &format!("over.{reason}"),
                &[("net", &format!("{:+}", view.bayar.unwrap_or(0)))],
            ),
        });
        out
    }

    fn is_over(&self) -> bool {
        self.over()
    }

    fn result(&self) -> Option<GameResult> {
        self.over().then(|| {
            meja::result(
                catalog(),
                "summary",
                self.payouts.iter().sum(),
                u32::from(self.played()),
            )
        })
    }

    fn parse_command(&self, command: &str) -> Result<Action, GameError> {
        let c = command.trim();
        let parts: Vec<&str> = c.split(' ').collect();
        let err = || GameError::Parse(c.into());
        match parts.as_slice() {
            ["cards", n] => Ok(Action::Cards(n.parse().map_err(|_| err())?)),
            ["bet", n] => Ok(Action::Bet(meja::amount(n).ok_or_else(err)?)),
            ["leave"] => Ok(Action::Leave),
            _ => Err(err()),
        }
    }

    fn format_action(&self, action: &Action) -> String {
        match action {
            Action::Cards(n) => format!("cards {n}"),
            Action::Bet(n) => format!("bet {n}"),
            Action::Leave => "leave".into(),
        }
    }
}
