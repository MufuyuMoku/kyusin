//! Big Six / Money Wheel (SPEC §6.4; M6a, D-069) di atas mesin papan
//! taruhan bersama.
//!
//! Roda 54 segmen gaya Las Vegas: 24 × `1` (1:1), 15 × `2` (2:1), 7 × `5`
//! (5:1), 4 × `10` (10:1), 2 × `20` (20:1), 1 `joker` dan 1 `logo` (40:1).
//! Taruhan dibayar bila roda berhenti di simbol yang sama.

use std::sync::OnceLock;

use kyusin_core::game::create_session;
use kyusin_core::i18n::Lang;
use kyusin_core::{Cartridge, GameRng, RegistryError};

use crate::meja::WagerRtp;
use crate::papan_taruhan::{Board, BoardRules};

pub const ID: &str = "big-six";

pub const SYMBOLS: &[&str] = &["1", "2", "5", "10", "20", "joker", "logo"];
const COUNTS: [usize; 7] = [24, 15, 7, 4, 2, 1, 1];
const PAYS: [i64; 7] = [1, 2, 5, 10, 20, 40, 40];

/// RTP tiap simbol (tepat; D-069). Angka manifest = `1` (taruhan terbaik
/// sekaligus yang paling sering; semua simbol di bawah 89%).
pub const RTP: &[WagerRtp] = &[
    WagerRtp {
        wager: "1",
        percent: 88.8889,
        manifest: true,
    },
    WagerRtp {
        wager: "2",
        percent: 83.3333,
        manifest: false,
    },
    WagerRtp {
        wager: "5",
        percent: 77.7778,
        manifest: false,
    },
    WagerRtp {
        wager: "10",
        percent: 81.4815,
        manifest: false,
    },
    WagerRtp {
        wager: "20",
        percent: 77.7778,
        manifest: false,
    },
    WagerRtp {
        wager: "joker",
        percent: 75.9259,
        manifest: false,
    },
    WagerRtp {
        wager: "logo",
        percent: 75.9259,
        manifest: false,
    },
];

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Wheel;

impl Wheel {
    /// Segmen roda berurutan searah jarum jam. Simbol yang jarang disebar
    /// supaya roda tampak wajar; peluangnya hanya bergantung jumlahnya.
    pub fn segments() -> &'static [&'static str] {
        static S: OnceLock<Vec<&'static str>> = OnceLock::new();
        S.get_or_init(|| {
            let mut left = COUNTS;
            let mut out = Vec::with_capacity(54);
            // Isi bergiliran dari simbol yang paling banyak tersisa,
            // relatif terhadap jumlah awalnya, supaya tersebar rata.
            for _ in 0..54 {
                let i = (0..7)
                    .filter(|&i| left[i] > 0)
                    .max_by(|&a, &b| {
                        let fa = left[a] as f64 / COUNTS[a] as f64;
                        let fb = left[b] as f64 / COUNTS[b] as f64;
                        fa.partial_cmp(&fb).unwrap().then(b.cmp(&a))
                    })
                    .expect("segmen tersisa");
                left[i] -= 1;
                out.push(SYMBOLS[i]);
            }
            out
        })
    }
}

impl BoardRules for Wheel {
    type Outcome = String;

    fn spots() -> &'static [&'static str] {
        SYMBOLS
    }

    fn verb() -> &'static str {
        "spin"
    }

    fn draw(rng: &mut GameRng) -> String {
        Wheel::segments()[rng.below(54) as usize].to_string()
    }

    fn settle(spot: &str, stake: i64, outcome: &String) -> i64 {
        let i = SYMBOLS.iter().position(|s| *s == spot).expect("simbol");
        if outcome == spot {
            stake * PAYS[i]
        } else {
            -stake
        }
    }

    fn outcomes() -> Vec<(String, f64)> {
        SYMBOLS
            .iter()
            .zip(COUNTS)
            .map(|(s, n)| (s.to_string(), n as f64 / 54.0))
            .collect()
    }

    fn valid(outcome: &String) -> bool {
        SYMBOLS.contains(&outcome.as_str())
    }

    fn describe(outcome: &String, _lang: Lang) -> String {
        outcome.clone()
    }
}

pub type BigSix = Board<Wheel>;

pub fn cartridge() -> Result<Cartridge, RegistryError> {
    Cartridge::new(
        include_str!("manifest.toml"),
        include_str!("../../../../tutorials/big-six.toml"),
        create_session::<BigSix>,
    )
}
