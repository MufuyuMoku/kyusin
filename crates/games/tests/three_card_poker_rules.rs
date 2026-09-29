//! Tes aturan dan pembayaran Three Card Poker (SPEC §6.3, §7; M5a),
//! ditulis sebelum mesinnya.
//!
//! Aturan meja (varian paling umum, lihat DECISIONS): satu dek dikocok per
//! ronde; satu ronde = satu pertandingan (commit-reveal per ronde). Taruhan
//! `ante` dan/atau `pairplus` (10–2.000, kelipatan 10), lalu `deal`: tiga
//! kartu untuk pemain dan tiga (tertutup) untuk bandar. Dengan ante, pemain
//! memilih `play` (taruhan play sebesar ante) atau `fold` (ante kalah).
//! Bandar memenuhi syarat dengan Q-high atau lebih. Tidak memenuhi: ante
//! dibayar 1:1, play kembali. Memenuhi: tangan lebih tinggi menang (ante
//! dan play 1:1), sama = seri. Ante bonus bila bermain (apa pun tangan
//! bandar): straight 1:1, three of a kind 4:1, straight flush 5:1. Pair
//! Plus (terpisah): pair 1, flush 3, straight 6, three of a kind 30,
//! straight flush 40; selain itu kalah.

use kyusin_core::{Lang, Session, TurnGame};
use kyusin_games::three_card_poker::{Config, ThreeCardPoker};
use serde_json::{Value, json};

fn tcp(cards: &[&str]) -> ThreeCardPoker {
    ThreeCardPoker::new(
        Config {
            kartu: Some(cards.iter().map(|c| c.to_string()).collect()),
        },
        [0; 32],
    )
    .unwrap()
}

fn act(g: &mut ThreeCardPoker, cmd: &str) {
    Session::act(g, 0, cmd).unwrap_or_else(|e| panic!("`{cmd}`: {e:?}"));
}

fn view(g: &ThreeCardPoker) -> Value {
    Session::view_data(g, 0)
}

fn legal(g: &ThreeCardPoker) -> Vec<String> {
    TurnGame::legal_actions(g, 0)
        .iter()
        .map(|a| a.usage())
        .collect()
}

/// Pemain `p` (3 kartu), bandar `d` (3 kartu), ante 100 lalu `play`.
fn played(p: [&str; 3], d: [&str; 3]) -> Value {
    let mut g = tcp(&[p[0], p[1], p[2], d[0], d[1], d[2]]);
    act(&mut g, "bet ante 100");
    act(&mut g, "deal");
    act(&mut g, "play");
    view(&g)
}

#[test]
fn deal_hides_the_dealer_until_the_decision() {
    let mut g = tcp(&["Ah", "Kd", "2c", "Qs", "Jh", "9d"]);
    act(&mut g, "bet ante 100");
    act(&mut g, "bet pairplus 50");
    assert_eq!(view(&g)["taruhan_meja"], 150);
    act(&mut g, "deal");
    let v = view(&g);
    assert_eq!(v["fase"], "keputusan");
    assert_eq!(v["pemain"], json!(["Ah", "Kd", "2c"]));
    assert_eq!(v["bandar"], json!(["??", "??", "??"]));
    assert_eq!(v["biaya"]["play"], 100);
    assert_eq!(v["netral"], "fold");
    assert_eq!(v["taruhan_meja"], 150);
    assert_eq!(v["ronde"], 1);
    assert_eq!(legal(&g), vec!["play", "fold"]);
}

#[test]
fn dealer_must_qualify_with_queen_high() {
    // Bandar J-high: tidak memenuhi → ante 1:1, play kembali.
    let v = played(["2h", "3d", "5c"], ["Jc", "9s", "4h"]);
    assert_eq!(v["memenuhi"], false);
    assert_eq!(v["bayar"]["ante"], 100);
    assert_eq!(v["bayar"]["play"], 0);
    assert_eq!(v["bersih"], 100);
    assert_eq!(v["bandar"], json!(["Jc", "9s", "4h"]));
    assert_eq!(v["fase"], "selesai");
    // Bandar Q-high memenuhi dan menang.
    let v = played(["2h", "3d", "5c"], ["Qc", "9s", "4h"]);
    assert_eq!(v["memenuhi"], true);
    assert_eq!(v["bayar"]["ante"], -100);
    assert_eq!(v["bayar"]["play"], -100);
    assert_eq!(v["bersih"], -200);
    assert_eq!(v["dipertaruhkan"], 200);
}

