//! Dragon Tiger (SPEC §6.3; M5a): satu kartu Naga lawan satu kartu Macan.
//!
//! Aturan (D-060): 8 dek, titik potong 75%, as terendah, jenis tidak
//! berpengaruh. Taruhan `dragon` dan `tiger` dibayar 1:1 dan kalah
//! setengah bila seri; `tie` dibayar 8:1. Satu sesi = satu shoe, provably
//! fair per shoe seperti Blackjack (D-056).

use std::collections::BTreeMap;
use std::sync::OnceLock;

use kyusin_core::game::{GameResult, create_session};
use kyusin_core::i18n::{Catalog, Lang};
use kyusin_core::{ActionSpec, Cartridge, GameError, PlayerId, RegistryError, Seed, TurnGame};
use serde::{Deserialize, Serialize};

use crate::cards::{Card, Rank, Shoe};
use crate::meja::{self, MejaView, STANDARD, ShoeConfig, Spots, Umum, WagerRtp};

pub const ID: &str = "dragon-tiger";
pub const DECKS: u8 = 8;
pub const PENETRATION: f64 = 0.75;
pub const SPOTS: &[&str] = &["dragon", "tiger", "tie"];
pub const TIE_PAYS: i64 = 8;

pub type Config = ShoeConfig;

/// RTP tiap taruhan (analitis, 8 dek; D-060). Angka manifest = dragon.
pub const RTP: &[WagerRtp] = &[
    WagerRtp {
        wager: "dragon",
        percent: 96.265,
        manifest: true,
    },
    WagerRtp {
        wager: "tiger",
        percent: 96.265,
        manifest: false,
    },
    WagerRtp {
        wager: "tie",
        percent: 67.229,
        manifest: false,
    },
];

pub fn catalog() -> &'static Catalog {
    static CATALOG: OnceLock<Catalog> = OnceLock::new();
    CATALOG.get_or_init(|| {
        Catalog::from_toml(include_str!("i18n.toml")).expect("dragon_tiger/i18n.toml tidak sah")
    })
}

