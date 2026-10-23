//! Tes aturan Bingo melawan bandar (SPEC §6.5, §7; M6b, D-071), ditulis
//! sebelum mesinnya.
//!
//! 75 bola, kartu 5×5 (B 1–15, I 16–30, N 31–45 dengan petak bebas di
//! tengah, G 46–60, O 61–75), 1–4 kartu per ronde, 40 bola ditarik. Pola
//! tertinggi yang terpenuhi dibayar: 1 garis 1,5 / 4 sudut 2 / 2 garis 3 /
//! 3 garis 4 / X 6 / bingkai 500 / penuh 5.000.

use kyusin_core::{Session, TurnGame};
use kyusin_games::bingo::{self, Bingo, Pattern, RTP, best, exact_rtp, valid_card};
use serde_json::{Value, json};

/// Kartu contoh, baris demi baris; 0 = petak bebas.
const CARD: [u8; 25] = [
    1, 16, 31, 46, 61, //
    2, 17, 32, 47, 62, //
    3, 18, 0, 48, 63, //
    4, 19, 34, 49, 64, //
    5, 20, 35, 50, 65,
];

const OTHER: [u8; 25] = [
    6, 21, 36, 51, 66, //
    7, 22, 37, 52, 67, //
    8, 23, 0, 53, 68, //
    9, 24, 39, 54, 69, //
    10, 25, 40, 55, 70,
];

fn cells(idx: &[usize]) -> Vec<u8> {
    idx.iter().map(|&i| CARD[i]).filter(|&n| n != 0).collect()
}

#[test]
fn patterns_pay_the_highest_one() {
    assert!(valid_card(&CARD));
    assert!(valid_card(&OTHER));
    assert_eq!(best(&CARD, &[]), None);
    assert_eq!(best(&CARD, &cells(&[0, 1, 2, 3, 4])), Some(Pattern::Line));
    // Kolom N memakai petak bebas: cukup empat angka.
    assert_eq!(best(&CARD, &cells(&[2, 7, 17, 22])), Some(Pattern::Line));
    assert_eq!(best(&CARD, &cells(&[0, 4, 20, 24])), Some(Pattern::Corners));
    assert_eq!(
        best(&CARD, &cells(&[0, 1, 2, 3, 4, 5, 6, 7, 8, 9])),
        Some(Pattern::TwoLines)
    );
    let x = [0, 6, 12, 18, 24, 4, 8, 16, 20];
    assert_eq!(best(&CARD, &cells(&x)), Some(Pattern::X));
    // X ditambah garis ketiga tetap X (6× > 4×).
    let mut x_row = x.to_vec();
    x_row.extend([1, 2, 3]);
    assert_eq!(best(&CARD, &cells(&x_row)), Some(Pattern::X));
    let three = [0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 13, 14];
    assert_eq!(best(&CARD, &cells(&three)), Some(Pattern::ThreeLines));
    let frame: Vec<usize> = (0..25).filter(|c| c / 5 == 0 || c / 5 == 4 || c % 5 == 0 || c % 5 == 4).collect();
    assert_eq!(best(&CARD, &cells(&frame)), Some(Pattern::Frame));
    assert_eq!(best(&CARD, &cells(&(0..25).collect::<Vec<_>>())), Some(Pattern::Full));
    let pays: Vec<u32> = [
        Pattern::Line,
        Pattern::Corners,
        Pattern::TwoLines,
        Pattern::ThreeLines,
        Pattern::X,
        Pattern::Frame,
        Pattern::Full,
    ]
    .iter()
    .map(|p| p.pays_tenths())
    .collect();
    assert_eq!(pays, [15, 20, 30, 40, 60, 5000, 50000]);
}

#[test]
fn exact_rtp_by_enumeration() {
    let r = exact_rtp();
    assert!((r - 95.434162).abs() < 0.000001, "{r}");
    assert_eq!(RTP.len(), 1);
    assert!(RTP[0].manifest);
    assert!((RTP[0].percent - r).abs() < 0.00006);
    assert_eq!(bingo::BALLS, 75);
    assert_eq!(bingo::DRAWN, 40);
    assert_eq!(bingo::MAX_CARDS, 4);
}

fn game(cards: &[[u8; 25]], balls: &[u8]) -> Bingo {
    Bingo::new(
        serde_json::from_value(json!({ "kartu": cards, "bola": balls })).unwrap(),
        [0; 32],
    )
    .unwrap()
}

fn act(g: &mut Bingo, cmd: &str) {
    Session::act(g, 0, cmd).unwrap_or_else(|e| panic!("`{cmd}`: {e:?}"));
}

