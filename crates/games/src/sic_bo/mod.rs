//! Sic Bo (SPEC §6.4; M6a, D-069): tiga dadu, tabel bayar Macau/Hong Kong,
//! di atas mesin papan taruhan bersama.
//!
//! `small`/`big` (total 4–10 / 11–17), `odd`/`even` 1:1, kalah bila
//! triple; `total-4` … `total-17` (50, 18, 14, 12, 8, 6, 6 : 1, simetris);
//! `combo-a-b` dua angka tertentu muncul 6:1; `double-n` 8:1 (triple juga
//! dihitung); `any-triple` 24:1; `triple-n` 150:1; `single-n` 1:1, 2:1,
//! 3:1 untuk satu, dua, tiga dadu yang cocok.

use std::sync::OnceLock;

use kyusin_core::game::create_session;
use kyusin_core::i18n::Lang;
use kyusin_core::{Cartridge, GameRng, RegistryError};

use crate::dadu::{self, count, total, triple};
use crate::meja::WagerRtp;
use crate::papan_taruhan::{Board, BoardRules, leak};

pub const ID: &str = "sic-bo";

/// RTP tiap jenis taruhan (tepat, enumerasi 216 hasil; D-069). Angka
/// manifest = kecil/besar.
pub const RTP: &[WagerRtp] = &[
    WagerRtp {
        wager: "small",
        percent: 97.2222,
        manifest: true,
    },
    WagerRtp {
        wager: "big",
        percent: 97.2222,
        manifest: false,
    },
    WagerRtp {
        wager: "odd",
        percent: 97.2222,
        manifest: false,
    },
    WagerRtp {
        wager: "combo-2-5",
        percent: 97.2222,
        manifest: false,
    },
    WagerRtp {
        wager: "total-4",
        percent: 70.8333,
        manifest: false,
    },
    WagerRtp {
        wager: "total-10",
        percent: 87.5,
        manifest: false,
    },
    WagerRtp {
        wager: "double-3",
        percent: 66.6667,
        manifest: false,
    },
    WagerRtp {
        wager: "any-triple",
        percent: 69.4444,
        manifest: false,
    },
    WagerRtp {
        wager: "triple-6",
        percent: 69.9074,
        manifest: false,
    },
    WagerRtp {
        wager: "single-4",
        percent: 92.1296,
        manifest: false,
    },
];

/// Bayaran total (x:1) untuk total 4–17.
pub fn total_pays(t: u8) -> Option<i64> {
    Some(match t.min(21 - t) {
        4 => 50,
        5 => 18,
        6 => 14,
        7 => 12,
        8 => 8,
        9 | 10 => 6,
        _ => return None,
    })
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SicBoRules;

impl BoardRules for SicBoRules {
    type Outcome = [u8; 3];

    fn spots() -> &'static [&'static str] {
        static S: OnceLock<&'static [&'static str]> = OnceLock::new();
        S.get_or_init(|| {
            let mut v: Vec<String> = ["small", "big", "odd", "even"].map(String::from).to_vec();
            v.extend((4..=17).map(|t| format!("total-{t}")));
            for a in 1..=6 {
                for b in a + 1..=6 {
                    v.push(format!("combo-{a}-{b}"));
                }
            }
            v.extend((1..=6).map(|n| format!("double-{n}")));
            v.push("any-triple".into());
            v.extend((1..=6).map(|n| format!("triple-{n}")));
            v.extend((1..=6).map(|n| format!("single-{n}")));
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
        let num = |s: &str| s.parse::<u8>().unwrap_or(0);
        let win = |pays: i64, ok: bool| if ok { stake * pays } else { -stake };
        let (kind, rest) = spot.split_once('-').unwrap_or((spot, ""));
        match kind {
            "small" => win(1, !triple(d) && t <= 10),
            "big" => win(1, !triple(d) && t >= 11),
            "odd" => win(1, !triple(d) && t % 2 == 1),
            "even" => win(1, !triple(d) && t % 2 == 0),
            "total" => {
                let want = num(rest);
                win(total_pays(want).unwrap_or(0), t == want)
            }
            "combo" => {
                let (a, b) = rest.split_once('-').unwrap_or(("0", "0"));
                win(6, count(d, num(a)) > 0 && count(d, num(b)) > 0)
            }
            "double" => win(8, count(d, num(rest)) >= 2),
            "any" => win(24, triple(d)),
            "triple" => win(150, triple(d) && d[0] == num(rest)),
            "single" => match count(d, num(rest)) {
                0 => -stake,
                k => stake * k,
            },
            _ => -stake,
        }
    }

    fn outcomes() -> Vec<([u8; 3], f64)> {
        dadu::all3()
    }

    fn valid(d: &[u8; 3]) -> bool {
        dadu::valid3(d)
    }

    fn describe(d: &[u8; 3], lang: Lang) -> String {
        match lang {
            Lang::Id => format!("{} {} {} (total {})", d[0], d[1], d[2], total(d)),
            Lang::En => format!("{} {} {} (total {})", d[0], d[1], d[2], total(d)),
        }
    }
}

pub type SicBo = Board<SicBoRules>;

pub fn cartridge() -> Result<Cartridge, RegistryError> {
    Cartridge::new(
        include_str!("manifest.toml"),
        include_str!("../../../../tutorials/sic-bo.toml"),
        create_session::<SicBo>,
    )
}
