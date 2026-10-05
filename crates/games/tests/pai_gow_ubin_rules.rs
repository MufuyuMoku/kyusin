//! Tes aturan Pai Gow ubin (SPEC §6.4, §7; M6a, D-069), ditulis sebelum
//! mesinnya.
//!
//! 32 ubin Cina, empat per pemain, disusun menjadi tangan tinggi dan
//! rendah (dua ubin). Urutan: pasangan Gee Joon > pasangan (Teen, Day, Yun,
//! Gor, Mooy, Chong, Bon, Foo, Ping, Tit, Look, lalu pasangan campuran 9,
//! 8, 7, 5) > Wong (Teen/Day + 9) > Gong (Teen/Day + 8) > nilai 9 … 0
//! (jumlah bulatan mod 10; ubin Gee Joon boleh 3 atau 6). Nilai sama:
//! ubin tertinggi; masih sama, atau sama-sama 0, dimenangkan bandar (copy).
//! Menang kedua tangan dibayar 1:1 dipotong komisi 5%, satu-satu push.

use std::cmp::Ordering;

use kyusin_core::{Session, TurnGame};
use kyusin_games::domino::Tile;
use kyusin_games::pai_gow_ubin::{HandKind, PaiGowUbin, RTP, compare, hand, house_way, set};
use serde_json::{Value, json};

fn t(s: &str) -> Tile {
    s.parse().unwrap()
}

fn h(a: &str, b: &str) -> kyusin_games::pai_gow_ubin::Hand {
    hand(t(a), t(b))
}

#[test]
fn the_chinese_set_has_32_tiles() {
    let tiles = set();
    assert_eq!(tiles.len(), 32);
    assert!(tiles.iter().all(|x| x.lo >= 1), "tanpa sisi kosong");
    let count = |s: &str| tiles.iter().filter(|x| **x == t(s)).count();
    for civil in ["6-6", "1-1", "4-4", "3-1", "5-5", "3-3", "2-2", "6-5", "6-4", "6-1", "5-1"] {
        assert_eq!(count(civil), 2, "{civil}");
    }
    for military in ["6-3", "5-4", "6-2", "5-3", "5-2", "4-3", "4-1", "3-2", "2-1", "4-2"] {
        assert_eq!(count(military), 1, "{military}");
    }
}

#[test]
fn hand_ranking() {
    let order = [
        (h("6-5", "4-1"), HandKind::Points), // 16 → 6
        (h("6-5", "5-3"), HandKind::Points), // 19 → 9
        (h("1-1", "5-3"), HandKind::Gong),
        (h("6-6", "5-3"), HandKind::Gong),
        (h("1-1", "6-3"), HandKind::Wong),
        (h("6-6", "5-4"), HandKind::Wong),
        (h("4-1", "3-2"), HandKind::Pair), // 5 campuran
        (h("5-2", "4-3"), HandKind::Pair),
        (h("6-2", "5-3"), HandKind::Pair),
        (h("6-3", "5-4"), HandKind::Pair),
        (h("5-1", "5-1"), HandKind::Pair), // Look
        (h("6-1", "6-1"), HandKind::Pair),
        (h("6-4", "6-4"), HandKind::Pair),
        (h("6-5", "6-5"), HandKind::Pair),
        (h("2-2", "2-2"), HandKind::Pair),
        (h("3-3", "3-3"), HandKind::Pair),
        (h("5-5", "5-5"), HandKind::Pair),
        (h("3-1", "3-1"), HandKind::Pair),
        (h("4-4", "4-4"), HandKind::Pair),
        (h("1-1", "1-1"), HandKind::Pair),
        (h("6-6", "6-6"), HandKind::Pair),
        (h("2-1", "4-2"), HandKind::GeeJoon),
    ];
    for w in order.windows(2) {
        assert_eq!(w[0].0.kind(), w[0].1, "{:?}", w[0].0);
        assert_eq!(compare(&w[1].0, &w[0].0), Ordering::Greater, "{:?} > {:?}", w[1].0, w[0].0);
    }
    assert_eq!(h("6-5", "4-1").points(), 6);
    // High nine (Teen + 7) adalah 9 dengan ubin tertinggi Teen.
    assert_eq!(h("6-6", "4-3").points(), 9);
    assert_eq!(compare(&h("6-6", "4-3"), &h("6-1", "1-1")), Ordering::Greater, "Teen > Day");
    // Gee Joon dihitung 3 atau 6, mana yang lebih baik.
    assert_eq!(h("2-1", "6-6").points(), 8);
    assert_eq!(h("4-2", "6-5").points(), 7);
}