/// 40 bola: X di kartu contoh (8 angka) + 32 angka yang tidak ada di kedua
/// kartu (11–15, 26–30, 41–45, 56–60, 71–75, dan 33, 38, 57… secukupnya).
fn balls() -> Vec<u8> {
    let mut v = cells(&[0, 6, 12, 18, 24, 4, 8, 16, 20]);
    for n in [11, 12, 13, 14, 15, 26, 27, 28, 29, 30, 41, 42, 43, 44, 45, 56, 57, 58, 59, 60, 71, 72, 73, 74, 75, 33, 38] {
        v.push(n);
    }
    // Lengkapi dengan angka kartu kedua yang tidak membentuk pola: satu per baris.
    for n in [6, 22, 53, 69, 25] {
        v.push(n);
    }
    assert_eq!(v.len(), 40);
    v
}

#[test]
fn a_round_with_two_cards() {
    let mut g = game(&[CARD, OTHER, CARD, OTHER], &balls());
    let v = Session::view_data(&g, 0);
    assert_eq!(v["fase"], "taruhan");
    assert_eq!(v["jumlah_kartu"], 1);
    // Kartu sudah terlihat sebelum bertaruh (berbeda turunan seed dari bola).
    assert_eq!(v["kartu"].as_array().unwrap().len(), 4);
    assert!(Session::act(&mut g, 0, "cards 5").is_err());
    assert!(Session::act(&mut g, 0, "cards 0").is_err());
    act(&mut g, "cards 2");
    let v = Session::view_data(&g, 0);
    assert_eq!(v["jumlah_kartu"], 2);
    assert_eq!(v["pengali"]["bet"], 2, "taruhan per kartu dikali jumlah kartu");
    act(&mut g, "bet 10");
    let v = Session::view_data(&g, 0);
    assert_eq!(v["fase"], "selesai");
    assert_eq!(v["bola"].as_array().unwrap().len(), 40);
    assert_eq!(v["pola"], json!(["x", null]));
    // Kartu 1: X 6× dari 10 = 60 (+50); kartu 2 kalah −10.
    assert_eq!(v["bayar_kartu"], json!([50, -10]));
    assert_eq!(v["bayar"], 40);
    assert_eq!(v["bersih"], 40);
    assert_eq!(v["dipertaruhkan"], 20);
    assert!(TurnGame::is_over(&g));
}

#[test]
fn config_and_commands_are_checked() {
    let mut bad = CARD;
    bad[0] = 16; // kolom B harus 1–15
    assert!(!valid_card(&bad));
    let mut dup = CARD;
    dup[5] = 1;
    assert!(!valid_card(&dup));
    let mut no_free = CARD;
    no_free[12] = 33;
    assert!(!valid_card(&no_free));
    let cfg = serde_json::from_value(json!({ "kartu": [bad], "bola": balls() })).unwrap();
    assert!(Bingo::new(cfg, [0; 32]).is_err());
    let cfg = serde_json::from_value(json!({ "bola": [1, 2, 3] })).unwrap();
    assert!(Bingo::new(cfg, [0; 32]).is_err());
    let g = game(&[CARD], &balls());
    for cmd in ["cards 3", "bet 10", "leave"] {
        let a = g.parse_command(cmd).unwrap();
        assert_eq!(g.format_action(&a), cmd);
    }
    let mut g = game(&[CARD], &balls());
    assert!(Session::act(&mut g, 0, "bet 15").is_err());
    act(&mut g, "leave");
    assert_eq!(Session::view_data(&g, 0)["alasan"], "berhenti");
}

#[test]
fn cards_and_balls_come_from_the_round_seed() {
    let round = |seed: u8| -> (Value, Value) {
        let mut g = Bingo::new(Default::default(), [seed; 32]).unwrap();
        let cards = Session::view_data(&g, 0)["kartu"].clone();
        Session::act(&mut g, 0, "bet 10").unwrap();
        (cards, Session::view_data(&g, 0)["bola"].clone())
    };
    let (c, b) = round(4);
    assert_eq!((c.clone(), b.clone()), round(4));
    assert_ne!(c, round(5).0);
    for card in c.as_array().unwrap() {
        let v: Vec<u8> = card.as_array().unwrap().iter().map(|x| x.as_u64().unwrap() as u8).collect();
        assert!(valid_card(&v.try_into().unwrap()));
    }
    let mut nums: Vec<u64> = b.as_array().unwrap().iter().map(|x| x.as_u64().unwrap()).collect();
    nums.sort_unstable();
    nums.dedup();
    assert_eq!(nums.len(), 40);
    assert!(nums.iter().all(|n| (1..=75).contains(n)));
}
