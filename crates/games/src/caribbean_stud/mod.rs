//! Caribbean Stud Poker (SPEC §6.3; M5a): lima kartu lawan bandar.
//!
//! Aturan (D-060): satu dek per ronde; bandar membuka satu kartu. Raise =
//! dua kali ante. Bandar memenuhi syarat dengan A-K atau lebih. Tabel raise
//! baku AS: 1-2-3-4-5-7-20-50-100. Tanpa jackpot progresif. Satu sesi =
//! satu ronde (commit-reveal per ronde).

use std::collections::BTreeMap;
use std::sync::OnceLock;

use kyusin_core::game::{GameResult, create_session};
use kyusin_core::i18n::{Catalog, Lang};
use kyusin_core::{ActionSpec, Cartridge, GameError, PlayerId, RegistryError, Seed, TurnGame};
use serde::{Deserialize, Serialize};

use crate::cards::{Card, Shoe};
use crate::meja::{self, MejaView, RoundConfig, STANDARD, Umum, WagerRtp};
use crate::poker::{Category, Value, eval5};

pub const ID: &str = "caribbean-stud";

pub type Config = RoundConfig;

/// RTP ante dengan strategi sederhana Wizard of Odds (D-060), dari simulasi
/// sangat besar di workflow `rtp.yml`.
pub const RTP: &[WagerRtp] = &[WagerRtp {
    wager: "ante",
    percent: 94.7760,
    manifest: true,
}];

pub fn catalog() -> &'static Catalog {
    static CATALOG: OnceLock<Catalog> = OnceLock::new();
    CATALOG.get_or_init(|| {
        Catalog::from_toml(include_str!("i18n.toml")).expect("caribbean_stud/i18n.toml tidak sah")
    })
}

pub fn cartridge() -> Result<Cartridge, RegistryError> {
    Cartridge::new(
        include_str!("manifest.toml"),
        include_str!("../../../../tutorials/caribbean-stud.toml"),
        create_session::<CaribbeanStud>,
    )
}

/// Pembayaran raise (kali raise) untuk tangan pemain yang menang.
pub fn raise_pays(v: Value) -> i64 {
    if v.is_royal() {
        return 100;
    }
    match v.category() {
        Category::HighCard | Category::Pair => 1,
        Category::TwoPair => 2,
        Category::Trips => 3,
        Category::Straight => 4,
        Category::Flush => 5,
        Category::FullHouse => 7,
        Category::Quads => 20,
        Category::StraightFlush => 50,
    }
}

/// Bandar memenuhi syarat: pair ke atas, atau kartu tinggi dengan as dan
/// king.
pub fn qualifies(v: Value) -> bool {
    v.category() > Category::HighCard || (v.rank(0) == 14 && v.rank(1) == 13)
}

