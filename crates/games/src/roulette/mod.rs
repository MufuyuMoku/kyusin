//! Roulette Eropa dan Amerika (SPEC §6.4; M6a, D-069) di atas mesin papan
//! taruhan bersama.
//!
//! Eropa: kantong 0–36 tanpa La Partage. Amerika: 0, 00, 1–36, ditambah
//! Top Line 0-00-1-2-3 (6:1). Taruhan: angka tunggal 35:1, split 17:1,
//! street dan trio 11:1, corner 8:1, line 5:1, lusin dan kolom 2:1,
//! merah/hitam/ganjil/genap/kecil/besar 1:1 (kalah pada 0 dan 00). Nama
//! tempat menyebut angkanya: `straight-17`, `split-17-18`,
//! `street-16-17-18`, `trio-0-1-2`, `corner-17-18-20-21`, `line-13-18`,
//! `topline`, `dozen-2`, `column-1`, `red`, `odd`, `low`, dst.

use std::sync::OnceLock;

use kyusin_core::game::create_session;
use kyusin_core::i18n::Lang;
use kyusin_core::{Cartridge, GameRng, RegistryError};

use crate::meja::WagerRtp;
use crate::papan_taruhan::{Board, BoardRules, leak};

pub const ID_EROPA: &str = "roulette-eropa";
pub const ID_AMERIKA: &str = "roulette-amerika";

/// Kantong 00 dalam angka internal.
pub const DOUBLE_ZERO: u8 = 37;

pub const RED: [u8; 18] = [
    1, 3, 5, 7, 9, 12, 14, 16, 18, 19, 21, 23, 25, 27, 30, 32, 34, 36,
];

/// RTP tiap jenis taruhan (tepat; D-069). Angka manifest = taruhan luar.
pub const RTP_EROPA: &[WagerRtp] = &[
    WagerRtp {
        wager: "red",
        percent: 97.2973,
        manifest: true,
    },
    WagerRtp {
        wager: "straight-17",
        percent: 97.2973,
        manifest: false,
    },
    WagerRtp {
        wager: "dozen-2",
        percent: 97.2973,
        manifest: false,
    },
];

pub const RTP_AMERIKA: &[WagerRtp] = &[
    WagerRtp {
        wager: "red",
        percent: 94.7368,
        manifest: true,
    },
    WagerRtp {
        wager: "straight-00",
        percent: 94.7368,
        manifest: false,
    },
    WagerRtp {
        wager: "topline",
        percent: 92.1053,
        manifest: false,
    },
];

/// Angka kantong dari labelnya (`0`–`36`, `00`).
pub fn pocket(label: &str) -> Option<u8> {
    if label == "00" {
        return Some(DOUBLE_ZERO);
    }
    let n: u8 = label.parse().ok()?;
    (n <= 36 && (n == 0 || !label.starts_with('0'))).then_some(n)
}

pub fn label(pocket: u8) -> String {
    if pocket == DOUBLE_ZERO {
        "00".into()
    } else {
        pocket.to_string()
    }
}

fn spots(american: bool) -> Vec<String> {
    let mut v = Vec::new();
    v.push("straight-0".to_string());
    if american {
        v.push("straight-00".into());
    }
    v.extend((1..=36).map(|n| format!("straight-{n}")));
    if american {
        for s in ["0-00", "0-1", "0-2", "00-2", "00-3"] {
            v.push(format!("split-{s}"));
        }
    } else {
        for s in ["0-1", "0-2", "0-3"] {
            v.push(format!("split-{s}"));
        }
    }
    for n in 1..=35u8 {
        if n % 3 != 0 {
            v.push(format!("split-{n}-{}", n + 1));
        }
    }
    for n in 1..=33u8 {
        v.push(format!("split-{n}-{}", n + 3));
    }
    for n in (1..=34u8).step_by(3) {
        v.push(format!("street-{n}-{}-{}", n + 1, n + 2));
    }
    if american {
        for s in ["0-1-2", "0-00-2", "00-2-3"] {
            v.push(format!("trio-{s}"));
        }
    } else {
        for s in ["0-1-2", "0-2-3"] {
            v.push(format!("trio-{s}"));
        }
    }
    for n in 1..=32u8 {
        if n % 3 != 0 {
            v.push(format!("corner-{n}-{}-{}-{}", n + 1, n + 3, n + 4));
        }
    }
    for n in (1..=31u8).step_by(3) {
        v.push(format!("line-{n}-{}", n + 5));
    }
    if american {
        v.push("topline".into());
    }
    for s in [
        "dozen-1", "dozen-2", "dozen-3", "column-1", "column-2", "column-3", "red", "black", "odd",
        "even", "low", "high",
    ] {
        v.push(s.into());
    }
    v
}

