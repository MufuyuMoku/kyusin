//! Strategi Red Dog: raise pada jarak 7 ke atas, selain itu call (strategi
//! terbaik; D-060). Dipakai untuk memverifikasi RTP (SPEC §7).

use kyusin_core::{Seed, TurnGame};
use kyusin_games::red_dog::{self, RedDog, View};

use crate::meja::{Rtp, Wager, simulate};

/// Jarak terkecil yang di-raise.
pub const RAISE_FROM: u8 = 7;

pub fn decide(v: &View, bet: i64) -> String {
    match v.meja.fase.as_str() {
        "naikkan" if v.jarak.is_some_and(|s| s >= RAISE_FROM) => "raise".into(),
        "naikkan" => "call".into(),
        _ => format!("bet {bet}"),
    }
}

pub fn wagers() -> Vec<Wager> {
    red_dog::RTP
        .iter()
        .map(|&rtp| Wager {
            game: red_dog::ID,
            rtp,
            run: |n, s: &Seed| -> Rtp {
                simulate(
                    n,
                    s,
                    100,
                    |seed| RedDog::new(Default::default(), seed).expect("shoe"),
                    |_, v| decide(v, 100),
                )
            },
        })
        .collect()
}
