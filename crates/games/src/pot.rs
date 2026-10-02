//! Pot utama dan side pot untuk game antar-pemain (SPEC §6.3, §7; M5b).
//!
//! Kontribusi tiap kursi dalam satu tangan dipecah menurut lapisan: setiap
//! lapisan berisi bagian semua kursi sampai batas kontribusi berikutnya,
//! dan hanya kursi yang belum fold dengan kontribusi sampai lapisan itu
//! yang berhak. Lapisan tanpa kursi yang berhak (chip kursi yang fold di
//! atas semua yang tersisa) digabung ke pot sebelumnya. Pot dengan kursi
//! berhak yang sama juga digabung.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Pot {
    pub amount: i64,
    /// Kursi yang berhak, terurut.
    pub eligible: Vec<u8>,
}

/// Pot dari kontribusi `contrib[s]` dan status `folded[s]` tiap kursi.
pub fn pots(contrib: &[i64], folded: &[bool]) -> Vec<Pot> {
    let mut levels: Vec<i64> = contrib.iter().copied().filter(|&c| c > 0).collect();
    levels.sort_unstable();
    levels.dedup();
    let mut out: Vec<Pot> = Vec::new();
    let mut below = 0;
    for level in levels {
        let amount: i64 = contrib.iter().map(|&c| c.min(level) - c.min(below)).sum();
        let eligible: Vec<u8> = (0..contrib.len())
            .filter(|&s| !folded[s] && contrib[s] >= level)
            .map(|s| s as u8)
            .collect();
        below = level;
        if amount == 0 {
            continue;
        }
        match out.last_mut() {
            Some(last) if eligible.is_empty() || last.eligible == eligible => last.amount += amount,
            _ if eligible.is_empty() => out.push(Pot {
                amount,
                eligible: Vec::new(),
            }),
            _ => out.push(Pot { amount, eligible }),
        }
    }
    out
}

/// Membagi semua pot ke tangan terbaik (`strength`, lebih besar = lebih
/// baik) di antara kursi yang berhak. Seri dibagi rata; sisa chip ganjil
/// satu per satu mulai dari kursi pertama setelah `dealer` searah jarum jam.
pub fn award<K: Ord>(pots: &[Pot], strength: impl Fn(u8) -> K, seats: u8, dealer: u8) -> Vec<i64> {
    let mut won = vec![0i64; seats as usize];
    for pot in pots {
        if pot.eligible.is_empty() {
            continue;
        }
        let best = pot
            .eligible
            .iter()
            .map(|&s| strength(s))
            .max()
            .expect("ada kursi");
        let mut winners: Vec<u8> = pot
            .eligible
            .iter()
            .copied()
            .filter(|&s| strength(s) == best)
            .collect();
        // Urutan searah jarum jam mulai dari kursi setelah dealer.
        winners.sort_by_key(|&s| (s + seats - dealer - 1) % seats);
        let n = winners.len() as i64;
        let share = pot.amount / n;
        let odd = pot.amount % n;
        for (i, s) in winners.iter().enumerate() {
            won[*s as usize] += share + i64::from((i as i64) < odd);
        }
    }
    won
}

/// Pot untuk game yang taruhannya tidak disamakan (Teen Patti): semua chip
/// masuk ke pot yang sama, dan lapisan hanya dibuat di kontribusi pemain
/// all-in yang belum pack, karena merekalah yang tidak bisa memenangkan
/// lebih dari lapisannya.
pub fn pots_all_in(contrib: &[i64], folded: &[bool], allin: &[bool]) -> Vec<Pot> {
    let mut caps: Vec<i64> = (0..contrib.len())
        .filter(|&s| allin[s] && !folded[s])
        .map(|s| contrib[s])
        .collect();
    caps.sort_unstable();
    caps.dedup();
    caps.push(i64::MAX);
    let mut out: Vec<Pot> = Vec::new();
    let mut below = 0;
    for cap in caps {
        let amount: i64 = contrib.iter().map(|&c| c.min(cap) - c.min(below)).sum();
        let eligible: Vec<u8> = (0..contrib.len())
            .filter(|&s| !folded[s] && (!allin[s] || contrib[s] >= cap))
            .map(|s| s as u8)
            .collect();
        below = cap;
        if amount == 0 {
            continue;
        }
        match out.last_mut() {
            Some(last) if eligible.is_empty() || last.eligible == eligible => last.amount += amount,
            _ => out.push(Pot { amount, eligible }),
        }
    }
    out
}
