//! Tes aturan dan pembayaran Casino Hold'em (SPEC §6.3, §7; M5a), ditulis
//! sebelum mesinnya.
//!
//! Aturan meja (varian paling umum, lihat DECISIONS): satu dek dikocok per
//! ronde; satu ronde = satu pertandingan. `bet <jumlah>` = ante (10–2.000,
//! kelipatan 10); dua kartu untuk pemain, dua (tertutup) untuk bandar, dan
//! tiga kartu bersama (flop). Pemain memilih `call` (dua kali ante) atau
//! `fold` (ante kalah). Setelah call: turn dan river dibuka, tangan terbaik
//! 5 dari 7 dibandingkan. Bandar memenuhi syarat dengan pair 4 atau lebih.
//! Tidak memenuhi: ante dibayar menurut tabel AnteWin, call kembali.
//! Memenuhi: pemain menang = ante menurut tabel + call 1:1; kalah = ante dan
//! call kalah; sama = seri. AnteWin: royal flush 100, straight flush 20,
//! four of a kind 10, full house 3, flush 2, straight atau kurang 1. Tanpa
//! taruhan sampingan AA bonus.

use kyusin_core::{Lang, Session, TurnGame};
use kyusin_games::casino_holdem::{CasinoHoldem, Config};
use serde_json::{Value, json};

fn ch(cards: &[&str]) -> CasinoHoldem {
    CasinoHoldem::new(
        Config {
            kartu: Some(cards.iter().map(|c| c.to_string()).collect()),
        },
        [0; 32],
    )
    .unwrap()
}

fn act(g: &mut CasinoHoldem, cmd: &str) {
    Session::act(g, 0, cmd).unwrap_or_else(|e| panic!("`{cmd}`: {e:?}"));
}

fn view(g: &CasinoHoldem) -> Value {
    Session::view_data(g, 0)
}

/// Urutan dek: pemain 2, bandar 2, flop 3, turn, river.
fn called(player: &str, dealer: &str, board: &str) -> Value {
    let cards: Vec<&str> = player
        .split(' ')
        .chain(dealer.split(' '))
        .chain(board.split(' '))
        .collect();
    let mut g = ch(&cards);
    act(&mut g, "bet 100");
    act(&mut g, "call");
    view(&g)
}

#[test]
fn deal_shows_the_flop_and_hides_the_dealer() {
    let mut g = ch(&["Ah", "Kh", "2c", "7d", "Qh", "Jh", "3s", "9c", "4d"]);
    act(&mut g, "bet 100");
    let v = view(&g);
    assert_eq!(v["fase"], "keputusan");
    assert_eq!(v["pemain"], json!(["Ah", "Kh"]));
    assert_eq!(v["bandar"], json!(["??", "??"]));
    assert_eq!(v["meja_kartu"], json!(["Qh", "Jh", "3s"]));
    assert_eq!(v["biaya"]["call"], 200);
    assert_eq!(v["netral"], "fold");
    assert_eq!(v["taruhan_meja"], 100);
}

#[test]
fn fold_loses_the_ante_and_shows_everything() {
    let mut g = ch(&["2h", "7c", "Ac", "Ad", "Qh", "Jh", "3s", "9c", "4d"]);
    act(&mut g, "bet 100");
    act(&mut g, "fold");
    let v = view(&g);
    assert_eq!(v["bayar"]["ante"], -100);
    assert_eq!(v["bersih"], -100);
    assert_eq!(v["meja_kartu"].as_array().unwrap().len(), 5);
    assert_eq!(v["bandar"], json!(["Ac", "Ad"]));
    assert_eq!(v["fase"], "selesai");
}

#[test]
fn dealer_qualifies_with_a_pair_of_fours() {
    // Bandar hanya kartu tinggi: tidak memenuhi → ante 1:1 (straight atau
    // kurang), call kembali.
    let v = called("Ah Kd", "2c 7d", "Qh Jh 3s 9c 5d");
    assert_eq!(v["memenuhi"], false);
    assert_eq!(v["bayar"]["ante"], 100);
    assert_eq!(v["bayar"]["call"], 0);
    // Bandar pair 3 (dari 3s di meja + 3c): tidak memenuhi.
    let v = called("Ah Kd", "3c 7d", "Qh Jh 3s 9c 5d");
    assert_eq!(v["memenuhi"], false);
    // Bandar pair 4: memenuhi dan menang dari A-high.
    let v = called("Ah Kd", "4c 7d", "Qh Jh 4s 9c 5d");
    assert_eq!(v["memenuhi"], true);
    assert_eq!(v["bayar"]["ante"], -100);
    assert_eq!(v["bayar"]["call"], -200);
    assert_eq!(v["bersih"], -300);
    assert_eq!(v["dipertaruhkan"], 300);
}

#[test]
fn ante_win_table() {
    // Bandar pair 5 memenuhi; pemain menang dengan berbagai tangan.
    let dealer = "5c 2d";
    let cases = [
        ("Ah Ad", "5h 9s Kc 7d 3h", 1),   // pair as
        ("7h 8d", "5h 9s Tc Jd 3h", 1),   // straight
        ("Ah 2h", "5h 9h Kc 7h 3d", 2),   // flush
        ("9d 9c", "5h 9s Kc Kd 3h", 3),   // full house
        ("9d 9c", "5h 9s 9h Kd 3h", 10),  // four of a kind
        ("6h 7h", "5h 8h 9h Kd 3c", 20),  // straight flush
        ("Ah Kh", "5h Th Jh Qh 3c", 100), // royal flush
    ];
    for (player, board, k) in cases {
        let v = called(player, dealer, board);
        assert_eq!(v["memenuhi"], true, "{player} {board}");
        assert_eq!(v["bayar"]["ante"], 100 * k, "{player} {board}");
        assert_eq!(v["bayar"]["call"], 200, "{player} {board}");
    }
    // Tidak memenuhi tetap memakai tabel: flush dibayar 2:1, call kembali.
    let v = called("Ah 2h", "Kc Qd", "5h 9h Jc 7h 3d");
    assert_eq!(v["memenuhi"], false);
    assert_eq!(v["bayar"]["ante"], 200);
    assert_eq!(v["bayar"]["call"], 0);
}

#[test]
fn equal_hands_push() {
    // Keduanya memakai straight di meja.
    let v = called("2h 3d", "2c 3s", "Th Jd Qc Kd Ah");
    assert_eq!(v["bayar"]["ante"], 0);
    assert_eq!(v["bayar"]["call"], 0);
    assert_eq!(v["tangan_pemain"], "straight");
}

#[test]
fn bets_and_leave() {
    let mut g = ch(&["Ah", "Kh", "2c", "7d", "Qh", "Jh", "3s", "9c", "4d"]);
    for bad in ["bet 5", "bet 2010", "call", "fold", "bet ante 10"] {
        assert!(Session::act(&mut g, 0, bad).is_err(), "{bad}");
    }
    act(&mut g, "leave");
    assert_eq!(view(&g)["alasan"], "berhenti");
}

#[test]
fn commands_round_trip_and_text_view() {
    let g = ch(&["Ah", "Kh", "2c", "7d", "Qh", "Jh", "3s", "9c", "4d"]);
    for cmd in ["bet 100", "call", "fold", "leave"] {
        let a = g.parse_command(cmd).unwrap();
        assert_eq!(g.format_action(&a), cmd);
    }
    assert!(Session::view_text(&g, 0, Lang::Id).contains("bet"));
    assert!(Session::view_text(&g, 0, Lang::En).contains("bet"));
}
