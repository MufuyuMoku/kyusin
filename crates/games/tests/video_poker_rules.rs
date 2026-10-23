//! Tes aturan dan pembayaran Video Poker (SPEC §6.5, §7; M6b, D-071),
//! ditulis sebelum mesinnya.
//!
//! Jacks or Better 9/6, Deuces Wild "Not So Ugly Ducks", Joker Poker Kings
//! or Better 18/7/5 (53 kartu). Satu taruhan dibayar seperti 5 koin (Royal
//! 800×). Strategi optimal = pilihan tahan dengan nilai harapan tepat
//! terbesar (enumerasi semua kartu pengganti).

use kyusin_core::{Session, TurnGame};
use kyusin_games::video_poker::{
    self, Hand, Variant, VpCard, evaluate, optimal::Solver,
};
use serde_json::{Value, json};

fn hand(cards: &str) -> [VpCard; 5] {
    let v: Vec<VpCard> = cards.split(' ').map(|c| c.parse().unwrap()).collect();
    v.try_into().unwrap()
}

fn eval(variant: Variant, cards: &str) -> Option<Hand> {
    evaluate(variant, &hand(cards))
}

#[test]
fn pay_tables() {
    use Hand::*;
    let job = Variant::JacksOrBetter;
    let table: Vec<(Hand, u32)> = [RoyalFlush, StraightFlush, FourOfAKind, FullHouse, Flush, Straight, ThreeOfAKind, TwoPair, HighPair]
        .iter()
        .map(|h| (*h, job.pays(*h)))
        .collect();
    assert_eq!(
        table,
        [(RoyalFlush, 800), (StraightFlush, 50), (FourOfAKind, 25), (FullHouse, 9), (Flush, 6), (Straight, 4), (ThreeOfAKind, 3), (TwoPair, 2), (HighPair, 1)]
    );
    let d = Variant::DeucesWild;
    let deuces: Vec<u32> = [RoyalFlush, FourDeuces, WildRoyal, FiveOfAKind, StraightFlush, FourOfAKind, FullHouse, Flush, Straight, ThreeOfAKind]
        .iter()
        .map(|h| d.pays(*h))
        .collect();
    assert_eq!(deuces, [800, 200, 25, 16, 10, 4, 4, 3, 2, 1]);
    assert_eq!(d.pays(TwoPair), 0);
    let j = Variant::JokerPoker;
    let joker: Vec<u32> = [RoyalFlush, FiveOfAKind, WildRoyal, StraightFlush, FourOfAKind, FullHouse, Flush, Straight, ThreeOfAKind, TwoPair, HighPair]
        .iter()
        .map(|h| j.pays(*h))
        .collect();
    assert_eq!(joker, [800, 200, 100, 50, 18, 7, 5, 3, 2, 1, 1]);
    assert_eq!(Variant::JacksOrBetter.deck().len(), 52);
    assert_eq!(Variant::DeucesWild.deck().len(), 52);
    assert_eq!(Variant::JokerPoker.deck().len(), 53);
}

#[test]
fn jacks_or_better_hands() {
    use Hand::*;
    let v = Variant::JacksOrBetter;
    assert_eq!(eval(v, "Th Jh Qh Kh Ah"), Some(RoyalFlush));
    assert_eq!(eval(v, "9h Th Jh Qh Kh"), Some(StraightFlush));
    assert_eq!(eval(v, "As 2h 3d 4c 5s"), Some(Straight), "roda");
    assert_eq!(eval(v, "Ts Jh Qd Kc As"), Some(Straight));
    assert_eq!(eval(v, "Qs Kh Ad 2c 3s"), None, "tidak melingkar");
    assert_eq!(eval(v, "7s 7h 7d 7c 2s"), Some(FourOfAKind));
    assert_eq!(eval(v, "7s 7h 7d 2c 2s"), Some(FullHouse));
    assert_eq!(eval(v, "2h 7h 9h Jh Kh"), Some(Flush));
    assert_eq!(eval(v, "7s 7h 7d 2c 3s"), Some(ThreeOfAKind));
    assert_eq!(eval(v, "7s 7h 3d 3c Ks"), Some(TwoPair));
    assert_eq!(eval(v, "Js Jh 4d 5c 9h"), Some(HighPair));
    assert_eq!(eval(v, "Ts Th 4d 5c 9h"), None, "sepuluh tidak cukup");
}

