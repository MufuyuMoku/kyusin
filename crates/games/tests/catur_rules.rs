//! Tes aturan catur, ditulis sebelum mesin aturannya (SPEC §11).
//!
//! Aturan FIDE lewat generator langkah cozy-chess (MIT). Notasi kanonik =
//! SAN (`e4`, `Nbd2`, `exd6`, `e8=Q`, `O-O`, dengan `+`/`#`); notasi
//! koordinat (`e2e4`, `e7e8q`, `e1g1`) dan `0-0` diterima sebagai alias.
//! Remis otomatis: pat, ulangan tiga kali, aturan 50 langkah, bahan tidak
//! cukup. `resign` = menyerah; `timeout` = waktu habis (hanya bila ada jam,
//! diajukan host). Putih = kursi 0.

use kyusin_core::{GameError, Lang, Session, TurnGame};
use kyusin_games::catur::{Catur, Config, Jam, pgn};

fn start() -> Catur {
    Catur::new(Config::default(), [0; 32]).unwrap()
}

fn fen(f: &str) -> Catur {
    Catur::new(
        Config {
            fen: Some(f.into()),
            jam: None,
        },
        [0; 32],
    )
    .unwrap()
}

fn legal(g: &Catur) -> Vec<String> {
    let seat = TurnGame::pending_players(g)[0];
    let mut v: Vec<String> = TurnGame::legal_actions(g, seat)
        .iter()
        .map(|a| a.usage())
        .filter(|u| u != "resign" && u != "timeout")
        .collect();
    v.sort();
    v
}

fn play(g: &mut Catur, moves: &[&str]) {
    for m in moves {
        let seat = TurnGame::pending_players(g)[0];
        Session::act(g, seat, m).unwrap_or_else(|e| panic!("{m}: {e}"));
    }
}

fn view(g: &Catur) -> serde_json::Value {
    Session::view_data(g, 0)
}

#[test]
fn starting_position() {
    let g = start();
    assert_eq!(TurnGame::seats(&g), 2);
    assert_eq!(TurnGame::pending_players(&g), vec![0]);
    let l = legal(&g);
    assert_eq!(l.len(), 20);
    assert!(l.contains(&"e4".into()) && l.contains(&"Nf3".into()));
    let v = view(&g);
    assert_eq!(v["papan"][0], "rnbqkbnr");
    assert_eq!(v["papan"][7], "RNBQKBNR");
    assert_eq!(v["langkah"].as_array().unwrap().len(), 20);
    assert!(TurnGame::legal_actions(&g, 0).iter().any(|a| a.usage() == "resign"));
    assert!(!TurnGame::legal_actions(&g, 0).iter().any(|a| a.usage() == "timeout"));
}

fn perft(g: &Catur, depth: u32) -> u64 {
    if depth == 0 || TurnGame::is_over(g) {
        return 1;
    }
    let seat = TurnGame::pending_players(g)[0];
    legal(g)
        .iter()
        .map(|m| {
            let mut next = g.clone();
            Session::act(&mut next, seat, m).unwrap();
            perft(&next, depth - 1)
        })
        .sum()
}

#[test]
fn perft_start_position() {
    let g = start();
    for (d, want) in [(1, 20), (2, 400), (3, 8902)] {
        assert_eq!(perft(&g, d), want, "kedalaman {d}");
    }
}

#[test]
fn perft_kiwipete_covers_castling_en_passant_promotion() {
    let g = fen("r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq - 0 1");
    assert_eq!(perft(&g, 1), 48);
    assert_eq!(perft(&g, 2), 2039);
}

