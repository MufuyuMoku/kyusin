//! Tes aturan Capsa Susun (SPEC §6.3, §7; M5b-2), ditulis sebelum mesinnya.
//! Aturan lokal dipilih klien (D-065):
//!
//! - 2–4 kursi, 13 kartu per kursi, disusun depan 3 / tengah 5 /
//!   belakang 5 dengan depan ≤ tengah ≤ belakang (salah susun ditolak);
//!   baris depan hanya kartu tinggi, pair, atau three of a kind;
//!   A-2-3-4-5 straight terendah;
//! - setiap pasangan pemain dibandingkan per baris: 1 poin per baris,
//!   seri persis 0; menang ketiga baris (sapu bersih) = poin baris dikali
//!   dua; royalti untuk baris yang menang: trips depan 3, full house tengah
//!   2, four of a kind tengah 8 / belakang 4, straight flush tengah 10 /
//!   belakang 5;
//! - kartu istimewa (otomatis, tanpa disusun): Naga 13 poin; enam pasang,
//!   tiga flush, tiga straight masing-masing 3 poin; urutan Naga > enam
//!   pasang > tiga flush > tiga straight, jenis sama seri;
//! - 1 poin = 10 chip; yang kalah membayar paling banyak tumpukannya, dan
//!   kekurangannya mengurangi bagian para pemenang secara proporsional.

use kyusin_core::{GameRng, Lang, Session, TurnGame};
use kyusin_games::capsa_susun::{
    Capsa, Config, Hand, POINT, Special, front_value, row_value, settle, special, suggest, valid,
    versus,
};
use kyusin_games::cards::Card;
use serde_json::{Value, json};

fn cards(text: &str) -> Vec<Card> {
    text.split_whitespace().map(|c| c.parse().unwrap()).collect()
}

fn hand(front: &str, mid: &str, back: &str) -> Hand {
    Hand::Normal {
        front: cards(front),
        mid: cards(mid),
        back: cards(back),
    }
}

#[test]
fn rows_compare_across_three_and_five_cards() {
    // Depan tidak mengenal flush atau straight.
    assert_eq!(front_value(&cards("Ah Kh Qh")).key(), "high_card");
    assert_eq!(front_value(&cards("Qs Qh Qd")).key(), "trips");
    // Pair 8 dengan kicker K di depan ≤ pair 8 K 5 3 di tengah.
    assert!(valid(&cards("8s 8h Kd"), &cards("8c 8d Kc 5s 3h"), &cards("Ah Ad 2c 2d 9s")));
    // Pair 8 kicker A di depan > pair 8 kicker K di tengah: salah susun.
    assert!(!valid(&cards("8s 8h Ad"), &cards("8c 8d Kc 5s 3h"), &cards("Ah Kd 2c 2d 9s")));
    // Trips di depan > two pair di tengah: salah susun.
    assert!(!valid(&cards("Qs Qh Qd"), &cards("8c 8d Kc Ks 3h"), &cards("Ah Ad Ac 2d 2s")));
    // Tengah lebih kuat dari belakang: salah susun.
    assert!(!valid(&cards("2s 3h 4d"), &cards("5h 6h 7h 8h 9h"), &cards("Ac Jc 8c 6c 5c")));
    // Wheel = straight terendah.
    assert!(row_value(&cards("Ah 2d 3c 4s 5h")) < row_value(&cards("2h 3d 4c 5s 6h")));
    assert!(row_value(&cards("Ah 2d 3c 4s 5h")) > row_value(&cards("Ah Ad Ac 2d 9s")));
}

