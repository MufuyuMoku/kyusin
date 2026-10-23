//! Kartu gosok (SPEC §6.5; M6b, D-071): kartu 3×3, cocokkan tiga jumlah.
//!
//! Setiap kartu independen dari seed ronde: hadiah diundi lebih dulu dari
//! tabel peluang (penyebut 100.000, [`PRIZES`]), lalu sembilan sel disusun
//! dari turunan seed yang sama ([`layout`]): menang = tepat tiga sel bernilai
//! hadiah itu dan nilai lain paling banyak dua kali; kalah = tidak ada nilai
//! yang muncul tiga kali. RTP tepat = jumlah peluang × hadiah = 95%.
//!
//! `bet <jumlah>` membeli kartu (dipotong saat dibeli, D-059), `scratch <n>`
//! atau `scratch all` membuka sel; hasil dibayar saat sel terakhir dibuka.

use std::sync::OnceLock;

use kyusin_core::game::{GameResult, create_session};
use kyusin_core::i18n::{Catalog, Lang};
use kyusin_core::rng::derive;
use kyusin_core::{
    ActionSpec, Cartridge, GameError, GameRng, Param, ParamKind, PlayerId, RegistryError, Seed,
    TurnGame,
};
use serde::{Deserialize, Serialize};

use crate::meja::{self, MejaView, STANDARD, Umum, WagerRtp};

pub const ID: &str = "kartu-gosok";
pub const DENOMINATOR: u32 = 100_000;

/// (pengali total, peluang dari 100.000), D-071.
pub const PRIZES: &[(u32, u32)] = &[
    (1, 16_000),
    (2, 10_000),
    (5, 3_000),
    (10, 1_500),
    (20, 600),
    (50, 200),
    (100, 50),
    (1000, 2),
];

pub const RTP: &[WagerRtp] = &[WagerRtp {
    wager: "bet",
    percent: 95.0,
    manifest: true,
}];

pub fn catalog() -> &'static Catalog {
    static CATALOG: OnceLock<Catalog> = OnceLock::new();
    CATALOG.get_or_init(|| {
        Catalog::from_toml(include_str!("i18n.toml")).expect("kartu_gosok/i18n.toml tidak sah")
    })
}

pub fn cartridge() -> Result<Cartridge, RegistryError> {
    Cartridge::new(
        include_str!("manifest.toml"),
        include_str!("../../../../tutorials/kartu-gosok.toml"),
        create_session::<KartuGosok>,
    )
}

/// Hadiah kartu (pengali total; 0 = kalah) dari seed ronde.
pub fn draw_prize(seed: Seed) -> u32 {
    let mut x = GameRng::from_seed(derive(&seed, "hadiah")).below(DENOMINATOR);
    for (m, w) in PRIZES {
        if x < *w {
            return *m;
        }
        x -= w;
    }
    0
}

/// Sembilan sel untuk hadiah `prize` (0 = kalah), dari seed ronde.
pub fn layout(prize: u32, seed: Seed) -> [u32; 9] {
    let mut rng = GameRng::from_seed(derive(&seed, "susun"));
    let mut pool: Vec<u32> = PRIZES
        .iter()
        .map(|p| p.0)
        .filter(|m| *m != prize)
        .flat_map(|m| [m, m])
        .collect();
    rng.shuffle(&mut pool);
    let mut cells: Vec<u32> = if prize == 0 {
        pool.truncate(9);
        pool
    } else {
        pool.truncate(6);
        pool.extend([prize; 3]);
        pool
    };
    rng.shuffle(&mut cells);
    cells.try_into().expect("sembilan sel")
}

