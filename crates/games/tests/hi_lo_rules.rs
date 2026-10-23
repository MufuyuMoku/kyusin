//! Tes aturan Hi-Lo (SPEC §6.5, §7; M6b, D-071), ditulis sebelum mesinnya.
//!
//! Dek tak terbatas: setiap kartu independen, 13 peringkat sama peluang
//! (A = 1, K = 13). Tebakan "lebih tinggi atau sama" / "lebih rendah atau
//! sama"; pengali = 0,97 ÷ peluang menang dibulatkan ke bawah 2 desimal
//! (dalam perseratus: ⌊97 × 13 ÷ n⌋, n = peringkat yang menang). Pengali
//! berjalan juga perseratus, dibulatkan ke bawah tiap langkah; 1.000× = cash
//! out otomatis.

use kyusin_core::{Session, TurnGame};
use kyusin_games::hi_lo::{self, HiLo, LIMITS, RTP, multiplier};
use serde_json::{Value, json};

#[test]
fn multipliers_from_win_chance() {
    // Peringkat 7: tinggi-atau-sama menang di 7 peringkat → ⌊1261/7⌋ = 180.
    assert_eq!(multiplier(7, true), 180);
    assert_eq!(multiplier(7, false), 180);
    // A: tinggi-atau-sama pasti menang → 0,97×; rendah-atau-sama hanya A.
    assert_eq!(multiplier(1, true), 97);
    assert_eq!(multiplier(1, false), 1261);
    assert_eq!(multiplier(13, false), 97);
    assert_eq!(multiplier(13, true), 1261);
    assert_eq!(multiplier(9, true), 252);
    assert_eq!(multiplier(9, false), 140);
    for r in 1..=13u8 {
        for hi in [true, false] {
            let n = if hi { 14 - r } else { r };
            // Pengembalian satu tebakan tidak pernah di atas 97%.
            assert!(u32::from(n) * multiplier(r, hi) <= 97 * 13, "{r} {hi}");
        }
    }
}

#[test]
fn announced_rtp_is_the_best_single_guess() {
    // Strategi terbaik: satu tebakan (pilihan yang lebih baik per kartu),
    // lalu cash out; setiap langkah tambahan mengalikan ≤ 0,97.
    let mut sum = 0u64;
    for r in 1..=13u8 {
        let hi = u64::from(14 - r) * u64::from(multiplier(r, true));
        let lo = u64::from(r) * u64::from(multiplier(r, false));
        sum += hi.max(lo);
    }
    let exact = sum as f64 / (13.0 * 13.0);
    assert!((exact - 96.934911).abs() < 0.000001, "{exact}");
    assert_eq!(RTP.len(), 1);
    assert!(RTP[0].manifest);
    assert!((RTP[0].percent - exact).abs() < 0.00006);
    assert_eq!((LIMITS.min, LIMITS.max, LIMITS.step), (100, 2000, 100));
    assert_eq!(hi_lo::CAP, 100_000);
}

fn game(cards: &[&str]) -> HiLo {
    HiLo::new(
        serde_json::from_value(json!({ "kartu": cards })).unwrap(),
        [0; 32],
    )
    .unwrap()
}

fn act(g: &mut HiLo, cmd: &str) {
    Session::act(g, 0, cmd).unwrap_or_else(|e| panic!("`{cmd}`: {e:?}"));
}

