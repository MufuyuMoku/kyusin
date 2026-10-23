//! Keno (SPEC §6.5; M6b, D-071): 80 angka, 20 ditarik, pemain memilih 1–10
//! angka. Bayaran tabel A (pengali total dalam persepuluhan), RTP tepat dari
//! distribusi hipergeometrik ([`exact_rtp`]).
//!
//! Satu pertandingan = satu tarikan: pemain menandai angka (`mark`,
//! `unmark`, `clear`), lalu `bet <jumlah>` memasang taruhan dan menarik 20
//! angka dari seed ronde. Game ini tidak tahu saldo; host membaca kontrak
//! chip [`Umum`] (D-058, D-059).

use std::sync::OnceLock;

use kyusin_core::game::{GameResult, create_session};
use kyusin_core::i18n::{Catalog, Lang};
use kyusin_core::{
    ActionSpec, Cartridge, GameError, GameRng, Param, ParamKind, PlayerId, RegistryError, Seed,
    TurnGame,
};
use serde::{Deserialize, Serialize};

use crate::meja::{self, MejaView, STANDARD, Umum, WagerRtp};

pub const ID: &str = "keno";
pub const NUMBERS: u8 = 80;
pub const DRAWN: usize = 20;
pub const MAX_SPOTS: usize = 10;

/// Tabel A (D-071): per jumlah angka dipilih, (kena, pengali × 10).
const TABLE: [&[(usize, u32)]; 11] = [
    &[],
    &[(1, 38)],
    &[(2, 158)],
    &[(2, 20), (3, 480)],
    &[(2, 10), (3, 55), (4, 1600)],
    &[(3, 30), (4, 150), (5, 8000)],
    &[(3, 20), (4, 65), (5, 900), (6, 18000)],
    &[(3, 15), (4, 30), (5, 230), (6, 3000), (7, 45000)],
    &[(4, 20), (5, 150), (6, 1100), (7, 11000), (8, 185000)],
    &[(4, 15), (5, 55), (6, 430), (7, 3600), (8, 36000), (9, 360000)],
    &[
        (0, 35),
        (5, 20),
        (6, 220),
        (7, 1450),
        (8, 11000),
        (9, 70000),
        (10, 800000),
    ],
];

/// RTP tepat tiap jumlah angka (hipergeometrik). Manifest = 4 angka
/// (terendah).
pub const RTP: &[WagerRtp] = &[
    rtp("spot-1", 95.0, false),
    rtp("spot-2", 95.0, false),
    rtp("spot-3", 94.3525, false),
    rtp("spot-4", 94.0642, true),
    rtp("spot-5", 94.9130, false),
    rtp("spot-6", 95.5916, false),
    rtp("spot-7", 94.7183, false),
    rtp("spot-8", 95.4780, false),
    rtp("spot-9", 95.2818, false),
    rtp("spot-10", 95.0068, false),
];

const fn rtp(wager: &'static str, percent: f64, manifest: bool) -> WagerRtp {
    WagerRtp {
        wager,
        percent,
        manifest,
    }
}

/// Pengali total (× 10) untuk `hits` kena dari `spots` angka; 0 = kalah.
pub fn pays_tenths(spots: usize, hits: usize) -> u32 {
    TABLE
        .get(spots)
        .and_then(|t| t.iter().find(|(h, _)| *h == hits))
        .map_or(0, |t| t.1)
}

fn choose(n: u64, k: u64) -> f64 {
    if k > n {
        return 0.0;
    }
    (0..k).fold(1.0, |acc, i| acc * (n - i) as f64 / (i + 1) as f64)
}

/// Peluang tepat `hits` dari `spots` angka termasuk di 20 angka yang ditarik.
pub fn probability(spots: usize, hits: usize) -> f64 {
    let (s, h) = (spots as u64, hits as u64);
    let n = u64::from(NUMBERS);
    let d = DRAWN as u64;
    if h > s || h > d {
        return 0.0;
    }
    choose(s, h) * choose(n - s, d - h) / choose(n, d)
}

