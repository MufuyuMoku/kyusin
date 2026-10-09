//! Tes aturan Roulette Eropa dan Amerika serta mesin papan taruhan bersama
//! (SPEC §6.4, §7; M6a, D-069), ditulis sebelum mesinnya.
//!
//! - Eropa: kantong 0–36 tanpa La Partage; Amerika: 0, 00, 1–36 dengan Top
//!   Line 0-00-1-2-3 6:1.
//! - Angka tunggal 35:1, split 17:1, street/trio 11:1, corner 8:1, line
//!   5:1, lusin/kolom 2:1, merah/hitam/ganjil/genap/kecil/besar 1:1 (kalah
//!   pada 0/00).
//! - Satu putaran per pertandingan: taruhan, `spin`, lalu selesai.

use kyusin_core::{Session, TurnGame};
use kyusin_games::papan_taruhan::{BoardRules, exact_rtp};
use kyusin_games::roulette::{Amerika, Eropa, RouletteAmerika, RouletteEropa, pocket};
use serde_json::{Value, json};

fn eu(result: &str) -> RouletteEropa {
    RouletteEropa::new(
        serde_json::from_value(json!({ "hasil": result })).unwrap(),
        [0; 32],
    )
    .unwrap()
}

fn us(result: &str) -> RouletteAmerika {
    RouletteAmerika::new(
        serde_json::from_value(json!({ "hasil": result })).unwrap(),
        [0; 32],
    )
    .unwrap()
}

fn net<G: TurnGame + Session>(mut g: G, bets: &[(&str, i64)]) -> (Value, i64) {
    for (spot, n) in bets {
        Session::act(&mut g, 0, &format!("bet {spot} {n}"))
            .unwrap_or_else(|e| panic!("bet {spot}: {e:?}"));
    }
    Session::act(&mut g, 0, "spin").unwrap();
    let v = Session::view_data(&g, 0);
    let bersih = v["bersih"].as_i64().unwrap();
    (v, bersih)
}

#[test]
fn pockets_and_colours() {
    assert_eq!(pocket("0"), Some(0));
    assert_eq!(pocket("00"), Some(37));
    assert_eq!(pocket("36"), Some(36));
    assert_eq!(pocket("37"), None);
    let reds = [
        1, 3, 5, 7, 9, 12, 14, 16, 18, 19, 21, 23, 25, 27, 30, 32, 34, 36,
    ];
    for n in 1..=36u8 {
        let v = net(eu(&n.to_string()), &[("red", 10)]).1;
        assert_eq!(v == 10, reds.contains(&n), "{n}");
    }
}

#[test]
fn inside_bets_pay_by_type() {
    assert_eq!(net(eu("17"), &[("straight-17", 10)]).1, 350);
    assert_eq!(net(eu("17"), &[("split-17-18", 10)]).1, 170);
    assert_eq!(net(eu("17"), &[("split-14-17", 10)]).1, 170);
    assert_eq!(net(eu("17"), &[("street-16-17-18", 10)]).1, 110);
    assert_eq!(net(eu("17"), &[("corner-17-18-20-21", 10)]).1, 80);
    assert_eq!(net(eu("17"), &[("line-13-18", 10)]).1, 50);
    assert_eq!(net(eu("2"), &[("trio-0-1-2", 10)]).1, 110);
    assert_eq!(net(eu("0"), &[("split-0-3", 10)]).1, 170);
    assert_eq!(net(eu("17"), &[("straight-18", 10)]).1, -10);
    // Amerika: 00 dan Top Line.
    assert_eq!(net(us("00"), &[("straight-00", 10)]).1, 350);
    assert_eq!(net(us("00"), &[("split-0-00", 10)]).1, 170);
    assert_eq!(net(us("2"), &[("trio-00-2-3", 10)]).1, 110);
    assert_eq!(net(us("3"), &[("topline", 10)]).1, 60);
    assert_eq!(net(us("4"), &[("topline", 10)]).1, -10);
}

#[test]
fn outside_bets_lose_on_zero() {
    let outside = [
        ("red", 1),
        ("black", 1),
        ("odd", 1),
        ("even", 1),
        ("low", 1),
        ("high", 1),
    ];
    for (spot, _) in outside {
        assert_eq!(net(eu("0"), &[(spot, 10)]).1, -10, "{spot} pada 0");
        assert_eq!(net(us("00"), &[(spot, 10)]).1, -10, "{spot} pada 00");
    }
    assert_eq!(net(eu("13"), &[("dozen-2", 10)]).1, 20);
    assert_eq!(
        net(eu("13"), &[("column-1", 10)]).1,
        20,
        "13 di kolom 1 (1, 4, …, 34)"
    );
    assert_eq!(net(eu("0"), &[("dozen-1", 10), ("column-1", 10)]).1, -20);
    assert_eq!(
        net(eu("19"), &[("high", 10), ("low", 10), ("odd", 10)]).1,
        10
    );
}