#[test]
fn qualified_dealer_is_compared() {
    // Pemain pair lawan bandar K-high: menang 1:1 + 1:1.
    let v = played(["9h", "9d", "2c"], ["Kc", "7s", "4h"]);
    assert_eq!(v["bayar"]["ante"], 100);
    assert_eq!(v["bayar"]["play"], 100);
    assert_eq!(v["tangan_pemain"], "pair");
    assert_eq!(v["tangan_bandar"], "high_card");
    // Seri persis: semuanya kembali.
    let v = played(["Kh", "7d", "4c"], ["Kc", "7s", "4h"]);
    assert_eq!(v["bayar"]["ante"], 0);
    assert_eq!(v["bayar"]["play"], 0);
    assert_eq!(v["bersih"], 0);
}

#[test]
fn ante_bonus_is_paid_even_when_losing() {
    // Pemain straight lawan bandar flush? Flush di bawah straight: pemain
    // menang + bonus straight 1:1.
    let v = played(["4h", "5d", "6c"], ["Kc", "7c", "4c"]);
    assert_eq!(v["bayar"]["bonus"], 100);
    assert_eq!(v["bersih"], 300);
    // Pemain three of a kind kalah dari straight flush: bonus 4:1 tetap.
    let v = played(["8h", "8d", "8c"], ["9s", "Ts", "Js"]);
    assert_eq!(v["bayar"]["ante"], -100);
    assert_eq!(v["bayar"]["play"], -100);
    assert_eq!(v["bayar"]["bonus"], 400);
    assert_eq!(v["bersih"], 200);
    // Straight flush: 5:1.
    let v = played(["Qh", "Kh", "Ah"], ["2c", "3d", "7s"]);
    assert_eq!(v["bayar"]["bonus"], 500);
    // Tanpa straight: tidak ada bonus.
    let v = played(["Ah", "Kh", "9h"], ["2c", "3d", "7s"]);
    assert_eq!(v["bayar"].get("bonus"), None);
}

#[test]
fn fold_loses_the_ante_but_pair_plus_still_pays() {
    let mut g = tcp(&["7h", "7d", "2c", "Ac", "Ks", "4h"]);
    act(&mut g, "bet ante 100");
    act(&mut g, "bet pairplus 20");
    act(&mut g, "deal");
    act(&mut g, "fold");
    let v = view(&g);
    assert_eq!(v["bayar"]["ante"], -100);
    assert_eq!(v["bayar"]["pairplus"], 20);
    assert_eq!(v["bayar"].get("play"), None);
    assert_eq!(v["bersih"], -80);
    assert_eq!(v["dipertaruhkan"], 120);
    assert!(TurnGame::is_over(&g));
}

#[test]
fn pair_plus_pay_table() {
    let cases = [
        (["2h", "5d", "9c"], -10),
        (["2h", "2d", "9c"], 10),
        (["2h", "5h", "9h"], 30),
        (["2h", "3d", "4c"], 60),
        (["Ah", "2d", "3c"], 60),
        (["5h", "5d", "5c"], 300),
        (["Jh", "Qh", "Kh"], 400),
    ];
    for (p, pay) in cases {
        let mut g = tcp(&[p[0], p[1], p[2], "Ac", "Ks", "4h"]);
        act(&mut g, "bet pairplus 10");
        act(&mut g, "deal");
        // Tanpa ante: tidak ada keputusan, ronde langsung selesai.
        let v = view(&g);
        assert_eq!(v["fase"], "selesai", "{p:?}");
        assert_eq!(v["bayar"]["pairplus"], pay, "{p:?}");
    }
}

#[test]
fn bets_and_leave() {
    let mut g = tcp(&["2h", "5d", "9c", "Ac", "Ks", "4h"]);
    assert!(!legal(&g).contains(&"deal".to_string()));
    for bad in ["bet ante 5", "bet play 10", "play", "fold"] {
        assert!(Session::act(&mut g, 0, bad).is_err(), "{bad}");
    }
    act(&mut g, "bet ante 10");
    act(&mut g, "clear");
    assert_eq!(view(&g)["taruhan_meja"], 0);
    act(&mut g, "leave");
    let v = view(&g);
    assert_eq!(v["alasan"], "berhenti");
    assert_eq!(v["ronde"], 0);
}

#[test]
fn commands_round_trip_and_text_view() {
    let g = tcp(&["2h", "5d", "9c", "Ac", "Ks", "4h"]);
    for cmd in [
        "bet ante 100",
        "bet pairplus 10",
        "clear",
        "deal",
        "play",
        "fold",
        "leave",
    ] {
        let a = g.parse_command(cmd).unwrap();
        assert_eq!(g.format_action(&a), cmd);
    }
    assert!(Session::view_text(&g, 0, Lang::Id).contains("ante"));
    assert!(Session::view_text(&g, 0, Lang::En).contains("ante"));
}
