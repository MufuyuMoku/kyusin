//! Tes aturan Reversi, ditulis sebelum mesin aturannya (SPEC §11).
//!
//! Aturan: papan 8×8, hitam (X, kursi 0) jalan duluan dari posisi awal
//! standar (d4, e5 putih; d5, e4 hitam). Langkah sah harus mengapit minimal
//! satu garis lawan; semua garis yang terapit dibalik. Pemain yang tidak
//! punya langkah sah wajib `pass`, dan `pass` hanya boleh saat itu.
//! Permainan selesai bila kedua pemain tidak bisa melangkah; pemenang =
//! bidak terbanyak, sama banyak = seri.

use kyusin_core::{GameError, Lang, Session, TurnGame};
use kyusin_games::reversi::{Config, Move, Reversi, Square};

fn start() -> Reversi {
    Reversi::new(Config::default(), [0; 32]).unwrap()
}

fn from(rows: [&str; 8], black_to_move: bool) -> Reversi {
    Reversi::new(
        Config {
            posisi: Some(rows.iter().map(|r| r.to_string()).collect()),
            giliran: Some(if black_to_move { "hitam" } else { "putih" }.into()),
        },
        [0; 32],
    )
    .unwrap()
}

fn legal(g: &Reversi, seat: u8) -> Vec<String> {
    let mut v: Vec<String> = TurnGame::legal_actions(g, seat)
        .iter()
        .map(|a| a.usage())
        .collect();
    v.sort();
    v
}

fn view(g: &Reversi) -> serde_json::Value {
    Session::view_data(g, 0)
}

#[test]
fn starting_position() {
    let g = start();
    assert_eq!(TurnGame::seats(&g), 2);
    assert_eq!(TurnGame::pending_players(&g), vec![0]);
    assert_eq!(legal(&g, 0), vec!["c4", "d3", "e6", "f5"]);
    assert!(legal(&g, 1).is_empty());
    let v = view(&g);
    assert_eq!(v["hitam"], 2);
    assert_eq!(v["putih"], 2);
    assert_eq!(v["papan"][3], "...OX...");
    assert_eq!(v["papan"][4], "...XO...");
}

#[test]
fn placing_flips_the_flanked_disc() {
    let mut g = start();
    Session::act(&mut g, 0, "d3").unwrap();
    let v = view(&g);
    assert_eq!(v["hitam"], 4);
    assert_eq!(v["putih"], 1);
    assert_eq!(v["papan"][2], "...X....");
    assert_eq!(v["papan"][3], "...XX...");
    assert_eq!(v["terakhir"], "d3");
    assert_eq!(v["dibalik"], serde_json::json!(["d4"]));
    assert_eq!(TurnGame::pending_players(&g), vec![1]);
    assert_eq!(legal(&g, 1), vec!["c3", "c5", "e3"]);
}

#[test]
fn illegal_moves_and_turn_order_are_rejected() {
    let mut g = start();
    assert_eq!(
        Session::act(&mut g, 0, "a1"),
        Err(GameError::Illegal("a1".into()))
    );
    assert_eq!(
        Session::act(&mut g, 0, "pass"),
        Err(GameError::Illegal("pass".into()))
    );
    assert_eq!(Session::act(&mut g, 1, "c3"), Err(GameError::NotPending(1)));
    assert_eq!(
        Session::act(&mut g, 0, "D3"),
        Err(GameError::Illegal("D3".into()))
    );
}

#[test]
fn flips_in_several_directions_at_once() {
    // Hitam di d4 mengapit ke timur (e4), selatan (d5), dan tenggara (e5).
    let mut g = from(
        [
            "........", "........", "........", "....OX..", "...OO...", "...XO...", "....X...",
            "........",
        ],
        true,
    );
    // Hitam main d4: timur e4 O lalu f4 X → apit; selatan d5 O lalu d6 X
    // → apit; tenggara e5 O lalu f6 kosong → tidak.
    Session::act(&mut g, 0, "d4").unwrap();
    let v = view(&g);
    let mut flipped: Vec<String> = serde_json::from_value(v["dibalik"].clone()).unwrap();
    flipped.sort();
    assert_eq!(flipped, vec!["d5", "e4"]);
    assert_eq!(v["papan"][3], "...XXX..");
    assert_eq!(v["papan"][4], "...XO...");
}

#[test]
fn pass_only_when_no_legal_move() {
    // Putih tidak punya langkah; hitam punya.
    let mut g = from(
        [
            "XXXXXXXX", "XXXXXXXX", "XXXXXXXX", "XXXXXXXX", "XXXXXXXO", "XXXXXXX.", "XXXXXXX.",
            "XXXXXX..",
        ],
        false,
    );
    assert_eq!(legal(&g, 1), vec!["pass"]);
    Session::act(&mut g, 1, "pass").unwrap();
    assert_eq!(TurnGame::pending_players(&g), vec![0]);
    assert!(!legal(&g, 0).contains(&"pass".to_string()));
}

#[test]
fn game_ends_when_neither_can_move() {
    let mut g = from(
        [
            "XXXXXXXX", "XXXXXXXX", "XXXXXXXX", "XXXXXXXX", "XXXXXXXX", "XXXXXXXX", "XXXXXXXO",
            "XXXXXXX.",
        ],
        true,
    );
    // Hitam h8 mengapit h7 ke utara.
    Session::act(&mut g, 0, "h8").unwrap();
    assert!(TurnGame::is_over(&g));
    assert!(TurnGame::pending_players(&g).is_empty());
    let r = TurnGame::result(&g).unwrap();
    assert_eq!(r.winners, vec![0]);
    assert_eq!(r.scores, vec![64, 0]);
    assert_eq!(Session::act(&mut g, 1, "pass"), Err(GameError::Over));
}

