//! Craps: RTP tiap taruhan dihitung per taruhan yang diselesaikan (menang,
//! kalah, atau seri; D-069). Pemain otomatis memasang taruhan terkecil di
//! satu tempat setiap kali tempat itu kosong dan terbuka, lalu melempar.
//! Satu pertandingan = satu penembak, jadi simulasi berganti penembak
//! sampai jumlah taruhan yang diselesaikan tercapai (SPEC §7).

use kyusin_core::rng::derive;
use kyusin_core::{Seed, TurnGame};
use kyusin_games::craps::{self, Craps};

use crate::meja::{Acc, Rtp, Wager};

pub fn simulate_wager(spot: &str, rounds: u64, seed: &Seed) -> Rtp {
    let mut acc = Acc::default();
    let mut shooter = 0u64;
    while acc.len() < rounds {
        let mut g =
            Craps::new(Default::default(), derive(seed, &format!("rtp:{shooter}"))).expect("craps");
        shooter += 1;
        while !g.is_over() && acc.len() < rounds {
            let v = g.view_for(0);
            let up = v.taruhan.get(spot).copied().unwrap_or(0);
            if up == 0
                && let Some(&[min, _, _]) = v.batas.get(spot)
            {
                let a = g.parse_command(&format!("bet {spot} {min}")).expect("bet");
                g.apply(0, a).expect("bet");
                continue;
            }
            assert!(up > 0, "{spot}: tempat tertutup tanpa taruhan");
            let a = g.parse_command("roll").expect("roll");
            g.apply(0, a).expect("roll");
            if let Some(&net) = g.view_for(0).bayar.get(spot) {
                acc.push(net as f64 / up as f64);
            }
        }
    }
    acc.finish()
}

pub fn wagers() -> Vec<Wager> {
    craps::RTP
        .iter()
        .map(|&rtp| Wager {
            game: craps::ID,
            rtp,
            run: match rtp.wager {
                "pass" => |n, s| simulate_wager("pass", n, s),
                "dont-pass" => |n, s| simulate_wager("dont-pass", n, s),
                "place-6" => |n, s| simulate_wager("place-6", n, s),
                "place-5" => |n, s| simulate_wager("place-5", n, s),
                "place-4" => |n, s| simulate_wager("place-4", n, s),
                "field" => |n, s| simulate_wager("field", n, s),
                "hard-6" => |n, s| simulate_wager("hard-6", n, s),
                "hard-4" => |n, s| simulate_wager("hard-4", n, s),
                w => panic!("craps: taruhan {w} tanpa simulator"),
            },
        })
        .collect()
}
