//! Casino lotere dan instan M6b (Keno, Bingo, kartu gosok, Hi-Lo): pemain
//! otomatis yang memverifikasi RTP tiap taruhan yang diumumkan (SPEC §7).
//! Keno, Bingo, dan kartu gosok tidak punya keputusan; Hi-Lo memakai
//! strategi terbaik yang didokumentasikan (satu tebakan dengan pengembalian
//! lebih besar, lalu cash out; D-072).

use kyusin_core::{Seed, TurnGame};
use kyusin_games::{bingo, hi_lo, kartu_gosok, keno};

use crate::meja::{Rtp, Wager, simulate};

/// Keno dengan `K` angka (1..=K), taruhan 10.
fn keno_spots<const K: u8>(rounds: u64, seed: &Seed) -> Rtp {
    simulate(
        rounds,
        seed,
        10,
        |s| keno::Keno::new(Default::default(), s).expect("keno"),
        |_, v| {
            if v.pilihan.len() < usize::from(K) {
                format!("mark {}", v.pilihan.len() + 1)
            } else {
                "bet 10".into()
            }
        },
    )
}

fn keno_wagers() -> Vec<Wager> {
    let runs: [fn(u64, &Seed) -> Rtp; 10] = [
        keno_spots::<1>,
        keno_spots::<2>,
        keno_spots::<3>,
        keno_spots::<4>,
        keno_spots::<5>,
        keno_spots::<6>,
        keno_spots::<7>,
        keno_spots::<8>,
        keno_spots::<9>,
        keno_spots::<10>,
    ];
    keno::RTP
        .iter()
        .zip(runs)
        .map(|(&rtp, run)| Wager {
            game: keno::ID,
            rtp,
            run,
        })
        .collect()
}

/// Strategi Hi-Lo terbaik: tebakan pertama dengan n × pengali terbesar,
/// lalu cash out.
pub fn hi_lo_choice(view: &hi_lo::View) -> String {
    match view.meja.fase.as_str() {
        "taruhan" => "bet 100".into(),
        _ if view.tebakan > 0 => "cashout".into(),
        _ => {
            let card = view
                .kartu
                .last()
                .and_then(|c| c.parse::<kyusin_games::cards::Card>().ok());
            let r = card.map_or(7, hi_lo::rank);
            let hi = u32::from(14 - r) * hi_lo::multiplier(r, true);
            let lo = u32::from(r) * hi_lo::multiplier(r, false);
            if hi >= lo { "higher" } else { "lower" }.into()
        }
    }
}

pub fn wagers() -> Vec<Wager> {
    let mut out = keno_wagers();
    out.push(Wager {
        game: bingo::ID,
        rtp: bingo::RTP[0],
        run: |n, s| {
            simulate(
                n,
                s,
                10,
                |seed| bingo::Bingo::new(Default::default(), seed).expect("bingo"),
                |_, _| "bet 10".into(),
            )
        },
    });
    out.push(Wager {
        game: kartu_gosok::ID,
        rtp: kartu_gosok::RTP[0],
        run: |n, s| {
            simulate(
                n,
                s,
                10,
                |seed| kartu_gosok::KartuGosok::new(Default::default(), seed).expect("kartu"),
                |_, v| match v.meja.fase.as_str() {
                    "gosok" => "scratch all".into(),
                    _ => "bet 10".into(),
                },
            )
        },
    });
    out.push(Wager {
        game: hi_lo::ID,
        rtp: hi_lo::RTP[0],
        run: |n, s| {
            simulate(
                n,
                s,
                100,
                |seed| hi_lo::HiLo::new(Default::default(), seed).expect("hi-lo"),
                |_, v| hi_lo_choice(v),
            )
        },
    });
    out
}
