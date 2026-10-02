//! Tes aturan dan pembayaran Let It Ride (SPEC §6.3, §7; M5a), ditulis
//! sebelum mesinnya.
//!
//! Aturan meja (varian paling umum, lihat DECISIONS): satu dek dikocok per
//! ronde; satu ronde = satu pertandingan. `bet <jumlah>` memasang jumlah
//! itu di tiga tempat (biaya 3 × jumlah; 10–2.000 per tempat, kelipatan
//! 10), lalu tiga kartu untuk pemain dan dua kartu bersama tertutup. Dua
//! kali pemain memilih `pull` (tempat 1, lalu tempat 2, kembali) atau
//! `ride`; setelah tiap keputusan satu kartu bersama dibuka. Tempat yang
//! tersisa dibayar menurut tangan lima kartu: pair 10 ke atas 1, two pair 2,
//! three of a kind 3, straight 5, flush 8, full house 11, four of a kind 50,
//! straight flush 200, royal flush 1.000; selain itu kalah. Tanpa taruhan
//! sampingan.

use kyusin_core::{Lang, Session, TurnGame};
use kyusin_games::let_it_ride::{Config, LetItRide};
use serde_json::{Value, json};

fn lir(cards: &[&str]) -> LetItRide {
    LetItRide::new(
        Config {
            kartu: Some(cards.iter().map(|c| c.to_string()).collect()),
        },
        [0; 32],
    )
    .unwrap()
}

fn act(g: &mut LetItRide, cmd: &str) {
    Session::act(g, 0, cmd).unwrap_or_else(|e| panic!("`{cmd}`: {e:?}"));
}

fn view(g: &LetItRide) -> Value {
    Session::view_data(g, 0)
}

fn play(hand: &str, first: &str, second: &str) -> Value {
    let cards: Vec<&str> = hand.split(' ').collect();
    let mut all = cards.clone();
    all.extend(first.split(' '));
    let mut g = lir(&all);
    act(&mut g, "bet 100");
    for d in second.split(' ') {
        act(&mut g, d);
    }
    view(&g)
}

#[test]
fn bet_places_three_spots_and_hides_the_community_cards() {
    let mut g = lir(&["Th", "Td", "4c", "9s", "2h"]);
    act(&mut g, "bet 100");
    let v = view(&g);
    assert_eq!(v["fase"], "keputusan");
    assert_eq!(v["pemain"], json!(["Th", "Td", "4c"]));
    assert_eq!(v["bersama"], json!(["??", "??"]));
    assert_eq!(v["tempat"], json!([100, 100, 100]));
    assert_eq!(v["taruhan_meja"], 300);
    assert_eq!(v["pengali"]["bet"], 3, "biaya bet = 3 × jumlah");
    assert_eq!(v["netral"], "pull");
    act(&mut g, "ride");
    let v = view(&g);
    assert_eq!(v["bersama"], json!(["9s", "??"]));
    assert_eq!(v["fase"], "keputusan");
    act(&mut g, "pull");
    let v = view(&g);
    assert_eq!(v["bersama"], json!(["9s", "2h"]));
    assert_eq!(v["tempat"], json!([100, 0, 100]));
    assert_eq!(v["fase"], "selesai");
    // Pair 10: 1:1 untuk dua tempat yang tersisa.
    assert_eq!(v["tangan"], "pair");
    assert_eq!(v["bayar"], 200);
    assert_eq!(v["bersih"], 200);
    assert_eq!(v["dipertaruhkan"], 200);
}

#[test]
fn pulling_returns_spots_and_low_pairs_lose() {
    // Pair 9 tidak dibayar; dua tempat ditarik, tempat ketiga kalah.
    let v = play("9h 9d 4c", "Ks 2h", "pull pull");
    assert_eq!(v["tempat"], json!([0, 0, 100]));
    assert_eq!(v["bayar"], -100);
    assert_eq!(v["bersih"], -100);
    assert_eq!(v["taruhan_meja"], 0);
    // Taruhan yang ditarik mengurangi chip yang dipertaruhkan.
    let mut g = lir(&["9h", "9d", "4c", "Ks", "2h"]);
    act(&mut g, "bet 100");
    act(&mut g, "pull");
    assert_eq!(view(&g)["taruhan_meja"], 200);
}

#[test]
fn pay_table() {
    let cases = [
        ("Ah Kd 4c", "9s 2h", -1),
        ("Jh Jd 4c", "9s 2h", 1),
        ("Jh Jd 4c", "4s 2h", 2),
        ("Jh Jd Jc", "4s 2h", 3),
        ("5h 6d 7c", "8s 9h", 5),
        ("Ah 2d 3c", "4s 5h", 5),
        ("2h 7h 9h", "Jh Kh", 8),
        ("Jh Jd Jc", "4s 4h", 11),
        ("Jh Jd Jc", "Js 4h", 50),
        ("5h 6h 7h", "8h 9h", 200),
        ("Th Jh Qh", "Kh Ah", 1000),
    ];
    for (hand, board, k) in cases {
        let v = play(hand, board, "ride ride");
        assert_eq!(v["bayar"], 300 * k, "{hand} {board}");
    }
}

#[test]
fn bets_and_leave() {
    let mut g = lir(&["Th", "Td", "4c", "9s", "2h"]);
    for bad in ["bet 5", "bet 2010", "pull", "ride", "bet spot 10"] {
        assert!(Session::act(&mut g, 0, bad).is_err(), "{bad}");
    }
    act(&mut g, "leave");
    assert_eq!(view(&g)["alasan"], "berhenti");
    assert!(TurnGame::is_over(&g));
}

#[test]
fn commands_round_trip_and_text_view() {
    let g = lir(&["Th", "Td", "4c", "9s", "2h"]);
    for cmd in ["bet 100", "pull", "ride", "leave"] {
        let a = g.parse_command(cmd).unwrap();
        assert_eq!(g.format_action(&a), cmd);
    }
    assert!(Session::view_text(&g, 0, Lang::Id).contains("bet"));
    assert!(Session::view_text(&g, 0, Lang::En).contains("bet"));
}