#[test]
fn copies_and_zero_go_to_the_dealer() {
    // Sama-sama 7 dengan ubin tertinggi 9 (6-3 dan 5-4 sepangkat): copy.
    assert_eq!(compare(&h("5-4", "6-2"), &h("6-3", "5-3")), Ordering::Equal);
    // Sama-sama 0: bandar, apa pun ubinnya.
    assert_eq!(h("6-4", "5-5").points(), 0);
    assert_eq!(h("6-3", "6-5").points(), 0);
    assert_eq!(compare(&h("6-4", "5-5"), &h("6-3", "6-5")), Ordering::Equal);
    // Nilai sama, ubin tertinggi beda: yang lebih tinggi menang.
    assert_eq!(compare(&h("6-6", "6-5"), &h("1-1", "6-5")), Ordering::Greater, "3 dengan Teen > 3 dengan Day");
}

fn hw(tiles: [&str; 4]) -> (Vec<String>, Vec<String>) {
    let (hi, lo) = house_way([t(tiles[0]), t(tiles[1]), t(tiles[2]), t(tiles[3])]);
    let mut a: Vec<String> = hi.iter().map(Tile::to_string).collect();
    let mut b: Vec<String> = lo.iter().map(Tile::to_string).collect();
    a.sort();
    b.sort();
    (a, b)
}

#[test]
fn house_way() {
    // Pasangan Teen dipertahankan bila memecahnya membuat dua tangan lebih rendah.
    assert_eq!(hw(["6-6", "6-6", "2-1", "4-1"]), (vec!["6-6".into(), "6-6".into()], vec!["2-1".into(), "4-1".into()]));
    // Teen dipecah bila hasilnya dua Wong (lebih baik dari pasangan + 8).
    assert_eq!(hw(["6-6", "6-6", "5-4", "6-3"]).1.len(), 2);
    let (a, b) = hw(["6-6", "6-6", "5-4", "6-3"]);
    assert!(a.contains(&"6-6".to_string()) && b.contains(&"6-6".to_string()), "{a:?} {b:?}");
    // Pasangan 9 campuran hanya dipecah bila hasilnya 9-9.
    assert_eq!(hw(["6-3", "5-4", "6-6", "2-2"]), (vec!["5-4".into(), "6-3".into()], vec!["2-2".into(), "6-6".into()]));
    // Pasangan 10 (Ping) tidak pernah dipecah.
    assert_eq!(hw(["6-4", "6-4", "6-6", "1-1"]).0, vec!["6-4".to_string(), "6-4".to_string()]);
    // Tanpa pasangan: Gong lebih dulu bila tangan rendahnya paling sedikit 4.
    assert_eq!(hw(["6-6", "5-3", "6-5", "4-1"]), (vec!["5-3".into(), "6-6".into()], vec!["4-1".into(), "6-5".into()]));
    // Selain itu tangan rendah sebesar mungkin: 6/6; tangan tinggi = yang
    // ubin tertingginya lebih tinggi (Foo > Ping).
    assert_eq!(hw(["6-5", "6-4", "5-1", "3-2"]), (vec!["3-2".into(), "6-5".into()], vec!["5-1".into(), "6-4".into()]));
}

fn game(tiles: &[&str]) -> PaiGowUbin {
    PaiGowUbin::new(serde_json::from_value(json!({ "ubin": tiles })).unwrap(), [0; 32]).unwrap()
}

