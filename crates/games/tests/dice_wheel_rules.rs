//! Tes aturan Sic Bo, Chuck-a-luck, Big Six, dan Fan-Tan (SPEC §6.4, §7;
//! M6a, D-069), ditulis sebelum mesinnya. Semuanya memakai mesin papan
//! taruhan bersama: satu putaran/lemparan per pertandingan, hasil dari
//! seed ronde atau `hasil` tetap di konfigurasi (tes dan tutorial).

use kyusin_core::{Session, TurnGame};
use kyusin_games::big_six::{BigSix, Wheel};
use kyusin_games::chuck_a_luck::{Chuck, ChuckALuck};
use kyusin_games::fan_tan::{FanTan, FanTanRules};
use kyusin_games::papan_taruhan::{BoardRules, exact_rtp};
use kyusin_games::sic_bo::{SicBo, SicBoRules};
use serde_json::{Value, json};

fn play<G: TurnGame + Send>(mut g: G, verb: &str, bets: &[(&str, i64)]) -> Value {
    for (spot, n) in bets {
        Session::act(&mut g, 0, &format!("bet {spot} {n}"))
            .unwrap_or_else(|e| panic!("bet {spot} {n}: {e:?}"));
    }
    Session::act(&mut g, 0, verb).unwrap();
    Session::view_data(&g, 0)
}

fn sic(dice: [u8; 3]) -> SicBo {
    SicBo::new(
        serde_json::from_value(json!({ "hasil": dice })).unwrap(),
        [0; 32],
    )
    .unwrap()
}

fn sic_net(dice: [u8; 3], spot: &str) -> i64 {
    play(sic(dice), "roll", &[(spot, 10)])["bersih"]
        .as_i64()
        .unwrap()
}

fn close(r: f64, want: f64, what: &str) {
    assert!((r - want).abs() < 1e-9, "{what}: {r} ≠ {want}");
}

#[test]
fn sic_bo_macau_paytable() {
    assert_eq!(sic_net([1, 2, 4], "small"), 10);
    assert_eq!(sic_net([2, 2, 2], "small"), -10, "triple kalah");
    assert_eq!(sic_net([6, 5, 1], "big"), 10);
    assert_eq!(sic_net([6, 5, 2], "odd"), 10);
    assert_eq!(sic_net([5, 5, 5], "odd"), -10);
    assert_eq!(sic_net([1, 1, 2], "total-4"), 500);
    assert_eq!(sic_net([6, 6, 5], "total-17"), 500);
    assert_eq!(sic_net([1, 2, 2], "total-5"), 180);
    assert_eq!(sic_net([1, 2, 3], "total-6"), 140);
    assert_eq!(sic_net([1, 2, 4], "total-7"), 120);
    assert_eq!(sic_net([1, 3, 4], "total-8"), 80);
    assert_eq!(sic_net([2, 3, 4], "total-9"), 60);
    assert_eq!(sic_net([3, 3, 4], "total-10"), 60);
    assert_eq!(sic_net([3, 3, 5], "total-11"), 60);
    assert_eq!(sic_net([1, 3, 6], "combo-1-3"), 60);
    assert_eq!(sic_net([3, 3, 1], "combo-1-3"), 60, "1 dan 3 muncul");
    assert_eq!(sic_net([1, 1, 6], "combo-1-3"), -10);
    assert_eq!(sic_net([4, 4, 2], "double-4"), 80);
    assert_eq!(sic_net([4, 4, 4], "double-4"), 80, "triple juga double");
    assert_eq!(sic_net([3, 3, 3], "any-triple"), 240);
    assert_eq!(sic_net([3, 3, 3], "triple-3"), 1500);
    assert_eq!(sic_net([3, 3, 3], "triple-4"), -10);
    assert_eq!(sic_net([5, 1, 2], "single-5"), 10);
    assert_eq!(sic_net([5, 5, 2], "single-5"), 20);
    assert_eq!(sic_net([5, 5, 5], "single-5"), 30);
    assert_eq!(sic_net([1, 2, 3], "single-5"), -10);
    assert!(
        Session::act(&mut sic([1, 2, 3]), 0, "bet total-3 10").is_err(),
        "tidak ada total 3"
    );
    let spots = SicBoRules::spots();
    assert_eq!(spots.len(), 4 + 14 + 15 + 6 + 1 + 6 + 6);
}

#[test]
fn sic_bo_exact_rtp() {
    let r = |s| exact_rtp::<SicBoRules>(s);
    for s in ["small", "big", "odd", "even", "combo-2-5"] {
        close(r(s), 100.0 * 210.0 / 216.0, s);
    }
    close(r("total-4"), 100.0 * 51.0 * 3.0 / 216.0, "total-4");
    close(r("total-10"), 100.0 * 7.0 * 27.0 / 216.0, "total-10");
    close(r("double-2"), 100.0 * 9.0 * 16.0 / 216.0, "double-2");
    close(r("any-triple"), 100.0 * 25.0 * 6.0 / 216.0, "any-triple");
    close(r("triple-6"), 100.0 * 151.0 / 216.0, "triple-6");
    close(r("single-1"), 100.0 * 199.0 / 216.0, "single-1");
    for s in SicBoRules::spots() {
        assert!(r(s) < 100.0, "{s}");
    }
}