#[derive(Debug, Clone, Default, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Config {
    /// Hadiah tetap (0 = kalah; tes dan tutorial).
    #[serde(default)]
    pub hadiah: Option<u32>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Action {
    Bet(i64),
    Scratch(usize),
    ScratchAll,
    Leave,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
enum Phase {
    Betting,
    Scratch,
    Over,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct KartuGosok {
    seed: Seed,
    fixed: Option<u32>,
    phase: Phase,
    prize: u32,
    cells: [u32; 9],
    open: [bool; 9],
    bet: i64,
    payout: Option<i64>,
    ended: Option<&'static str>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct View {
    #[serde(flatten)]
    pub meja: Umum,
    /// Nilai sel yang sudah digosok (`null` = belum).
    pub sel: Vec<Option<u32>>,
    /// Hadiah (pengali total) setelah semua sel dibuka.
    pub hadiah: Option<u32>,
    pub taruhan: i64,
    pub bayar: Option<i64>,
}

impl MejaView for View {
    fn meja(&self) -> &Umum {
        &self.meja
    }
}

impl KartuGosok {
    fn reveal(&mut self, i: usize) {
        self.open[i] = true;
        if self.open.iter().all(|o| *o) {
            self.payout = Some(self.bet * i64::from(self.prize) - self.bet);
            self.phase = Phase::Over;
            self.ended = Some("ronde_selesai");
        }
    }
}

impl TurnGame for KartuGosok {
    type Config = Config;
    type Action = Action;
    type View = View;

    fn new(config: Config, seed: Seed) -> Result<Self, GameError> {
        if let Some(p) = config.hadiah
            && p != 0
            && !PRIZES.iter().any(|x| x.0 == p)
        {
            return Err(GameError::Config(format!(
                "hadiah `{p}` tidak ada di tabel"
            )));
        }
        Ok(KartuGosok {
            seed,
            fixed: config.hadiah,
            phase: Phase::Betting,
            prize: 0,
            cells: [0; 9],
            open: [false; 9],
            bet: 0,
            payout: None,
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
            Phase::Betting => vec![STANDARD.single(), ActionSpec::fixed("leave")],
            Phase::Scratch => {
                let closed: Vec<String> = (0..9)
                    .filter(|i| !self.open[*i])
                    .map(|i| (i + 1).to_string())
                    .collect();
                vec![
                    ActionSpec::template(
                        "scratch",
                        vec![Param {
                            name: "sel".into(),
                            kind: ParamKind::Choice { options: closed },
                        }],
                    ),
                    ActionSpec::fixed("scratch all"),
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
        match (self.phase, action) {
            (Phase::Betting, Action::Bet(n)) if STANDARD.accepts(n) => {
                self.prize = self.fixed.unwrap_or_else(|| draw_prize(self.seed));
                self.cells = layout(self.prize, self.seed);
                self.bet = n;
                self.phase = Phase::Scratch;
            }
            (Phase::Betting, Action::Leave) => {
                self.phase = Phase::Over;
                self.ended = Some("berhenti");
            }
            (Phase::Scratch, Action::Scratch(i)) if (1..=9).contains(&i) && !self.open[i - 1] => {
                self.reveal(i - 1);
            }
            (Phase::Scratch, Action::ScratchAll) => {
                for i in 0..9 {
                    if !self.open[i] {
                        self.reveal(i);
                    }
                }
            }
            _ => return Err(GameError::Illegal(self.format_action(&action))),
        }
        Ok(())
    }

    fn view_for(&self, _player: PlayerId) -> View {
        let fase = match self.phase {
            Phase::Betting => "taruhan",
            Phase::Scratch => "gosok",
            Phase::Over => "selesai",
        };
        let mut meja = Umum::new(fase, STANDARD);
        let bought = self.phase != Phase::Betting && self.bet > 0;
        meja.bersih = self.payout.unwrap_or(0);
        meja.ronde = u32::from(self.payout.is_some());
        meja.taruhan_meja = if self.phase == Phase::Scratch {
            self.bet
        } else {
            0
        };
        meja.dipertaruhkan = self.bet;
        if self.phase == Phase::Scratch {
            meja.netral = Some("scratch all".into());
        }
        meja.selesai = self.phase == Phase::Over;
        meja.alasan = self.ended.map(str::to_string);
        View {
            meja,
            sel: if bought {
                (0..9)
                    .map(|i| self.open[i].then_some(self.cells[i]))
                    .collect()
            } else {
                Vec::new()
            },
            hadiah: self.payout.is_some().then_some(self.prize),
            taruhan: self.bet,
            bayar: self.payout,
        }
    }

    fn to_text(view: &View, lang: Lang) -> String {
        let c = catalog();
        let mut out = String::new();
        for row in view.sel.chunks(3) {
            let cells: Vec<String> = row
                .iter()
                .map(|x| x.map_or("[ ?? ]".into(), |m| format!("[{m:>4}×]")))
                .collect();
            out.push_str(&cells.join(" "));
            out.push('\n');
        }
        out.push_str(
            &match (view.meja.fase.as_str(), view.meja.alasan.as_deref()) {
                ("taruhan", _) => c.text(lang, "place_bet", &[]),
                ("gosok", _) => c.text(lang, "scratch", &[]),
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
            ["scratch", "all"] => Ok(Action::ScratchAll),
            ["scratch", n] => Ok(Action::Scratch(n.parse().map_err(|_| err())?)),
            ["leave"] => Ok(Action::Leave),
            _ => Err(err()),
        }
    }

    fn format_action(&self, action: &Action) -> String {
        match action {
            Action::Bet(n) => format!("bet {n}"),
            Action::Scratch(i) => format!("scratch {i}"),
            Action::ScratchAll => "scratch all".into(),
            Action::Leave => "leave".into(),
        }
    }
}
