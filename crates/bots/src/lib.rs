//! Bot per game (SPEC §5.1, §5.3, §8). Bot adalah [`Player`] yang memilih
//! perintah teks sendiri; acaknya dari RNG turunan seed ronde, jadi
//! pertandingan bisa diulang persis dari seed yang sama.

use kyusin_core::rng::derive;
use kyusin_core::{Player, Seed};

pub mod catur;
pub mod reversi;

/// Jumlah level bot untuk sebuah game (0 = tidak ada bot).
pub fn levels(game: &str) -> u8 {
    match game {
        kyusin_games::reversi::ID => reversi::LEVELS,
        kyusin_games::catur::ID => catur::LEVELS,
        _ => 0,
    }
}

/// Perkiraan rating tiap level dari kalibrasi (SPEC §8), bila ada.
/// Indeks 0 = level 1.
pub fn ratings(game: &str) -> Vec<Option<i64>> {
    let data = match game {
        kyusin_games::catur::ID => include_str!("../../../data/calibration/catur.json"),
        _ => return vec![None; levels(game) as usize],
    };
    let doc: serde_json::Value = serde_json::from_str(data).unwrap_or_default();
    (1..=levels(game))
        .map(|level| {
            doc["levels"]
                .as_array()?
                .iter()
                .find(|l| l["level"] == level)?["elo"]
                .as_f64()
                .map(|e| e as i64)
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