#[test]
fn guess_then_cash_out() {
    let mut g = game(&["7h", "9s", "9d", "2c"]);
    assert_eq!(Session::view_data(&g, 0)["fase"], "taruhan");
    assert!(Session::act(&mut g, 0, "bet 150").is_err(), "kelipatan 100");
    act(&mut g, "bet 100");
    let v = Session::view_data(&g, 0);
    assert_eq!(v["fase"], "tebak");
    assert_eq!(v["kartu"], json!(["7h"]));
    assert_eq!(v["pengali"], 100);
    assert_eq!(v["pengali_tinggi"], 180);
    assert_eq!(v["pengali_rendah"], 180);
    assert_eq!(v["taruhan_meja"], 100);
    assert!(Session::act(&mut g, 0, "cashout").is_err(), "belum menebak");
    act(&mut g, "higher");
    let v = Session::view_data(&g, 0);
    assert_eq!(v["kartu"], json!(["7h", "9s"]));
    assert_eq!(v["pengali"], 180);
    // 9 lalu 9: sama = menang; ⌊180 × 252 / 100⌋ = 453.
    act(&mut g, "higher");
    assert_eq!(Session::view_data(&g, 0)["pengali"], 453);
    assert_eq!(Session::view_data(&g, 0)["netral"], "cashout");
    act(&mut g, "cashout");
    let v = Session::view_data(&g, 0);
    assert_eq!(v["fase"], "selesai");
    assert_eq!(v["alasan"], "cashout");
    assert_eq!(v["bayar"], 353);
    assert_eq!(v["bersih"], 353);
    assert_eq!(v["dipertaruhkan"], 100);
    assert!(TurnGame::is_over(&g));
}

#[test]
fn a_wrong_guess_loses_the_bet() {
    let mut g = game(&["7h", "9s", "2c"]);
    act(&mut g, "bet 200");
    act(&mut g, "higher");
    act(&mut g, "higher");
    let v = Session::view_data(&g, 0);
    assert_eq!(v["fase"], "selesai");
    assert_eq!(v["alasan"], "kalah");
    assert_eq!(v["kartu"], json!(["7h", "9s", "2c"]));
    assert_eq!(v["bayar"], -200);
}

#[test]
fn the_cap_cashes_out_automatically() {
    // A, lalu rendah-atau-sama (12,61×) tiga kali: 100 → 1.261 → 15.901 →
    // 200.511 > 100.000, jadi pengali dipotong ke 1.000× dan cash out.
    let mut g = game(&["Ah", "As", "Ad", "Ac"]);
    act(&mut g, "bet 100");
    act(&mut g, "lower");
    act(&mut g, "lower");
    assert_eq!(Session::view_data(&g, 0)["pengali"], 15_901);
    act(&mut g, "lower");
    let v = Session::view_data(&g, 0);
    assert_eq!(v["fase"], "selesai");
    assert_eq!(v["alasan"], "batas");
    assert_eq!(v["pengali"], 100_000);
    assert_eq!(v["bayar"], 99_900);
}

#[test]
fn cards_come_from_the_round_seed() {
    let run = |seed: u8| -> Value {
        let mut g = HiLo::new(Default::default(), [seed; 32]).unwrap();
        Session::act(&mut g, 0, "bet 100").unwrap();
        while Session::view_data(&g, 0)["fase"] == "tebak" {
            let v = Session::view_data(&g, 0);
            if v["kartu"].as_array().unwrap().len() > 6 {
                Session::act(&mut g, 0, "cashout").unwrap();
                break;
            }
            let cmd = if v["pengali_tinggi"].as_u64() <= v["pengali_rendah"].as_u64() {
                "higher"
            } else {
                "lower"
            };
            Session::act(&mut g, 0, cmd).unwrap();
        }
        Session::view_data(&g, 0)["kartu"].clone()
    };
    assert_eq!(run(1), run(1));
    let all: std::collections::BTreeSet<String> = (0..30).map(|s| run(s)[0].to_string()).collect();
    assert!(all.len() > 6, "kartu pertama beragam: {all:?}");
    let g = game(&["7h"]);
    for cmd in ["bet 100", "higher", "lower", "cashout", "leave"] {
        let a = g.parse_command(cmd).unwrap();
        assert_eq!(g.format_action(&a), cmd);
    }
    // Sebelum tebakan pertama, aksi netral = tebakan dengan peluang lebih besar.
    let mut g = game(&["3h", "Ks"]);
    act(&mut g, "bet 100");
    assert_eq!(Session::view_data(&g, 0)["netral"], "higher");
    let mut g = game(&["Jh", "Ks"]);
    act(&mut g, "bet 100");
    assert_eq!(Session::view_data(&g, 0)["netral"], "lower");
}
