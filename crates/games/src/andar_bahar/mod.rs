//! Andar Bahar (SPEC §6.3; M5a): tebak sisi yang menerima kartu kembar
//! kartu tengah.
//!
//! Aturan (keputusan klien, D-060): satu dek dikocok per ronde; kartu
//! pertama setelah kartu tengah selalu ke Andar, lalu bergantian. Andar
//! dibayar 0,9:1, Bahar 1:1. Satu sesi = satu ronde, jadi commit-reveal
//! SPEC §5.4 berlaku per ronde.

use std::collections::BTreeMap;
use std::sync::OnceLock;

use kyusin_core::game::{GameResult, create_session};
use kyusin_core::i18n::{Catalog, Lang};
use kyusin_core::{ActionSpec, Cartridge, GameError, PlayerId, RegistryError, Seed, TurnGame};
use serde::{Deserialize, Serialize};

use crate::cards::{Card, Shoe};
use crate::meja::{self, MejaView, RoundConfig, STANDARD, Spots, Umum, WagerRtp};

pub const ID: &str = "andar-bahar";
pub const SPOTS: &[&str] = &["andar", "bahar"];

pub type Config = RoundConfig;

/// RTP tiap taruhan (analitis, satu dek; D-060). Angka manifest = bahar.
pub const RTP: &[WagerRtp] = &[
    WagerRtp {
        wager: "andar",
        percent: 97.8511,
        manifest: false,
    },
    WagerRtp {
        wager: "bahar",
        percent: 96.9988,
        manifest: true,
    },
];

pub fn catalog() -> &'static Catalog {
    static CATALOG: OnceLock<Catalog> = OnceLock::new();
    CATALOG.get_or_init(|| {
        Catalog::from_toml(include_str!("i18n.toml")).expect("andar_bahar/i18n.toml tidak sah")
    })
}