#[test]
fn rows_scoop_and_royalties() {
    // a: trips depan, full house tengah, quads belakang.
    // b: kartu tinggi depan, flush tengah, straight flush belakang.
    let a = hand("Qs Qh Qd", "9s 9h 9d 4c 4h", "2c 2d 2h 2s 7c");
    let b = hand("Ah Kd 3c", "Ac Jc 8c 6c 5c", "5h 6h 7h 8h 9h");
    // depan +1 +3, tengah +1 +2, belakang -1 -5 (royalti straight flush b).
    assert_eq!(versus(&a, &b), 1);
    assert_eq!(versus(&b, &a), -1);
    // Sapu bersih: 3 × 2 = 6, royalti tidak dikali.
    let c = hand("2s 3h 5d", "4s 4h 7d 8c 9h", "Ts Th Td 3c 3d");
    let d = hand("2c 3c 6d", "5s 5h 7c 8d 9d", "Js Jh Jd 4c 4h");
    assert_eq!(versus(&d, &c), 6);
    assert_eq!(versus(&c, &d), -6);
    let e = hand("Js Jh Jd", "5s 5h 5d 8d 8h", "Ks Kh Kd Kc 3s");
    let f = hand("2c 3c 4d", "6s 6h 7c 8c 9d", "Ts Th 7d 4c 4h");
    assert_eq!(versus(&e, &f), 6 + 3 + 2 + 4);
    // Seri persis di satu baris = 0 dan bukan sapu bersih.
    let g = hand("2s 3h 4d", "5s 5h 7d 8c 9h", "Ts Th Td 3c 3d");
    let h = hand("2c 3d 4h", "4s 4c 7c 8d 9d", "Js Jh Jd 5c 5d");
    assert_eq!(versus(&h, &g), 0 + -1 + 1);
    let i = hand("2c 3c 4h", "6s 6h 7c 8d 9d", "Js Jh Jd 4c 4d");
    assert_eq!(versus(&i, &g), 2, "seri depan, menang tengah dan belakang");
}

#[test]
fn special_hands() {
    let naga = cards("2c 3d 4h 5s 6c 7d 8h 9s Tc Jd Qh Ks Ac");
    let pairs = cards("2c 2d 3h 3s 4c 4d 5h 5s 6c 6d 7h 7s Ac");
    let flushes = cards("2h 5h 9h 3s 6s 8s Ts Qs 4c 7c 9c Jc Kc");
    let straights = cards("3h 4d 5c 3s 4h 5d 6c 7s 8h 9d Tc Jd Qs");
    let plain = cards("Qs Qh Qd 9s 9h 9d 4c 4h 2c 2d 2h 2s 7c");
    assert_eq!(special(&naga), Some(Special::Naga));
    assert_eq!(special(&pairs), Some(Special::EnamPasang));
    assert_eq!(special(&flushes), Some(Special::TigaFlush));
    assert_eq!(special(&straights), Some(Special::TigaStraight));
    assert_eq!(special(&plain), None);
    // Four of a kind dihitung dua pasang: 2222 3333 44 55 + 3 kartu.
    assert_eq!(
        special(&cards("2c 2d 2h 2s 3c 3d 3h 3s 4c 4d 5h 5s Ac")),
        Some(Special::EnamPasang)
    );
    assert!(Special::Naga > Special::EnamPasang);
    assert!(Special::EnamPasang > Special::TigaFlush);
    assert!(Special::TigaFlush > Special::TigaStraight);
    assert_eq!(Special::Naga.points(), 13);
    assert_eq!(Special::TigaStraight.points(), 3);

    let normal = hand("Qs Qh Qd", "9s 9h 9d 4c 4h", "2c 2d 2h 2s 7c");
    assert_eq!(versus(&Hand::Special(Special::Naga), &normal), 13);
    assert_eq!(versus(&normal, &Hand::Special(Special::TigaFlush)), -3);
    assert_eq!(
        versus(&Hand::Special(Special::Naga), &Hand::Special(Special::EnamPasang)),
        13
    );
    assert_eq!(
        versus(&Hand::Special(Special::TigaFlush), &Hand::Special(Special::TigaStraight)),
        3
    );
    assert_eq!(
        versus(&Hand::Special(Special::TigaFlush), &Hand::Special(Special::TigaFlush)),
        0
    );
}

#[test]
fn settlement_is_zero_sum_and_capped_by_stacks() {
    assert_eq!(POINT, 10);
    // p[i][j] = poin yang didapat i dari j.
    let p = vec![vec![0, 6, -2], vec![-6, 0, 3], vec![2, -3, 0]];
    assert_eq!(settle(&p, &[2000, 2000, 2000]), vec![40, -30, -10]);
    // Kursi 1 hanya punya 20: yang dibayar 20 + 10, semuanya ke kursi 0.
    assert_eq!(settle(&p, &[2000, 20, 2000]), vec![30, -20, -10]);
    // Dua pemenang (klaim 30 dan 10) dari satu yang kalah dengan 20 chip:
    // dibagi proporsional 15 dan 5.
    let q = vec![vec![0, 0, 3], vec![0, 0, 1], vec![-3, -1, 0]];
    assert_eq!(settle(&q, &[2000, 2000, 20]), vec![15, 5, -20]);
    // Sisa pembagian (bulat ke bawah) ke klaim terbesar lebih dulu.
    let r = vec![vec![0, 0, 2], vec![0, 0, 1], vec![-2, -1, 0]];
    assert_eq!(settle(&r, &[2000, 2000, 10]), vec![7, 3, -10]);
}

