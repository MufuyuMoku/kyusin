//! Bot Reversi: selalu sah, dan level yang lebih tinggi lebih kuat
//! (SPEC §7.2). Setiap game kompetitif wajib punya minimal 3 level.

use kyusin_core::fair::{FairRound, commitment};
use kyusin_core::{Match, Player};

fn record(seed: u8) -> kyusin_core::fair::FairRecord {
    let mut r = FairRound::new("host", [seed; 32], vec!["pemain".into()]);
    r.commit("pemain", commitment(&[seed ^ 0x5a; 32])).unwrap();
    r.close_commits().unwrap();
    r.reveal("pemain", [seed ^ 0x5a; 32]).unwrap();
    r.close_reveals().unwrap();
    r.record().unwrap()
}

/// Memainkan satu pertandingan bot lawan bot; mengembalikan skor [hitam, putih].
fn play(black: u8, white: u8, seed: u8) -> Vec<i64> {
    let c = kyusin_games::reversi::cartridge().unwrap();
    let fair = record(seed);
    let round = fair.round_seed_bytes().unwrap();
    let players: Vec<Box<dyn Player>> = vec![
        kyusin_bots::create("reversi", black, &round, 0).unwrap(),
        kyusin_bots::create("reversi", white, &round, 1).unwrap(),
    ];
    let mut m = Match::new(&c, serde_json::Value::Null, fair, players).unwrap();
    let mut plies = 0;
    while !m.session().is_over() {
        m.step_auto()
            .expect("bot harus selalu memilih langkah sah")
            .unwrap();
        plies += 1;
        assert!(plies < 130);
    }
    // Pertandingan bot juga harus lolos verify.
    assert!(m.replay().verify(&c).ok);
    m.session().result().unwrap().scores
}

/// Berapa kali `strong` menang melawan `weak` dari `games` pertandingan,
/// bergantian warna.
fn wins(strong: u8, weak: u8, games: u8) -> u32 {
    (0..games)
        .map(|i| {
            let s = if i % 2 == 0 {
                let r = play(strong, weak, i);
                r[0] > r[1]
            } else {
                let r = play(weak, strong, i);
                r[1] > r[0]
            };
            s as u32
        })
        .sum()
}

#[test]
fn every_competitive_game_has_three_levels() {
    let registry = kyusin_games::builtin().unwrap();
    for m in registry.manifests().filter(|m| m.competitive) {
        assert!(
            kyusin_bots::levels(&m.id) >= 3,
            "{} butuh 3 level bot",
            m.id
        );
        for level in 1..=kyusin_bots::levels(&m.id) {
            assert!(kyusin_bots::create(&m.id, level, &[0; 32], 0).is_some());
        }
    }
    assert!(kyusin_bots::create("reversi", 0, &[0; 32], 0).is_none());
    assert!(kyusin_bots::create("reversi", 5, &[0; 32], 0).is_none());
}

#[test]
fn all_levels_play_legal_games_with_both_colours() {
    for a in 1..=4 {
        for b in 1..=4 {
            let s = play(a, b, a * 10 + b);
            assert_eq!(s.len(), 2);
        }
    }
}

#[test]
fn same_seed_same_game() {
    assert_eq!(play(1, 2, 7), play(1, 2, 7));
}

#[test]
fn level_two_beats_level_one() {
    let w = wins(2, 1, 20);
    eprintln!("level 2 vs 1: {w}/20");
    assert!(w >= 14, "level 2 hanya menang {w}/20 lawan level 1");
}

#[test]
fn level_three_beats_level_one() {
    let w = wins(3, 1, 10);
    eprintln!("level 3 vs 1: {w}/10");
    assert!(w >= 9, "level 3 hanya menang {w}/10 lawan level 1");
}

#[test]
fn level_three_beats_level_two() {
    let w = wins(3, 2, 10);
    eprintln!("level 3 vs 2: {w}/10");
    assert!(w >= 7, "level 3 hanya menang {w}/10 lawan level 2");
}

#[test]
fn level_four_beats_level_three() {
    let w = wins(4, 3, 10);
    eprintln!("level 4 vs 3: {w}/10");
    assert!(w >= 7, "level 4 hanya menang {w}/10 lawan level 3");
}