#[test]
fn deuces_are_wild() {
    use Hand::*;
    let v = Variant::DeucesWild;
    assert_eq!(eval(v, "Th Jh Qh Kh Ah"), Some(RoyalFlush));
    assert_eq!(eval(v, "2s 2h 2d 2c 9h"), Some(FourDeuces));
    assert_eq!(eval(v, "2s Jh Qh Kh Ah"), Some(WildRoyal));
    assert_eq!(eval(v, "2s 2h 7d 7c 7h"), Some(FiveOfAKind));
    assert_eq!(eval(v, "2s 5h 6h 7h 8h"), Some(StraightFlush));
    assert_eq!(eval(v, "2s 7h 7d 7c 9h"), Some(FourOfAKind));
    assert_eq!(eval(v, "2s 7h 7d 9c 9h"), Some(FullHouse));
    assert_eq!(eval(v, "2s 3h 7h 9h Jh"), Some(Flush));
    assert_eq!(eval(v, "2s 3h 4d 5c 6h"), Some(Straight));
    assert_eq!(eval(v, "2s 2h 4d 9c Kh"), Some(ThreeOfAKind));
    assert_eq!(eval(v, "2s 5h 7d 9c Kh"), None, "pasangan tidak dibayar");
    assert_eq!(eval(v, "7s 7h 3d 3c Ks"), None, "dua pasang tidak dibayar");
}

#[test]
fn the_joker_is_wild_kings_or_better() {
    use Hand::*;
    let v = Variant::JokerPoker;
    assert_eq!(eval(v, "Th Jh Qh Kh Ah"), Some(RoyalFlush));
    assert_eq!(eval(v, "JK Jh Qh Kh Ah"), Some(WildRoyal));
    assert_eq!(eval(v, "JK Ks Kh Kd Kc"), Some(FiveOfAKind));
    assert_eq!(eval(v, "JK 5h 6h 7h 8h"), Some(StraightFlush));
    assert_eq!(eval(v, "JK 7s 7h 7d 2c"), Some(FourOfAKind));
    assert_eq!(eval(v, "JK 7s 7h 2d 2c"), Some(FullHouse));
    assert_eq!(eval(v, "JK 7s 7h 4d 2c"), Some(ThreeOfAKind));
    assert_eq!(eval(v, "JK Ks 7h 4d 2c"), Some(HighPair));
    assert_eq!(eval(v, "JK As 7h 4d 2c"), Some(HighPair));
    assert_eq!(eval(v, "JK Qs 7h 4d 2c"), None, "pasangan Q di bawah K");
    assert_eq!(eval(v, "Ks Kh 7h 4d 2c"), Some(HighPair));
    assert_eq!(eval(v, "7s 7h 3d 3c Ks"), Some(TwoPair));
    assert_eq!(eval(v, "JK 3s 4h 5d 6c"), Some(Straight));
}

#[test]
fn exact_ev_and_optimal_holds() {
    let s = Solver::new(Variant::JacksOrBetter);
    let flush = hand("Ah Kh Qh Jh 2h");
    // Tahan semua: flush 6. Tahan empat ke royal: (800 + 7×6 + 3×4 + 12×1)/47.
    assert_eq!(s.ev(&flush, 0b11111), (6, 1));
    assert_eq!(s.ev(&flush, 0b01111), (866, 47));
    assert_eq!(s.best(&flush), 0b01111);
    assert_eq!(s.best(&hand("Js Jh 4d 7c 9h")), 0b00011, "pasangan jack");
    assert_eq!(s.best(&hand("Th Jh Qh Kh Ah")), 0b11111);
    let d = Solver::new(Variant::DeucesWild);
    assert_eq!(d.best(&hand("2s 2h 7c 9d Kh")), 0b00011, "tahan dua deuce");
    let j = Solver::new(Variant::JokerPoker);
    assert_eq!(j.best(&hand("JK Jh Qh Kh Ah")), 0b11111);
    // Nilai harapan tahan-semua = bayaran tangan itu.
    assert_eq!(j.ev(&hand("JK 7s 7h 4d 2c"), 0b11111), (2, 1));
}

