//! RNG yang disuntikkan ke game: ChaCha20 dengan seed eksplisit (SPEC §3,
//! §5.2). Keadaannya (seed + posisi kata) bisa diserialisasi, jadi keadaan
//! game yang memuat RNG tetap bisa di-hash dan diputar ulang persis.

use rand_chacha::ChaCha20Rng;
use rand_core::{Rng, SeedableRng};
use serde::{Deserialize, Deserializer, Serialize, Serializer};

use crate::game::Seed;
use crate::hash::{hex, sha256};

#[derive(Clone, Debug)]
pub struct GameRng {
    inner: ChaCha20Rng,
}

impl GameRng {
    pub fn from_seed(seed: Seed) -> Self {
        GameRng {
            inner: ChaCha20Rng::from_seed(seed),
        }
    }

    pub fn next_u32(&mut self) -> u32 {
        self.inner.next_u32()
    }

    pub fn next_u64(&mut self) -> u64 {
        self.inner.next_u64()
    }

    /// Bilangan acak seragam di `0..n` (tanpa bias modulo). `n` harus > 0.
    pub fn below(&mut self, n: u32) -> u32 {
        assert!(n > 0, "below(0)");
        let zone = u32::MAX - (u32::MAX % n);
        loop {
            let v = self.next_u32();
            if v < zone {
                return v % n;
            }
        }
    }

    /// Fisher–Yates.
    pub fn shuffle<T>(&mut self, items: &mut [T]) {
        for i in (1..items.len()).rev() {
            let j = self.below(i as u32 + 1) as usize;
            items.swap(i, j);
        }
    }
}

impl PartialEq for GameRng {
    fn eq(&self, other: &Self) -> bool {
        self.inner.get_seed() == other.inner.get_seed()
            && self.inner.get_word_pos() == other.inner.get_word_pos()
    }
}

impl Eq for GameRng {}

#[derive(Serialize, Deserialize)]
struct RngState {
    seed: String,
    word_pos: String,
}

impl Serialize for GameRng {
    fn serialize<S: Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
        RngState {
            seed: hex(&self.inner.get_seed()),
            word_pos: self.inner.get_word_pos().to_string(),
        }
        .serialize(s)
    }
}

impl<'de> Deserialize<'de> for GameRng {
    fn deserialize<D: Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        use serde::de::Error;
        let st = RngState::deserialize(d)?;
        let seed = crate::hash::unhex32(&st.seed).map_err(D::Error::custom)?;
        let pos: u128 = st.word_pos.parse().map_err(D::Error::custom)?;
        let mut inner = ChaCha20Rng::from_seed(seed);
        inner.set_word_pos(pos);
        Ok(GameRng { inner })
    }
}

/// Seed turunan untuk keperluan terpisah (misalnya RNG bot per kursi), supaya
/// satu seed ronde tidak dipakai ulang untuk dua aliran acak.
pub fn derive(seed: &Seed, label: &str) -> Seed {
    let mut bytes = seed.to_vec();
    bytes.extend_from_slice(label.as_bytes());
    sha256(&bytes)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn same_seed_same_stream() {
        let mut a = GameRng::from_seed([7; 32]);
        let mut b = GameRng::from_seed([7; 32]);
        let xs: Vec<u64> = (0..10).map(|_| a.next_u64()).collect();
        let ys: Vec<u64> = (0..10).map(|_| b.next_u64()).collect();
        assert_eq!(xs, ys);
        assert_ne!(GameRng::from_seed([8; 32]).next_u64(), xs[0]);
    }

    #[test]
    fn serde_round_trip_keeps_position() {
        let mut a = GameRng::from_seed([1; 32]);
        a.next_u64();
        a.next_u32();
        let json = serde_json::to_string(&a).unwrap();
        let mut b: GameRng = serde_json::from_str(&json).unwrap();
        assert_eq!(a, b);
        assert_eq!(a.next_u64(), b.next_u64());
    }

    #[test]
    fn below_is_in_range_and_covers_all() {
        let mut r = GameRng::from_seed([3; 32]);
        let mut seen = [false; 6];
        for _ in 0..600 {
            let v = r.below(6) as usize;
            seen[v] = true;
        }
        assert!(seen.iter().all(|s| *s));
    }

    #[test]
    fn shuffle_is_a_permutation() {
        let mut r = GameRng::from_seed([4; 32]);
        let mut v: Vec<u32> = (0..52).collect();
        r.shuffle(&mut v);
        let mut sorted = v.clone();
        sorted.sort();
        assert_eq!(sorted, (0..52).collect::<Vec<_>>());
        assert_ne!(v, sorted);
    }

    #[test]
    fn derive_differs_by_label() {
        assert_ne!(derive(&[0; 32], "bot:0"), derive(&[0; 32], "bot:1"));
    }
}