fn act(g: &mut PaiGowUbin, cmd: &str) {
    Session::act(g, 0, cmd).unwrap_or_else(|e| panic!("`{cmd}`: {e:?}"));
}

#[test]
fn a_round_with_commission_push_and_copy() {
    // Pemain: pasangan Teen / 9+8 = 7; bandar (tanpa pasangan) house way 6/4.
    let mut g = game(&["6-6", "6-6", "5-4", "6-2", "6-5", "4-1", "5-2", "6-1"]);
    assert_eq!(Session::view_data(&g, 0)["fase"], "taruhan");
    assert!(Session::act(&mut g, 0, "bet 30").is_err(), "kelipatan 20");
    act(&mut g, "bet 100");
    let v = Session::view_data(&g, 0);
    assert_eq!(v["fase"], "susun");
    assert_eq!(v["pemain"].as_array().unwrap().len(), 4);
    assert_eq!(v["bandar"], json!(["??", "??", "??", "??"]));
    assert_eq!(v["netral"], "houseway");
    assert!(Session::act(&mut g, 0, "set 1-1 2-2").is_err(), "bukan ubinmu");
    act(&mut g, "set 5-4 6-2");
    let v = Session::view_data(&g, 0);
    assert_eq!(v["fase"], "selesai");
    assert_eq!(v["hasil"], json!(["menang", "menang"]));
    assert_eq!(v["bersih"], 95, "1:1 dipotong 5%");
    assert_eq!(v["bandar"].as_array().unwrap().len(), 4);

    // Satu menang satu kalah: push. Pemain: pasangan Teen / 6+5 = 1;
    // bandar: Day + 6-5 = 3 dan 5-3 + 6-4 = 8 (Gong ditolak karena tangan
    // rendahnya 1).
    let mut g = game(&["6-6", "6-6", "5-1", "4-1", "1-1", "5-3", "6-5", "6-4"]);
    act(&mut g, "bet 100");
    act(&mut g, "set 5-1 4-1");
    let v = Session::view_data(&g, 0);
    assert_eq!(v["hasil"], json!(["menang", "kalah"]));
    assert_eq!(v["bersih"], 0);

    // Copy dimenangkan bandar: tangan rendah pemain 5-4 + 6-2 = 7 (ubin
    // tertinggi 9) lawan 6-3 + 5-3 = 7 (ubin tertinggi 9).
    let mut g = game(&["5-4", "6-2", "6-1", "6-1", "6-3", "5-3", "5-5", "5-5"]);
    act(&mut g, "bet 100");
    act(&mut g, "set 5-4 6-2");
    let v = Session::view_data(&g, 0);
    assert_eq!(v["hasil"], json!(["kalah", "kalah"]), "Tit < Mooy; 7 lawan 7: copy");
    assert_eq!(v["bersih"], -100);
}

#[test]
fn announced_rtp_is_below_100() {
    assert_eq!(RTP.iter().filter(|r| r.manifest).count(), 1);
    for r in RTP {
        assert!(r.percent < 100.0 && r.percent > 95.0, "{}: {}", r.wager, r.percent);
    }
}

#[test]
fn rounds_come_from_the_round_seed() {
    let deal = |seed: u8| {
        let mut g = PaiGowUbin::new(Default::default(), [seed; 32]).unwrap();
        Session::act(&mut g, 0, "bet 20").unwrap();
        Session::view_data(&g, 0)["pemain"].clone()
    };
    assert_eq!(deal(3), deal(3));
    assert_ne!(deal(3), deal(4));
    let g = game(&["6-6", "6-6", "5-4", "2-2", "6-5", "4-1", "5-2", "3-2"]);
    for cmd in ["bet 20", "set 6-6 5-4", "houseway", "leave"] {
        let a = g.parse_command(cmd).unwrap();
        assert_eq!(g.format_action(&a), cmd);
    }
    let _: Value = Session::view_data(&g, 0);
    assert!(!TurnGame::is_over(&g));
}
