//! Casino War (SPEC §6.3; M5a): kartu pemain lawan kartu bandar.
//!
//! Aturan (D-060): 6 dek, titik potong 75%, as tertinggi. `ante` 1:1;
//! `tie` 10:1 bila kartu pertama seri. Seri: `surrender` (kehilangan
//! setengah ante) atau `war` (raise sebesar ante; bandar membuang tiga
//! kartu, lalu satu kartu masing-masing). Setelah perang, kartu pemain
//! lebih tinggi atau sama: raise dibayar 1:1 dan ante kembali; kartu bandar
//! lebih tinggi: ante dan raise kalah. Satu sesi = satu shoe, provably
//! fair per shoe (D-056).

use std::collections::BTreeMap;
use std::sync::OnceLock;

use kyusin_core::game::{GameResult, create_session};
use kyusin_core::i18n::{Catalog, Lang};
use kyusin_core::{ActionSpec, Cartridge, GameError, PlayerId, RegistryError, Seed, TurnGame};
use serde::{Deserialize, Serialize};

use crate::cards::{Card, Shoe};
use crate::meja::{self, MejaView, STANDARD, ShoeConfig, Spots, Umum, WagerRtp};
use crate::poker::rank_value;

pub const ID: &str = "casino-war";
pub const DECKS: u8 = 6;
pub const PENETRATION: f64 = 0.75;
pub const SPOTS: &[&str] = &["ante", "tie"];
pub const TIE_PAYS: i64 = 10;
pub const BURN: u8 = 3;

pub type Config = ShoeConfig;

/// RTP tiap taruhan (analitis, 6 dek, selalu perang; D-060).
pub const RTP: &[WagerRtp] = &[
    WagerRtp {
        wager: "ante",
        percent: 97.123,
        manifest: true,
    },
    WagerRtp {
        wager: "tie",
        percent: 81.350,
        manifest: false,
    },
];

pub fn catalog() -> &'static Catalog {
    static CATALOG: OnceLock<Catalog> = OnceLock::new();
    CATALOG.get_or_init(|| {
        Catalog::from_toml(include_str!("i18n.toml")).expect("casino_war/i18n.toml tidak sah")
    })
}

