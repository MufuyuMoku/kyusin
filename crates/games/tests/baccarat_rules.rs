//! Tes aturan dan pembayaran Baccarat Punto Banco (SPEC §6.3, §7; M5a),
//! ditulis sebelum mesinnya.
//!
//! Aturan meja (varian paling umum, lihat DECISIONS): 8 dek; shoe selesai
//! di akhir ronde setelah titik potong (16 kartu sebelum akhir shoe).
//! Taruhan `player` (1:1), `banker` (0,95:1, komisi 5%), `tie` (8:1; player
//! dan banker kembali bila seri); tiap tempat 20–2.000, kelipatan 20 supaya
//! komisi selalu utuh. Kartu dibagi pemain, bankir, pemain, bankir. Nilai:
//! as 1, 2–9 sesuai angka, 10/J/Q/K 0; total = jumlah mod 10. Natural (8
//! atau 9 di dua kartu salah satu pihak): tidak ada kartu ketiga. Pemain
//! menarik pada 0–5, berdiri pada 6–7. Bila pemain berdiri, bankir menarik
//! pada 0–5. Bila pemain menarik kartu ketiga bernilai p: bankir 0–2
//! menarik; 3 menarik kecuali p = 8; 4 bila p 2–7; 5 bila p 4–7; 6 bila p
//! 6–7; 7 berdiri.

use kyusin_core::{Lang, Session, TurnGame};
use kyusin_games::baccarat::{Baccarat, Config};
use serde_json::{Value, json};

fn bac(cards: &[&str], cut: usize) -> Baccarat {
    Baccarat::new(
        Config {
            dek: None,
            shoe: Some(cards.iter().map(|c| c.to_string()).collect()),
            potong: Some(cut),
        },
        [0; 32],
    )
    .unwrap()
}

fn act(g: &mut Baccarat, cmd: &str) {
    Session::act(g, 0, cmd).unwrap_or_else(|e| panic!("`{cmd}`: {e:?}"));
}

fn view(g: &Baccarat) -> Value {
    Session::view_data(g, 0)
}

/// Satu ronde dengan kartu `cards`, taruhan player 100; mengembalikan view.
fn round(cards: &[&str]) -> Value {
    let mut all: Vec<&str> = cards.to_vec();
    all.extend(["2c"; 6]);
    let mut g = bac(&all, 100);
    act(&mut g, "bet player 100");
    act(&mut g, "deal");
    view(&g)
}

#[test]
fn default_shoe_is_eight_decks_cut_16_from_the_end() {
    let g = Baccarat::new(Config::default(), [5; 32]).unwrap();
    let v = view(&g);
    assert_eq!(v["sisa"], 416);
    assert_eq!(v["potong"], 400);
    assert_eq!(v["min_taruhan"], 20);
    assert_eq!(v["langkah_taruhan"], 20);
}

#[test]
fn naturals_stop_the_deal() {
    // Pemain 4+5 = 9 natural; bankir 3+3 = 6.
    let v = round(&["4h", "3c", "5d", "3s"]);
    assert_eq!(v["pemain"], json!(["4h", "5d"]));
    assert_eq!(v["bankir"], json!(["3c", "3s"]));
    assert_eq!(v["nilai_pemain"], 9);
    assert_eq!(v["nilai_bankir"], 6);
    assert_eq!(v["pemenang"], "player");
    assert_eq!(v["bayar"]["player"], 100);
    // Bankir natural 8 (K+8), pemain 5 tidak menarik.
    let v = round(&["2h", "Kc", "3d", "8s"]);
    assert_eq!(v["pemain"], json!(["2h", "3d"]));
    assert_eq!(v["pemenang"], "banker");
    assert_eq!(v["bayar"]["player"], -100);
}

#[test]
fn player_draws_on_zero_to_five_and_stands_on_six_seven() {
    // Pemain 6 berdiri; bankir 5 menarik (pemain berdiri → bankir 0–5).
    let v = round(&["6h", "2c", "Kd", "3s", "4h"]);
    assert_eq!(v["pemain"], json!(["6h", "Kd"]));
    assert_eq!(v["bankir"], json!(["2c", "3s", "4h"]));
    assert_eq!(v["nilai_bankir"], 9);
    assert_eq!(v["pemenang"], "banker");
    // Pemain 7 berdiri; bankir 6 berdiri; pemain menang.
    let v = round(&["7h", "3c", "Kd", "3s"]);
    assert_eq!(v["pemain"], json!(["7h", "Kd"]));
    assert_eq!(v["bankir"], json!(["3c", "3s"]));
    assert_eq!(v["pemenang"], "player");
    // Pemain 5 menarik.
    let v = round(&["2h", "Kc", "3d", "Ks", "Ah"]);
    assert_eq!(v["pemain"], json!(["2h", "3d", "Ah"]));
}