#[test]
fn a_position_with_no_moves_at_all_is_over_immediately_and_can_be_a_draw() {
    let g = from(
        [
            "XXXXXXXX", "XXXXXXXX", "XXXXXXXX", "XXXXXXXX", "OOOOOOOO", "OOOOOOOO", "OOOOOOOO",
            "OOOOOOOO",
        ],
        true,
    );
    assert!(TurnGame::is_over(&g));
    let r = TurnGame::result(&g).unwrap();
    assert!(r.winners.is_empty(), "seri");
    assert_eq!(r.scores, vec![32, 32]);
}

#[test]
fn invalid_custom_position_is_rejected() {
    let bad = Config {
        posisi: Some(vec!["XXXX".into(); 8]),
        giliran: None,
    };
    assert!(matches!(
        Reversi::new(bad, [0; 32]),
        Err(GameError::Config(_))
    ));
    let bad_char = Config {
        posisi: Some(vec!["XXXXXXXZ".into(); 8]),
        giliran: None,
    };
    assert!(Reversi::new(bad_char, [0; 32]).is_err());
    let bad_turn = Config {
        posisi: None,
        giliran: Some("hijau".into()),
    };
    assert!(Reversi::new(bad_turn, [0; 32]).is_err());
}

/// Jumlah urutan langkah (termasuk pass wajib) sampai kedalaman tertentu,
/// angka baku Othello.
fn perft(g: &Reversi, depth: u32) -> u64 {
    if depth == 0 || TurnGame::is_over(g) {
        return 1;
    }
    let seat = TurnGame::pending_players(g)[0];
    TurnGame::legal_actions(g, seat)
        .iter()
        .map(|a| {
            let mut next = g.clone();
            Session::act(&mut next, seat, &a.usage()).unwrap();
            perft(&next, depth - 1)
        })
        .sum()
}

#[test]
fn perft_matches_known_counts() {
    let g = start();
    let expected = [4, 12, 56, 244, 1396, 8200];
    for (d, want) in expected.iter().enumerate() {
        assert_eq!(perft(&g, d as u32 + 1), *want, "kedalaman {}", d + 1);
    }
}

#[test]
fn command_round_trip_for_every_square_and_pass() {
    let g = start();
    for i in 0..64u8 {
        let sq = Square::new(i);
        let text = g.format_action(&Move::Place(sq));
        assert_eq!(g.parse_command(&text).unwrap(), Move::Place(sq));
        assert_eq!(text.len(), 2);
    }
    assert_eq!(g.format_action(&Move::Pass), "pass");
    assert_eq!(g.parse_command("pass").unwrap(), Move::Pass);
    assert_eq!(
        g.format_action(&Move::Place(Square::parse("a1").unwrap())),
        "a1"
    );
    assert_eq!(
        g.format_action(&Move::Place(Square::parse("h8").unwrap())),
        "h8"
    );
    for bad in ["", "i1", "a9", "a0", "aa", "d3 d4", "PASS"] {
        assert!(g.parse_command(bad).is_err(), "{bad}");
    }
}

#[test]
fn random_playouts_conserve_discs_and_terminate() {
    let mut rng = kyusin_core::GameRng::from_seed([9; 32]);
    for _ in 0..200 {
        let mut g = start();
        let mut placed = 0;
        let mut plies = 0;
        while !TurnGame::is_over(&g) {
            let seat = TurnGame::pending_players(&g)[0];
            let moves = TurnGame::legal_actions(&g, seat);
            let pick = moves[rng.below(moves.len() as u32) as usize].usage();
            if pick != "pass" {
                placed += 1;
            }
            Session::act(&mut g, seat, &pick).unwrap();
            plies += 1;
            assert!(plies <= 130);
            let v = view(&g);
            let total = v["hitam"].as_u64().unwrap() + v["putih"].as_u64().unwrap();
            assert_eq!(total, 4 + placed);
        }
        let r = TurnGame::result(&g).unwrap();
        assert_eq!((r.scores[0] + r.scores[1]) as u64, 4 + placed);
    }
}

#[test]
fn text_view_in_both_languages() {
    let g = start();
    let id = Session::view_text(&g, 0, Lang::Id);
    let en = Session::view_text(&g, 0, Lang::En);
    for t in [&id, &en] {
        assert!(t.contains("a   b   c   d   e   f   g   h"), "{t}");
        assert!(t.contains('┌') && t.contains('┘'));
        assert!(t.contains('●') && t.contains('○'));
    }
    assert!(id.contains("Hitam 2") && id.contains("Putih 2"), "{id}");
    assert!(en.contains("Black 2") && en.contains("White 2"), "{en}");
}

#[test]
fn same_moves_same_state_hash() {
    let mut a = start();
    let mut b = start();
    for (seat, m) in [(0, "d3"), (1, "c5"), (0, "f6")] {
        Session::act(&mut a, seat, m).unwrap();
        Session::act(&mut b, seat, m).unwrap();
    }
    assert_eq!(Session::state_hash(&a), Session::state_hash(&b));
    let next = TurnGame::legal_actions(&a, 1)[0].usage();
    Session::act(&mut a, 1, &next).unwrap();
    assert_ne!(Session::state_hash(&a), Session::state_hash(&b));
}
