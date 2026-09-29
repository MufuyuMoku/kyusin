//! Baccarat Punto Banco (SPEC §6.3; M5a).
//!
//! Aturan (D-060): 8 dek, titik potong 16 kartu sebelum akhir shoe.
//! `player` 1:1, `banker` 0,95:1 (komisi 5%), `tie` 8:1 (player/banker
//! kembali bila seri). Taruhan 20–2.000, kelipatan 20, supaya komisi selalu
//! utuh. Kartu ketiga mengikuti tabel baku (lihat [`banker_draws`]). Satu
//! sesi = satu shoe, provably fair per shoe (D-056).

use std::collections::BTreeMap;
use std::sync::OnceLock;

use kyusin_core::game::{GameResult, create_session};
use kyusin_core::i18n::{Catalog, Lang};
use kyusin_core::{ActionSpec, Cartridge, GameError, PlayerId, RegistryError, Seed, TurnGame};
use serde::{Deserialize, Serialize};

use crate::cards::{Card, Rank, Shoe};
use crate::meja::{self, Limits, MejaView, ShoeConfig, Spots, Umum, WagerRtp};

pub const ID: &str = "baccarat";
pub const DECKS: u8 = 8;
/// Shoe selesai di akhir ronde setelah tersisa 16 kartu atau kurang.
pub const CUT_FROM_END: usize = 16;
pub const SPOTS: &[&str] = &["player", "banker", "tie"];
pub const LIMITS: Limits = Limits {
    min: 20,
    max: 2000,
    step: 20,
};
pub const TIE_PAYS: i64 = 8;

pub type Config = ShoeConfig;

/// RTP tiap taruhan (enumerasi tepat 8 dek; D-060). Manifest = player.
pub const RTP: &[WagerRtp] = &[
    WagerRtp {
        wager: "player",
        percent: 98.7649,
        manifest: true,
    },
    WagerRtp {
        wager: "banker",
        percent: 98.9421,
        manifest: false,
    },
    WagerRtp {
        wager: "tie",
        percent: 85.6404,
        manifest: false,
    },
];

pub fn catalog() -> &'static Catalog {
    static CATALOG: OnceLock<Catalog> = OnceLock::new();
    CATALOG.get_or_init(|| {
        Catalog::from_toml(include_str!("i18n.toml")).expect("baccarat/i18n.toml tidak sah")
    })
}

