//! Red Dog (SPEC §6.3; M5a): tebak apakah kartu ketiga jatuh di antara dua
//! kartu.
//!
//! Aturan (D-060): 8 dek, titik potong 75%, as tertinggi. Berurutan: seri.
//! Sepasang: kartu ketiga; three of a kind 11:1, selain itu seri. Selain
//! itu pemain boleh `raise` (menggandakan taruhan) atau `call`; kartu ketiga
//! di antara keduanya dibayar menurut jarak (1 → 5:1, 2 → 4:1, 3 → 2:1,
//! 4–11 → 1:1). Satu sesi = satu shoe, provably fair per shoe (D-056).

use std::sync::OnceLock;

use kyusin_core::game::{GameResult, create_session};
use kyusin_core::i18n::{Catalog, Lang};
use kyusin_core::{ActionSpec, Cartridge, GameError, PlayerId, RegistryError, Seed, TurnGame};
use serde::{Deserialize, Serialize};

use crate::cards::{Card, Shoe};
use crate::meja::{self, MejaView, STANDARD, ShoeConfig, Umum, WagerRtp};
use crate::poker::rank_value;

pub const ID: &str = "red-dog";
pub const DECKS: u8 = 8;
pub const PENETRATION: f64 = 0.75;
pub const TRIPS_PAYS: i64 = 11;

pub type Config = ShoeConfig;

/// RTP dengan strategi raise pada jarak 7 ke atas (analitis, 8 dek; D-060).
pub const RTP: &[WagerRtp] = &[WagerRtp {
    wager: "bet",
    percent: 97.2491,
    manifest: true,
}];

pub fn catalog() -> &'static Catalog {
    static CATALOG: OnceLock<Catalog> = OnceLock::new();
    CATALOG.get_or_init(|| {
        Catalog::from_toml(include_str!("i18n.toml")).expect("red_dog/i18n.toml tidak sah")
    })
}

pub fn cartridge() -> Result<Cartridge, RegistryError> {
    Cartridge::new(
        include_str!("manifest.toml"),
        include_str!("../../../../tutorials/red-dog.toml"),
        create_session::<RedDog>,
    )
}