pub fn cartridge() -> Result<Cartridge, RegistryError> {
    Cartridge::new(
        include_str!("manifest.toml"),
        include_str!("../../../../tutorials/casino-war.toml"),
        create_session::<CasinoWar>,
    )
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Action {
    Bet(&'static str, i64),
    Clear,
    Deal,
    War,
    Surrender,
    Leave,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
enum Phase {
    Betting,
    War,
    Over,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
struct Round {
    player: Vec<Card>,
    dealer: Vec<Card>,
    burned: u8,
    bets: BTreeMap<String, i64>,
    payouts: BTreeMap<String, i64>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct CasinoWar {
    shoe: Shoe,
    cut: usize,
    phase: Phase,
    spots: Spots,
    round: Option<Round>,
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
    /// Ronde berjalan atau terakhir: kartu pemain dan bandar (kartu kedua =
    /// kartu perang), jumlah kartu yang dibuang, taruhan, dan hasilnya.
    pub pemain: Vec<String>,
    pub bandar: Vec<String>,
    pub dibuang: u8,
    pub taruhan_ronde: BTreeMap<String, i64>,
    pub bayar: BTreeMap<String, i64>,
}

impl MejaView for View {
    fn meja(&self) -> &Umum {
        &self.meja
    }
}

fn value(c: &Card) -> u8 {
    rank_value(c.rank)
}

impl CasinoWar {
    fn ante(&self) -> i64 {
        self.round
            .as_ref()
            .and_then(|r| r.bets.get("ante").copied())
            .unwrap_or(0)
    }

    fn deal(&mut self) -> Result<(), GameError> {
        let bets = self.spots.take();
        let p = meja::draw(&mut self.shoe)?;
        let d = meja::draw(&mut self.shoe)?;
        self.rounds += 1;
        let ante = bets.get("ante").copied().unwrap_or(0);
        let mut payouts = BTreeMap::new();
        let tie = value(&p) == value(&d);
        if let Some(&t) = bets.get("tie") {
            payouts.insert("tie".to_string(), if tie { TIE_PAYS * t } else { -t });
        }
        if !tie {
            payouts.insert(
                "ante".to_string(),
                if value(&p) > value(&d) { ante } else { -ante },
            );
        }
        self.round = Some(Round {
            player: vec![p],
            dealer: vec![d],
            burned: 0,
            bets,
            payouts,
        });
        if tie {
            self.phase = Phase::War;
            Ok(())
        } else {
            self.finish();
            Ok(())
        }
    }

    fn war(&mut self) -> Result<(), GameError> {
        let ante = self.ante();
        for _ in 0..BURN {
            meja::draw(&mut self.shoe)?;
        }
        let p = meja::draw(&mut self.shoe)?;
        let d = meja::draw(&mut self.shoe)?;
        let r = self.round.as_mut().expect("ronde berjalan");
        r.burned = BURN;
        r.player.push(p);
        r.dealer.push(d);
        r.bets.insert("raise".into(), ante);
        let (a, raise) = if value(&p) >= value(&d) {
            (0, ante)
        } else {
            (-ante, -ante)
        };
        r.payouts.insert("ante".into(), a);
        r.payouts.insert("raise".into(), raise);
        self.finish();
        Ok(())
    }

    fn finish(&mut self) {
        if let Some(r) = &self.round {
            self.net += r.payouts.values().sum::<i64>();
        }
        if self.shoe.dealt() >= self.cut {
            self.phase = Phase::Over;
            self.ended = Some("shoe_habis");
        } else {
            self.phase = Phase::Betting;
        }
    }
}

impl TurnGame for CasinoWar {
    type Config = Config;
    type Action = Action;
    type View = View;

    fn new(config: Config, seed: Seed) -> Result<Self, GameError> {
        let (shoe, cut) = config.build(DECKS, PENETRATION, seed)?;
        Ok(CasinoWar {
            shoe,
            cut,
            phase: Phase::Betting,
            spots: Spots::new(SPOTS, STANDARD),
            round: None,
            net: 0,
            rounds: 0,
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
            Phase::Betting => {
                let mut out = self.spots.legal();
                if !self.spots.is_empty() {
                    out.push(ActionSpec::fixed("clear"));
                }
                if self.spots.get("ante") > 0 {
                    out.push(ActionSpec::fixed("deal"));
                }
                out.push(ActionSpec::fixed("leave"));
                out
            }
            Phase::War => vec![ActionSpec::fixed("war"), ActionSpec::fixed("surrender")],
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
        let illegal = |a: &Action, g: &CasinoWar| GameError::Illegal(g.format_action(a));
        match (self.phase, &action) {
            (Phase::Betting, Action::Bet(spot, n)) => {
                if self.spots.place(spot, *n) {
                    Ok(())
                } else {
                    Err(illegal(&action, self))
                }
            }
            (Phase::Betting, Action::Clear) => {
                self.spots.clear();
                Ok(())
            }
            (Phase::Betting, Action::Deal) if self.spots.get("ante") > 0 => self.deal(),
            (Phase::Betting, Action::Leave) => {
                self.spots.clear();
                self.phase = Phase::Over;
                self.ended = Some("berhenti");
                Ok(())
            }
            (Phase::War, Action::War) => self.war(),
            (Phase::War, Action::Surrender) => {
                let ante = self.ante();
                if let Some(r) = self.round.as_mut() {
                    r.payouts.insert("ante".into(), -ante / 2);
                }
                self.finish();
                Ok(())
            }
            _ => Err(illegal(&action, self)),
        }
    }

    fn view_for(&self, _player: PlayerId) -> View {
        let fase = match self.phase {
            Phase::Betting => "taruhan",
            Phase::War => "perang",
            Phase::Over => "selesai",
        };
        let mut meja = Umum::new(fase, STANDARD).shoe(&self.shoe, self.cut);
        let r = self.round.as_ref();
        meja.bersih = self.net;
        meja.ronde = self.rounds;
        meja.taruhan_meja = match self.phase {
            Phase::Betting => self.spots.total(),
            Phase::War => r.map(|r| r.bets.values().sum()).unwrap_or(0),
            Phase::Over => 0,
        };
        meja.dipertaruhkan = r.map(|r| r.bets.values().sum()).unwrap_or(0);
        if self.phase == Phase::War {
            meja.biaya.insert("war".into(), self.ante());
            meja.netral = Some("surrender".into());
        }
        meja.selesai = self.phase == Phase::Over;
        meja.alasan = self.ended.map(str::to_string);
        let cards = |v: Option<&Vec<Card>>| {
            v.map(|c| c.iter().map(Card::to_string).collect())
                .unwrap_or_default()
        };
        View {
            meja,
            taruhan: self.spots.bets().clone(),
            pemain: cards(r.map(|r| &r.player)),
            bandar: cards(r.map(|r| &r.dealer)),
            dibuang: r.map(|r| r.burned).unwrap_or(0),
            taruhan_ronde: r.map(|r| r.bets.clone()).unwrap_or_default(),
            bayar: r.map(|r| r.payouts.clone()).unwrap_or_default(),
        }
    }

    fn to_text(view: &View, lang: Lang) -> String {
        let c = catalog();
        let mut out = String::new();
        if !view.pemain.is_empty() {
            out.push_str(&c.text(
                lang,
                "cards",
                &[
                    ("player", &view.pemain.join(" ")),
                    ("dealer", &view.bandar.join(" ")),
                ],
            ));
            out.push('\n');
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
        out.push_str(&c.text(
            lang,
            "net",
            &[
                ("net", &format!("{:+}", view.meja.bersih)),
                ("left", &view.meja.sisa.unwrap_or(0).to_string()),
            ],
        ));
        out.push('\n');
        out.push_str(
            &match (view.meja.fase.as_str(), view.meja.alasan.as_deref()) {
                ("perang", _) => c.text(lang, "war", &[]),
                (_, Some(reason)) => c.text(lang, &format!("over.{reason}"), &[]),
                _ => c.text(lang, "place_bet", &[]),
            },
        );
        out
    }

    fn is_over(&self) -> bool {
        self.phase == Phase::Over
    }

    fn result(&self) -> Option<GameResult> {
        (self.phase == Phase::Over)
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
            ["war"] => Ok(Action::War),
            ["surrender"] => Ok(Action::Surrender),
            ["leave"] => Ok(Action::Leave),
            _ => Err(err()),
        }
    }

    fn format_action(&self, action: &Action) -> String {
        match action {
            Action::Bet(spot, n) => format!("bet {spot} {n}"),
            Action::Clear => "clear".into(),
            Action::Deal => "deal".into(),
            Action::War => "war".into(),
            Action::Surrender => "surrender".into(),
            Action::Leave => "leave".into(),
        }
    }
}