/// RTP tepat (persen) untuk `spots` angka.
pub fn exact_rtp(spots: usize) -> f64 {
    (0..=spots)
        .map(|h| probability(spots, h) * f64::from(pays_tenths(spots, h)) / 10.0)
        .sum::<f64>()
        * 100.0
}

pub fn catalog() -> &'static Catalog {
    static CATALOG: OnceLock<Catalog> = OnceLock::new();
    CATALOG.get_or_init(|| {
        Catalog::from_toml(include_str!("i18n.toml")).expect("keno/i18n.toml tidak sah")
    })
}

pub fn cartridge() -> Result<Cartridge, RegistryError> {
    Cartridge::new(
        include_str!("manifest.toml"),
        include_str!("../../../../tutorials/keno.toml"),
        create_session::<Keno>,
    )
}

/// Menarik `count` angka berbeda 1..=`max` dari seed (urutan tarikan).
pub fn draw_distinct(seed: Seed, max: u8, count: usize) -> Vec<u8> {
    let mut all: Vec<u8> = (1..=max).collect();
    GameRng::from_seed(seed).shuffle(&mut all);
    all.truncate(count);
    all
}

/// `count` angka berbeda, semuanya di 1..=`max`.
pub fn distinct_in_range(list: &[u8], max: u8, count: usize) -> bool {
    let mut s = list.to_vec();
    s.sort_unstable();
    s.dedup();
    list.len() == count && s.len() == count && s.iter().all(|n| (1..=max).contains(n))
}

