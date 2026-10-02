//! Casino Hold'em (SPEC §6.3; M5a): hold'em dua kartu melawan bandar.
//!
//! Aturan (D-060): satu dek per ronde. Ante; setelah flop pemain `call`
//! (dua kali ante) atau `fold`. Bandar memenuhi syarat dengan pair 4 atau
//! lebih. Tabel AnteWin 100-20-10-3-2-1. Tanpa AA bonus. Satu sesi = satu
//! ronde (commit-reveal per ronde).

use std::collections::BTreeMap;
use std::sync::OnceLock;

use kyusin_core::game::{GameResult, create_session};
use kyusin_core::i18n::{Catalog, Lang};
use kyusin_core::{ActionSpec, Cartridge, GameError, PlayerId, RegistryError, Seed, TurnGame};
use serde::{Deserialize, Serialize};

use crate::cards::{Card, Shoe};
use crate::meja::{self, MejaView, RoundConfig, STANDARD, Umum, WagerRtp};
use crate::poker::{Category, Value, eval_best};

pub const ID: &str = "casino-holdem";

pub type Config = RoundConfig;

/// RTP ante dengan strategi sederhana KyuSin (D-060), dari simulasi 1 miliar
/// ronde di workflow `rtp.yml` (run 36945993998).
pub const RTP: &[WagerRtp] = &[WagerRtp {
    wager: "ante",
    percent: 97.0750,
    manifest: true,
}];

pub fn catalog() -> &'static Catalog {
    static CATALOG: OnceLock<Catalog> = OnceLock::new();
    CATALOG.get_or_init(|| {
        Catalog::from_toml(include_str!("i18n.toml")).expect("casino_holdem/i18n.toml tidak sah")
    })
}

pub fn cartridge() -> Result<Cartridge, RegistryError> {
    Cartridge::new(
        include_str!("manifest.toml"),
        include_str!("../../../../tutorials/casino-holdem.toml"),
        create_session::<CasinoHoldem>,
    )
}

/// Tabel AnteWin (kali ante).
pub fn ante_pays(v: Value) -> i64 {
    if v.is_royal() {
        return 100;
    }
    match v.category() {
        Category::StraightFlush => 20,
        Category::Quads => 10,
        Category::FullHouse => 3,
        Category::Flush => 2,
        _ => 1,
    }
}

/// Bandar memenuhi syarat: pair 4 atau lebih.
pub fn qualifies(v: Value) -> bool {
    match v.category() {
        Category::HighCard => false,
        Category::Pair => v.rank(0) >= 4,
        _ => true,
    }
}

