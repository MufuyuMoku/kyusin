//! Texas Hold'em No-Limit (SPEC §6.3; M5b-1): dua kartu tangan, tangan
//! terbaik lima dari tujuh kartu. Mesinnya di [`crate::poker_meja`].

use kyusin_core::game::create_session;
use kyusin_core::{Cartridge, RegistryError};

use crate::cards::Card;
use crate::poker::{Value, eval_best};
use crate::poker_meja::{PokerTable, Variant};

pub const ID: &str = "texas-holdem";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Texas;

impl Variant for Texas {
    const ID: &'static str = ID;
    const HOLE: usize = 2;
    const POT_LIMIT: bool = false;

    fn best(hole: &[Card], board: &[Card]) -> Value {
        let all: Vec<Card> = hole.iter().chain(board.iter()).copied().collect();
        eval_best(&all)
    }
}

pub type TexasHoldem = PokerTable<Texas>;

pub fn cartridge() -> Result<Cartridge, RegistryError> {
    Cartridge::new(
        include_str!("manifest.toml"),
        include_str!("../../../../tutorials/texas-holdem.toml"),
        create_session::<TexasHoldem>,
    )
}
