//! Strategi sederhana KyuSin untuk Casino Hold'em (D-060). Call bila salah
//! satu berlaku, selain itu fold:
//! 1. pair atau lebih yang memakai paling sedikit satu kartu tangan (pair
//!    di tangan, atau kartu tangan berpasangan dengan flop), atau straight
//!    ke atas; pair yang hanya ada di flop tidak dihitung;
//! 2. empat kartu flush dengan paling sedikit satu kartu tangan;
//! 3. straight draw: empat dari lima peringkat berurutan di antara lima
//!    kartu (as boleh tinggi atau rendah);
//! 4. kartu tangan tertinggi J atau lebih;
//! 5. kedua kartu tangan 5 atau lebih dan yang tertinggi 9 atau lebih.
//!
//! Dipilih dengan membandingkan beberapa aturan sederhana pada simulasi
//! dengan bilangan acak yang sama; strategi optimal (menghitung setiap
//! keputusan) memberi RTP sekitar 0,7 poin lebih tinggi. RTP manifest
//! berlaku untuk strategi ini (SPEC §7).

use kyusin_core::{Seed, TurnGame};
use kyusin_games::cards::Card;
use kyusin_games::casino_holdem::{self, CasinoHoldem, View};
use kyusin_games::poker::{Category, eval_best, rank_value};

use crate::meja::{Wager, simulate};

pub fn calls(hole: &[Card], flop: &[Card]) -> bool {
    let h: Vec<u8> = hole.iter().map(|c| rank_value(c.rank)).collect();
    let f: Vec<u8> = flop.iter().map(|c| rank_value(c.rank)).collect();
    let all: Vec<Card> = hole.iter().chain(flop.iter()).copied().collect();
    // 1. Pair yang memakai kartu tangan, atau straight ke atas.
    if h[0] == h[1]
        || h.iter().any(|r| f.contains(r))
        || eval_best(&all).category() >= Category::Straight
    {
        return true;
    }
    // 2. Empat kartu flush dengan kartu tangan.
    if hole
        .iter()
        .any(|c| all.iter().filter(|x| x.suit == c.suit).count() >= 4)
    {
        return true;
    }
    // 3. Straight draw: empat dari lima peringkat berurutan.
    let mut ranks: Vec<u8> = h.iter().chain(f.iter()).copied().collect();
    if ranks.contains(&14) {
        ranks.push(1);
    }
    if (1..=10u8).any(|lo| (lo..lo + 5).filter(|r| ranks.contains(r)).count() >= 4) {
        return true;
    }
    // 4–5. Kartu tinggi.
    let (hi, lo) = (h[0].max(h[1]), h[0].min(h[1]));
    hi >= 11 || (hi >= 9 && lo >= 5)
}

pub fn decide(v: &View, bet: i64) -> String {
    match v.meja.fase.as_str() {
        "keputusan" => {
            let hole: Vec<Card> = v.pemain.iter().filter_map(|c| c.parse().ok()).collect();
            let flop: Vec<Card> = v.meja_kartu.iter().filter_map(|c| c.parse().ok()).collect();
            if calls(&hole, &flop) { "call" } else { "fold" }.into()
        }
        _ => format!("bet {bet}"),
    }
}

pub fn wagers() -> Vec<Wager> {
    casino_holdem::RTP
        .iter()
        .map(|&rtp| Wager {
            game: casino_holdem::ID,
            rtp,
            run: |n, s: &Seed| {
                simulate(
                    n,
                    s,
                    100,
                    |seed| CasinoHoldem::new(Default::default(), seed).expect("dek"),
                    |_, v| decide(v, 100),
                )
            },
        })
        .collect()
}
