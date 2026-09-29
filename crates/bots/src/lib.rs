//! Bot per game (SPEC §5.1, §5.3, §8). Bot adalah [`Player`] yang memilih
//! perintah teks sendiri; acaknya dari RNG turunan seed ronde, jadi
//! pertandingan bisa diulang persis dari seed yang sama.

use kyusin_core::rng::derive;
use kyusin_core::{Player, Seed};

pub mod andar_bahar;
pub mod baccarat;
pub mod blackjack;
pub mod calibration;
pub mod casino_war;
pub mod catur;
mod catur_search;
pub mod dragon_tiger;
pub mod meja;
pub mod reversi;

/// Jumlah level bot untuk sebuah game (0 = tidak ada bot).
pub fn levels(game: &str) -> u8 {
    match game {
        kyusin_games::reversi::ID => reversi::LEVELS,
        kyusin_games::catur::ID => catur::LEVELS,
        _ => 0,
    }
}

/// Rating lokal tiap level untuk ditampilkan (SPEC §8), bila ada.
/// Indeks 0 = level 1. Taksiran ekstrapolasi tidak ditampilkan (D-047).
pub fn ratings(game: &str) -> Vec<Option<i64>> {
    (1..=levels(game))
        .map(|l| {
            calibration::level(game, l)
                .filter(|r| !r.extrapolated)
                .map(|r| r.elo.round() as i64)
        })
        .collect()
}

/// Membuat bot level `level` (mulai 1) untuk kursi `seat`, dengan RNG
/// turunan seed ronde.
pub fn create(game: &str, level: u8, round_seed: &Seed, seat: u8) -> Option<Box<dyn Player>> {
    let seed = derive(round_seed, &format!("bot:{seat}"));
    match game {
        kyusin_games::reversi::ID if (1..=reversi::LEVELS).contains(&level) => {
            Some(Box::new(reversi::ReversiBot::new(level, seed)))
        }
        kyusin_games::catur::ID if (1..=catur::LEVELS).contains(&level) => {
            Some(Box::new(catur::ChessBot::new(level, seed)))
        }
        _ => None,
    }
}
