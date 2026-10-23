//! Tes aturan kartu gosok (SPEC §6.5, §7; M6b, D-071), ditulis sebelum
//! mesinnya.
//!
//! Kartu 3×3, cocokkan tiga jumlah. Hadiah diundi lebih dulu dari seed
//! ronde (penyebut 100.000), lalu sembilan sel disusun: menang = tepat tiga
//! sel bernilai hadiah, nilai lain paling banyak dua kali; kalah = tidak ada
//! nilai yang muncul tiga kali.

use std::collections::BTreeMap;

use kyusin_core::{Session, TurnGame};
use kyusin_games::kartu_gosok::{self, DENOMINATOR, KartuGosok, PRIZES, RTP, layout};
use serde_json::{Value, json};

#[test]
fn prize_table_and_exact_rtp() {
    assert_eq!(
        PRIZES,
        &[
            (1, 16_000),
            (2, 10_000),
            (5, 3_000),
            (10, 1_500),
            (20, 600),
            (50, 200),
            (100, 50),
            (1000, 2)
        ]
    );
    assert_eq!(DENOMINATOR, 100_000);
    let back: u64 = PRIZES.iter().map(|(m, w)| u64::from(*m) * u64::from(*w)).sum();
    assert_eq!(back, 95_000, "RTP tepat 95%");
    let wins: u32 = PRIZES.iter().map(|p| p.1).sum();
    assert_eq!(wins, 31_352);
    assert_eq!(RTP.len(), 1);
    assert!(RTP[0].manifest);
    assert_eq!(RTP[0].percent, 95.0);
}

fn counts(cells: &[u32; 9]) -> BTreeMap<u32, usize> {
    let mut m = BTreeMap::new();
    for c in cells {
        *m.entry(*c).or_default() += 1;
    }
    m
}

#[test]
fn layouts_show_exactly_the_drawn_prize() {
    for seed in 0..2000u32 {
        let mut s = [0u8; 32];
        s[..4].copy_from_slice(&seed.to_le_bytes());
        for prize in [0, 1, 2, 5, 10, 20, 50, 100, 1000] {
            let cells = layout(prize, s);
            assert!(cells.iter().all(|c| PRIZES.iter().any(|p| p.0 == *c)), "{cells:?}");
            let c = counts(&cells);
            let triples: Vec<u32> = c.iter().filter(|(_, n)| **n >= 3).map(|(v, _)| *v).collect();
            if prize == 0 {
                assert!(triples.is_empty(), "kalah tanpa tiga sama: {cells:?}");
            } else {
                assert_eq!(triples, [prize], "{cells:?}");
                assert_eq!(c[&prize], 3);
            }
        }
    }
}

fn game(prize: u32) -> KartuGosok {
    KartuGosok::new(
        serde_json::from_value(json!({ "hadiah": prize })).unwrap(),
        [7; 32],
    )
    .unwrap()
}

fn act(g: &mut KartuGosok, cmd: &str) {
    Session::act(g, 0, cmd).unwrap_or_else(|e| panic!("`{cmd}`: {e:?}"));
}

#[test]
fn scratching_reveals_then_settles() {
    let mut g = game(20);
    assert_eq!(Session::view_data(&g, 0)["fase"], "taruhan");
    assert!(Session::act(&mut g, 0, "scratch 1").is_err(), "belum membeli kartu");
    act(&mut g, "bet 50");
    let v = Session::view_data(&g, 0);
    assert_eq!(v["fase"], "gosok");
    assert_eq!(v["taruhan_meja"], 50, "dipotong saat dibeli");
    assert_eq!(v["sel"], json!([null, null, null, null, null, null, null, null, null]));
    assert_eq!(v["netral"], "scratch all");
    act(&mut g, "scratch 5");
    assert!(Session::act(&mut g, 0, "scratch 5").is_err(), "sudah digosok");
    assert!(Session::act(&mut g, 0, "scratch 10").is_err());
    let v = Session::view_data(&g, 0);
    assert!(v["sel"][4].is_u64());
    assert_eq!(v["sel"].as_array().unwrap().iter().filter(|c| c.is_null()).count(), 8);
    assert!(v["bayar"].is_null(), "hadiah belum dibuka");
    act(&mut g, "scratch all");
    let v = Session::view_data(&g, 0);
    assert_eq!(v["fase"], "selesai");
    assert_eq!(v["hadiah"], 20);
    assert_eq!(v["bayar"], 20 * 50 - 50);
    assert_eq!(v["bersih"], 950);
    assert_eq!(v["dipertaruhkan"], 50);
    assert!(TurnGame::is_over(&g));
    // Kalah: bersih −taruhan setelah sel terakhir digosok satu per satu.
    let mut g = game(0);
    act(&mut g, "bet 10");
    for i in 1..=9 {
        act(&mut g, &format!("scratch {i}"));
    }
    let v = Session::view_data(&g, 0);
    assert_eq!(v["fase"], "selesai");
    assert_eq!(v["bayar"], -10);
}

#[test]
fn prizes_come_from_the_round_seed() {
    let card = |seed: u8| -> Value {
        let mut g = KartuGosok::new(Default::default(), [seed; 32]).unwrap();
        Session::act(&mut g, 0, "bet 10").unwrap();
        Session::act(&mut g, 0, "scratch all").unwrap();
        Session::view_data(&g, 0)["sel"].clone()
    };
    assert_eq!(card(3), card(3));
    let distinct: std::collections::BTreeSet<String> = (0..20).map(|s| card(s).to_string()).collect();
    assert!(distinct.len() > 15);
    // Frekuensi menang dari 20.000 seed dalam 4σ dari 31,352%.
    let n = 20_000u32;
    let wins = (0..n)
        .filter(|i| {
            let mut s = [0u8; 32];
            s[..4].copy_from_slice(&i.to_le_bytes());
            kartu_gosok::draw_prize(s) > 0
        })
        .count() as f64;
    let p = 0.31352;
    let sd = (p * (1.0 - p) / f64::from(n)).sqrt();
    assert!((wins / f64::from(n) - p).abs() < 4.0 * sd, "{wins}");
    let g = game(0);
    for cmd in ["bet 10", "scratch 3", "scratch all", "leave"] {
        let a = g.parse_command(cmd).unwrap();
        assert_eq!(g.format_action(&a), cmd);
    }
    // Hadiah tetap di konfigurasi harus ada di tabel.
    let cfg = serde_json::from_value(json!({ "hadiah": 3 })).unwrap();
    assert!(KartuGosok::new(cfg, [0; 32]).is_err());
}