#[test]
fn san_disambiguation_captures_checks() {
    // Dua kuda bisa ke d2: harus Nbd2 / Nfd2.
    let g = fen("4k3/8/8/8/8/8/8/1N2KN2 w - - 0 1");
    let l = legal(&g);
    assert!(l.contains(&"Nbd2".into()) && l.contains(&"Nfd2".into()), "{l:?}");
    // Benteng di baris yang sama tetapi kolom berbeda, dan di kolom yang sama.
    let g = fen("4k3/8/8/8/R7/8/8/R3K3 w - - 0 1");
    let l = legal(&g);
    assert!(l.contains(&"R1a2".into()) && l.contains(&"R4a2".into()), "{l:?}");
    // Tangkapan pion dan skak.
    let mut g = start();
    play(&mut g, &["e4", "d5"]);
    assert!(legal(&g).contains(&"exd5".into()));
    play(&mut g, &["Bb5+"]);
    assert_eq!(view(&g)["terakhir"]["san"], "Bb5+");
    assert!(view(&g)["skak"].as_bool().unwrap());
}

#[test]
fn coordinate_and_zero_castling_aliases() {
    let mut a = start();
    let mut b = start();
    play(&mut a, &["e4", "e5", "Nf3", "Nc6", "Bc4", "Bc5", "O-O"]);
    play(&mut b, &["e2e4", "e7e5", "g1f3", "b8c6", "f1c4", "f8c5", "e1g1"]);
    assert_eq!(Session::state_hash(&a), Session::state_hash(&b));
    let mut c = start();
    play(&mut c, &["e4", "e5", "Nf3", "Nc6", "Bc4", "Bc5", "0-0"]);
    assert_eq!(Session::state_hash(&a), Session::state_hash(&c));
    let v = view(&a);
    assert_eq!(v["papan"][7], "RNBQ1RK1");
    assert_eq!(v["terakhir"]["san"], "O-O");
    assert_eq!(v["terakhir"]["dari"], "e1");
    assert_eq!(v["terakhir"]["ke"], "g1");
}

#[test]
fn queenside_castling_and_castling_through_check() {
    let g = fen("r3k2r/8/8/8/8/8/8/R3K2R w KQkq - 0 1");
    let l = legal(&g);
    assert!(l.contains(&"O-O".into()) && l.contains(&"O-O-O".into()));
    let mut q = g.clone();
    play(&mut q, &["O-O-O"]);
    assert_eq!(view(&q)["papan"][7], "2KR3R");
    // Benteng hitam di f8 mengawasi f1: rokade pendek putih terlarang.
    let g = fen("4kr2/8/8/8/8/8/8/R3K2R w KQ - 0 1");
    let l = legal(&g);
    assert!(!l.contains(&"O-O".into()) && l.contains(&"O-O-O".into()), "{l:?}");
}

#[test]
fn en_passant() {
    let mut g = start();
    play(&mut g, &["e4", "a6", "e5", "d5"]);
    assert!(legal(&g).contains(&"exd6".into()));
    play(&mut g, &["exd6"]);
    let v = view(&g);
    assert_eq!(v["papan"][3], "........", "pion d5 hilang");
    assert_eq!(v["papan"][2], "p..P....");
    assert_eq!(v["terakhir"]["en_passant"], true);
}

#[test]
fn promotion_including_underpromotion() {
    let g = fen("8/P6k/8/8/8/8/8/K7 w - - 0 1");
    let l = legal(&g);
    for p in ["a8=Q", "a8=R", "a8=B", "a8=N"] {
        assert!(l.iter().any(|m| m.starts_with(p)), "{p} di {l:?}");
    }
    let mut q = g.clone();
    play(&mut q, &["a7a8n"]);
    assert_eq!(view(&q)["papan"][0], "N.......");
    assert_eq!(view(&q)["terakhir"]["promosi"], "n");
    let moves = view(&g)["langkah"].as_array().unwrap().clone();
    let promo: Vec<_> = moves.iter().filter(|m| m["dari"] == "a7").collect();
    assert_eq!(promo.len(), 4);
}