#[test]
fn the_suggested_arrangement_is_always_valid() {
    let mut rng = GameRng::from_seed([3; 32]);
    for _ in 0..300 {
        let mut deck = Card::deck();
        for i in (1..deck.len()).rev() {
            deck.swap(i, rng.below(i as u32 + 1) as usize);
        }
        let h: Vec<Card> = deck[..13].to_vec();
        let s = suggest(&h);
        assert!(valid(&s[..3], &s[3..8], &s[8..]), "{s:?}");
        let mut a = s.clone();
        let mut b = h.clone();
        a.sort();
        b.sort();
        assert_eq!(a, b, "memakai ketiga belas kartu");
    }
}

/// Dek tetap untuk dua kursi, dealer 0: kartu dibagi bergantian mulai
/// kursi 1.
fn deal2(seat0: &str, seat1: &str) -> Vec<String> {
    let (a, b) = (cards(seat0), cards(seat1));
    b.iter()
        .zip(a.iter())
        .flat_map(|(x, y)| [x.to_string(), y.to_string()])
        .collect()
}

const SEAT0: &str = "Qs Qh Qd 9s 9h 9d 4c 4h 2c 2d 2h 2s 7c";
const SEAT1: &str = "Ah Kd 3c 5h 6h 7h 8h 9c Ac Jc 8c 6c 5c";

fn capsa(seats: u8, deck: Vec<String>) -> Capsa {
    Capsa::new(
        Config {
            kursi: Some(seats),
            dealer: Some(0),
            dek: Some(vec![deck]),
            ..Config::default()
        },
        [0; 32],
    )
    .unwrap()
}

fn act(g: &mut Capsa, seat: u8, cmd: &str) {
    Session::act(g, seat, cmd).unwrap_or_else(|e| panic!("kursi {seat} `{cmd}`: {e:?}"));
}

fn view(g: &Capsa, seat: u8) -> Value {
    Session::view_data(g, seat)
}

#[test]
fn arranging_scoring_and_the_session() {
    let mut g = capsa(2, deal2(SEAT0, SEAT1));
    let v = view(&g, 0);
    assert_eq!(v["fase"], "susun");
    assert_eq!(v["taruhan_meja"], 2000);
    assert_eq!(v["poin_chip"], 10);
    let mut mine: Vec<String> = v["kursi"][0]["kartu"]
        .as_array()
        .unwrap()
        .iter()
        .map(|c| c.as_str().unwrap().to_string())
        .collect();
    mine.sort();
    let mut want: Vec<String> = SEAT0.split(' ').map(String::from).collect();
    want.sort();
    assert_eq!(mine, want);
    assert_eq!(v["kursi"][1]["kartu"], json!(vec!["??"; 13]));
    assert_eq!(v["saran"].as_array().unwrap().len(), 13);
    let mut p = TurnGame::pending_players(&g);
    p.sort();
    assert_eq!(p, vec![0, 1], "susun serentak");
    assert_eq!(v["netral"], "auto");

    // Salah susun, kartu orang lain, dan jumlah kartu salah ditolak.
    assert!(Session::act(&mut g, 0, "arrange Qs Qh Qd 2c 2d 2h 2s 7c 9s 9h 9d 4c 4h").is_err());
    assert!(Session::act(&mut g, 0, "arrange Ah Qh Qd 9s 9h 9d 4c 4h 2c 2d 2h 2s 7c").is_err());
    assert!(Session::act(&mut g, 0, "arrange Qs Qh Qd").is_err());

    act(&mut g, 1, "arrange Ah Kd 3c 5h 6h 7h 8h 9c Ac Jc 8c 6c 5c");
    assert_eq!(TurnGame::pending_players(&g), vec![0]);
    assert_eq!(view(&g, 0)["kursi"][1]["siap"], true);
    act(&mut g, 0, "arrange Qs Qh Qd 9s 9h 9d 4c 4h 2c 2d 2h 2s 7c");
    let v = view(&g, 0);
    assert_eq!(v["fase"], "antara");
    // Sapu bersih 6 + royalti 3 + 2 + 4 = 15 poin = 150 chip.
    assert_eq!(v["kursi"][0]["poin"], 15);
    assert_eq!(v["kursi"][1]["poin"], -15);
    assert_eq!(v["kursi"][0]["tumpukan"], 2150);
    assert_eq!(v["kursi"][1]["tumpukan"], 1850);
    assert_eq!(
        v["kursi"][1]["baris"],
        json!([["Ah", "Kd", "3c"], ["5h", "6h", "7h", "8h", "9c"], ["Ac", "Jc", "8c", "6c", "5c"]])
    );
    assert_eq!(v["kursi"][0]["nama_baris"], json!(["trips", "full_house", "quads"]));
    assert_eq!(v["netral"], "leave");
    act(&mut g, 1, "next");
    act(&mut g, 0, "leave");
    let v = view(&g, 0);
    assert_eq!(v["fase"], "selesai");
    assert_eq!(v["bersih"], 150);
    assert_eq!(TurnGame::result(&g).unwrap().scores.iter().sum::<i64>(), 0);
    for cmd in ["auto", "next", "leave", "arrange Qs Qh Qd 9s 9h 9d 4c 4h 2c 2d 2h 2s 7c"] {
        let a = g.parse_command(cmd).unwrap();
        assert_eq!(g.format_action(&a), cmd);
    }
    assert!(Session::view_text(&g, 0, Lang::Id).contains("Qs"));
}

