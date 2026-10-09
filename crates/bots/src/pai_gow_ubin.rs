//! Pai Gow ubin: pemain otomatis memakai house way yang sama dengan bandar
//! (D-069). Simulasi untuk memverifikasi RTP (SPEC §7) dan enumerasi tepat
//! semua pembagian ubin (berat; hanya di workflow `rtp.yml`).

use kyusin_core::{Seed, TurnGame};
use kyusin_games::pai_gow_ubin::{self, Hand, PaiGowUbin, compare, hand, house_way, set};
use std::cmp::Ordering;

use crate::meja::{Wager, simulate};

pub fn wagers() -> Vec<Wager> {
    pai_gow_ubin::RTP
        .iter()
        .map(|&rtp| Wager {
            game: pai_gow_ubin::ID,
            rtp,
            run: |n, s: &Seed| {
                simulate(
                    n,
                    s,
                    100,
                    |seed| PaiGowUbin::new(Default::default(), seed).expect("ubin"),
                    |_, v| match v.meja.fase.as_str() {
                        "susun" => "houseway".into(),
                        _ => "bet 100".into(),
                    },
                )
            },
        })
        .collect()
}

/// Jumlah pembagian (pemain, bandar) yang dimenangkan pemain di kedua
/// tangan, yang dimenangkan bandar di kedua tangan, dan semuanya.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct Count {
    pub win: u64,
    pub lose: u64,
    pub total: u64,
}

/// RTP tepat (persen) dari [`Count`]: menang dibayar 0,95, kalah −1.
pub fn percent(c: Count) -> f64 {
    100.0 * (1.0 + (0.95 * c.win as f64 - c.lose as f64) / c.total as f64)
}

/// Enumerasi semua C(32,4) × C(28,4) pembagian, house way lawan house way.
pub fn exact_count(threads: usize) -> Count {
    let tiles = set();
    let mut combos: Vec<(u32, Hand, Hand)> = Vec::with_capacity(35_960);
    for a in 0..32 {
        for b in a + 1..32 {
            for c in b + 1..32 {
                for d in c + 1..32 {
                    let (hi, lo) = house_way([tiles[a], tiles[b], tiles[c], tiles[d]]);
                    combos.push((
                        1 << a | 1 << b | 1 << c | 1 << d,
                        hand(hi[0], hi[1]),
                        hand(lo[0], lo[1]),
                    ));
                }
            }
        }
    }
    let combos = &combos;
    let threads = threads.max(1);
    std::thread::scope(|scope| {
        let parts: Vec<_> = (0..threads)
            .map(|t| {
                scope.spawn(move || {
                    let mut c = Count::default();
                    for (pm, phi, plo) in combos.iter().skip(t).step_by(threads) {
                        for (dm, dhi, dlo) in combos {
                            if pm & dm != 0 {
                                continue;
                            }
                            c.total += 1;
                            let hi = compare(phi, dhi) == Ordering::Greater;
                            let lo = compare(plo, dlo) == Ordering::Greater;
                            match (hi, lo) {
                                (true, true) => c.win += 1,
                                (false, false) => c.lose += 1,
                                _ => {}
                            }
                        }
                    }
                    c
                })
            })
            .collect();
        parts.into_iter().fold(Count::default(), |acc, h| {
            let c = h.join().expect("utas enumerasi");
            Count {
                win: acc.win + c.win,
                lose: acc.lose + c.lose,
                total: acc.total + c.total,
            }
        })
    })
}