#[test]
fn checkmate() {
    let mut g = start();
    play(&mut g, &["f3", "e5", "g4", "Qh4#"]);
    assert!(TurnGame::is_over(&g));
    let r = TurnGame::result(&g).unwrap();
    assert_eq!(r.winners, vec![1]);
    let v = view(&g);
    assert_eq!(v["alasan"], "skakmat");
    assert_eq!(v["riwayat"], serde_json::json!(["f3", "e5", "g4", "Qh4#"]));
}

#[test]
fn stalemate_is_a_draw() {
    let g = fen("7k/5Q2/6K1/8/8/8/8/8 b - - 0 1");
    assert!(TurnGame::is_over(&g));
    assert!(TurnGame::result(&g).unwrap().winners.is_empty());
    assert_eq!(view(&g)["alasan"], "pat");
}

#[test]
fn threefold_repetition_is_a_draw() {
    let mut g = start();
    play(&mut g, &["Nf3", "Nf6", "Ng1", "Ng8", "Nf3", "Nf6", "Ng1"]);
    assert!(!TurnGame::is_over(&g));
    play(&mut g, &["Ng8"]);
    assert!(TurnGame::is_over(&g));
    assert_eq!(view(&g)["alasan"], "ulangan");
}

#[test]
fn fifty_move_rule_is_a_draw() {
    let mut g = fen("4k3/8/8/8/8/8/8/R3K3 w - - 99 80");
    play(&mut g, &["Ra2"]);
    assert!(TurnGame::is_over(&g));
    assert_eq!(view(&g)["alasan"], "50_langkah");
}

#[test]
fn insufficient_material_is_a_draw() {
    for f in [
        "4k3/8/8/8/8/8/8/4K3 w - - 0 1",
        "4k3/8/8/8/8/8/8/4KB2 w - - 0 1",
        "4k3/8/8/8/8/8/8/4KN2 b - - 0 1",
        "4kb2/8/8/8/8/8/8/2B1K3 w - - 0 1",
    ] {
        let g = fen(f);
        assert!(TurnGame::is_over(&g), "{f}");
        assert_eq!(view(&g)["alasan"], "bahan", "{f}");
    }
    // Gajah beda warna petak masih bisa mat: bukan remis otomatis.
    assert!(!TurnGame::is_over(&fen("4k1b1/8/8/8/8/8/8/2B1K3 w - - 0 1")));
}

#[test]
fn resign_and_timeout() {
    let mut g = start();
    play(&mut g, &["e4"]);
    Session::act(&mut g, 1, "resign").unwrap();
    assert_eq!(TurnGame::result(&g).unwrap().winners, vec![0]);
    assert_eq!(view(&g)["alasan"], "menyerah");

    let clocked = Config {
        fen: None,
        jam: Some(Jam {
            menit: 5,
            tambahan_detik: 3,
        }),
    };
    let mut g = Catur::new(clocked.clone(), [0; 32]).unwrap();
    assert!(TurnGame::legal_actions(&g, 0).iter().any(|a| a.usage() == "timeout"));
    Session::act(&mut g, 0, "timeout").unwrap();
    assert_eq!(TurnGame::result(&g).unwrap().winners, vec![1]);
    assert_eq!(view(&g)["alasan"], "waktu");

    // Waktu habis, tetapi lawan tinggal raja: remis.
    let mut g = Catur::new(
        Config {
            fen: Some("4k3/8/8/8/8/8/8/Q3K3 b - - 0 1".into()),
            ..clocked
        },
        [0; 32],
    )
    .unwrap();
    Session::act(&mut g, 1, "timeout").unwrap();
    assert!(TurnGame::result(&g).unwrap().winners.is_empty());

    let mut plain = start();
    assert_eq!(
        Session::act(&mut plain, 0, "timeout"),
        Err(GameError::Illegal("timeout".into()))
    );
}

#[test]
fn illegal_input_rejected() {
    let mut g = start();
    for bad in ["e5", "Ke2", "e2e5", "Nf4", "O-O", "", "xyz"] {
        assert!(Session::act(&mut g, 0, bad).is_err(), "{bad}");
    }
    assert_eq!(Session::act(&mut g, 1, "e5"), Err(GameError::NotPending(1)));
}

