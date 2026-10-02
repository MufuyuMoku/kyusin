//! Omaha Pot-Limit (SPEC §6.3; M5b-1): empat kartu tangan; tangan terbaik
//! memakai tepat dua kartu tangan dan tepat tiga kartu meja. Mesinnya di
//! [`crate::poker_meja`].

use kyusin_core::game::create_session;
use kyusin_core::{Cartridge, RegistryError};

use crate::cards::Card;
use crate::poker::{Value, eval5};
use crate::poker_meja::{PokerTable, Variant};

pub const ID: &str = "omaha";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct OmahaRules;

impl Variant for OmahaRules {
    const ID: &'static str = ID;
    const HOLE: usize = 4;
    const POT_LIMIT: bool = true;

    fn best(hole: &[Card], board: &[Card]) -> Value {
        let mut best = Value(0);
        for a in 0..hole.len() {
            for b in a + 1..hole.len() {
                for x in 0..board.len() {
                    for y in x + 1..board.len() {
                        for z in y + 1..board.len() {
                            let v = eval5(&[hole[a], hole[b], board[x], board[y], board[z]]);
                            best = best.max(v);
                        }
                    }
                }
            }
        }
        best
    }
}

pub type Omaha = PokerTable<OmahaRules>;

pub fn cartridge() -> Result<Cartridge, RegistryError> {
    Cartridge::new(
        include_str!("manifest.toml"),
        include_str!("../../../../tutorials/omaha.toml"),
        create_session::<Omaha>,
    )
}