pub fn cartridge() -> Result<Cartridge, RegistryError> {
    Cartridge::new(
        include_str!("manifest.toml"),
        include_str!("../../../../tutorials/baccarat.toml"),
        create_session::<Baccarat>,
    )
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Action {
    Bet(&'static str, i64),
    Clear,
    Deal,
    Leave,
}

/// Nilai kartu baccarat: as 1, 2–9, 10/J/Q/K 0.
pub fn card_value(c: Card) -> u8 {
    match c.rank {
        Rank::Ace => 1,
        Rank::Ten | Rank::Jack | Rank::Queen | Rank::King => 0,
        r => r as u8 + 1,
    }
}

pub fn total(cards: &[Card]) -> u8 {
    cards.iter().map(|c| card_value(*c)).sum::<u8>() % 10
}

/// Apakah bankir menarik kartu ketiga, dengan total `banker` dan kartu
/// ketiga pemain `p3` (`None` bila pemain berdiri).
pub fn banker_draws(banker: u8, p3: Option<u8>) -> bool {
    match p3 {
        None => banker <= 5,
        Some(p) => match banker {
            0..=2 => true,
            3 => p != 8,
            4 => (2..=7).contains(&p),
            5 => (4..=7).contains(&p),
            6 => (6..=7).contains(&p),
            _ => false,
        },
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
struct Round {
    player: Vec<Card>,
    banker: Vec<Card>,
    winner: &'static str,
    bets: BTreeMap<String, i64>,
    payouts: BTreeMap<String, i64>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Baccarat {
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
    pub taruhan: BTreeMap<String, i64>,
    /// Ronde terakhir.
    pub pemain: Vec<String>,
    pub bankir: Vec<String>,
    pub nilai_pemain: Option<u8>,
    pub nilai_bankir: Option<u8>,
    /// `player`, `banker`, atau `tie`.
    pub pemenang: Option<String>,
    pub taruhan_terakhir: BTreeMap<String, i64>,
    pub bayar: BTreeMap<String, i64>,
}

impl MejaView for View {
    fn meja(&self) -> &Umum {
        &self.meja
    }
}

impl Baccarat {
    fn over(&self) -> bool {
        self.ended.is_some()
    }

    fn deal(&mut self) -> Result<(), GameError> {
        let mut player = Vec::with_capacity(3);
        let mut banker = Vec::with_capacity(3);
        for _ in 0..2 {
            player.push(meja::draw(&mut self.shoe)?);
            banker.push(meja::draw(&mut self.shoe)?);
        }
        let (p, b) = (total(&player), total(&banker));
        if p < 8 && b < 8 {
            let p3 = if p <= 5 {
                let c = meja::draw(&mut self.shoe)?;
                player.push(c);
                Some(card_value(c))
            } else {
                None
            };
            if banker_draws(b, p3) {
                banker.push(meja::draw(&mut self.shoe)?);
            }
        }
        let (p, b) = (total(&player), total(&banker));
        let winner = match p.cmp(&b) {
            std::cmp::Ordering::Greater => "player",
            std::cmp::Ordering::Less => "banker",
            std::cmp::Ordering::Equal => "tie",
        };
        let bets = self.spots.take();
        let payouts: BTreeMap<String, i64> = bets
            .iter()
            .map(|(spot, &bet)| {
                let pay = match (spot.as_str(), winner) {
                    ("tie", "tie") => TIE_PAYS * bet,
                    ("tie", _) => -bet,
                    (_, "tie") => 0,
                    ("player", "player") => bet,
                    ("banker", "banker") => bet * 19 / 20,
                    _ => -bet,
                };
                (spot.clone(), pay)
            })
            .collect();
        self.net += payouts.values().sum::<i64>();
        self.rounds += 1;
        self.last = Some(Round {
            player,
            banker,
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

impl TurnGame for Baccarat {
    type Config = Config;
    type Action = Action;
    type View = View;

    fn new(config: Config, seed: Seed) -> Result<Self, GameError> {
        let (shoe, cut) = config.build(DECKS, 1.0, seed)?;
        let cut = match config.potong {
            Some(c) => c,
            None => cut.saturating_sub(CUT_FROM_END),
        };
        Ok(Baccarat {
            shoe,
            cut,
            spots: Spots::new(SPOTS, LIMITS),
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
        let mut meja = Umum::new(if self.over() { "selesai" } else { "taruhan" }, LIMITS)
            .shoe(&self.shoe, self.cut);
        let r = self.last.as_ref();
        meja.bersih = self.net;
        meja.ronde = self.rounds;
        meja.taruhan_meja = self.spots.total();
        meja.dipertaruhkan = r.map(|r| r.bets.values().sum()).unwrap_or(0);
        meja.selesai = self.over();
        meja.alasan = self.ended.map(str::to_string);
        let cards = |v: &[Card]| v.iter().map(Card::to_string).collect();
        View {
            meja,
            taruhan: self.spots.bets().clone(),
            pemain: r.map(|r| cards(&r.player)).unwrap_or_default(),
            bankir: r.map(|r| cards(&r.banker)).unwrap_or_default(),
            nilai_pemain: r.map(|r| total(&r.player)),
            nilai_bankir: r.map(|r| total(&r.banker)),
            pemenang: r.map(|r| r.winner.to_string()),
            taruhan_terakhir: r.map(|r| r.bets.clone()).unwrap_or_default(),
            bayar: r.map(|r| r.payouts.clone()).unwrap_or_default(),
        }
    }

    fn to_text(view: &View, lang: Lang) -> String {
        let c = catalog();
        let mut out = String::new();
        if let (Some(p), Some(b), Some(w)) = (view.nilai_pemain, view.nilai_bankir, &view.pemenang)
        {
            out.push_str(&c.text(
                lang,
                "cards",
                &[
                    ("player", &format!("{} = {p}", view.pemain.join(" "))),
                    ("banker", &format!("{} = {b}", view.bankir.join(" "))),
                    ("winner", &c.text(lang, &format!("winner.{w}"), &[])),
                ],
            ));
            out.push('\n');
            for (spot, pay) in &view.bayar {
                out.push_str(&format!(
                    "  {spot} {} → {pay:+}\n",
                    view.taruhan_terakhir.get(spot).copied().unwrap_or(0)
                ));
            }
        }
        let bets: Vec<String> = SPOTS
            .iter()
            .map(|s| format!("{s} {}", view.taruhan.get(*s).copied().unwrap_or(0)))
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
