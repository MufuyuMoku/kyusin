//! Let It Ride (SPEC §6.3; M5a): tiga tempat taruhan, dua di antaranya
//! boleh ditarik.
//!
//! Aturan (D-060): satu dek per ronde; tabel baku 1-2-3-5-8-11-50-200-1000
//! (pair 10 ke atas sampai royal flush); tanpa taruhan sampingan. Satu sesi
//! = satu ronde (commit-reveal per ronde).

use std::sync::OnceLock;

use kyusin_core::game::{GameResult, create_session};
use kyusin_core::i18n::{Catalog, Lang};
use kyusin_core::{ActionSpec, Cartridge, GameError, PlayerId, RegistryError, Seed, TurnGame};
use serde::{Deserialize, Serialize};

use crate::cards::{Card, Shoe};
use crate::meja::{self, MejaView, RoundConfig, STANDARD, Umum, WagerRtp};
use crate::poker::{Category, Value, eval5};

pub const ID: &str = "let-it-ride";
/// `bet <jumlah>` memasang jumlah itu di tiga tempat.
pub const SPOTS: i64 = 3;

pub type Config = RoundConfig;

/// RTP per satu tempat (satuan taruhan) dengan strategi optimal, dari
/// enumerasi tepat di workflow `rtp.yml` (D-060).
pub const RTP: &[WagerRtp] = &[WagerRtp {
    wager: "bet",
    percent: 96.4943,
    manifest: true,
}];

pub fn catalog() -> &'static Catalog {
    static CATALOG: OnceLock<Catalog> = OnceLock::new();
    CATALOG.get_or_init(|| {
        Catalog::from_toml(include_str!("i18n.toml")).expect("let_it_ride/i18n.toml tidak sah")
    })
}

pub fn cartridge() -> Result<Cartridge, RegistryError> {
    Cartridge::new(
        include_str!("manifest.toml"),
        include_str!("../../../../tutorials/let-it-ride.toml"),
        create_session::<LetItRide>,
    )
}

