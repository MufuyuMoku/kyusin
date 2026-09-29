//! Three Card Poker (SPEC §6.3; M5a): Ante/Play dan Pair Plus.
//!
//! Aturan (D-060): satu dek per ronde. Bandar memenuhi syarat dengan Q-high
//! atau lebih. Ante bonus 1-4-5 (straight, three of a kind, straight
//! flush); Pair Plus 1-3-6-30-40 (pair, flush, straight, three of a kind,
//! straight flush). Satu sesi = satu ronde (commit-reveal per ronde).

use std::collections::BTreeMap;
use std::sync::OnceLock;

use kyusin_core::game::{GameResult, create_session};
use kyusin_core::i18n::{Catalog, Lang};
use kyusin_core::{ActionSpec, Cartridge, GameError, PlayerId, RegistryError, Seed, TurnGame};
use serde::{Deserialize, Serialize};

use crate::cards::{Card, Shoe};
use crate::meja::{self, MejaView, RoundConfig, STANDARD, Spots, Umum, WagerRtp};
use crate::poker::{Category3, Value3, eval3};

pub const ID: &str = "three-card-poker";
pub const SPOTS: &[&str] = &["ante", "pairplus"];

pub type Config = RoundConfig;

/// RTP tiap taruhan (D-060). Ante/Play: enumerasi tepat semua pasangan
/// tangan dengan strategi Q-6-4 (workflow `rtp.yml`); Pair Plus: analitis.
pub const RTP: &[WagerRtp] = &[
    WagerRtp {
        wager: "ante",
        percent: 96.6264,
        manifest: true,
    },
    WagerRtp {
        wager: "pairplus",
        percent: 92.7240,
        manifest: false,
    },
];

pub fn catalog() -> &'static Catalog {
    static CATALOG: OnceLock<Catalog> = OnceLock::new();
    CATALOG.get_or_init(|| {
        Catalog::from_toml(include_str!("i18n.toml")).expect("three_card_poker/i18n.toml tidak sah")
    })
}

pub fn cartridge() -> Result<Cartridge, RegistryError> {
    Cartridge::new(
        include_str!("manifest.toml"),
        include_str!("../../../../tutorials/three-card-poker.toml"),
        create_session::<ThreeCardPoker>,
    )
}

/// Ante bonus (kali ante) untuk tangan pemain yang bermain.
pub fn ante_bonus(c: Category3) -> i64 {
    match c {
        Category3::Straight => 1,
        Category3::Trips => 4,
        Category3::StraightFlush => 5,
        _ => 0,
    }
}

/// Pembayaran Pair Plus (kali taruhan); 0 = kalah.
pub fn pair_plus(c: Category3) -> i64 {
    match c {
        Category3::Pair => 1,
        Category3::Flush => 3,
        Category3::Straight => 6,
        Category3::Trips => 30,
        Category3::StraightFlush => 40,
        Category3::HighCard => 0,
    }
}

/// Bandar memenuhi syarat: Q-high atau lebih.
pub fn qualifies(v: Value3) -> bool {
    v.category() > Category3::HighCard || v.rank(0) >= 12
}