#[test]
fn special_hands_are_declared_automatically() {
    let naga = "2c 3d 4h 5s 6c 7d 8h 9s Tc Jd Qh Ks Ac";
    let mut g = capsa(2, deal2("2d 2h 2s 3c 3h 3s 4c 4d 4s 5c 5d 5h 6d", naga));
    // Kursi 1 punya Naga: tidak perlu menyusun.
    assert_eq!(TurnGame::pending_players(&g), vec![0]);
    assert_eq!(view(&g, 0)["kursi"][1]["istimewa"], "naga");
    act(&mut g, 0, "auto");
    let v = view(&g, 0);
    assert_eq!(v["kursi"][1]["poin"], 13);
    assert_eq!(v["kursi"][0]["tumpukan"], 2000 - 130);
}

#[test]
fn hands_are_dealt_from_the_session_seed() {
    let mk = |seed: u8| Capsa::new(Config::default(), [seed; 32]).unwrap();
    let (a, b, c) = (mk(1), mk(1), mk(2));
    assert_eq!(view(&a, 0)["kursi"][0]["kartu"], view(&b, 0)["kursi"][0]["kartu"]);
    assert_ne!(view(&a, 0)["kursi"][0]["kartu"], view(&c, 0)["kursi"][0]["kartu"]);
    assert_eq!(view(&a, 0)["kursi"].as_array().unwrap().len(), 4, "bawaan 4 kursi");
    assert!(Capsa::new(Config { kursi: Some(5), ..Config::default() }, [0; 32]).is_err());
}

/// Properti: total chip meja tidak pernah berubah (SPEC §7 poin 1), juga
/// saat ada kursi yang habis.
#[test]
fn chips_are_conserved_under_random_play() {
    for seed in 0..40u8 {
        let mut rng = GameRng::from_seed([seed; 32]);
        let n = 2 + seed % 3;
        let stacks: Vec<i64> = (0..n).map(|s| 30 + 300 * i64::from(s)).collect();
        let total: i64 = stacks.iter().sum();
        let mut g = Capsa::new(
            Config {
                kursi: Some(n),
                tumpukan: Some(stacks),
                tangan_maks: Some(15),
                ..Config::default()
            },
            [seed; 32],
        )
        .unwrap();
        let mut steps = 0;
        while !TurnGame::is_over(&g) && steps < 2000 {
            let seat = TurnGame::pending_players(&g)[0];
            let v = view(&g, seat);
            let cmd = if v["fase"] == "susun" {
                if rng.below(2) == 0 {
                    "auto".to_string()
                } else {
                    let s: Vec<&str> = v["saran"].as_array().unwrap().iter().map(|c| c.as_str().unwrap()).collect();
                    format!("arrange {}", s.join(" "))
                }
            } else {
                "next".to_string()
            };
            act(&mut g, seat, &cmd);
            let on_table: i64 = view(&g, 0)["kursi"]
                .as_array()
                .unwrap()
                .iter()
                .map(|k| k["tumpukan"].as_i64().unwrap())
                .sum();
            assert_eq!(on_table, total, "sesi {seed} langkah {steps}");
            steps += 1;
        }
        assert!(TurnGame::is_over(&g), "sesi {seed}");
        assert_eq!(TurnGame::result(&g).unwrap().scores.iter().sum::<i64>(), 0);
    }
}
