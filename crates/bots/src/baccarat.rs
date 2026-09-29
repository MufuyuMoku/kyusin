//! Baccarat tidak punya keputusan selain tempat taruhan: pemain otomatis
//! memasang satu tempat 100 chip per ronde untuk memverifikasi RTP-nya
//! (SPEC §7).

use kyusin_core::{Seed, TurnGame};
use kyusin_games::baccarat::{self, Baccarat};

use crate::meja::{Rtp, Wager, simulate};

pub fn simulate_wager(wager: &str, rounds: u64, seed: &Seed) -> Rtp {
    simulate(
        rounds,
        seed,
        100,
        |s| Baccarat::new(Default::default(), s).expect("shoe"),
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
    baccarat::RTP
        .iter()
        .map(|&rtp| Wager {
            game: baccarat::ID,
            rtp,
            run: match rtp.wager {
                "player" => |n, s| simulate_wager("player", n, s),
                "banker" => |n, s| simulate_wager("banker", n, s),
                _ => |n, s| simulate_wager("tie", n, s),
            },
        })
        .collect()
}