#[test]
fn every_legal_move_round_trips() {
    for f in [
        "r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq - 0 1",
        "8/P6k/8/8/8/8/8/K7 w - - 0 1",
        "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1",
    ] {
        let g = fen(f);
        for san in legal(&g) {
            let mv = g.parse_command(&san).unwrap();
            assert_eq!(g.format_action(&mv), san, "{f}");
        }
    }
}

#[test]
fn text_view_in_both_languages() {
    let g = start();
    let id = Session::view_text(&g, 0, Lang::Id);
    let en = Session::view_text(&g, 0, Lang::En);
    for t in [&id, &en] {
        assert!(t.contains("a b c d e f g h"), "{t}");
        assert!(t.contains("r n b q k b n r"), "{t}");
    }
    assert!(id.contains("Giliranmu (putih)"), "{id}");
    assert!(en.contains("Your turn (white)"), "{en}");
}

#[test]
fn random_playouts_terminate_deterministically() {
    let mut rng = kyusin_core::GameRng::from_seed([5; 32]);
    for _ in 0..20 {
        let mut g = start();
        let mut plies = 0;
        while !TurnGame::is_over(&g) {
            let l = legal(&g);
            let m = l[rng.below(l.len() as u32) as usize].clone();
            play(&mut g, &[&m]);
            plies += 1;
            assert!(plies < 1200, "harus berakhir lewat aturan remis");
        }
        assert!(view(&g)["alasan"].is_string());
    }
}

#[test]
fn pgn_export_import_round_trip() {
    let mut g = start();
    play(&mut g, &["e4", "e5", "Nf3", "Nc6", "Bb5", "a6", "Bxc6", "dxc6", "O-O"]);
    let text = pgn::export(&g, &pgn::Tags::new("Kamu", "Bot level 2", "2026.09.28"));
    assert!(text.contains("[White \"Kamu\"]"), "{text}");
    assert!(text.contains("[Result \"*\"]"), "{text}");
    assert!(text.contains("1. e4 e5 2. Nf3 Nc6 3. Bb5 a6 4. Bxc6 dxc6 5. O-O *"), "{text}");
    let game = pgn::import(&text).unwrap();
    assert_eq!(game.moves, legal_history(&g));
}

fn legal_history(g: &Catur) -> Vec<String> {
    serde_json::from_value(view(g)["riwayat"].clone()).unwrap()
}

#[test]
fn pgn_import_tolerates_comments_variations_nags_and_fen() {
    let text = r#"[Event "Uji"]
[FEN "4k3/8/8/8/8/8/8/R3K3 w - - 0 1"]
[SetUp "1"]

1. Ra7 {sebuah komentar} Kf8 $1 (1... Kd8 2. Ra8#) 2. Kd2 Kg8 1/2-1/2"#;
    let game = pgn::import(text).unwrap();
    assert_eq!(game.moves, vec!["Ra7", "Kf8", "Kd2", "Kg8"]);
    assert_eq!(game.fen.as_deref(), Some("4k3/8/8/8/8/8/8/R3K3 w - - 0 1"));
    assert_eq!(game.result, "1/2-1/2");
}

#[test]
fn pgn_import_reports_the_illegal_move() {
    let err = pgn::import("1. e4 e5 2. Ke3 *").unwrap_err();
    assert!(err.contains("Ke3"), "{err}");
}

#[test]
fn finished_game_pgn_has_result() {
    let mut g = start();
    play(&mut g, &["f3", "e5", "g4", "Qh4#"]);
    let text = pgn::export(&g, &pgn::Tags::new("A", "B", "2026.09.28"));
    assert!(text.contains("[Result \"0-1\"]") && text.trim_end().ends_with("0-1"), "{text}");
}