pub fn cartridge() -> Result<Cartridge, RegistryError> {
    Cartridge::new(
        include_str!("manifest.toml"),
        include_str!("../../../../tutorials/dragon-tiger.toml"),
        create_session::<DragonTiger>,
    )
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Action {
    Bet(&'static str, i64),
    Clear,
    Deal,
    Leave,
}

/// Nilai kartu: as 1 sampai K 13.
pub fn card_value(c: Card) -> u8 {
    match c.rank {
        Rank::Ace => 1,
        r => r as u8 + 1,
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
struct Round {
    dragon: Card,
    tiger: Card,
    winner: &'static str,
    bets: BTreeMap<String, i64>,
    payouts: BTreeMap<String, i64>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct DragonTiger {
    shoe: Shoe,
    cut: usize,
    spots: Spots,
    last: Option<Round>,
    net: i64,
    rounds: u32,
    ended: Option<&'static str>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct View {
    #[serde(flatten)]
    pub meja: Umum,
    /// Taruhan yang sedang disusun untuk ronde berikutnya.
    pub taruhan: BTreeMap<String, i64>,
    /// Ronde terakhir: kartu Naga, kartu Macan, pemenang (`naga`, `macan`,
    /// `seri`), taruhannya, dan hasil tiap tempat.
    pub naga: Option<String>,
    pub macan: Option<String>,
    pub pemenang: Option<String>,
    pub taruhan_terakhir: BTreeMap<String, i64>,
    pub bayar: BTreeMap<String, i64>,
}

impl MejaView for View {
    fn meja(&self) -> &Umum {
        &self.meja
    }
}

impl DragonTiger {
    fn over(&self) -> bool {
        self.ended.is_some()
    }

    fn deal(&mut self) -> Result<(), GameError> {
        let dragon = meja::draw(&mut self.shoe)?;
        let tiger = meja::draw(&mut self.shoe)?;
        let (d, t) = (card_value(dragon), card_value(tiger));
        let winner = match d.cmp(&t) {
            std::cmp::Ordering::Greater => "naga",
            std::cmp::Ordering::Less => "macan",
            std::cmp::Ordering::Equal => "seri",
        };
        let bets = self.spots.take();
        let payouts: BTreeMap<String, i64> = bets
            .iter()
            .map(|(spot, &b)| {
                let p = match (spot.as_str(), winner) {
                    ("tie", "seri") => TIE_PAYS * b,
                    ("tie", _) => -b,
                    (_, "seri") => -b / 2,
                    ("dragon", "naga") | ("tiger", "macan") => b,
                    _ => -b,
                };
                (spot.clone(), p)
            })
            .collect();
        self.net += payouts.values().sum::<i64>();
        self.rounds += 1;
        self.last = Some(Round {
            dragon,
            tiger,
            winner,
            bets,
            payouts,
        });
        if self.shoe.dealt() >= self.cut {
            self.ended = Some("shoe_habis");
        }
        Ok(())
    }
}

impl TurnGame for DragonTiger {
    type Config = Config;
    type Action = Action;
    type View = View;

    fn new(config: Config, seed: Seed) -> Result<Self, GameError> {
        let (shoe, cut) = config.build(DECKS, PENETRATION, seed)?;
        Ok(DragonTiger {
            shoe,
            cut,
            spots: Spots::new(SPOTS, STANDARD),
            last: None,
            net: 0,
            rounds: 0,
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
        let mut out = self.spots.legal();
        if !self.spots.is_empty() {
            out.push(ActionSpec::fixed("clear"));
            out.push(ActionSpec::fixed("deal"));
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
        match action {
            Action::Bet(spot, n) => {
                if self.spots.place(spot, n) {
                    Ok(())
                } else {
                    Err(GameError::Illegal(format!("bet {spot} {n}")))
                }
            }
            Action::Clear => {
                self.spots.clear();
                Ok(())
            }
            Action::Deal if !self.spots.is_empty() => self.deal(),
            Action::Deal => Err(GameError::Illegal("deal".into())),
            Action::Leave => {
                self.spots.clear();
                self.ended = Some("berhenti");
                Ok(())
            }
        }
    }

    fn view_for(&self, _player: PlayerId) -> View {
        let mut meja = Umum::new(if self.over() { "selesai" } else { "taruhan" }, STANDARD)
            .shoe(&self.shoe, self.cut);
        meja.bersih = self.net;
        meja.ronde = self.rounds;
        meja.taruhan_meja = self.spots.total();
        meja.dipertaruhkan = self
            .last
            .as_ref()
            .map(|r| r.bets.values().sum())
            .unwrap_or(0);
        meja.selesai = self.over();
        meja.alasan = self.ended.map(str::to_string);
        let last = self.last.as_ref();
        View {
            meja,
            taruhan: self.spots.bets().clone(),
            naga: last.map(|r| r.dragon.to_string()),
            macan: last.map(|r| r.tiger.to_string()),
            pemenang: last.map(|r| r.winner.to_string()),
            taruhan_terakhir: last.map(|r| r.bets.clone()).unwrap_or_default(),
            bayar: last.map(|r| r.payouts.clone()).unwrap_or_default(),
        }
    }

    fn to_text(view: &View, lang: Lang) -> String {
        let c = catalog();
        let mut out = String::new();
        if let (Some(d), Some(t), Some(w)) = (&view.naga, &view.macan, &view.pemenang) {
            out.push_str(&c.text(
                lang,
                "cards",
                &[
                    ("dragon", d),
                    ("tiger", t),
                    ("winner", &c.text(lang, &format!("winner.{w}"), &[])),
                ],
            ));
            out.push('\n');
            for (spot, pay) in &view.bayar {
                out.push_str(&format!(
                    "  {} {} → {pay:+}\n",
                    c.text(lang, &format!("spot.{spot}"), &[]),
                    view.taruhan_terakhir.get(spot).copied().unwrap_or(0)
                ));
            }
        }
        let bets: Vec<String> = SPOTS
            .iter()
            .map(|s| {
                format!(
                    "{} {}",
                    c.text(lang, &format!("spot.{s}"), &[]),
                    view.taruhan.get(*s).copied().unwrap_or(0)
                )
            })
            .collect();
        out.push_str(&c.text(lang, "bets", &[("bets", &bets.join(" · "))]));
        out.push('\n');
        out.push_str(&c.text(
            lang,
            "net",
            &[
                ("net", &format!("{:+}", view.meja.bersih)),
                ("left", &view.meja.sisa.unwrap_or(0).to_string()),
            ],
        ));
        out.push('\n');
        out.push_str(&match view.meja.alasan.as_deref() {
            Some(reason) => c.text(lang, &format!("over.{reason}"), &[]),
            None => c.text(lang, "place_bet", &[]),
        });
        out
    }

    fn is_over(&self) -> bool {
        self.over()
    }

    fn result(&self) -> Option<GameResult> {
        self.over()
            .then(|| meja::result(catalog(), "summary", self.net, self.rounds))
    }

    fn parse_command(&self, command: &str) -> Result<Action, GameError> {
        let c = command.trim();
        let parts: Vec<&str> = c.split(' ').collect();
        let err = || GameError::Parse(c.into());
        match parts.as_slice() {
            ["bet", spot, n] => {
                let spot = SPOTS.iter().find(|s| **s == *spot).ok_or_else(err)?;
                Ok(Action::Bet(spot, meja::amount(n).ok_or_else(err)?))
            }
            ["clear"] => Ok(Action::Clear),
            ["deal"] => Ok(Action::Deal),
            ["leave"] => Ok(Action::Leave),
            _ => Err(err()),
        }
    }

    fn format_action(&self, action: &Action) -> String {
        match action {
            Action::Bet(spot, n) => format!("bet {spot} {n}"),
            Action::Clear => "clear".into(),
            Action::Deal => "deal".into(),
            Action::Leave => "leave".into(),
        }
    }
}
