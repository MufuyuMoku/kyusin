//! Tes aturan dan pembayaran Caribbean Stud Poker (SPEC §6.3, §7; M5a),
//! ditulis sebelum mesinnya.
//!
//! Aturan meja (varian paling umum, lihat DECISIONS): satu dek dikocok per
//! ronde; satu ronde = satu pertandingan. `bet <jumlah>` = ante (10–2.000,
//! kelipatan 10), lalu lima kartu untuk pemain dan lima untuk bandar (satu
//! terbuka: kartu bandar pertama). Pemain memilih `raise` (dua kali ante)
//! atau `fold` (ante kalah). Bandar memenuhi syarat dengan A-K atau lebih
//! (paling sedikit as dan king, atau pair ke atas). Tidak memenuhi: ante
//! 1:1, raise kembali. Memenuhi: tangan lebih tinggi menang; ante 1:1 dan
//! raise dibayar menurut tabel (pair atau kurang 1, two pair 2, three of a
//! kind 3, straight 4, flush 5, full house 7, four of a kind 20, straight
//! flush 50, royal flush 100); kalah = ante dan raise kalah; sama = seri.
//! Tanpa jackpot progresif.

use kyusin_core::{Lang, Session, TurnGame};
use kyusin_games::caribbean_stud::{CaribbeanStud, Config};
use serde_json::{Value, json};

fn cs(cards: &[&str]) -> CaribbeanStud {
    CaribbeanStud::new(
        Config {
            kartu: Some(cards.iter().map(|c| c.to_string()).collect()),
        },
        [0; 32],
    )
    .unwrap()
}

fn act(g: &mut CaribbeanStud, cmd: &str) {
    Session::act(g, 0, cmd).unwrap_or_else(|e| panic!("`{cmd}`: {e:?}"));
}

fn view(g: &CaribbeanStud) -> Value {
    Session::view_data(g, 0)
}

fn raised(player: &str, dealer: &str) -> Value {
    let cards: Vec<&str> = player.split(' ').chain(dealer.split(' ')).collect();
    let mut g = cs(&cards);
    act(&mut g, "bet 100");
    act(&mut g, "raise");
    view(&g)
}

#[test]
fn deal_shows_one_dealer_card() {
    let mut g = cs(&["Ah", "Kd", "9c", "7s", "2h", "Qc", "Jd", "8s", "6h", "3c"]);
    act(&mut g, "bet 100");
    let v = view(&g);
    assert_eq!(v["fase"], "keputusan");
    assert_eq!(v["pemain"], json!(["Ah", "Kd", "9c", "7s", "2h"]));
    assert_eq!(v["bandar"], json!(["Qc", "??", "??", "??", "??"]));
    assert_eq!(v["biaya"]["raise"], 200);
    assert_eq!(v["netral"], "fold");
    assert_eq!(v["taruhan_meja"], 100);
    assert_eq!(v["tangan_pemain"], "high_card");
}

#[test]
fn fold_loses_the_ante() {
    let mut g = cs(&["Ah", "Kd", "9c", "7s", "2h", "Qc", "Jd", "8s", "6h", "3c"]);
    act(&mut g, "bet 100");
    act(&mut g, "fold");
    let v = view(&g);
    assert_eq!(v["bayar"]["ante"], -100);
    assert_eq!(v["bersih"], -100);
    assert_eq!(v["dipertaruhkan"], 100);
    assert_eq!(v["fase"], "selesai");
    assert_eq!(v["bandar"].as_array().unwrap().len(), 5);
    assert_eq!(v["bandar"][1], "Jd", "bandar dibuka setelah selesai");
}

#[test]
fn dealer_needs_ace_king() {
    // Bandar A-Q-9-7-3: tidak memenuhi → ante 1:1, raise kembali.
    let v = raised("2h 5d 9h Tc Js", "Ac Qd 9s 7h 3c");
    assert_eq!(v["memenuhi"], false);
    assert_eq!(v["bayar"]["ante"], 100);
    assert_eq!(v["bayar"]["raise"], 0);
    assert_eq!(v["bersih"], 100);
    assert_eq!(v["dipertaruhkan"], 300);
    // Bandar A-K-5-4-2 memenuhi dan menang dari J-high.
    let v = raised("2h 5d 9h Tc Js", "Ac Kd 5s 4h 2c");
    assert_eq!(v["memenuhi"], true);
    assert_eq!(v["bayar"]["ante"], -100);
    assert_eq!(v["bayar"]["raise"], -200);
    assert_eq!(v["bersih"], -300);
}

#[test]
fn raise_pays_by_the_player_hand() {
    // Bandar pair 2 (memenuhi); pemain menang dengan berbagai tangan.
    let dealer = "2c 2d 7s 8h 9c";
    let cases = [
        ("Ah Kd Qc Jh 3s", -1),          // kartu tinggi kalah dari pair
        ("3h 3d Qh Jd 5s", 1),           // pair
        ("3h 3d Qh Qd 5s", 2),           // two pair
        ("3h 3d 3s Qd 5s", 3),           // three of a kind
        ("Th Jd Qh Kd As", 4),           // straight
        ("3h 5h Qh Th Ah", 5),           // flush
        ("3h 3d 3s Qd Qs", 7),           // full house
        ("Ts Td Th Tc 5s", 20),          // four of a kind
        ("8s 9s Ts Js Qs", 50),          // straight flush
        ("Th Jh Qh Kh Ah", 100),         // royal flush
    ];
    for (player, k) in cases {
        let v = raised(player, dealer);
        if k < 0 {
            assert_eq!(v["bayar"]["raise"], -200, "{player}");
        } else {
            assert_eq!(v["bayar"]["ante"], 100, "{player}");
            assert_eq!(v["bayar"]["raise"], 200 * k, "{player}");
        }
    }
    // Sama persis: seri.
    let v = raised("As Kd 9h 7h 3s", "Ac Kh 9d 7c 3d");
    assert_eq!(v["bayar"]["ante"], 0);
    assert_eq!(v["bayar"]["raise"], 0);
}

#[test]
fn bets_and_leave() {
    let mut g = cs(&["Ah", "Kd", "9c", "7s", "2h", "Qc", "Jd", "8s", "6h", "3c"]);
    for bad in ["bet 5", "bet 2010", "raise", "fold", "bet ante 10"] {
        assert!(Session::act(&mut g, 0, bad).is_err(), "{bad}");
    }
    act(&mut g, "leave");
    assert_eq!(view(&g)["alasan"], "berhenti");
    assert!(TurnGame::is_over(&g));
}

#[test]
fn commands_round_trip_and_text_view() {
    let g = cs(&["Ah", "Kd", "9c", "7s", "2h", "Qc", "Jd", "8s", "6h", "3c"]);
    for cmd in ["bet 100", "raise", "fold", "leave"] {
        let a = g.parse_command(cmd).unwrap();
        assert_eq!(g.format_action(&a), cmd);
    }
    assert!(Session::view_text(&g, 0, Lang::Id).contains("bet"));
    assert!(Session::view_text(&g, 0, Lang::En).contains("bet"));
}
