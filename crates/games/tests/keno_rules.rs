//! Tes aturan Keno (SPEC §6.5, §7; M6b, D-071), ditulis sebelum mesinnya.
//!
//! 80 angka, 20 ditarik, pemain memilih 1–10 angka. Tabel A: pengali total
//! (dalam persepuluhan) per jumlah angka yang kena; RTP dihitung tepat dari
//! distribusi hipergeometrik.

use kyusin_core::{Session, TurnGame};
use kyusin_games::keno::{self, Keno, RTP, exact_rtp, pays_tenths, probability};
use serde_json::{Value, json};

/// Tabel A (D-071) dalam persepuluhan: (spot, kena, pengali × 10).
const TABLE: &[(usize, usize, u32)] = &[
    (1, 1, 38),
    (2, 2, 158),
    (3, 2, 20),
    (3, 3, 480),
    (4, 2, 10),
    (4, 3, 55),
    (4, 4, 1600),
    (5, 3, 30),
    (5, 4, 150),
    (5, 5, 8000),
    (6, 3, 20),
    (6, 4, 65),
    (6, 5, 900),
    (6, 6, 18000),
    (7, 3, 15),
    (7, 4, 30),
    (7, 5, 230),
    (7, 6, 3000),
    (7, 7, 45000),
    (8, 4, 20),
    (8, 5, 150),
    (8, 6, 1100),
    (8, 7, 11000),
    (8, 8, 185000),
    (9, 4, 15),
    (9, 5, 55),
    (9, 6, 430),
    (9, 7, 3600),
    (9, 8, 36000),
    (9, 9, 360000),
    (10, 0, 35),
    (10, 5, 20),
    (10, 6, 220),
    (10, 7, 1450),
    (10, 8, 11000),
    (10, 9, 70000),
    (10, 10, 800000),
];

#[test]
fn pay_table_is_table_a() {
    for spots in 1..=10 {
        for hits in 0..=spots {
            let want = TABLE
                .iter()
                .find(|(s, h, _)| *s == spots && *h == hits)
                .map_or(0, |t| t.2);
            assert_eq!(pays_tenths(spots, hits), want, "{spots} spot, {hits} kena");
        }
    }
    assert_eq!(keno::NUMBERS, 80);
    assert_eq!(keno::DRAWN, 20);
    assert_eq!(keno::MAX_SPOTS, 10);
}

#[test]
fn exact_rtp_matches_the_announced_figures() {
    // Peluang tiap spot berjumlah 1.
    for spots in 1..=10 {
        let total: f64 = (0..=spots).map(|h| probability(spots, h)).sum();
        assert!((total - 1.0).abs() < 1e-12, "{spots}");
    }
    let want = [
        95.0, 95.0, 94.3525, 94.0642, 94.9130, 95.5916, 94.7183, 95.4780, 95.2818, 95.0068,
    ];
    for (i, w) in want.iter().enumerate() {
        let spots = i + 1;
        let r = exact_rtp(spots);
        assert!((r - w).abs() < 0.00006, "{spots} spot: {r}");
        let announced = RTP
            .iter()
            .find(|x| x.wager == format!("spot-{spots}"))
            .unwrap_or_else(|| panic!("RTP spot-{spots}"));
        assert!((announced.percent - r).abs() < 0.00006);
        assert!(announced.percent < 100.0);
    }
    // Angka manifest = spot dengan RTP terendah (4 spot).
    let marked: Vec<_> = RTP.iter().filter(|r| r.manifest).collect();
    assert_eq!(marked.len(), 1);
    assert_eq!(marked[0].wager, "spot-4");
}

fn game(drawn: &[u8]) -> Keno {
    Keno::new(
        serde_json::from_value(json!({ "hasil": drawn })).unwrap(),
        [0; 32],
    )
    .unwrap()
}

fn act(g: &mut Keno, cmd: &str) {
    Session::act(g, 0, cmd).unwrap_or_else(|e| panic!("`{cmd}`: {e:?}"));
}

const DRAW: [u8; 20] = [
    3, 7, 12, 18, 21, 25, 33, 37, 40, 44, 48, 52, 55, 61, 64, 68, 71, 75, 79, 80,
];

