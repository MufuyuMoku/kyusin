//! Bot per game (SPEC §5.1, §5.3, §8). Bot adalah [`Player`] yang memilih
//! perintah teks sendiri; acaknya dari RNG turunan seed ronde, jadi
//! pertandingan bisa diulang persis dari seed yang sama.

use kyusin_core::rng::derive;
use kyusin_core::{Player, Seed};

pub mod reversi;

/// Jumlah level bot untuk sebuah game (0 = tidak ada bot).
pub fn levels(game: &str) -> u8 {
    match game {
        kyusin_games::reversi::ID => reversi::LEVELS,
        _ => 0,
    }
}

/// Membuat bot level `level` (mulai 1) untuk kursi `seat`, dengan RNG
/// turunan seed ronde.
pub fn create(game: &str, level: u8, round_seed: &Seed, seat: u8) -> Option<Box<dyn Player>> {
    let seed = derive(round_seed, &format!("bot:{seat}"));
    match game {
        kyusin_games::reversi::ID if (1..=reversi::LEVELS).contains(&level) => {
            Some(Box::new(reversi::ReversiBot::new(level, seed)))
        }
        _ => None,
    }
}