#[test]
fn banker_third_card_tableau() {
    // (bankir total, kartu ketiga pemain) → menarik?
    let cases = [
        (2, "8", true),
        (3, "8", false),
        (3, "9", true),
        (4, "A", false),
        (4, "2", true),
        (4, "7", true),
        (4, "8", false),
        (5, "3", false),
        (5, "4", true),
        (5, "7", true),
        (5, "8", false),
        (6, "5", false),
        (6, "6", true),
        (6, "7", true),
        (6, "8", false),
        (7, "6", false),
    ];
    for (banker, p3, draws) in cases {
        // Pemain K+K = 0 menarik p3; bankir dari K + total.
        let b2 = if banker == 0 {
            "K"
        } else {
            &banker.to_string()
        };
        let b2 = format!("{}d", if b2 == "1" { "A" } else { b2 });
        let p3c = format!("{p3}h");
        let cards = ["Kh", "Kc", "Ks", b2.as_str(), p3c.as_str(), "9c"];
        let v = round(&cards);
        let n = v["bankir"].as_array().unwrap().len();
        assert_eq!(n == 3, draws, "bankir {banker}, pemain menarik {p3}: {v}");
    }
}

#[test]
fn payouts_commission_and_ties() {
    // Bankir menang: banker dibayar 0,95 (100 → 95).
    let mut g = bac(
        &["2h", "Kc", "3d", "8s", "9h", "9c", "Kh", "Kd", "2c", "2d"],
        100,
    );
    act(&mut g, "bet banker 100");
    act(&mut g, "bet tie 20");
    act(&mut g, "deal");
    let v = view(&g);
    assert_eq!(v["bayar"]["banker"], 95);
    assert_eq!(v["bayar"]["tie"], -20);
    assert_eq!(v["bersih"], 75);
    // Seri 9-9: tie 8:1, player dan banker kembali.
    act(&mut g, "bet banker 100");
    act(&mut g, "bet player 40");
    act(&mut g, "bet tie 20");
    act(&mut g, "deal");
    let v = view(&g);
    assert_eq!(v["pemenang"], "tie");
    assert_eq!(v["bayar"]["tie"], 160);
    assert_eq!(v["bayar"]["banker"], 0);
    assert_eq!(v["bayar"]["player"], 0);
    assert_eq!(v["bersih"], 235);
    assert_eq!(v["dipertaruhkan"], 160);
}

#[test]
fn bets_limits_and_shoe_end() {
    let mut g = bac(&["4h", "3c", "5d", "3s", "4h", "3c", "5d", "3s"], 4);
    for bad in [
        "bet player 10",
        "bet player 30",
        "bet banker 2020",
        "bet pair 20",
    ] {
        assert!(Session::act(&mut g, 0, bad).is_err(), "{bad}");
    }
    act(&mut g, "bet player 20");
    assert_eq!(view(&g)["taruhan_meja"], 20);
    act(&mut g, "deal");
    let v = view(&g);
    assert_eq!(v["fase"], "selesai");
    assert_eq!(v["alasan"], "shoe_habis");
    let mut g = bac(&["4h", "3c"], 100);
    act(&mut g, "bet tie 20");
    act(&mut g, "leave");
    assert_eq!(view(&g)["taruhan_meja"], 0);
    assert!(TurnGame::is_over(&g));
}

#[test]
fn commands_round_trip_and_text_view() {
    let g = bac(&["4h", "3c"], 100);
    for cmd in [
        "bet player 20",
        "bet banker 40",
        "bet tie 20",
        "clear",
        "deal",
        "leave",
    ] {
        let a = g.parse_command(cmd).unwrap();
        assert_eq!(g.format_action(&a), cmd);
    }
    assert!(Session::view_text(&g, 0, Lang::Id).contains("banker"));
    assert!(Session::view_text(&g, 0, Lang::En).contains("banker"));
}
