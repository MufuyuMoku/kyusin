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
//!   `pengali[kata kerja]` (bawaan 1);
//! - `netral`: aksi tanpa tambahan taruhan yang dimainkan host bila pemain
//!   membuang pertandingan di tengah ronde.
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
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
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
    /// Aksi yang dimainkan host bila pemain membuang pertandingan di tengah
    /// ronde: pilihan yang tidak menambah taruhan (seperti `stand` di
    /// Blackjack, D-058/D-059). `None` di antara ronde (`leave`).
    pub netral: Option<String>,
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

/// Tempat taruhan yang disusun sebelum kartu dibagi (`bet <tempat>
/// <jumlah>` berkali-kali sampai batas per tempat, `clear` menarik semua).
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Spots {
    names: &'static [&'static str],
    limits: Limits,
    bets: BTreeMap<String, i64>,
}

impl Spots {
    pub fn new(names: &'static [&'static str], limits: Limits) -> Spots {
        Spots {
            names,
            limits,
            bets: BTreeMap::new(),
        }
    }

    pub fn get(&self, spot: &str) -> i64 {
        self.bets.get(spot).copied().unwrap_or(0)
    }

    pub fn total(&self) -> i64 {
        self.bets.values().sum()
    }

    pub fn is_empty(&self) -> bool {
        self.bets.is_empty()
    }

    /// Taruhan yang terpasang (hanya tempat berisi).
    pub fn bets(&self) -> &BTreeMap<String, i64> {
        &self.bets
    }

    /// Mengambil semua taruhan untuk dibagi; tempat kosong lagi.
    pub fn take(&mut self) -> BTreeMap<String, i64> {
        std::mem::take(&mut self.bets)
    }

    pub fn clear(&mut self) {
        self.bets.clear();
    }

    /// Templat `bet <tempat> <jumlah>` untuk tempat yang belum penuh.
    pub fn legal(&self) -> Vec<ActionSpec> {
        self.names
            .iter()
            .filter_map(|s| self.limits.spot(s, self.get(s)))
            .collect()
    }

    /// Menambah taruhan; `false` bila tempat atau jumlahnya tidak sah.
    pub fn place(&mut self, spot: &str, n: i64) -> bool {
        let Some(name) = self.names.iter().find(|s| **s == spot) else {
            return false;
        };
        let now = self.get(name);
        if n < self.limits.min || n % self.limits.step != 0 || now + n > self.limits.max {
            return false;
        }
        self.bets.insert((*name).to_string(), now + n);
        true
    }
}

/// View meja yang punya bagian [`Umum`] (untuk simulasi RTP dan host).
pub trait MejaView {
    fn meja(&self) -> &Umum;

    /// Ronde yang sudah selesai (sama dengan cara host menghitungnya).
    fn settled(&self) -> u32 {
        let m = self.meja();
        match m.fase.as_str() {
            "taruhan" | "selesai" => m.ronde,
            _ => m.ronde.saturating_sub(1),
        }
    }
}

/// RTP satu jenis taruhan: nama, persen, dan apakah ini angka manifest.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct WagerRtp {
    pub wager: &'static str,
    pub percent: f64,
    pub manifest: bool,
}

/// Nama tangan poker (`pair`, `royal_flush`, …) dalam bahasa `lang`.
pub fn hand_name(lang: kyusin_core::i18n::Lang, key: &str) -> String {
    static CATALOG: std::sync::OnceLock<Catalog> = std::sync::OnceLock::new();
    CATALOG
        .get_or_init(|| Catalog::from_toml(include_str!("meja_i18n.toml")).expect("meja_i18n.toml"))
        .text(lang, key, &[])
}
