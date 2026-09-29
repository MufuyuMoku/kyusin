//! Bagian bersama meja casino melawan bandar (M5a; SPEC §6.3, §6.7).
//!
//! Setiap game meja tetap mesin keadaan murni yang tidak tahu saldo. Host
//! membaca kontrak chip dari bagian view [`Umum`] (D-058, D-059):
//! - `fase`: `taruhan` (menyusun taruhan), fase game, atau `selesai`;
//! - `ronde`: ronde yang sudah dibagi; `bersih`: hasil bersih ronde yang
//!   sudah selesai dalam sesi ini;
//! - `taruhan_meja`: chip yang sedang dipertaruhkan (termasuk taruhan yang
//!   sudah dipasang sebelum kartu dibagi);
//! - `dipertaruhkan`: total yang dipertaruhkan di ronde terakhir yang
//!   selesai (untuk ringkasan casino);
//! - `biaya`: biaya chip aksi tetap (`play`, `raise`, `war`, …); perintah
//!   berangka (`bet <tempat> <jumlah>`) berbiaya angkanya dikali
//!   `pengali[kata kerja]` (bawaan 1).
//!
//! Dua jenis sesi: game ber-shoe (Baccarat, Dragon Tiger, Casino War, Red
//! Dog) memakai satu shoe per sesi dengan provably fair per shoe seperti
//! Blackjack (D-056); game yang dikocok ulang tiap ronde memakai satu
//! ronde per sesi, jadi commit-reveal SPEC §5.4 berlaku per ronde.

use std::collections::BTreeMap;

use kyusin_core::game::GameResult;
use kyusin_core::i18n::{Catalog, Localized};
use kyusin_core::{ActionSpec, GameError, GameRng, Param, ParamKind, Seed};
use serde::{Deserialize, Serialize};

use crate::cards::{Card, Shoe};

/// Batas taruhan satu tempat taruhan.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Limits {
    pub min: i64,
    pub max: i64,
    pub step: i64,
}

/// Batas meja bawaan, sama dengan Blackjack (D-056): 10–2.000, kelipatan 10.
pub const STANDARD: Limits = Limits {
    min: 10,
    max: 2000,
    step: 10,
};

impl Limits {
    pub fn accepts(&self, n: i64) -> bool {
        n >= self.min && n <= self.max && n % self.step == 0
    }

    fn amount(&self, max: i64) -> Param {
        Param {
            name: "jumlah".into(),
            kind: ParamKind::Int {
                min: self.min,
                max,
                step: self.step,
            },
        }
    }

    /// `bet <jumlah>`: taruhan tunggal yang langsung membagi kartu.
    pub fn single(&self) -> ActionSpec {
        ActionSpec::template("bet", vec![self.amount(self.max)])
    }

    /// `bet <tempat> <jumlah>` untuk satu tempat taruhan yang sudah berisi
    /// `current`; `None` bila tempat itu sudah penuh.
    pub fn spot(&self, spot: &str, current: i64) -> Option<ActionSpec> {
        let room = self.max - current;
        let max = room - room % self.step;
        (max >= self.min).then(|| {
            ActionSpec::template(
                "bet",
                vec![
                    Param {
                        name: "tempat".into(),
                        kind: ParamKind::Choice {
                            options: vec![spot.into()],
                        },
                    },
                    self.amount(max),
                ],
            )
        })
    }
}

/// Bilangan bulat tanpa tanda.
pub fn amount(text: &str) -> Option<i64> {
    (!text.is_empty() && text.bytes().all(|b| b.is_ascii_digit()))
        .then(|| text.parse().ok())
        .flatten()
}

pub fn parse_cards(list: &[String]) -> Result<Vec<Card>, GameError> {
    list.iter()
        .map(|c| {
            c.parse::<Card>()
                .map_err(|_| GameError::Config(format!("kartu `{c}`")))
        })
        .collect()
}

