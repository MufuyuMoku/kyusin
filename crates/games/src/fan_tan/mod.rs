//! Fan-Tan gaya Macau (SPEC §6.4; M6a, D-069) di atas mesin papan taruhan
//! bersama.
//!
//! Bandar menutup segenggam kancing (40–119, jadi sisanya seragam), lalu
//! menghitungnya empat-empat; sisa 1–4 (sisa 0 = 4) adalah hasilnya.
//! Taruhan (komisi 5% dari kemenangan):
//! - `fan-n`: satu angka, 3:1;
//! - `nim-a-b`: menang 2:1 bila `a`, seri bila `b`;
//! - `kwok-a-b`: dua angka berdampingan (1-2, 2-3, 3-4, 1-4), 1:1;
//! - `tan-a-b-c`: menang 1:2 bila `a` atau `b`, seri bila `c`;
//! - `ssh-a-b-c`: tiga angka, 1:3.
//!
//! Taruhan kelipatan 120 (120–1.920) supaya komisi pada bayaran 1:2 dan 1:3
//! selalu utuh.

use std::sync::OnceLock;

use kyusin_core::game::create_session;
use kyusin_core::i18n::Lang;
use kyusin_core::{Cartridge, GameRng, RegistryError};

use crate::meja::{Limits, WagerRtp};
use crate::papan_taruhan::{Board, BoardRules, leak};

pub const ID: &str = "fan-tan";
pub const LIMITS: Limits = Limits {
    min: 120,
    max: 1920,
    step: 120,
};
/// Jumlah kancing paling sedikit dan banyaknya kemungkinan (kelipatan 4).
pub const BEANS_MIN: u32 = 40;
pub const BEANS_SPAN: u32 = 80;

/// RTP tiap jenis taruhan (tepat; D-069). Angka manifest = Fan.
pub const RTP: &[WagerRtp] = &[
    WagerRtp {
        wager: "fan-2",
        percent: 96.25,
        manifest: true,
    },
    WagerRtp {
        wager: "nim-1-2",
        percent: 97.5,
        manifest: false,
    },
    WagerRtp {
        wager: "kwok-2-3",
        percent: 97.5,
        manifest: false,
    },
    WagerRtp {
        wager: "tan-1-2-3",
        percent: 98.75,
        manifest: false,
    },
    WagerRtp {
        wager: "ssh-2-3-4",
        percent: 98.75,
        manifest: false,
    },
];

/// Angka hasil (1–4) dari jumlah kancing.
pub fn number(beans: u32) -> u8 {
    ((beans + 3) % 4 + 1) as u8
}

/// Kemenangan bersih setelah komisi 5%.
fn after_commission(win: i64) -> i64 {
    win - win / 20
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FanTanRules;

impl BoardRules for FanTanRules {
    type Outcome = u32;

    fn spots() -> &'static [&'static str] {
        static S: OnceLock<&'static [&'static str]> = OnceLock::new();
        S.get_or_init(|| {
            let mut v: Vec<String> = (1..=4).map(|n| format!("fan-{n}")).collect();
            for a in 1..=4 {
                for b in 1..=4 {
                    if a != b {
                        v.push(format!("nim-{a}-{b}"));
                    }
                }
            }
            for k in ["1-2", "2-3", "3-4", "1-4"] {
                v.push(format!("kwok-{k}"));
            }
            for a in 1..=4 {
                for b in a + 1..=4 {
                    for c in 1..=4 {
                        if c != a && c != b {
                            v.push(format!("tan-{a}-{b}-{c}"));
                        }
                    }
                }
            }
            for skip in (1..=4).rev() {
                let three: Vec<String> = (1..=4)
                    .filter(|n| *n != skip)
                    .map(|n| n.to_string())
                    .collect();
                v.push(format!("ssh-{}", three.join("-")));
            }
            leak(v)
        })
    }

    fn limits(_spot: &str) -> Limits {
        LIMITS
    }

    fn verb() -> &'static str {
        "open"
    }

    fn draw(rng: &mut GameRng) -> u32 {
        BEANS_MIN + rng.below(BEANS_SPAN)
    }

    fn settle(spot: &str, stake: i64, beans: &u32) -> i64 {
        let n = number(*beans);
        let parts: Vec<u8> = spot
            .split('-')
            .skip(1)
            .filter_map(|x| x.parse().ok())
            .collect();
        let kind = spot.split('-').next().unwrap_or("");
        let has = |x: &u8| *x == n;
        match kind {
            "fan" if has(&parts[0]) => after_commission(3 * stake),
            "nim" if has(&parts[0]) => after_commission(2 * stake),
            "nim" if has(&parts[1]) => 0,
            "kwok" if parts.iter().any(has) => after_commission(stake),
            "tan" if parts[..2].iter().any(has) => after_commission(stake / 2),
            "tan" if has(&parts[2]) => 0,
            "ssh" if parts.iter().any(has) => after_commission(stake / 3),
            _ => -stake,
        }
    }

    fn outcomes() -> Vec<(u32, f64)> {
        (BEANS_MIN..BEANS_MIN + BEANS_SPAN)
            .map(|b| (b, 1.0 / BEANS_SPAN as f64))
            .collect()
    }

    fn valid(beans: &u32) -> bool {
        *beans >= 1
    }

    fn extra(beans: &u32) -> serde_json::Map<String, serde_json::Value> {
        let mut m = serde_json::Map::new();
        m.insert("angka".into(), number(*beans).into());
        m
    }

    fn describe(beans: &u32, lang: Lang) -> String {
        match lang {
            Lang::Id => format!("{beans} kancing, sisa {}", number(*beans)),
            Lang::En => format!("{beans} buttons, remainder {}", number(*beans)),
        }
    }
}

pub type FanTan = Board<FanTanRules>;

pub fn cartridge() -> Result<Cartridge, RegistryError> {
    Cartridge::new(
        include_str!("manifest.toml"),
        include_str!("../../../../tutorials/fan-tan.toml"),
        create_session::<FanTan>,
    )
}