/// Kantong yang dicakup sebuah taruhan dan bayarannya (x:1).
pub fn coverage(spot: &str) -> Option<(Vec<u8>, i64)> {
    let numbers = |rest: &str| -> Option<Vec<u8>> { rest.split('-').map(pocket).collect() };
    let (kind, rest) = spot.split_once('-').unwrap_or((spot, ""));
    Some(match kind {
        "straight" => (numbers(rest)?, 35),
        "split" => (numbers(rest)?, 17),
        "street" | "trio" => (numbers(rest)?, 11),
        "corner" => (numbers(rest)?, 8),
        "line" => {
            let ends = numbers(rest)?;
            ((ends[0]..=ends[1]).collect(), 5)
        }
        "topline" => (vec![0, DOUBLE_ZERO, 1, 2, 3], 6),
        "dozen" => {
            let d: u8 = rest.parse().ok()?;
            ((12 * (d - 1) + 1..=12 * d).collect(), 2)
        }
        "column" => {
            let c: u8 = rest.parse().ok()?;
            ((0..12).map(|r| 3 * r + c).collect(), 2)
        }
        "red" => (RED.to_vec(), 1),
        "black" => ((1..=36).filter(|n| !RED.contains(n)).collect(), 1),
        "odd" => ((1..=36).filter(|n| n % 2 == 1).collect(), 1),
        "even" => ((1..=36).filter(|n| n % 2 == 0).collect(), 1),
        "low" => ((1..=18).collect(), 1),
        "high" => ((19..=36).collect(), 1),
        _ => return None,
    })
}

fn settle(spot: &str, stake: i64, outcome: &str) -> i64 {
    let (covered, pays) = coverage(spot).expect("tempat roulette");
    match pocket(outcome) {
        Some(p) if covered.contains(&p) => stake * pays,
        _ => -stake,
    }
}

fn describe(outcome: &str, lang: Lang) -> String {
    let p = pocket(outcome).unwrap_or(0);
    let colour = match (p, lang) {
        (0 | DOUBLE_ZERO, Lang::Id) => "hijau",
        (0 | DOUBLE_ZERO, Lang::En) => "green",
        (n, Lang::Id) if RED.contains(&n) => "merah",
        (n, Lang::En) if RED.contains(&n) => "red",
        (_, Lang::Id) => "hitam",
        (_, Lang::En) => "black",
    };
    format!("{outcome} ({colour})")
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Eropa;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Amerika;

impl BoardRules for Eropa {
    type Outcome = String;

    fn spots() -> &'static [&'static str] {
        static S: OnceLock<&'static [&'static str]> = OnceLock::new();
        S.get_or_init(|| leak(spots(false)))
    }

    fn verb() -> &'static str {
        "spin"
    }

    fn draw(rng: &mut GameRng) -> String {
        label(rng.below(37) as u8)
    }

    fn settle(spot: &str, stake: i64, outcome: &String) -> i64 {
        settle(spot, stake, outcome)
    }

    fn outcomes() -> Vec<(String, f64)> {
        (0..37u8).map(|p| (label(p), 1.0 / 37.0)).collect()
    }

    fn valid(outcome: &String) -> bool {
        pocket(outcome).is_some_and(|p| p != DOUBLE_ZERO)
    }

    fn describe(outcome: &String, lang: Lang) -> String {
        describe(outcome, lang)
    }
}

impl BoardRules for Amerika {
    type Outcome = String;

    fn spots() -> &'static [&'static str] {
        static S: OnceLock<&'static [&'static str]> = OnceLock::new();
        S.get_or_init(|| leak(spots(true)))
    }

    fn verb() -> &'static str {
        "spin"
    }

    fn draw(rng: &mut GameRng) -> String {
        label(rng.below(38) as u8)
    }

    fn settle(spot: &str, stake: i64, outcome: &String) -> i64 {
        settle(spot, stake, outcome)
    }

    fn outcomes() -> Vec<(String, f64)> {
        (0..38u8).map(|p| (label(p), 1.0 / 38.0)).collect()
    }

    fn valid(outcome: &String) -> bool {
        pocket(outcome).is_some()
    }

    fn describe(outcome: &String, lang: Lang) -> String {
        describe(outcome, lang)
    }
}

pub type RouletteEropa = Board<Eropa>;
pub type RouletteAmerika = Board<Amerika>;

pub fn cartridge_eropa() -> Result<Cartridge, RegistryError> {
    Cartridge::new(
        include_str!("eropa.toml"),
        include_str!("../../../../tutorials/roulette-eropa.toml"),
        create_session::<RouletteEropa>,
    )
}

pub fn cartridge_amerika() -> Result<Cartridge, RegistryError> {
    Cartridge::new(
        include_str!("amerika.toml"),
        include_str!("../../../../tutorials/roulette-amerika.toml"),
        create_session::<RouletteAmerika>,
    )
}
