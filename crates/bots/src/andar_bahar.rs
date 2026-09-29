//! Andar Bahar tidak punya keputusan selain sisi: pemain otomatis memasang
//! satu sisi 100 chip per ronde untuk memverifikasi RTP-nya (SPEC §7).

use kyusin_core::{Seed, TurnGame};
use kyusin_games::andar_bahar::{self, AndarBahar};

use crate::meja::{Rtp, Wager, simulate};

pub fn simulate_wager(wager: &str, rounds: u64, seed: &Seed) -> Rtp {
    simulate(
        rounds,
        seed,
        100,
        |s| AndarBahar::new(Default::default(), s).expect("dek"),
        |_, v| {
            if v.taruhan.is_empty() {
                format!("bet {wager} 100")
            } else {
                "deal".into()
            }
        },
    )
}

pub fn wagers() -> Vec<Wager> {
    andar_bahar::RTP
        .iter()
        .map(|&rtp| Wager {
            game: andar_bahar::ID,
            rtp,
            run: match rtp.wager {
                "andar" => |n, s| simulate_wager("andar", n, s),
                _ => |n, s| simulate_wager("bahar", n, s),
            },
        })
        .collect()
}
