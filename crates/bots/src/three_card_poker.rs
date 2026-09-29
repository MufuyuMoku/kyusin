//! Strategi Three Card Poker: play dengan Q-6-4 atau lebih baik, fold
//! selain itu (strategi terbaik untuk Ante/Play; D-060). Dipakai untuk
//! menghitung dan memverifikasi RTP (SPEC §7).

use kyusin_core::{Seed, TurnGame};
use kyusin_games::cards::Card;
use kyusin_games::poker::{Value3, eval3};
use kyusin_games::three_card_poker::{self, ThreeCardPoker, View};

use crate::meja::{Rtp, Wager, simulate};

/// Tangan terlemah yang dimainkan: Q-6-4 (jenis berbeda).
pub fn threshold() -> Value3 {
    let q64: Vec<Card> = ["Qh", "6d", "4c"]
        .iter()
        .map(|c| c.parse().unwrap())
        .collect();
    eval3(&q64)
}

pub fn plays(hand: Value3) -> bool {
    hand >= threshold()
}

pub fn decide(v: &View, wager: &str) -> String {
    match v.meja.fase.as_str() {
        "keputusan" => {
            let cards: Vec<Card> = v.pemain.iter().filter_map(|c| c.parse().ok()).collect();
            if plays(eval3(&cards)) { "play" } else { "fold" }.into()
        }
        _ if v.taruhan.is_empty() => format!("bet {wager} 100"),
        _ => "deal".into(),
    }
}

pub fn simulate_wager(wager: &'static str, rounds: u64, seed: &Seed) -> Rtp {
    simulate(
        rounds,
        seed,
        100,
        |s| ThreeCardPoker::new(Default::default(), s).expect("dek"),
        |_, v| decide(v, wager),
    )
}

pub fn wagers() -> Vec<Wager> {
    three_card_poker::RTP
        .iter()
        .map(|&rtp| Wager {
            game: three_card_poker::ID,
            rtp,
            run: match rtp.wager {
                "ante" => |n, s| simulate_wager("ante", n, s),
                _ => |n, s| simulate_wager("pairplus", n, s),
            },
        })
        .collect()
}

/// RTP Ante/Play tepat: semua 22.100 tangan pemain × 18.424 tangan bandar
/// dari sisa dek, dengan strategi [`plays`]. Dijalankan di workflow
/// `rtp.yml` (407 juta perbandingan), bukan di tes setiap push.
pub fn exact_ante_play() -> f64 {
    use kyusin_games::three_card_poker::{ante_bonus, showdown};
    let deck = Card::deck();
    let mut value = vec![Value3(0); 52 * 52 * 52];
    let idx = |a: usize, b: usize, c: usize| (a * 52 + b) * 52 + c;
    for a in 0..52 {
        for b in a + 1..52 {
            for c in b + 1..52 {
                value[idx(a, b, c)] = eval3(&[deck[a], deck[b], deck[c]]);
            }
        }
    }
    let mut total = 0.0f64;
    let mut hands = 0u64;
    for a in 0..52 {
        for b in a + 1..52 {
            for c in b + 1..52 {
                let p = value[idx(a, b, c)];
                hands += 1;
                if !plays(p) {
                    total -= 1.0;
                    continue;
                }
                let bonus = ante_bonus(p.category()) as f64;
                let (mut sum, mut n) = (0i64, 0i64);
                for x in (0..52).filter(|x| ![a, b, c].contains(x)) {
                    for y in (x + 1..52).filter(|y| ![a, b, c].contains(y)) {
                        for z in (y + 1..52).filter(|z| ![a, b, c].contains(z)) {
                            let (ante, play) = showdown(p, value[idx(x, y, z)], 1);
                            sum += ante + play;
                            n += 1;
                        }
                    }
                }
                total += sum as f64 / n as f64 + bonus;
            }
        }
    }
    100.0 * (1.0 + total / hands as f64)
}
