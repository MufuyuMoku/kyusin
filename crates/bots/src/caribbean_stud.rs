//! Strategi sederhana Caribbean Stud, diadaptasi dari Wizard of Odds (D-060): raise
//! dengan pair atau lebih; fold di bawah A-K; dengan A-K raise bila
//! (1) kartu terbuka bandar 2–Q dan sama dengan salah satu kartu pemain,
//! (2) kartu terbuka bandar as atau king dan pemain punya Q atau J, atau
//! (3) kartu terbuka bandar tidak sama dengan kartu pemain mana pun dan
//! kartu tertinggi keempat pemain lebih tinggi darinya. Dipakai untuk
//! menghitung dan memverifikasi RTP (SPEC §7).

use kyusin_core::{Seed, TurnGame};
use kyusin_games::cards::Card;
use kyusin_games::caribbean_stud::{self, CaribbeanStud, View};
use kyusin_games::poker::{Category, eval5, rank_value};

use crate::meja::{Wager, simulate};

pub fn raises(player: &[Card], up: Card) -> bool {
    let v = eval5(player);
    if v.category() >= Category::Pair {
        return true;
    }
    if v.rank(0) != 14 || v.rank(1) != 13 {
        return false;
    }
    let u = rank_value(up.rank);
    let mine: Vec<u8> = player.iter().map(|c| rank_value(c.rank)).collect();
    let matches = mine.contains(&u);
    (u <= 12 && matches)
        || (u >= 13 && mine.iter().any(|&r| r == 12 || r == 11))
        || (!matches && v.rank(3) > u)
}

pub fn decide(v: &View, bet: i64) -> String {
    match v.meja.fase.as_str() {
        "keputusan" => {
            let cards: Vec<Card> = v.pemain.iter().filter_map(|c| c.parse().ok()).collect();
            let up: Card = v.bandar[0].parse().expect("kartu terbuka bandar");
            if raises(&cards, up) { "raise" } else { "fold" }.into()
        }
        _ => format!("bet {bet}"),
    }
}

pub fn wagers() -> Vec<Wager> {
    caribbean_stud::RTP
        .iter()
        .map(|&rtp| Wager {
            game: caribbean_stud::ID,
            rtp,
            run: |n, s: &Seed| {
                simulate(
                    n,
                    s,
                    100,
                    |seed| CaribbeanStud::new(Default::default(), seed).expect("dek"),
                    |_, v| decide(v, 100),
                )
            },
        })
        .collect()
}
