//! Mesin domino bersama (SPEC §6.3; M5b-2): satu set double-six berisi 28
//! kartu, dipakai Domino QiuQiu sekarang dan Gaple nanti. Kartu ditulis
//! `besar-kecil` (misalnya `6-4`, `0-0`); urutan masukan bebas.

use std::fmt;
use std::str::FromStr;

use kyusin_core::GameRng;
use serde::{Deserialize, Serialize};

/// Angka terbesar di satu sisi kartu.
pub const MAX_PIPS: u8 = 6;

/// Satu kartu domino; `hi >= lo`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct Tile {
    pub hi: u8,
    pub lo: u8,
}

impl Tile {
    /// Kartu dari dua sisi dalam urutan apa pun.
    pub fn new(a: u8, b: u8) -> Tile {
        debug_assert!(a <= MAX_PIPS && b <= MAX_PIPS);
        Tile {
            hi: a.max(b),
            lo: a.min(b),
        }
    }

    /// Jumlah bulatan kedua sisi.
    pub fn pips(self) -> u8 {
        self.hi + self.lo
    }

    /// Kartu balak (kedua sisi sama).
    pub fn is_double(self) -> bool {
        self.hi == self.lo
    }

    /// Satu set lengkap, berurutan dari 0-0 sampai 6-6.
    pub fn set() -> Vec<Tile> {
        (0..=MAX_PIPS)
            .flat_map(|hi| (0..=hi).map(move |lo| Tile { hi, lo }))
            .collect()
    }
}

impl fmt::Display for Tile {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}-{}", self.hi, self.lo)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParseTileError(pub String);

impl FromStr for Tile {
    type Err = ParseTileError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let err = || ParseTileError(s.into());
        let (a, b) = s.split_once('-').ok_or_else(err)?;
        let side = |x: &str| {
            x.parse::<u8>()
                .ok()
                .filter(|&n| n <= MAX_PIPS && x.len() == 1)
                .ok_or_else(err)
        };
        Ok(Tile::new(side(a)?, side(b)?))
    }
}

impl Serialize for Tile {
    fn serialize<S: serde::Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
        s.collect_str(self)
    }
}

impl<'de> Deserialize<'de> for Tile {
    fn deserialize<D: serde::Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        let text = String::deserialize(d)?;
        text.parse()
            .map_err(|_| serde::de::Error::custom(format!("kartu domino `{text}`")))
    }
}

/// Tumpukan kartu domino tertutup yang dibagikan dari atas.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Boneyard {
    tiles: Vec<Tile>,
    next: usize,
}

impl Boneyard {
    /// Satu set yang dikocok Fisher–Yates dengan `rng`.
    pub fn shuffled(rng: &mut GameRng) -> Self {
        let mut tiles = Tile::set();
        rng.shuffle(&mut tiles);
        Boneyard { tiles, next: 0 }
    }

    /// Urutan tetap (kartu pertama dibagi lebih dulu); untuk tes dan tutorial.
    pub fn from_tiles(tiles: Vec<Tile>) -> Self {
        Boneyard { tiles, next: 0 }
    }

    pub fn draw(&mut self) -> Option<Tile> {
        let tile = self.tiles.get(self.next).copied();
        if tile.is_some() {
            self.next += 1;
        }
        tile
    }

    /// Kartu yang belum dibagi.
    pub fn len(&self) -> usize {
        self.tiles.len() - self.next
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Seluruh urutan kartu, termasuk yang sudah dibagi.
    pub fn tiles(&self) -> &[Tile] {
        &self.tiles
    }
}