#[test]
fn a_round_marks_numbers_then_draws() {
    let mut g = game(&DRAW);
    let v = Session::view_data(&g, 0);
    assert_eq!(v["fase"], "taruhan");
    assert!(Session::act(&mut g, 0, "bet 10").is_err(), "tanpa angka");
    for n in ["3", "7", "12", "50"] {
        act(&mut g, &format!("mark {n}"));
    }
    assert!(Session::act(&mut g, 0, "mark 7").is_err(), "sudah dipilih");
    assert!(Session::act(&mut g, 0, "mark 0").is_err());
    assert!(Session::act(&mut g, 0, "mark 81").is_err());
    act(&mut g, "unmark 50");
    act(&mut g, "mark 64");
    let v = Session::view_data(&g, 0);
    assert_eq!(v["pilihan"], json!([3, 7, 12, 64]));
    // Tabel bayar untuk jumlah angka yang dipilih: [kena, pengali × 10].
    assert_eq!(v["tabel"], json!([[2, 10], [3, 55], [4, 1600]]));
    assert!(Session::act(&mut g, 0, "bet 15").is_err(), "kelipatan 10");
    act(&mut g, "bet 10");
    let v = Session::view_data(&g, 0);
    assert_eq!(v["fase"], "selesai");
    assert_eq!(v["tarikan"].as_array().unwrap().len(), 20);
    assert_eq!(v["kena"], json!([3, 7, 12, 64]));
    // 4 dari 4: 160×, bersih 1.600 − 10.
    assert_eq!(v["bayar"], 1590);
    assert_eq!(v["bersih"], 1590);
    assert_eq!(v["dipertaruhkan"], 10);
    assert!(TurnGame::is_over(&g));
}

#[test]
fn half_multipliers_and_losses() {
    // 4 spot, 3 kena: 5,5× dari 20 = 110, bersih +90.
    let mut g = game(&DRAW);
    for n in [3, 7, 12, 13] {
        act(&mut g, &format!("mark {n}"));
    }
    act(&mut g, "bet 20");
    assert_eq!(Session::view_data(&g, 0)["bayar"], 90);
    // 10 spot, 0 kena: 3,5×.
    let mut g = game(&DRAW);
    for n in [1, 2, 4, 5, 6, 8, 9, 10, 11, 13] {
        act(&mut g, &format!("mark {n}"));
    }
    assert!(Session::act(&mut g, 0, "mark 14").is_err(), "paling banyak 10");
    act(&mut g, "bet 100");
    let v = Session::view_data(&g, 0);
    assert_eq!(v["kena"], json!([]));
    assert_eq!(v["bayar"], 250);
    // 1 spot meleset: kalah taruhan.
    let mut g = game(&DRAW);
    act(&mut g, "mark 1");
    act(&mut g, "bet 50");
    assert_eq!(Session::view_data(&g, 0)["bayar"], -50);
}

#[test]
fn clear_leave_and_commands() {
    let mut g = game(&DRAW);
    act(&mut g, "mark 5");
    act(&mut g, "clear");
    assert_eq!(Session::view_data(&g, 0)["pilihan"], json!([]));
    for cmd in ["mark 5", "unmark 5", "clear", "bet 10", "leave"] {
        let a = g.parse_command(cmd).unwrap();
        assert_eq!(g.format_action(&a), cmd);
    }
    act(&mut g, "leave");
    let v = Session::view_data(&g, 0);
    assert_eq!(v["fase"], "selesai");
    assert_eq!(v["alasan"], "berhenti");
    assert_eq!(v["bersih"], 0);
}

#[test]
fn the_draw_comes_from_the_round_seed() {
    let draw = |seed: u8| -> Value {
        let mut g = Keno::new(Default::default(), [seed; 32]).unwrap();
        Session::act(&mut g, 0, "mark 1").unwrap();
        Session::act(&mut g, 0, "bet 10").unwrap();
        Session::view_data(&g, 0)["tarikan"].clone()
    };
    let a = draw(9);
    assert_eq!(a, draw(9));
    assert_ne!(a, draw(10));
    let mut nums: Vec<u64> = a.as_array().unwrap().iter().map(|x| x.as_u64().unwrap()).collect();
    nums.sort_unstable();
    nums.dedup();
    assert_eq!(nums.len(), 20);
    assert!(nums.iter().all(|n| (1..=80).contains(n)));
    // Hasil tetap di konfigurasi harus 20 angka berbeda 1–80.
    for bad in [json!([1, 2, 3]), json!((1..=20).map(|_| 5).collect::<Vec<_>>())] {
        let cfg = serde_json::from_value(json!({ "hasil": bad })).unwrap();
        assert!(Keno::new(cfg, [0; 32]).is_err());
    }
}