/// Hasil ante + play (tanpa bonus) untuk ante `a` bila pemain bermain.
pub fn showdown(player: Value3, dealer: Value3, a: i64) -> (i64, i64) {
    if !qualifies(dealer) {
        return (a, 0);
    }
    match player.cmp(&dealer) {
        std::cmp::Ordering::Greater => (a, a),
        std::cmp::Ordering::Less => (-a, -a),
        std::cmp::Ordering::Equal => (0, 0),
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Action {
    Bet(&'static str, i64),
    Clear,
    Deal,
    Play,
    Fold,
    Leave,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
enum Phase {
    Betting,
    Decide,
    Over,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
struct Round {
    player: Vec<Card>,
    dealer: Vec<Card>,
    revealed: bool,
    bets: BTreeMap<String, i64>,
    payouts: BTreeMap<String, i64>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ThreeCardPoker {
    deck: Shoe,
    phase: Phase,
    spots: Spots,
    round: Option<Round>,
    ended: Option<&'static str>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct View {
    #[serde(flatten)]
    pub meja: Umum,
    pub taruhan: BTreeMap<String, i64>,
    pub pemain: Vec<String>,
    /// `??` sampai dibuka.
    pub bandar: Vec<String>,
    pub tangan_pemain: Option<String>,
    pub tangan_bandar: Option<String>,
    /// Bandar memenuhi syarat (Q-high), setelah dibuka.
    pub memenuhi: Option<bool>,
    pub taruhan_ronde: BTreeMap<String, i64>,
    pub bayar: BTreeMap<String, i64>,
}

impl MejaView for View {
    fn meja(&self) -> &Umum {
        &self.meja
    }
}

impl ThreeCardPoker {
    fn deal(&mut self) -> Result<(), GameError> {
        let bets = self.spots.take();
        let mut player = Vec::with_capacity(3);
        let mut dealer = Vec::with_capacity(3);
        for _ in 0..3 {
            player.push(meja::draw(&mut self.deck)?);
        }
        for _ in 0..3 {
            dealer.push(meja::draw(&mut self.deck)?);
        }
        let mut payouts = BTreeMap::new();
        if let Some(&pp) = bets.get("pairplus") {
            let k = pair_plus(eval3(&player).category());
            payouts.insert("pairplus".into(), if k > 0 { k * pp } else { -pp });
        }
        let ante = bets.contains_key("ante");
        self.round = Some(Round {
            player,
            dealer,
            revealed: !ante,
            bets,
            payouts,
        });
        if ante {
            self.phase = Phase::Decide;
        } else {
            self.end();
        }
        Ok(())
    }

    fn decide(&mut self, play: bool) {
        let r = self.round.as_mut().expect("ronde berjalan");
        r.revealed = true;
        let a = r.bets.get("ante").copied().unwrap_or(0);
        if play {
            r.bets.insert("play".into(), a);
            let (p, d) = (eval3(&r.player), eval3(&r.dealer));
            let (ante, play) = showdown(p, d, a);
            r.payouts.insert("ante".into(), ante);
            r.payouts.insert("play".into(), play);
            let bonus = ante_bonus(p.category());
            if bonus > 0 {
                r.payouts.insert("bonus".into(), bonus * a);
            }
        } else {
            r.payouts.insert("ante".into(), -a);
        }
        self.end();
    }

    fn end(&mut self) {
        self.phase = Phase::Over;
        self.ended = Some("ronde_selesai");
    }

    fn net(&self) -> i64 {
        match (&self.round, self.phase) {
            (Some(r), Phase::Over) => r.payouts.values().sum(),
            _ => 0,
        }
    }
}

impl TurnGame for ThreeCardPoker {
    type Config = Config;
    type Action = Action;
    type View = View;

    fn new(config: Config, seed: Seed) -> Result<Self, GameError> {
        Ok(ThreeCardPoker {
            deck: config.deck(seed)?,
            phase: Phase::Betting,
            spots: Spots::new(SPOTS, STANDARD),
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
            Phase::Betting => {
                let mut out = self.spots.legal();
                if !self.spots.is_empty() {
                    out.push(ActionSpec::fixed("clear"));
                    out.push(ActionSpec::fixed("deal"));
                }
                out.push(ActionSpec::fixed("leave"));
                out
            }
            Phase::Decide => vec![ActionSpec::fixed("play"), ActionSpec::fixed("fold")],
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
        match (self.phase, &action) {
            (Phase::Betting, Action::Bet(spot, n)) if self.spots.place(spot, *n) => Ok(()),
            (Phase::Betting, Action::Clear) => {
                self.spots.clear();
                Ok(())
            }
            (Phase::Betting, Action::Deal) if !self.spots.is_empty() => self.deal(),
            (Phase::Betting, Action::Leave) => {
                self.spots.clear();
                self.phase = Phase::Over;
                self.ended = Some("berhenti");
                Ok(())
            }
            (Phase::Decide, Action::Play) => {
                self.decide(true);
                Ok(())
            }
            (Phase::Decide, Action::Fold) => {
                self.decide(false);
                Ok(())
            }
            _ => Err(GameError::Illegal(self.format_action(&action))),
        }
    }

    fn view_for(&self, _player: PlayerId) -> View {
        let fase = match self.phase {
            Phase::Betting => "taruhan",
            Phase::Decide => "keputusan",
            Phase::Over => "selesai",
        };
        let mut meja = Umum::new(fase, STANDARD);
        let r = self.round.as_ref();
        meja.bersih = self.net();
        meja.ronde = u32::from(r.is_some());
        meja.taruhan_meja = match self.phase {
            Phase::Betting => self.spots.total(),
            Phase::Decide => r.map(|r| r.bets.values().sum()).unwrap_or(0),
            Phase::Over => 0,
        };
        meja.dipertaruhkan = r.map(|r| r.bets.values().sum()).unwrap_or(0);
        if self.phase == Phase::Decide {
            let a = r.and_then(|r| r.bets.get("ante").copied()).unwrap_or(0);
            meja.biaya.insert("play".into(), a);
            meja.netral = Some("fold".into());
        }
        meja.selesai = self.phase == Phase::Over;
        meja.alasan = self.ended.map(str::to_string);
        let revealed = r.is_some_and(|r| r.revealed);
        View {
            meja,
            taruhan: self.spots.bets().clone(),
            pemain: r
                .map(|r| r.player.iter().map(Card::to_string).collect())
                .unwrap_or_default(),
            bandar: r
                .map(|r| {
                    r.dealer
                        .iter()
                        .map(|c| if revealed { c.to_string() } else { "??".into() })
                        .collect()
                })
                .unwrap_or_default(),
            tangan_pemain: r.map(|r| eval3(&r.player).category().key().to_string()),
            tangan_bandar: r
                .filter(|_| revealed)
                .map(|r| eval3(&r.dealer).category().key().to_string()),
            memenuhi: r.filter(|_| revealed).map(|r| qualifies(eval3(&r.dealer))),
            taruhan_ronde: r.map(|r| r.bets.clone()).unwrap_or_default(),
            bayar: r
                .filter(|_| self.phase == Phase::Over)
                .map(|r| r.payouts.clone())
                .unwrap_or_default(),
        }
    }

    fn to_text(view: &View, lang: Lang) -> String {
        let c = catalog();
        let mut out = String::new();
        if !view.pemain.is_empty() {
            let hand = |k: &Option<String>| {
                k.as_ref()
                    .map(|k| format!(" ({})", c.text(lang, &format!("hand.{k}"), &[])))
                    .unwrap_or_default()
            };
            out.push_str(&c.text(
                lang,
                "cards",
                &[
                    (
                        "player",
                        &format!("{}{}", view.pemain.join(" "), hand(&view.tangan_pemain)),
                    ),
                    (
                        "dealer",
                        &format!("{}{}", view.bandar.join(" "), hand(&view.tangan_bandar)),
                    ),
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
        out.push_str(
            &match (view.meja.fase.as_str(), view.meja.alasan.as_deref()) {
                ("keputusan", _) => c.text(lang, "decide", &[]),
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
            ["play"] => Ok(Action::Play),
            ["fold"] => Ok(Action::Fold),
            ["leave"] => Ok(Action::Leave),
            _ => Err(err()),
        }
    }

    fn format_action(&self, action: &Action) -> String {
        match action {
            Action::Bet(spot, n) => format!("bet {spot} {n}"),
            Action::Clear => "clear".into(),
            Action::Deal => "deal".into(),
            Action::Play => "play".into(),
            Action::Fold => "fold".into(),
            Action::Leave => "leave".into(),
        }
    }
}