/// Pembayaran per tempat (kali taruhan) untuk tangan lima kartu; -1 = kalah.
pub fn pays(v: Value) -> i64 {
    if v.is_royal() {
        return 1000;
    }
    match v.category() {
        Category::StraightFlush => 200,
        Category::Quads => 50,
        Category::FullHouse => 11,
        Category::Flush => 8,
        Category::Straight => 5,
        Category::Trips => 3,
        Category::TwoPair => 2,
        Category::Pair if v.rank(0) >= 10 => 1,
        _ => -1,
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Action {
    Bet(i64),
    Pull,
    Ride,
    Leave,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
enum Phase {
    Betting,
    First,
    Second,
    Over,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
struct Round {
    player: Vec<Card>,
    board: Vec<Card>,
    /// Taruhan tiap tempat; 0 = ditarik.
    spots: [i64; 3],
    payout: Option<i64>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct LetItRide {
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
    /// Dua kartu bersama; `??` sampai dibuka.
    pub bersama: Vec<String>,
    /// Taruhan tiga tempat; 0 = ditarik.
    pub tempat: Vec<i64>,
    /// Keputusan ke berapa (1 atau 2) saat fase `keputusan`.
    pub keputusan: Option<u8>,
    pub tangan: Option<String>,
    pub bayar: Option<i64>,
}

impl MejaView for View {
    fn meja(&self) -> &Umum {
        &self.meja
    }
}

impl LetItRide {
    fn deal(&mut self, bet: i64) -> Result<(), GameError> {
        let take = |n: usize, deck: &mut Shoe| -> Result<Vec<Card>, GameError> {
            (0..n).map(|_| meja::draw(deck)).collect()
        };
        let player = take(3, &mut self.deck)?;
        let board = take(2, &mut self.deck)?;
        self.round = Some(Round {
            player,
            board,
            spots: [bet; 3],
            payout: None,
        });
        self.phase = Phase::First;
        Ok(())
    }

    fn decide(&mut self, pull: bool) {
        let r = self.round.as_mut().expect("ronde berjalan");
        let spot = if self.phase == Phase::First { 0 } else { 1 };
        if pull {
            r.spots[spot] = 0;
        }
        if self.phase == Phase::First {
            self.phase = Phase::Second;
            return;
        }
        let all: Vec<Card> = r.player.iter().chain(r.board.iter()).copied().collect();
        let k = pays(eval5(&all));
        r.payout = Some(r.spots.iter().map(|s| s * k).sum());
        self.phase = Phase::Over;
        self.ended = Some("ronde_selesai");
    }
}

impl TurnGame for LetItRide {
    type Config = Config;
    type Action = Action;
    type View = View;

    fn new(config: Config, seed: Seed) -> Result<Self, GameError> {
        Ok(LetItRide {
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
            Phase::First | Phase::Second => {
                vec![ActionSpec::fixed("pull"), ActionSpec::fixed("ride")]
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
            (Phase::Betting, Action::Bet(n)) if STANDARD.accepts(n) => self.deal(n),
            (Phase::Betting, Action::Leave) => {
                self.phase = Phase::Over;
                self.ended = Some("berhenti");
                Ok(())
            }
            (Phase::First | Phase::Second, Action::Pull) => {
                self.decide(true);
                Ok(())
            }
            (Phase::First | Phase::Second, Action::Ride) => {
                self.decide(false);
                Ok(())
            }
            _ => Err(GameError::Illegal(self.format_action(&action))),
        }
    }

    fn view_for(&self, _player: PlayerId) -> View {
        let fase = match self.phase {
            Phase::Betting => "taruhan",
            Phase::First | Phase::Second => "keputusan",
            Phase::Over => "selesai",
        };
        let mut meja = Umum::new(fase, STANDARD);
        meja.pengali.insert("bet".into(), SPOTS);
        let r = self.round.as_ref();
        let open = match self.phase {
            Phase::Betting | Phase::First => 0,
            Phase::Second => 1,
            Phase::Over => 2,
        };
        let at_risk: i64 = r.map(|r| r.spots.iter().sum()).unwrap_or(0);
        meja.bersih = r.and_then(|r| r.payout).unwrap_or(0);
        meja.ronde = u32::from(r.is_some());
        meja.taruhan_meja = if matches!(self.phase, Phase::First | Phase::Second) {
            at_risk
        } else {
            0
        };
        meja.dipertaruhkan = at_risk;
        if matches!(self.phase, Phase::First | Phase::Second) {
            meja.netral = Some("pull".into());
        }
        meja.selesai = self.phase == Phase::Over;
        meja.alasan = self.ended.map(str::to_string);
        View {
            meja,
            pemain: r
                .map(|r| r.player.iter().map(Card::to_string).collect())
                .unwrap_or_default(),
            bersama: r
                .map(|r| {
                    r.board
                        .iter()
                        .enumerate()
                        .map(|(i, c)| if i < open { c.to_string() } else { "??".into() })
                        .collect()
                })
                .unwrap_or_default(),
            tempat: r.map(|r| r.spots.to_vec()).unwrap_or_default(),
            keputusan: match self.phase {
                Phase::First => Some(1),
                Phase::Second => Some(2),
                _ => None,
            },
            tangan: r.filter(|r| r.payout.is_some()).map(|r| {
                let all: Vec<Card> = r.player.iter().chain(r.board.iter()).copied().collect();
                eval5(&all).key().to_string()
            }),
            bayar: r.and_then(|r| r.payout),
        }
    }

    fn to_text(view: &View, lang: Lang) -> String {
        let c = catalog();
        let mut out = String::new();
        if !view.pemain.is_empty() {
            let hand = view
                .tangan
                .as_ref()
                .map(|k| format!(" ({})", meja::hand_name(lang, k)))
                .unwrap_or_default();
            out.push_str(
                &c.text(
                    lang,
                    "cards",
                    &[
                        ("player", &view.pemain.join(" ")),
                        ("board", &format!("{}{hand}", view.bersama.join(" "))),
                        (
                            "spots",
                            &view
                                .tempat
                                .iter()
                                .map(i64::to_string)
                                .collect::<Vec<_>>()
                                .join(" / "),
                        ),
                    ],
                ),
            );
            out.push('\n');
            if let Some(p) = view.bayar {
                out.push_str(&format!("  → {p:+}\n"));
            }
        }
        out.push_str(
            &match (view.meja.fase.as_str(), view.meja.alasan.as_deref()) {
                ("keputusan", _) => c.text(
                    lang,
                    "decide",
                    &[("n", &view.keputusan.unwrap_or(1).to_string())],
                ),
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
            let net = self.round.as_ref().and_then(|r| r.payout).unwrap_or(0);
            meja::result(catalog(), "summary", net, u32::from(self.round.is_some()))
        })
    }

    fn parse_command(&self, command: &str) -> Result<Action, GameError> {
        let c = command.trim();
        let parts: Vec<&str> = c.split(' ').collect();
        let err = || GameError::Parse(c.into());
        match parts.as_slice() {
            ["bet", n] => Ok(Action::Bet(meja::amount(n).ok_or_else(err)?)),
            ["pull"] => Ok(Action::Pull),
            ["ride"] => Ok(Action::Ride),
            ["leave"] => Ok(Action::Leave),
            _ => Err(err()),
        }
    }

    fn format_action(&self, action: &Action) -> String {
        match action {
            Action::Bet(n) => format!("bet {n}"),
            Action::Pull => "pull".into(),
            Action::Ride => "ride".into(),
            Action::Leave => "leave".into(),
        }
    }
}