/// Hasil ante dan raise (raise = 2 × ante) bila pemain raise.
pub fn showdown(player: Value, dealer: Value, ante: i64) -> (i64, i64) {
    if !qualifies(dealer) {
        return (ante, 0);
    }
    match player.cmp(&dealer) {
        std::cmp::Ordering::Greater => (ante, 2 * ante * raise_pays(player)),
        std::cmp::Ordering::Less => (-ante, -2 * ante),
        std::cmp::Ordering::Equal => (0, 0),
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Action {
    Bet(i64),
    Raise,
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
    ante: i64,
    raised: bool,
    payouts: BTreeMap<String, i64>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct CaribbeanStud {
    deck: Shoe,
    phase: Phase,
    round: Option<Round>,
    ended: Option<&'static str>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct View {
    #[serde(flatten)]
    pub meja: Umum,
    pub pemain: Vec<String>,
    /// Kartu bandar; hanya yang pertama terbuka sampai ronde selesai.
    pub bandar: Vec<String>,
    pub tangan_pemain: Option<String>,
    pub tangan_bandar: Option<String>,
    pub memenuhi: Option<bool>,
    pub ante: i64,
    pub raise: i64,
    pub bayar: BTreeMap<String, i64>,
}

impl MejaView for View {
    fn meja(&self) -> &Umum {
        &self.meja
    }
}

impl CaribbeanStud {
    fn deal(&mut self, ante: i64) -> Result<(), GameError> {
        let mut player = Vec::with_capacity(5);
        let mut dealer = Vec::with_capacity(5);
        for _ in 0..5 {
            player.push(meja::draw(&mut self.deck)?);
        }
        for _ in 0..5 {
            dealer.push(meja::draw(&mut self.deck)?);
        }
        self.round = Some(Round {
            player,
            dealer,
            ante,
            raised: false,
            payouts: BTreeMap::new(),
        });
        self.phase = Phase::Decide;
        Ok(())
    }

    fn decide(&mut self, raise: bool) {
        let r = self.round.as_mut().expect("ronde berjalan");
        if raise {
            r.raised = true;
            let (ante, raise) = showdown(eval5(&r.player), eval5(&r.dealer), r.ante);
            r.payouts.insert("ante".into(), ante);
            r.payouts.insert("raise".into(), raise);
        } else {
            r.payouts.insert("ante".into(), -r.ante);
        }
        self.phase = Phase::Over;
        self.ended = Some("ronde_selesai");
    }

    fn net(&self) -> i64 {
        self.round
            .as_ref()
            .map(|r| r.payouts.values().sum())
            .unwrap_or(0)
    }
}

impl TurnGame for CaribbeanStud {
    type Config = Config;
    type Action = Action;
    type View = View;

    fn new(config: Config, seed: Seed) -> Result<Self, GameError> {
        Ok(CaribbeanStud {
            deck: config.deck(seed)?,
            phase: Phase::Betting,
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
            Phase::Betting => vec![STANDARD.single(), ActionSpec::fixed("leave")],
            Phase::Decide => vec![ActionSpec::fixed("raise"), ActionSpec::fixed("fold")],
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
            (Phase::Decide, Action::Raise) => {
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
        let over = self.phase == Phase::Over;
        let ante = r.map(|r| r.ante).unwrap_or(0);
        let raise = if r.is_some_and(|r| r.raised) {
            2 * ante
        } else {
            0
        };
        meja.bersih = self.net();
        meja.ronde = u32::from(r.is_some());
        meja.taruhan_meja = if self.phase == Phase::Decide { ante } else { 0 };
        meja.dipertaruhkan = ante + raise;
        if self.phase == Phase::Decide {
            meja.biaya.insert("raise".into(), 2 * ante);
            meja.netral = Some("fold".into());
        }
        meja.selesai = over;
        meja.alasan = self.ended.map(str::to_string);
        View {
            meja,
            pemain: r
                .map(|r| r.player.iter().map(Card::to_string).collect())
                .unwrap_or_default(),
            bandar: r
                .map(|r| {
                    r.dealer
                        .iter()
                        .enumerate()
                        .map(|(i, c)| {
                            if over || i == 0 {
                                c.to_string()
                            } else {
                                "??".into()
                            }
                        })
                        .collect()
                })
                .unwrap_or_default(),
            tangan_pemain: r.map(|r| eval5(&r.player).key().to_string()),
            tangan_bandar: r
                .filter(|_| over)
                .map(|r| eval5(&r.dealer).key().to_string()),
            memenuhi: r
                .filter(|r| over && r.raised)
                .map(|r| qualifies(eval5(&r.dealer))),
            ante,
            raise,
            bayar: r.map(|r| r.payouts.clone()).unwrap_or_default(),
        }
    }

    fn to_text(view: &View, lang: Lang) -> String {
        let c = catalog();
        let mut out = String::new();
        if !view.pemain.is_empty() {
            let hand = |k: &Option<String>| {
                k.as_ref()
                    .map(|k| format!(" ({})", crate::meja::hand_name(lang, k)))
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
                out.push_str(&format!("  {spot} → {pay:+}\n"));
            }
        }
        out.push_str(
            &match (view.meja.fase.as_str(), view.meja.alasan.as_deref()) {
                ("keputusan", _) => {
                    c.text(lang, "decide", &[("raise", &(2 * view.ante).to_string())])
                }
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
            ["bet", n] => Ok(Action::Bet(meja::amount(n).ok_or_else(err)?)),
            ["raise"] => Ok(Action::Raise),
            ["fold"] => Ok(Action::Fold),
            ["leave"] => Ok(Action::Leave),
            _ => Err(err()),
        }
    }

    fn format_action(&self, action: &Action) -> String {
        match action {
            Action::Bet(n) => format!("bet {n}"),
            Action::Raise => "raise".into(),
            Action::Fold => "fold".into(),
            Action::Leave => "leave".into(),
        }
    }
}
