//! Hi-Lo (SPEC §6.5; M6b, D-071): tebak apakah kartu berikutnya lebih
//! tinggi atau sama, atau lebih rendah atau sama.
//!
//! Dek tak terbatas: kartu ke-k diundi independen dari `derive(seed,
//! "kartu:k")` (13 peringkat sama peluang, A = 1, K = 13; jenis kartu hanya
//! hiasan). Pengali tebakan = 0,97 ÷ peluang menang, dibulatkan ke bawah ke
//! perseratus ([`multiplier`]); pengali berjalan juga perseratus, dibulatkan
//! ke bawah tiap langkah. Cash out kapan saja setelah tebakan pertama;
//! pengali 1.000× = cash out otomatis.
//!
//! Satu pertandingan = satu ronde: taruhan dipotong saat dipasang (D-059),
//! kartu dibuka satu per satu, seed ronde dibuka saat kalah atau cash out.
//! Pertandingan yang ditunda dilanjutkan dari titik simpan, jadi menutup
//! aplikasi tidak membatalkan tebakan.

use std::sync::OnceLock;

use kyusin_core::game::{GameResult, create_session};
use kyusin_core::i18n::{Catalog, Lang};
use kyusin_core::rng::derive;
use kyusin_core::{
    ActionSpec, Cartridge, GameError, GameRng, PlayerId, RegistryError, Seed, TurnGame,
};
use serde::{Deserialize, Serialize};

use crate::cards::Card;
use crate::meja::{self, Limits, MejaView, Umum, WagerRtp};

pub const ID: &str = "hi-lo";
/// Kelipatan 100 supaya taruhan × pengali (perseratus) selalu utuh.
pub const LIMITS: Limits = Limits {
    min: 100,
    max: 2000,
    step: 100,
};
/// Pengali tertinggi (perseratus): 1.000×.
pub const CAP: u32 = 100_000;

/// RTP strategi terbaik: satu tebakan (pilihan yang lebih baik per kartu),
/// lalu cash out; tepat (D-072).
pub const RTP: &[WagerRtp] = &[WagerRtp {
    wager: "bet",
    percent: 96.9349,
    manifest: true,
}];

/// Pengali (perseratus) tebakan dari kartu berperingkat `rank` (1–13):
/// ⌊97 × 13 ÷ n⌋, n = jumlah peringkat yang menang.
pub fn multiplier(rank: u8, higher: bool) -> u32 {
    let n = u32::from(if higher { 14 - rank } else { rank });
    97 * 13 / n
}

/// Peringkat kartu: A = 1 … K = 13.
pub fn rank(card: Card) -> u8 {
    card.rank as u8 + 1
}

/// Kartu ke-`k` ronde ini (dek tak terbatas).
pub fn card_at(seed: &Seed, k: usize) -> Card {
    let i = GameRng::from_seed(derive(seed, &format!("kartu:{k}"))).below(52);
    Card::deck()[i as usize]
}

pub fn catalog() -> &'static Catalog {
    static CATALOG: OnceLock<Catalog> = OnceLock::new();
    CATALOG.get_or_init(|| {
        Catalog::from_toml(include_str!("i18n.toml")).expect("hi_lo/i18n.toml tidak sah")
    })
}

pub fn cartridge() -> Result<Cartridge, RegistryError> {
    Cartridge::new(
        include_str!("manifest.toml"),
        include_str!("../../../../tutorials/hi-lo.toml"),
        create_session::<HiLo>,
    )
}

