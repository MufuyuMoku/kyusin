//! Bot catur: selalu sah, menemukan mat satu langkah, tidak menyerahkan
//! menteri, dan level lebih tinggi lebih kuat (SPEC §7.2, §8).

use cozy_chess::Board;
use kyusin_bots::catur::ChessBot;
use kyusin_core::fair::{FairRound, commitment};
use kyusin_core::{Match, Player};

fn record(seed: u8) -> kyusin_core::fair::FairRecord {
    let mut r = FairRound::new("host", [seed; 32], vec!["pemain".into()]);
    r.commit("pemain", commitment(&[seed ^ 0x33; 32])).unwrap();
    r.close_commits().unwrap();
    r.reveal("pemain", [seed ^ 0x33; 32]).unwrap();
    r.close_reveals().unwrap();
    r.record().unwrap()
}

/// Satu partai bot lawan bot dari posisi awal; poin putih ×2 (2/1/0).
fn play(white: u8, black: u8, seed: u8) -> i64 {
    let c = kyusin_games::catur::cartridge().unwrap();
    let fair = record(seed);
    let round = fair.round_seed_bytes().unwrap();
    let players: Vec<Box<dyn Player>> = vec![
        kyusin_bots::create("catur", white, &round, 0).unwrap(),
        kyusin_bots::create("catur", black, &round, 1).unwrap(),
    ];
    let mut m = Match::new(&c, serde_json::Value::Null, fair, players).unwrap();
    let mut plies = 0;
    while !m.session().is_over() && plies < 300 {
        m.step_auto()
            .expect("bot harus memilih langkah sah")
            .unwrap();
        plies += 1;
    }
    if !m.session().is_over() {
        return 1; // dihentikan: dihitung remis
    }
    assert!(m.replay().verify(&c).ok);
    m.session().result().unwrap().scores[0]
}

/// Poin (×2) `strong` melawan `weak`, bergantian warna.
fn points(strong: u8, weak: u8, games: u8) -> i64 {
    (0..games)
        .map(|i| {
            if i % 2 == 0 {
                play(strong, weak, i)
            } else {
                2 - play(weak, strong, i)
            }
        })
        .sum()
}

#[test]
fn finds_mate_in_one_at_every_level_above_one() {
    // Putih: Qh5xf7# (mat skolastik).
    let board: Board = "r1bqkb1r/pppp1ppp/2n2n2/4p2Q/2B1P3/8/PPPP1PPP/RNB1K1NR w KQkq - 4 4"
        .parse()
        .unwrap();
    for level in 2..=4 {
        let mv = ChessBot::new(level, [1; 32]).choose(&board).unwrap();
        assert_eq!(mv.to.to_string(), "f7", "level {level}");
    }
}

#[test]
fn does_not_hang_the_queen() {
    // Menteri putih di d4 diserang pion e5; level 3 harus menyelamatkannya.
    let board: Board = "rnbqkbnr/pppp1ppp/8/4p3/3Q4/8/PPP1PPPP/RNB1KBNR w KQkq - 0 3"
        .parse()
        .unwrap();
    let mv = ChessBot::new(3, [2; 32]).choose(&board).unwrap();
    assert_eq!(mv.from.to_string(), "d4");
}

#[test]
fn all_levels_play_legal_games() {
    for (w, b) in [(1, 4), (4, 1), (2, 3)] {
        play(w, b, w * 10 + b);
    }
}

#[test]
fn same_seed_same_game() {
    assert_eq!(play(2, 3, 9), play(2, 3, 9));
}

#[test]
fn stronger_levels_score_more() {
    let p21 = points(2, 1, 8);
    eprintln!("level 2 vs 1: {}/{}", p21 as f64 / 2.0, 8);
    assert!(p21 >= 12, "level 2 hanya {} dari 8 poin", p21 as f64 / 2.0);
    let p32 = points(3, 2, 8);
    eprintln!("level 3 vs 2: {}/{}", p32 as f64 / 2.0, 8);
    assert!(p32 >= 10, "level 3 hanya {} dari 8 poin", p32 as f64 / 2.0);
    let p43 = points(4, 3, 6);
    eprintln!("level 4 vs 3: {}/{}", p43 as f64 / 2.0, 6);
    assert!(p43 >= 7, "level 4 hanya {} dari 6 poin", p43 as f64 / 2.0);
}