/// Hasil ante dan call (call = 2 × ante) bila pemain call.
pub fn showdown(player: Value, dealer: Value, ante: i64) -> (i64, i64) {
    if !qualifies(dealer) {
        return (ante * ante_pays(player), 0);
    }
    match player.cmp(&dealer) {
        std::cmp::Ordering::Greater => (ante * ante_pays(player), 2 * ante),
        std::cmp::Ordering::Less => (-ante, -2 * ante),
        std::cmp::Ordering::Equal => (0, 0),
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Action {
    Bet(i64),
    Call,
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
    board: Vec<Card>,
    ante: i64,
    called: bool,
    payouts: BTreeMap<String, i64>,
}

impl Round {
    fn best(&self, hole: &[Card]) -> Value {
        let all: Vec<Card> = hole.iter().chain(self.board.iter()).copied().collect();
        eval_best(&all)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct CasinoHoldem {
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
    /// `??` sampai ronde selesai.
    pub bandar: Vec<String>,
    /// Kartu bersama yang sudah dibuka (flop, lalu turn dan river).
    pub meja_kartu: Vec<String>,
    pub tangan_pemain: Option<String>,
    pub tangan_bandar: Option<String>,
    pub memenuhi: Option<bool>,
    pub ante: i64,
    pub call: i64,
    pub bayar: BTreeMap<String, i64>,
}

impl MejaView for View {
    fn meja(&self) -> &Umum {
        &self.meja
    }
}

impl CasinoHoldem {
    fn deal(&mut self, ante: i64) -> Result<(), GameError> {
        let take = |n: usize, deck: &mut Shoe| -> Result<Vec<Card>, GameError> {
            (0..n).map(|_| meja::draw(deck)).collect()
        };
        let player = take(2, &mut self.deck)?;
        let dealer = take(2, &mut self.deck)?;
        let board = take(3, &mut self.deck)?;
        self.round = Some(Round {
            player,
            dealer,
            board,
            ante,
            called: false,
            payouts: BTreeMap::new(),
        });
        self.phase = Phase::Decide;
        Ok(())
    }

    fn decide(&mut self, call: bool) -> Result<(), GameError> {
        // Turn dan river selalu dibuka di akhir ronde.
        for _ in 0..2 {
            let c = meja::draw(&mut self.deck)?;
            self.round.as_mut().expect("ronde berjalan").board.push(c);
        }
        let r = self.round.as_mut().expect("ronde berjalan");
        if call {
            r.called = true;
            let (p, d) = (r.best(&r.player), r.best(&r.dealer));
            let (ante, call) = showdown(p, d, r.ante);
            r.payouts.insert("ante".into(), ante);
            r.payouts.insert("call".into(), call);
        } else {
            r.payouts.insert("ante".into(), -r.ante);
        }
        self.phase = Phase::Over;
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

impl TurnGame for CasinoHoldem {
    type Config = Config;
    type Action = Action;
    type View = View;

    fn new(config: Config, seed: Seed) -> Result<Self, GameError> {
        Ok(CasinoHoldem {
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
            Phase::Decide => vec![ActionSpec::fixed("call"), ActionSpec::fixed("fold")],
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
            (Phase::Decide, Action::Call) => self.decide(true),
            (Phase::Decide, Action::Fold) => self.decide(false),
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
        let call = if r.is_some_and(|r| r.called) {
            2 * ante
        } else {
            0
        };
        meja.bersih = self.net();
        meja.ronde = u32::from(r.is_some());
        meja.taruhan_meja = if self.phase == Phase::Decide { ante } else { 0 };
        meja.dipertaruhkan = ante + call;
        if self.phase == Phase::Decide {
            meja.biaya.insert("call".into(), 2 * ante);
            meja.netral = Some("fold".into());
        }
        meja.selesai = over;
        meja.alasan = self.ended.map(str::to_string);
        let cards = |v: &[Card]| v.iter().map(Card::to_string).collect::<Vec<_>>();
        View {
            meja,
            pemain: r.map(|r| cards(&r.player)).unwrap_or_default(),
            bandar: r
                .map(|r| {
                    if over {
                        cards(&r.dealer)
                    } else {
                        vec!["??".into(); r.dealer.len()]
                    }
                })
                .unwrap_or_default(),
            meja_kartu: r.map(|r| cards(&r.board)).unwrap_or_default(),
            tangan_pemain: r.map(|r| r.best(&r.player).key().to_string()),
            tangan_bandar: r
                .filter(|_| over)
                .map(|r| r.best(&r.dealer).key().to_string()),
            memenuhi: r
                .filter(|r| over && r.called)
                .map(|r| qualifies(r.best(&r.dealer))),
            ante,
            call,
            bayar: r.map(|r| r.payouts.clone()).unwrap_or_default(),
        }
    }

    fn to_text(view: &View, lang: Lang) -> String {
        let c = catalog();
        let mut out = String::new();
        if !view.pemain.is_empty() {
            let hand = |k: &Option<String>| {
                k.as_ref()
                    .map(|k| format!(" ({})", meja::hand_name(lang, k)))
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
                    ("board", &view.meja_kartu.join(" ")),
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
                    c.text(lang, "decide", &[("call", &(2 * view.ante).to_string())])
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
            ["call"] => Ok(Action::Call),
            ["fold"] => Ok(Action::Fold),
            ["leave"] => Ok(Action::Leave),
            _ => Err(err()),
        }
    }

    fn format_action(&self, action: &Action) -> String {
        match action {
            Action::Bet(n) => format!("bet {n}"),
            Action::Call => "call".into(),
            Action::Fold => "fold".into(),
            Action::Leave => "leave".into(),
        }
    }
}