#[derive(Debug, Clone, Default, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Config {
    /// 20 angka tarikan tetap (tes dan tutorial).
    #[serde(default)]
    pub hasil: Option<Vec<u8>>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Action {
    Mark(u8),
    Unmark(u8),
    Clear,
    Bet(i64),
    Leave,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Keno {
    seed: Seed,
    fixed: Option<Vec<u8>>,
    marked: Vec<u8>,
    drawn: Vec<u8>,
    bet: i64,
    payout: Option<i64>,
    ended: Option<&'static str>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct View {
    #[serde(flatten)]
    pub meja: Umum,
    /// Angka yang dipilih, urut naik.
    pub pilihan: Vec<u8>,
    /// Tabel bayar untuk jumlah angka yang dipilih: [kena, pengali × 10].
    pub tabel: Vec<[u32; 2]>,
    /// 20 angka yang ditarik (urutan tarikan), setelah ronde berjalan.
    pub tarikan: Vec<u8>,
    /// Angka pilihan yang ditarik, urut naik.
    pub kena: Vec<u8>,
    pub taruhan: i64,
    pub bayar: Option<i64>,
}

impl MejaView for View {
    fn meja(&self) -> &Umum {
        &self.meja
    }
}

impl Keno {
    fn over(&self) -> bool {
        self.ended.is_some()
    }

    fn hits(&self) -> Vec<u8> {
        self.marked
            .iter()
            .copied()
            .filter(|n| self.drawn.contains(n))
            .collect()
    }
}

impl TurnGame for Keno {
    type Config = Config;
    type Action = Action;
    type View = View;

    fn new(config: Config, seed: Seed) -> Result<Self, GameError> {
        if let Some(d) = &config.hasil
            && !distinct_in_range(d, NUMBERS, DRAWN)
        {
            return Err(GameError::Config("hasil: 20 angka berbeda 1–80".into()));
        }
        Ok(Keno {
            seed,
            fixed: config.hasil,
            marked: Vec::new(),
            drawn: Vec::new(),
            bet: 0,
            payout: None,
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
        let numbers = |list: Vec<u8>| Param {
            name: "angka".into(),
            kind: ParamKind::Choice {
                options: list.iter().map(u8::to_string).collect(),
            },
        };
        let mut out = Vec::new();
        if self.marked.len() < MAX_SPOTS {
            let free: Vec<u8> = (1..=NUMBERS).filter(|n| !self.marked.contains(n)).collect();
            out.push(ActionSpec::template("mark", vec![numbers(free)]));
        }
        if !self.marked.is_empty() {
            out.push(ActionSpec::template(
                "unmark",
                vec![numbers(self.marked.clone())],
            ));
            out.push(ActionSpec::fixed("clear"));
            out.push(STANDARD.single());
        }
        out.push(ActionSpec::fixed("leave"));
        out
    }

    fn apply(&mut self, player: PlayerId, action: Action) -> Result<(), GameError> {
        if self.over() {
            return Err(GameError::Over);
        }
        if player != 0 {
            return Err(GameError::NotPending(player));
        }
        let illegal = GameError::Illegal(self.format_action(&action));
        match action {
            Action::Mark(n)
                if (1..=NUMBERS).contains(&n)
                    && !self.marked.contains(&n)
                    && self.marked.len() < MAX_SPOTS =>
            {
                self.marked.push(n);
                self.marked.sort_unstable();
            }
            Action::Unmark(n) if self.marked.contains(&n) => self.marked.retain(|m| *m != n),
            Action::Clear => self.marked.clear(),
            Action::Bet(n) if !self.marked.is_empty() && STANDARD.accepts(n) => {
                self.drawn = self
                    .fixed
                    .clone()
                    .unwrap_or_else(|| draw_distinct(self.seed, NUMBERS, DRAWN));
                self.bet = n;
                let hits = self.hits().len();
                let back = n * i64::from(pays_tenths(self.marked.len(), hits)) / 10;
                self.payout = Some(back - n);
                self.ended = Some("ronde_selesai");
            }
            Action::Leave => self.ended = Some("berhenti"),
            _ => return Err(illegal),
        }
        Ok(())
    }

    fn view_for(&self, _player: PlayerId) -> View {
        let fase = if self.over() { "selesai" } else { "taruhan" };
        let mut meja = Umum::new(fase, STANDARD);
        meja.bersih = self.payout.unwrap_or(0);
        meja.ronde = u32::from(self.payout.is_some());
        meja.dipertaruhkan = self.bet;
        meja.selesai = self.over();
        meja.alasan = self.ended.map(str::to_string);
        View {
            meja,
            pilihan: self.marked.clone(),
            tabel: TABLE
                .get(self.marked.len())
                .map(|t| t.iter().map(|(h, m)| [*h as u32, *m]).collect())
                .unwrap_or_default(),
            tarikan: self.drawn.clone(),
            kena: self.hits(),
            taruhan: self.bet,
            bayar: self.payout,
        }
    }

    fn to_text(view: &View, lang: Lang) -> String {
        let c = catalog();
        let list = |v: &[u8]| v.iter().map(u8::to_string).collect::<Vec<_>>().join(" ");
        let mut out = c.text(lang, "marked", &[("list", &list(&view.pilihan))]);
        out.push('\n');
        if !view.tarikan.is_empty() {
            let mut d = view.tarikan.clone();
            d.sort_unstable();
            out.push_str(&c.text(lang, "drawn", &[("list", &list(&d))]));
            out.push('\n');
            out.push_str(&c.text(
                lang,
                "hits",
                &[
                    ("n", &view.kena.len().to_string()),
                    ("list", &list(&view.kena)),
                ],
            ));
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
                self.payout.unwrap_or(0),
                u32::from(self.payout.is_some()),
            )
        })
    }

    fn parse_command(&self, command: &str) -> Result<Action, GameError> {
        let c = command.trim();
        let parts: Vec<&str> = c.split(' ').collect();
        let err = || GameError::Parse(c.into());
        let number = |s: &str| s.parse::<u8>().map_err(|_| err());
        match parts.as_slice() {
            ["mark", n] => Ok(Action::Mark(number(n)?)),
            ["unmark", n] => Ok(Action::Unmark(number(n)?)),
            ["clear"] => Ok(Action::Clear),
            ["bet", n] => Ok(Action::Bet(meja::amount(n).ok_or_else(err)?)),
            ["leave"] => Ok(Action::Leave),
            _ => Err(err()),
        }
    }

    fn format_action(&self, action: &Action) -> String {
        match action {
            Action::Mark(n) => format!("mark {n}"),
            Action::Unmark(n) => format!("unmark {n}"),
            Action::Clear => "clear".into(),
            Action::Bet(n) => format!("bet {n}"),
            Action::Leave => "leave".into(),
        }
    }
}
