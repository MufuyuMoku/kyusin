//! Strategi Casino War: selalu `war` saat seri (lebih baik daripada
//! `surrender`; D-060). Dipakai untuk memverifikasi RTP (SPEC §7).

use kyusin_core::{Seed, TurnGame};
use kyusin_games::casino_war::{self, CasinoWar, View};

use crate::meja::{Rtp, Wager, simulate};

/// Perintah strategi: pasang `wager` 100 (ante juga, karena wajib), bagi,
/// dan selalu berperang.
pub fn decide(v: &View, wager: &str) -> String {
    match v.meja.fase.as_str() {
        "perang" => "war".into(),
        _ if v.taruhan.get(wager).is_none() => format!("bet {wager} 100"),
        _ if v.taruhan.get("ante").is_none() => "bet ante 100".into(),
        _ => "deal".into(),
    }
}

pub fn simulate_wager(wager: &'static str, rounds: u64, seed: &Seed) -> Rtp {
    if wager == "ante" {
        return simulate(
            rounds,
            seed,
            100,
            |s| CasinoWar::new(Default::default(), s).expect("shoe"),
            |_, v| decide(v, "ante"),
        );
    }
    // Taruhan tie diukur sendiri: hasil tie dibagi taruhan tie, tanpa ante.
    let mut acc = crate::meja::Acc::default();
    let mut session = 0u64;
    while acc.len() < rounds {
        let s = kyusin_core::rng::derive(seed, &format!("rtp:{session}"));
        session += 1;
        let mut g = CasinoWar::new(Default::default(), s).expect("shoe");
        while !g.is_over() && acc.len() < rounds {
            let v = g.view_for(0);
            let cmd = decide(&v, wager);
            let a = g.parse_command(&cmd).expect("perintah strategi");
            let before = v.meja.ronde;
            let was_betting = v.meja.fase == "taruhan";
            g.apply(0, a).expect("aksi strategi sah");
            let after = g.view_for(0);
            // Taruhan tie selesai begitu kartu pertama dibagi.
            if was_betting && after.meja.ronde > before {
                let x = after.bayar.get(wager).copied().unwrap_or(0) as f64 / 100.0;
                acc.push(x);
            }
        }
    }
    acc.finish()
}

pub fn wagers() -> Vec<Wager> {
    casino_war::RTP
        .iter()
        .map(|&rtp| Wager {
            game: casino_war::ID,
            rtp,
            run: match rtp.wager {
                "ante" => |n, s| simulate_wager("ante", n, s),
                _ => |n, s| simulate_wager("tie", n, s),
            },
        })
        .collect()
}