/// Konfigurasi game ber-shoe; `shoe` dan `potong` untuk tes dan tutorial.
#[derive(Debug, Clone, Default, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ShoeConfig {
    #[serde(default)]
    pub dek: Option<u8>,
    /// Urutan kartu tetap, kartu pertama dibagi lebih dulu.
    #[serde(default)]
    pub shoe: Option<Vec<String>>,
    /// Jumlah kartu terpakai yang mengakhiri shoe.
    #[serde(default)]
    pub potong: Option<usize>,
}

impl ShoeConfig {
    /// Shoe beserta titik potongnya. `penetration` = bagian shoe yang
    /// dipakai sebelum shoe selesai.
    pub fn build(
        &self,
        decks: u8,
        penetration: f64,
        seed: Seed,
    ) -> Result<(Shoe, usize), GameError> {
        let shoe = match &self.shoe {
            Some(cards) => Shoe::from_cards(parse_cards(cards)?),
            None => {
                let decks = self.dek.unwrap_or(decks);
                if decks == 0 || decks > 8 {
                    return Err(GameError::Config(format!("dek {decks}")));
                }
                Shoe::shuffled(decks, &mut GameRng::from_seed(seed))
            }
        };
        let cut = self
            .potong
            .unwrap_or_else(|| (shoe.len() as f64 * penetration).round() as usize);
        Ok((shoe, cut))
    }
}

/// Konfigurasi game satu ronde per sesi; `kartu` = urutan dek tetap untuk
/// tes dan tutorial.
#[derive(Debug, Clone, Default, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RoundConfig {
    #[serde(default)]
    pub kartu: Option<Vec<String>>,
}

impl RoundConfig {
    /// Satu dek 52 kartu yang dikocok dari seed ronde, atau urutan tetap.
    pub fn deck(&self, seed: Seed) -> Result<Shoe, GameError> {
        match &self.kartu {
            Some(cards) => Ok(Shoe::from_cards(parse_cards(cards)?)),
            None => Ok(Shoe::shuffled(1, &mut GameRng::from_seed(seed))),
        }
    }
}

pub fn draw(shoe: &mut Shoe) -> Result<Card, GameError> {
    shoe.draw()
        .ok_or_else(|| GameError::Illegal("kartu habis".into()))
}

/// Bagian view yang sama untuk semua meja (kontrak chip host).
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Umum {
    pub fase: String,
    pub bersih: i64,
    pub ronde: u32,
    pub taruhan_meja: i64,
    pub dipertaruhkan: i64,
    pub biaya: BTreeMap<String, i64>,
    pub pengali: BTreeMap<String, i64>,
    pub min_taruhan: i64,
    pub maks_taruhan: i64,
    pub langkah_taruhan: i64,
    pub selesai: bool,
    /// `shoe_habis`, `berhenti`, atau `ronde_selesai`.
    pub alasan: Option<String>,
    /// Game ber-shoe: kartu terpakai, sisa, dan titik potong.
    pub kartu_terpakai: Option<usize>,
    pub sisa: Option<usize>,
    pub potong: Option<usize>,
}

impl Umum {
    pub fn new(fase: &str, limits: Limits) -> Umum {
        Umum {
            fase: fase.into(),
            min_taruhan: limits.min,
            maks_taruhan: limits.max,
            langkah_taruhan: limits.step,
            ..Umum::default()
        }
    }

    pub fn shoe(mut self, shoe: &Shoe, cut: usize) -> Umum {
        self.kartu_terpakai = Some(shoe.dealt());
        self.sisa = Some(shoe.remaining());
        self.potong = Some(cut);
        self
    }
}

/// Hasil sesi: menang bila bersih positif. `key` = kunci ringkasan di
/// katalog game dengan `{rounds}` dan `{net}`.
pub fn result(catalog: &'static Catalog, key: &'static str, net: i64, rounds: u32) -> GameResult {
    GameResult {
        winners: if net > 0 { vec![0] } else { Vec::new() },
        scores: vec![net],
        summary: Localized::build(|lang| {
            catalog.text(
                lang,
                key,
                &[
                    ("rounds", &rounds.to_string()),
                    ("net", &format!("{net:+}")),
                ],
            )
        }),
    }
}

/// Hasil tiap tempat taruhan dalam satu ronde.
pub type Payouts = BTreeMap<String, i64>;
