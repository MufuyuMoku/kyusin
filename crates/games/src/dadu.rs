//! Dadu bersama untuk Sic Bo, Chuck-a-luck, dan Craps (SPEC §6.4; M6a).

use kyusin_core::GameRng;

/// Satu dadu enam sisi.
pub fn roll(rng: &mut GameRng) -> u8 {
    rng.below(6) as u8 + 1
}

/// Tiga dadu.
pub fn roll3(rng: &mut GameRng) -> [u8; 3] {
    [roll(rng), roll(rng), roll(rng)]
}

/// Ke-216 hasil tiga dadu, masing-masing berpeluang 1/216.
pub fn all3() -> Vec<([u8; 3], f64)> {
    let mut v = Vec::with_capacity(216);
    for a in 1..=6 {
        for b in 1..=6 {
            for c in 1..=6 {
                v.push(([a, b, c], 1.0 / 216.0));
            }
        }
    }
    v
}

pub fn valid3(d: &[u8; 3]) -> bool {
    d.iter().all(|x| (1..=6).contains(x))
}

/// Banyak dadu yang menunjukkan `n`.
pub fn count(d: &[u8; 3], n: u8) -> i64 {
    d.iter().filter(|x| **x == n).count() as i64
}

pub fn triple(d: &[u8; 3]) -> bool {
    d[0] == d[1] && d[1] == d[2]
}

pub fn total(d: &[u8; 3]) -> u8 {
    d.iter().sum()
}