#[test]
fn chuck_a_luck_one_two_ten() {
    let g = |dice: [u8; 3]| {
        ChuckALuck::new(
            serde_json::from_value(json!({ "hasil": dice })).unwrap(),
            [0; 32],
        )
        .unwrap()
    };
    let n = |dice, spot| {
        play(g(dice), "roll", &[(spot, 10)])["bersih"]
            .as_i64()
            .unwrap()
    };
    assert_eq!(n([4, 1, 2], "single-4"), 10);
    assert_eq!(n([4, 4, 2], "single-4"), 20);
    assert_eq!(n([4, 4, 4], "single-4"), 100);
    assert_eq!(n([1, 2, 3], "single-4"), -10);
    assert_eq!(n([2, 2, 2], "any-triple"), 300);
    assert_eq!(n([6, 6, 6], "big"), -10);
    assert_eq!(n([6, 5, 1], "big"), 10);
    close(
        exact_rtp::<Chuck>("single-3"),
        100.0 * 206.0 / 216.0,
        "single",
    );
    close(
        exact_rtp::<Chuck>("any-triple"),
        100.0 * 31.0 * 6.0 / 216.0,
        "any-triple",
    );
    close(exact_rtp::<Chuck>("small"), 100.0 * 210.0 / 216.0, "small");
}

#[test]
fn big_six_wheel() {
    let segments = Wheel::segments();
    assert_eq!(segments.len(), 54);
    let count = |sym: &str| segments.iter().filter(|s| **s == sym).count();
    assert_eq!(
        ["1", "2", "5", "10", "20", "joker", "logo"].map(count),
        [24, 15, 7, 4, 2, 1, 1]
    );
    let g = |sym: &str| {
        BigSix::new(
            serde_json::from_value(json!({ "hasil": sym })).unwrap(),
            [0; 32],
        )
        .unwrap()
    };
    let n = |sym, spot| {
        play(g(sym), "spin", &[(spot, 10)])["bersih"]
            .as_i64()
            .unwrap()
    };
    assert_eq!(n("1", "1"), 10);
    assert_eq!(n("2", "2"), 20);
    assert_eq!(n("5", "5"), 50);
    assert_eq!(n("10", "10"), 100);
    assert_eq!(n("20", "20"), 200);
    assert_eq!(n("joker", "joker"), 400);
    assert_eq!(n("logo", "logo"), 400);
    assert_eq!(n("joker", "logo"), -10);
    assert_eq!(n("2", "1"), -10);
    for (spot, want) in [
        ("1", 48.0),
        ("2", 45.0),
        ("5", 42.0),
        ("10", 44.0),
        ("20", 42.0),
        ("joker", 41.0),
        ("logo", 41.0),
    ] {
        close(exact_rtp::<Wheel>(spot), 100.0 * want / 54.0, spot);
    }
}

#[test]
fn fan_tan_macau_with_commission() {
    let g = |beans: u32| {
        FanTan::new(
            serde_json::from_value(json!({ "hasil": beans })).unwrap(),
            [0; 32],
        )
        .unwrap()
    };
    let v = play(g(41), "open", &[("fan-1", 120)]);
    assert_eq!(v["hasil"], 41);
    assert_eq!(v["angka"], 1, "41 kancing: sisa 1");
    assert_eq!(v["bersih"], 342, "3:1 dipotong 5%");
    assert_eq!(
        play(g(44), "open", &[("fan-4", 120)])["angka"],
        4,
        "sisa 0 = 4"
    );
    let n = |beans, spot| {
        play(g(beans), "open", &[(spot, 120)])["bersih"]
            .as_i64()
            .unwrap()
    };
    assert_eq!(n(42, "nim-2-3"), 228);
    assert_eq!(n(43, "nim-2-3"), 0, "angka seri");
    assert_eq!(n(41, "nim-2-3"), -120);
    assert_eq!(n(42, "kwok-1-2"), 114);
    assert_eq!(n(44, "kwok-1-4"), 114);
    assert_eq!(n(43, "kwok-1-2"), -120);
    assert_eq!(n(41, "tan-1-2-3"), 57);
    assert_eq!(n(43, "tan-1-2-3"), 0);
    assert_eq!(n(44, "tan-1-2-3"), -120);
    assert_eq!(n(42, "ssh-1-2-3"), 38);
    assert_eq!(n(44, "ssh-1-2-3"), -120);
    // Kelipatan 120 supaya komisi selalu utuh.
    assert!(Session::act(&mut g(41), 0, "bet fan-1 100").is_err());
    assert!(
        Session::act(&mut g(41), 0, "bet kwok-1-3 120").is_err(),
        "kwok hanya angka berdampingan"
    );
    for (spot, want) in [
        ("fan-2", 96.25),
        ("nim-1-2", 97.5),
        ("kwok-2-3", 97.5),
        ("tan-1-2-3", 98.75),
        ("ssh-2-3-4", 98.75),
    ] {
        close(exact_rtp::<FanTanRules>(spot), want, spot);
    }
}