#[derive(Debug, Clone, Default, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Config {
    /// Urutan kartu tetap (tes dan tutorial).
    #[serde(default)]
    pub kartu: Option<Vec<String>>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Action {
    Bet(i64),
    Higher,
    Lower,
    CashOut,
    Leave,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
enum Phase {
    Betting,
    Guess,
    Over,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct HiLo {
    seed: Seed,
    fixed: Option<Vec<Card>>,
    phase: Phase,
    cards: Vec<Card>,
    mult: u32,
    guesses: u32,
    bet: i64,
    payout: Option<i64>,
    ended: Option<&'static str>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct View {
    #[serde(flatten)]
    pub meja: Umum,
    /// Kartu yang sudah dibuka, berurutan.
    pub kartu: Vec<String>,
    /// Pengali berjalan (perseratus; 100 = 1×).
    pub pengali: u32,
    /// Pengali tebakan dari kartu terakhir (perseratus).
    pub pengali_tinggi: Option<u32>,
    pub pengali_rendah: Option<u32>,
    /// Tebakan benar sejauh ini.
    pub tebakan: u32,
    pub taruhan: i64,
    pub bayar: Option<i64>,
}

impl MejaView for View {
    fn meja(&self) -> &Umum {
        &self.meja
    }
}

impl HiLo {
    fn next_card(&self) -> Result<Card, GameError> {
        let k = self.cards.len();
        match &self.fixed {
            Some(list) => list
                .get(k)
                .copied()
                .ok_or_else(|| GameError::Config("kartu tetap habis".into())),
            None => Ok(card_at(&self.seed, k)),
        }
    }

    fn current(&self) -> Option<u8> {
        self.cards.last().map(|c| rank(*c))
    }

    fn finish(&mut self, reason: &'static str, payout: i64) {
        self.payout = Some(payout);
        self.ended = Some(reason);
        self.phase = Phase::Over;
    }

    fn guess(&mut self, higher: bool) -> Result<(), GameError> {
        let now = self.current().expect("kartu pertama");
        let next = self.next_card()?;
        self.cards.push(next);
        let r = rank(next);
        let win = if higher { r >= now } else { r <= now };
        if !win {
            self.finish("kalah", -self.bet);
            return Ok(());
        }
        let m = u64::from(multiplier(now, higher));
        let mult = (u64::from(self.mult) * m / 100).min(u64::from(CAP)) as u32;
        self.mult = mult;
        self.guesses += 1;
        if mult >= CAP {
            self.finish("batas", self.bet * i64::from(CAP) / 100 - self.bet);
        }
        Ok(())
    }

    /// Tebakan dengan peluang menang lebih besar (seri: lebih tinggi).
    fn safer(&self) -> &'static str {
        let r = self.current().unwrap_or(7);
        if 14 - r >= r { "higher" } else { "lower" }
    }
}

impl TurnGame for HiLo {
    type Config = Config;
    type Action = Action;
    type View = View;

    fn new(config: Config, seed: Seed) -> Result<Self, GameError> {
        let fixed = config.kartu.as_deref().map(meja::parse_cards).transpose()?;
        Ok(HiLo {
            seed,
            fixed,
            phase: Phase::Betting,
            cards: Vec::new(),
            mult: 100,
            guesses: 0,
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
            Phase::Betting => vec![LIMITS.single(), ActionSpec::fixed("leave")],
            Phase::Guess => {
                let mut out = vec![ActionSpec::fixed("higher"), ActionSpec::fixed("lower")];
                if self.guesses > 0 {
                    out.push(ActionSpec::fixed("cashout"));
                }
                out
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
            (Phase::Betting, Action::Bet(n)) if LIMITS.accepts(n) => {
                let first = self.next_card()?;
                self.cards.push(first);
                self.bet = n;
                self.phase = Phase::Guess;
                Ok(())
            }
            (Phase::Betting, Action::Leave) => {
                self.phase = Phase::Over;
                self.ended = Some("berhenti");
                Ok(())
            }
            (Phase::Guess, Action::Higher) => self.guess(true),
            (Phase::Guess, Action::Lower) => self.guess(false),
            (Phase::Guess, Action::CashOut) if self.guesses > 0 => {
                let back = self.bet * i64::from(self.mult) / 100;
                self.finish("cashout", back - self.bet);
                Ok(())
            }
            _ => Err(GameError::Illegal(self.format_action(&action))),
        }
    }

    fn view_for(&self, _player: PlayerId) -> View {
        let fase = match self.phase {
            Phase::Betting => "taruhan",
            Phase::Guess => "tebak",
            Phase::Over => "selesai",
        };
        let mut meja = Umum::new(fase, LIMITS);
        meja.bersih = self.payout.unwrap_or(0);
        meja.ronde = u32::from(self.payout.is_some() && self.bet > 0);
        meja.taruhan_meja = if self.phase == Phase::Guess {
            self.bet
        } else {
            0
        };
        meja.dipertaruhkan = self.bet;
        if self.phase == Phase::Guess {
            meja.netral = Some(
                if self.guesses > 0 {
                    "cashout"
                } else {
                    self.safer()
                }
                .into(),
            );
        }
        meja.selesai = self.phase == Phase::Over;
        meja.alasan = self.ended.map(str::to_string);
        let guessing = self.phase == Phase::Guess;
        View {
            meja,
            kartu: self.cards.iter().map(Card::to_string).collect(),
            pengali: self.mult,
            pengali_tinggi: self
                .current()
                .filter(|_| guessing)
                .map(|r| multiplier(r, true)),
            pengali_rendah: self
                .current()
                .filter(|_| guessing)
                .map(|r| multiplier(r, false)),
            tebakan: self.guesses,
            taruhan: self.bet,
            bayar: self.payout,
        }
    }

    fn to_text(view: &View, lang: Lang) -> String {
        let c = catalog();
        let mut out = String::new();
        let x = |m: u32| format!("{}.{:02}×", m / 100, m % 100);
        if !view.kartu.is_empty() {
            out.push_str(&c.text(
                lang,
                "cards",
                &[("cards", &view.kartu.join(" ")), ("mult", &x(view.pengali))],
            ));
            out.push('\n');
        }
        if let (Some(h), Some(l)) = (view.pengali_tinggi, view.pengali_rendah) {
            out.push_str(&c.text(lang, "guess", &[("hi", &x(h)), ("lo", &x(l))]));
            out.push('\n');
        }
        out.push_str(
            &match (view.meja.fase.as_str(), view.meja.alasan.as_deref()) {
                ("taruhan", _) => c.text(lang, "place_bet", &[]),
                ("tebak", _) => String::new(),
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
                u32::from(self.bet > 0),
            )
        })
    }

    fn parse_command(&self, command: &str) -> Result<Action, GameError> {
        let c = command.trim();
        let parts: Vec<&str> = c.split(' ').collect();
        let err = || GameError::Parse(c.into());
        match parts.as_slice() {
            ["bet", n] => Ok(Action::Bet(meja::amount(n).ok_or_else(err)?)),
            ["higher"] => Ok(Action::Higher),
            ["lower"] => Ok(Action::Lower),
            ["cashout"] => Ok(Action::CashOut),
            ["leave"] => Ok(Action::Leave),
            _ => Err(err()),
        }
    }

    fn format_action(&self, action: &Action) -> String {
        match action {
            Action::Bet(n) => format!("bet {n}"),
            Action::Higher => "higher".into(),
            Action::Lower => "lower".into(),
            Action::CashOut => "cashout".into(),
            Action::Leave => "leave".into(),
        }
    }
}
