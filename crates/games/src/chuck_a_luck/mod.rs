//! Chuck-a-luck (SPEC §6.4; M6a, D-069): tiga dadu dalam sangkar, di atas
//! mesin papan taruhan bersama.
//!
//! `single-n` dibayar 1:1, 2:1, 10:1 untuk satu, dua, tiga dadu yang
//! cocok; `any-triple` 30:1; `big` (11–17) dan `small` (4–10) 1:1, kalah
//! bila triple.

use std::sync::OnceLock;

use kyusin_core::game::create_session;
use kyusin_core::i18n::Lang;
use kyusin_core::{Cartridge, GameRng, RegistryError};

use crate::dadu::{self, count, total, triple};
use crate::meja::WagerRtp;
use crate::papan_taruhan::{Board, BoardRules, leak};

pub const ID: &str = "chuck-a-luck";

/// RTP tiap jenis taruhan (tepat, enumerasi 216 hasil; D-069). Angka
/// manifest = taruhan angka.
pub const RTP: &[WagerRtp] = &[
    WagerRtp {
        wager: "single-3",
        percent: 95.3704,
        manifest: true,
    },
    WagerRtp {
        wager: "any-triple",
        percent: 86.1111,
        manifest: false,
    },
    WagerRtp {
        wager: "small",
        percent: 97.2222,
        manifest: false,
    },
    WagerRtp {
        wager: "big",
        percent: 97.2222,
        manifest: false,
    },
];

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Chuck;

impl BoardRules for Chuck {
    type Outcome = [u8; 3];

    fn spots() -> &'static [&'static str] {
        static S: OnceLock<&'static [&'static str]> = OnceLock::new();
        S.get_or_init(|| {
            let mut v: Vec<String> = (1..=6).map(|n| format!("single-{n}")).collect();
            v.extend(["any-triple", "small", "big"].map(String::from));
            leak(v)
        })
    }

    fn verb() -> &'static str {
        "roll"
    }

    fn draw(rng: &mut GameRng) -> [u8; 3] {
        dadu::roll3(rng)
    }

    fn settle(spot: &str, stake: i64, d: &[u8; 3]) -> i64 {
        let t = total(d);
        match spot.split_once('-') {
            Some(("single", n)) => match count(d, n.parse().unwrap_or(0)) {
                0 => -stake,
                1 => stake,
                2 => 2 * stake,
                _ => 10 * stake,
            },
            _ => {
                let (pays, ok) = match spot {
                    "any-triple" => (30, triple(d)),
                    "small" => (1, !triple(d) && t <= 10),
                    "big" => (1, !triple(d) && t >= 11),
                    _ => (0, false),
                };
                if ok { stake * pays } else { -stake }
            }
        }
    }

    fn outcomes() -> Vec<([u8; 3], f64)> {
        dadu::all3()
    }

    fn valid(d: &[u8; 3]) -> bool {
        dadu::valid3(d)
    }

    fn describe(d: &[u8; 3], _lang: Lang) -> String {
        format!("{} {} {}", d[0], d[1], d[2])
    }
}

pub type ChuckALuck = Board<Chuck>;

pub fn cartridge() -> Result<Cartridge, RegistryError> {
    Cartridge::new(
        include_str!("manifest.toml"),
        include_str!("../../../../tutorials/chuck-a-luck.toml"),
        create_session::<ChuckALuck>,
    )
}