/// Pembayaran menurut jarak (spread): 1 → 5, 2 → 4, 3 → 2, 4+ → 1.
pub fn spread_pays(spread: u8) -> i64 {
    match spread {
        1 => 5,
        2 => 4,
        3 => 2,
        _ => 1,
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Action {
    Bet(i64),
    Raise,
    Call,
    Leave,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
enum Phase {
    Betting,
    Raise,
    Over,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
struct Round {
    cards: Vec<Card>,
    bet: i64,
    raised: bool,
    outcome: Option<&'static str>,
    payout: Option<i64>,
}

impl Round {
    fn spread(&self) -> Option<u8> {
        let (a, b) = (
            rank_value(self.cards[0].rank),
            rank_value(self.cards[1].rank),
        );
        let (lo, hi) = (a.min(b), a.max(b));
        (hi > lo + 1).then(|| hi - lo - 1)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct RedDog {
    shoe: Shoe,
    cut: usize,
    phase: Phase,
    round: Option<Round>,
    net: i64,
    rounds: u32,
    ended: Option<&'static str>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct View {
    #[serde(flatten)]
    pub meja: Umum,
    /// Kartu ronde berjalan atau terakhir.
    pub kartu: Vec<String>,
    /// Taruhan ronde itu (termasuk raise).
    pub taruhan: i64,
    pub naik: bool,
    /// Jarak dan pengali pembayarannya, bila ada jarak.
    pub jarak: Option<u8>,
    pub kali: Option<i64>,
    /// `seri`, `tiga`, `menang`, atau `kalah` setelah ronde selesai.
    pub hasil: Option<String>,
    pub bayar: Option<i64>,
}

impl MejaView for View {
    fn meja(&self) -> &Umum {
        &self.meja
    }
}

impl RedDog {
    fn finish(&mut self, outcome: &'static str, payout: i64) {
        if let Some(r) = self.round.as_mut() {
            r.outcome = Some(outcome);
            r.payout = Some(payout);
        }
        self.net += payout;
        if self.shoe.dealt() >= self.cut {
            self.phase = Phase::Over;
            self.ended = Some("shoe_habis");
        } else {
            self.phase = Phase::Betting;
        }
    }

    fn deal(&mut self, bet: i64) -> Result<(), GameError> {
        let a = meja::draw(&mut self.shoe)?;
        let b = meja::draw(&mut self.shoe)?;
        self.rounds += 1;
        self.round = Some(Round {
            cards: vec![a, b],
            bet,
            raised: false,
            outcome: None,
            payout: None,
        });
        let (x, y) = (rank_value(a.rank), rank_value(b.rank));
        if x == y {
            let c = meja::draw(&mut self.shoe)?;
            if let Some(r) = self.round.as_mut() {
                r.cards.push(c);
            }
            if rank_value(c.rank) == x {
                self.finish("tiga", TRIPS_PAYS * bet);
            } else {
                self.finish("seri", 0);
            }
        } else if x.abs_diff(y) == 1 {
            self.finish("seri", 0);
        } else {
            self.phase = Phase::Raise;
        }
        Ok(())
    }

    fn third(&mut self, raise: bool) -> Result<(), GameError> {
        let c = meja::draw(&mut self.shoe)?;
        let r = self.round.as_mut().expect("ronde berjalan");
        if raise {
            r.bet *= 2;
            r.raised = true;
        }
        r.cards.push(c);
        let (a, b) = (rank_value(r.cards[0].rank), rank_value(r.cards[1].rank));
        let (lo, hi) = (a.min(b), a.max(b));
        let v = rank_value(c.rank);
        let spread = r.spread().unwrap_or(0);
        let bet = r.bet;
        if v > lo && v < hi {
            self.finish("menang", spread_pays(spread) * bet);
        } else {
            self.finish("kalah", -bet);
        }
        Ok(())
    }
}

impl TurnGame for RedDog {
    type Config = Config;
    type Action = Action;
    type View = View;

    fn new(config: Config, seed: Seed) -> Result<Self, GameError> {
        let (shoe, cut) = config.build(DECKS, PENETRATION, seed)?;
        Ok(RedDog {
            shoe,
            cut,
            phase: Phase::Betting,
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
            Phase::Betting => vec![STANDARD.single(), ActionSpec::fixed("leave")],
            Phase::Raise => vec![ActionSpec::fixed("raise"), ActionSpec::fixed("call")],
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
            (Phase::Betting, Action::Bet(n)) if STANDARD.accepts(n) => self.deal(n),
            (Phase::Betting, Action::Leave) => {
                self.phase = Phase::Over;
                self.ended = Some("berhenti");
                Ok(())
            }
            (Phase::Raise, Action::Raise) => self.third(true),
            (Phase::Raise, Action::Call) => self.third(false),
            _ => Err(GameError::Illegal(self.format_action(&action))),
        }
    }

    fn view_for(&self, _player: PlayerId) -> View {
        let fase = match self.phase {
            Phase::Betting => "taruhan",
            Phase::Raise => "naikkan",
            Phase::Over => "selesai",
        };
        let mut meja = Umum::new(fase, STANDARD).shoe(&self.shoe, self.cut);
        let r = self.round.as_ref();
        meja.bersih = self.net;
        meja.ronde = self.rounds;
        meja.taruhan_meja = if self.phase == Phase::Raise {
            r.map(|r| r.bet).unwrap_or(0)
        } else {
            0
        };
        meja.dipertaruhkan = r.map(|r| r.bet).unwrap_or(0);
        if self.phase == Phase::Raise {
            meja.biaya
                .insert("raise".into(), r.map(|r| r.bet).unwrap_or(0));
            meja.netral = Some("call".into());
        }
        meja.selesai = self.phase == Phase::Over;
        meja.alasan = self.ended.map(str::to_string);
        let spread = r.and_then(Round::spread);
        View {
            meja,
            kartu: r
                .map(|r| r.cards.iter().map(Card::to_string).collect())
                .unwrap_or_default(),
            taruhan: r.map(|r| r.bet).unwrap_or(0),
            naik: r.is_some_and(|r| r.raised),
            jarak: spread,
            kali: spread.map(spread_pays),
            hasil: r.and_then(|r| r.outcome.map(str::to_string)),
            bayar: r.and_then(|r| r.payout),
        }
    }

    fn to_text(view: &View, lang: Lang) -> String {
        let c = catalog();
        let mut out = String::new();
        if !view.kartu.is_empty() {
            out.push_str(&c.text(lang, "cards", &[("cards", &view.kartu.join(" "))]));
            if let (Some(s), Some(k)) = (view.jarak, view.kali) {
                out.push_str(&c.text(
                    lang,
                    "spread",
                    &[("spread", &s.to_string()), ("pays", &k.to_string())],
                ));
            }
            out.push('\n');
            if let (Some(h), Some(b)) = (&view.hasil, view.bayar) {
                out.push_str(&format!(
                    "{} {b:+}\n",
                    c.text(lang, &format!("result.{h}"), &[])
                ));
            }
        }
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
                ("naikkan", _) => c.text(lang, "raise", &[]),
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
            ["bet", n] => Ok(Action::Bet(meja::amount(n).ok_or_else(err)?)),
            ["raise"] => Ok(Action::Raise),
            ["call"] => Ok(Action::Call),
            ["leave"] => Ok(Action::Leave),
            _ => Err(err()),
        }
    }

    fn format_action(&self, action: &Action) -> String {
        match action {
            Action::Bet(n) => format!("bet {n}"),
            Action::Raise => "raise".into(),
            Action::Call => "call".into(),
            Action::Leave => "leave".into(),
        }
    }
}
