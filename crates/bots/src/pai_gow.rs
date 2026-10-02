//! Pai Gow Poker: pemain otomatis memakai house way yang sama dengan bandar
//! (D-060). Dipakai untuk menghitung dan memverifikasi RTP (SPEC §7).

use kyusin_core::{Seed, TurnGame};
use kyusin_games::pai_gow::{self, PaiGow};

use crate::meja::{Wager, simulate};

pub fn wagers() -> Vec<Wager> {
    pai_gow::RTP
        .iter()
        .map(|&rtp| Wager {
            game: pai_gow::ID,
            rtp,
            run: |n, s: &Seed| {
                simulate(
                    n,
                    s,
                    100,
                    |seed| PaiGow::new(Default::default(), seed).expect("dek"),
                    |_, v| match v.meja.fase.as_str() {
                        "susun" => "houseway".into(),
                        _ => "bet 100".into(),
                    },
                )
            },
        })
        .collect()
}
