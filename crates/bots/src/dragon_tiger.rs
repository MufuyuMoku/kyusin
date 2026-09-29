//! Dragon Tiger tidak punya keputusan: pemain otomatis hanya memasang satu
//! jenis taruhan 100 chip tiap ronde, untuk memverifikasi RTP-nya (SPEC §7).

use kyusin_core::{Seed, TurnGame};
use kyusin_games::dragon_tiger::{self, DragonTiger};

use crate::meja::{Rtp, Wager, simulate};

pub fn simulate_wager(wager: &str, rounds: u64, seed: &Seed) -> Rtp {
    simulate(
        rounds,
        seed,
        100,
        |s| DragonTiger::new(Default::default(), s).expect("shoe"),
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
    dragon_tiger::RTP
        .iter()
        .map(|&rtp| Wager {
            game: dragon_tiger::ID,
            rtp,
            run: match rtp.wager {
                "dragon" => |n, s| simulate_wager("dragon", n, s),
                "tiger" => |n, s| simulate_wager("tiger", n, s),
                _ => |n, s| simulate_wager("tie", n, s),
            },
        })
        .collect()
}