#[test]
fn announced_rtp_is_below_100_and_near_the_source() {
    for (v, source) in [
        (Variant::JacksOrBetter, 99.54),
        (Variant::DeucesWild, 99.73),
        (Variant::JokerPoker, 98.93),
    ] {
        let list = v.rtp();
        assert_eq!(list.len(), 1);
        assert!(list[0].manifest);
        assert!(list[0].percent < 100.0);
        assert!((list[0].percent - source).abs() < 0.01, "{v:?}");
    }
}

fn game(cards: &[&str]) -> video_poker::JacksOrBetterGame {
    TurnGame::new(
        serde_json::from_value(json!({ "kartu": cards })).unwrap(),
        [0; 32],
    )
    .unwrap()
}

#[test]
fn deal_hold_draw() {
    let mut g = game(&["Js", "Jh", "4d", "7c", "9h", "Jd", "2c", "3s", "8h", "Kd"]);
    assert_eq!(Session::view_data(&g, 0)["fase"], "taruhan");
    assert!(Session::act(&mut g, 0, "bet 15").is_err());
    Session::act(&mut g, 0, "bet 20").unwrap();
    let v = Session::view_data(&g, 0);
    assert_eq!(v["fase"], "tahan");
    assert_eq!(v["kartu"], json!(["Js", "Jh", "4d", "7c", "9h"]));
    assert_eq!(v["tangan"], "high_pair", "tangan awal sudah dinilai");
    assert_eq!(v["taruhan_meja"], 20);
    assert_eq!(v["netral"], "draw 1 2 3 4 5");
    assert!(Session::act(&mut g, 0, "draw 6").is_err());
    assert!(Session::act(&mut g, 0, "draw 1 1").is_err());
    Session::act(&mut g, 0, "draw 1 2").unwrap();
    let v = Session::view_data(&g, 0);
    assert_eq!(v["fase"], "selesai");
    assert_eq!(v["kartu"], json!(["Js", "Jh", "Jd", "2c", "3s"]));
    assert_eq!(v["tahan"], json!([true, true, false, false, false]));
    assert_eq!(v["tangan"], "three_of_a_kind");
    assert_eq!(v["bayar"], 3 * 20 - 20);
    assert_eq!(v["bersih"], 40);
    assert_eq!(v["dipertaruhkan"], 20);
    assert!(TurnGame::is_over(&g));
    // Tanpa tahan: lima kartu baru; tahan semua: kartu tetap.
    let mut g = game(&["Js", "Jh", "4d", "7c", "9h", "Jd", "2c", "3s", "8h", "Kd"]);
    Session::act(&mut g, 0, "bet 10").unwrap();
    Session::act(&mut g, 0, "draw").unwrap();
    assert_eq!(Session::view_data(&g, 0)["kartu"], json!(["Jd", "2c", "3s", "8h", "Kd"]));
    assert_eq!(Session::view_data(&g, 0)["bayar"], -10);
}

#[test]
fn deals_come_from_the_round_seed() {
    let deal = |seed: u8| -> Value {
        let mut g = video_poker::JokerPokerGame::new(Default::default(), [seed; 32]).unwrap();
        Session::act(&mut g, 0, "bet 10").unwrap();
        Session::view_data(&g, 0)["kartu"].clone()
    };
    assert_eq!(deal(2), deal(2));
    assert_ne!(deal(2), deal(3));
    let g = game(&["Js", "Jh", "4d", "7c", "9h", "Jd", "2c", "3s", "8h", "Kd"]);
    for cmd in ["bet 10", "draw", "draw 1 3 5", "leave"] {
        let a = g.parse_command(cmd).unwrap();
        assert_eq!(g.format_action(&a), cmd);
    }
    // Kartu tetap harus sah untuk variannya (joker hanya di Joker Poker).
    let cfg = serde_json::from_value(json!({ "kartu": ["JK", "Jh", "4d", "7c", "9h"] })).unwrap();
    assert!(video_poker::JacksOrBetterGame::new(cfg, [0; 32]).is_err());
    let cfg = serde_json::from_value(json!({ "kartu": ["Js", "Js", "4d", "7c", "9h"] })).unwrap();
    assert!(video_poker::JacksOrBetterGame::new(cfg, [0; 32]).is_err());
    assert_eq!(video_poker::JacksOrBetterGame::ID, "jacks-or-better");
    assert_eq!(video_poker::DeucesWildGame::ID, "deuces-wild");
    assert_eq!(video_poker::JokerPokerGame::ID, "joker-poker");
}