#[test]
fn spot_lists_are_complete() {
    let count =
        |spots: &[&str], prefix: &str| spots.iter().filter(|s| s.starts_with(prefix)).count();
    let e = Eropa::spots();
    assert_eq!(count(e, "straight-"), 37);
    assert_eq!(count(e, "split-"), 57 + 3);
    assert_eq!(count(e, "street-"), 12);
    assert_eq!(count(e, "trio-"), 2);
    assert_eq!(count(e, "corner-"), 22);
    assert_eq!(count(e, "line-"), 11);
    assert_eq!(count(e, "topline"), 0);
    let a = Amerika::spots();
    assert_eq!(count(a, "straight-"), 38);
    assert_eq!(count(a, "split-"), 57 + 5);
    assert_eq!(count(a, "trio-"), 3);
    assert_eq!(count(a, "topline"), 1);
    for s in [
        "dozen-1", "dozen-3", "column-2", "red", "black", "odd", "even", "low", "high",
    ] {
        assert!(e.contains(&s) && a.contains(&s), "{s}");
    }
}

#[test]
fn exact_rtp_of_every_bet() {
    for s in Eropa::spots() {
        let r = exact_rtp::<Eropa>(s);
        assert!((r - 100.0 * 36.0 / 37.0).abs() < 1e-9, "Eropa {s}: {r}");
    }
    for s in Amerika::spots() {
        let r = exact_rtp::<Amerika>(s);
        let want = if *s == "topline" {
            100.0 * 35.0 / 38.0
        } else {
            100.0 * 36.0 / 38.0
        };
        assert!((r - want).abs() < 1e-9, "Amerika {s}: {r}");
    }
}

#[test]
fn one_spin_per_match_and_the_chip_contract() {
    let mut g = eu("17");
    let v = Session::view_data(&g, 0);
    assert_eq!(v["fase"], "taruhan");
    assert_eq!(v["hasil"], Value::Null);
    let mut legal: Vec<String> = TurnGame::legal_actions(&g, 0)
        .iter()
        .map(|a| a.usage())
        .collect();
    legal.sort();
    assert!(legal.contains(&"leave".to_string()));
    assert!(
        !legal.contains(&"spin".to_string()),
        "spin hanya setelah ada taruhan"
    );
    // Batas meja 10–2.000 kelipatan 10 per tempat.
    assert!(Session::act(&mut g, 0, "bet red 5").is_err());
    assert!(Session::act(&mut g, 0, "bet red 2010").is_err());
    assert!(Session::act(&mut g, 0, "bet nowhere 10").is_err());
    Session::act(&mut g, 0, "bet red 100").unwrap();
    Session::act(&mut g, 0, "bet straight-17 10").unwrap();
    let v = Session::view_data(&g, 0);
    assert_eq!(v["taruhan_meja"], 110);
    assert_eq!(v["taruhan"], json!({ "red": 100, "straight-17": 10 }));
    assert_eq!(v["netral"], "spin");
    Session::act(&mut g, 0, "clear").unwrap();
    assert_eq!(Session::view_data(&g, 0)["taruhan_meja"], 0);
    Session::act(&mut g, 0, "bet black 100").unwrap();
    Session::act(&mut g, 0, "spin").unwrap();
    let v = Session::view_data(&g, 0);
    assert_eq!(v["fase"], "selesai");
    assert_eq!(v["alasan"], "ronde_selesai");
    assert_eq!(v["hasil"], "17");
    assert_eq!(v["bersih"], 100);
    assert_eq!(v["dipertaruhkan"], 100);
    assert_eq!(v["bayar"], json!({ "black": 100 }));
    assert!(TurnGame::is_over(&g));
    assert_eq!(TurnGame::result(&g).unwrap().scores, vec![100]);
    for cmd in ["bet red 10", "clear", "spin", "leave"] {
        let a = g.parse_command(cmd).unwrap();
        assert_eq!(g.format_action(&a), cmd);
    }
}

#[test]
fn results_come_from_the_round_seed() {
    let spin = |seed: u8| {
        let mut g = RouletteAmerika::new(Default::default(), [seed; 32]).unwrap();
        Session::act(&mut g, 0, "bet red 10").unwrap();
        Session::act(&mut g, 0, "spin").unwrap();
        Session::view_data(&g, 0)["hasil"].clone()
    };
    assert_eq!(spin(5), spin(5));
    let distinct: std::collections::BTreeSet<String> =
        (0..40).map(|s| spin(s).to_string()).collect();
    assert!(distinct.len() > 10, "{distinct:?}");
}