pub fn cartridge() -> Result<Cartridge, RegistryError> {
    Cartridge::new(
        include_str!("manifest.toml"),
        include_str!("../../../../tutorials/andar-bahar.toml"),
        create_session::<AndarBahar>,
    )
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Action {
    Bet(&'static str, i64),
    Clear,
    Deal,
    Leave,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
struct Round {
    middle: Card,
    andar: Vec<Card>,
    bahar: Vec<Card>,
    winner: &'static str,
    bets: BTreeMap<String, i64>,
    payouts: BTreeMap<String, i64>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct AndarBahar {
    deck: Shoe,
    spots: Spots,
    round: Option<Round>,
    ended: Option<&'static str>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct View {
    #[serde(flatten)]
    pub meja: Umum,
    pub taruhan: BTreeMap<String, i64>,
    pub tengah: Option<String>,
    pub andar: Vec<String>,
    pub bahar: Vec<String>,
    /// `andar` atau `bahar` setelah dibagi.
    pub pemenang: Option<String>,
    pub taruhan_ronde: BTreeMap<String, i64>,
    pub bayar: BTreeMap<String, i64>,
}

impl MejaView for View {
    fn meja(&self) -> &Umum {
        &self.meja
    }
}

/// Hasil bersih taruhan yang menang: Andar 0,9:1, Bahar 1:1.
pub fn win(spot: &str, bet: i64) -> i64 {
    if spot == "andar" { bet * 9 / 10 } else { bet }
}

impl AndarBahar {
    fn deal(&mut self) -> Result<(), GameError> {
        let bets = self.spots.take();
        let middle = meja::draw(&mut self.deck)?;
        let (mut andar, mut bahar) = (Vec::new(), Vec::new());
        let winner = loop {
            let c = meja::draw(&mut self.deck)?;
            let to_andar = andar.len() == bahar.len();
            if to_andar {
                andar.push(c);
            } else {
                bahar.push(c);
            }
            if c.rank == middle.rank {
                break if to_andar { "andar" } else { "bahar" };
            }
        };
        let payouts = bets
            .iter()
            .map(|(spot, &b)| {
                let p = if spot == winner { win(spot, b) } else { -b };
                (spot.clone(), p)
            })
            .collect();
        self.round = Some(Round {
            middle,
            andar,
            bahar,
            winner,
            bets,
            payouts,
        });
        self.ended = Some("ronde_selesai");
        Ok(())
    }

    fn net(&self) -> i64 {
        self.round
            .as_ref()
            .map(|r| r.payouts.values().sum())
            .unwrap_or(0)
    }
}

impl TurnGame for AndarBahar {
    type Config = Config;
    type Action = Action;
    type View = View;

    fn new(config: Config, seed: Seed) -> Result<Self, GameError> {
        Ok(AndarBahar {
            deck: config.deck(seed)?,
            spots: Spots::new(SPOTS, STANDARD),
            round: None,
            ended: None,
        })
    }

    fn seats(&self) -> u8 {
        1
    }

    fn pending_players(&self) -> Vec<PlayerId> {
        if self.ended.is_some() {
            Vec::new()
        } else {
            vec![0]
        }
    }

    fn legal_actions(&self, player: PlayerId) -> Vec<ActionSpec> {
        if player != 0 || self.ended.is_some() {
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
        if self.ended.is_some() {
            return Err(GameError::Over);
        }
        if player != 0 {
            return Err(GameError::NotPending(player));
        }
        match action {
            Action::Bet(spot, n) if self.spots.place(spot, n) => Ok(()),
            Action::Bet(spot, n) => Err(GameError::Illegal(format!("bet {spot} {n}"))),
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
        let over = self.ended.is_some();
        let mut meja = Umum::new(if over { "selesai" } else { "taruhan" }, STANDARD);
        let r = self.round.as_ref();
        meja.bersih = self.net();
        meja.ronde = u32::from(r.is_some());
        meja.taruhan_meja = self.spots.total();
        meja.dipertaruhkan = r.map(|r| r.bets.values().sum()).unwrap_or(0);
        meja.selesai = over;
        meja.alasan = self.ended.map(str::to_string);
        let cards = |v: &[Card]| v.iter().map(Card::to_string).collect();
        View {
            meja,
            taruhan: self.spots.bets().clone(),
            tengah: r.map(|r| r.middle.to_string()),
            andar: r.map(|r| cards(&r.andar)).unwrap_or_default(),
            bahar: r.map(|r| cards(&r.bahar)).unwrap_or_default(),
            pemenang: r.map(|r| r.winner.to_string()),
            taruhan_ronde: r.map(|r| r.bets.clone()).unwrap_or_default(),
            bayar: r.map(|r| r.payouts.clone()).unwrap_or_default(),
        }
    }

    fn to_text(view: &View, lang: Lang) -> String {
        let c = catalog();
        let mut out = String::new();
        if let Some(middle) = &view.tengah {
            out.push_str(&c.text(lang, "middle", &[("card", middle)]));
            out.push('\n');
            out.push_str(&format!("andar: {}\n", view.andar.join(" ")));
            out.push_str(&format!("bahar: {}\n", view.bahar.join(" ")));
            for (spot, pay) in &view.bayar {
                out.push_str(&format!(
                    "  {spot} {} → {pay:+}\n",
                    view.taruhan_ronde.get(spot).copied().unwrap_or(0)
                ));
            }
        }
        let bets: Vec<String> = SPOTS
            .iter()
            .map(|s| format!("{s} {}", view.taruhan.get(*s).copied().unwrap_or(0)))
            .collect();
        out.push_str(&c.text(lang, "bets", &[("bets", &bets.join(" · "))]));
        out.push('\n');
        out.push_str(&match view.meja.alasan.as_deref() {
            Some(reason) => c.text(
                lang,
                &format!("over.{reason}"),
                &[("net", &format!("{:+}", view.meja.bersih))],
            ),
            None => c.text(lang, "place_bet", &[]),
        });
        out
    }

    fn is_over(&self) -> bool {
        self.ended.is_some()
    }

    fn result(&self) -> Option<GameResult> {
        self.ended.map(|_| {
            meja::result(
                catalog(),
                "summary",
                self.net(),
                u32::from(self.round.is_some()),
            )
        })
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
