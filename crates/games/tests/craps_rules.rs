//! Tes aturan Craps (SPEC §6.4, §7; M6a, D-069), ditulis sebelum mesinnya.
//!
//! Satu pertandingan = satu penembak, sampai seven-out atau berhenti.
//! Taruhan: Pass/Don't Pass (12 seri), Come/Don't Come, Odds 3-4-5x (lay
//! sampai 6x), Place 4-5-6-8-9-10, Field (2 → 2:1, 12 → 3:1), Hardways.
//! Place, Hardways, dan Odds Come "off" di lemparan come-out. Taruhan yang
//! menang tetap terpasang kecuali Pass/Come/Don't (lunas). `take <tempat>`
//! menarik taruhan yang boleh ditarik; `leave` hanya bila tidak ada taruhan
//! kontrak (Pass/Come yang sudah punya angka).

use kyusin_core::{Session, TurnGame};
use kyusin_games::craps::{Craps, RTP};
use serde_json::{Value, json};

fn craps(rolls: &[[u8; 2]]) -> Craps {
    Craps::new(
        serde_json::from_value(json!({ "lemparan": rolls })).unwrap(),
        [0; 32],
    )
    .unwrap()
}

fn act(g: &mut Craps, cmd: &str) {
    Session::act(g, 0, cmd).unwrap_or_else(|e| panic!("`{cmd}`: {e:?}"));
}

fn view(g: &Craps) -> Value {
    Session::view_data(g, 0)
}

fn net(g: &Craps) -> i64 {
    view(g)["bersih"].as_i64().unwrap()
}

#[test]
fn come_out_naturals_craps_and_bar_twelve() {
    for (roll, pass, dont) in [
        ([3, 4], 10, -10),
        ([5, 6], 10, -10),
        ([1, 1], -10, 10),
        ([1, 2], -10, 10),
        ([6, 6], -10, 0),
    ] {
        let mut g = craps(&[roll]);
        act(&mut g, "bet pass 10");
        act(&mut g, "bet dont-pass 10");
        act(&mut g, "roll");
        let v = view(&g);
        assert_eq!(v["bayar"]["pass"], pass, "{roll:?}");
        assert_eq!(v["bayar"]["dont-pass"], dont, "{roll:?}");
        assert_eq!(v["titik"], Value::Null, "{roll:?}: tetap come-out");
        assert!(!TurnGame::is_over(&g), "penembak masih melempar");
    }
}

#[test]
fn point_odds_and_making_the_point() {
    let mut g = craps(&[[4, 2], [5, 1]]);
    act(&mut g, "bet pass 10");
    assert!(
        Session::act(&mut g, 0, "bet odds-pass 10").is_err(),
        "odds hanya setelah ada titik"
    );
    act(&mut g, "roll");
    assert_eq!(view(&g)["titik"], 6);
    assert!(
        Session::act(&mut g, 0, "take pass").is_err(),
        "pass dengan titik adalah kontrak"
    );
    // 5x pada 6: paling banyak 50 di belakang pass 10.
    assert!(Session::act(&mut g, 0, "bet odds-pass 60").is_err());
    act(&mut g, "bet odds-pass 50");
    act(&mut g, "roll");
    let v = view(&g);
    assert_eq!(v["bayar"]["pass"], 10);
    assert_eq!(v["bayar"]["odds-pass"], 60, "6:5");
    assert_eq!(v["titik"], Value::Null);
    assert_eq!(net(&g), 70);
}

#[test]
fn seven_out_ends_the_shooter() {
    let mut g = craps(&[[2, 2], [3, 4]]);
    act(&mut g, "bet pass 10");
    act(&mut g, "bet dont-pass 10");
    act(&mut g, "roll");
    act(&mut g, "bet odds-pass 30");
    act(&mut g, "bet odds-dont-pass 60");
    act(&mut g, "bet place-6 30");
    act(&mut g, "roll");
    let v = view(&g);
    assert_eq!(v["bayar"]["pass"], -10);
    assert_eq!(v["bayar"]["odds-pass"], -30);
    assert_eq!(v["bayar"]["dont-pass"], 10);
    assert_eq!(v["bayar"]["odds-dont-pass"], 30, "lay 1:2 pada 4");
    assert_eq!(v["bayar"]["place-6"], -30);
    assert_eq!(v["fase"], "selesai");
    assert_eq!(v["alasan"], "seven_out");
    assert_eq!(v["taruhan_meja"], 0);
    assert!(TurnGame::is_over(&g));
}

#[test]
fn come_bets_travel_to_their_number() {
    let mut g = craps(&[[2, 2], [4, 5], [6, 3], [1, 1], [3, 4]]);
    act(&mut g, "bet pass 10");
    act(&mut g, "roll"); // titik 4
    act(&mut g, "bet come 10");
    act(&mut g, "roll"); // 9: come pindah ke 9
    let v = view(&g);
    assert_eq!(v["taruhan"]["come-9"], 10);
    assert_eq!(v["taruhan"]["come"], Value::Null);
    act(&mut g, "bet odds-come-9 40");
    act(&mut g, "roll"); // 9 lagi: come-9 menang 1:1, odds 3:2
    let v = view(&g);
    assert_eq!(v["bayar"]["come-9"], 10);
    assert_eq!(v["bayar"]["odds-come-9"], 60);
    act(&mut g, "bet come 10");
    act(&mut g, "roll"); // 2: come kalah di lemparan pertamanya
    assert_eq!(view(&g)["bayar"]["come"], -10);
    act(&mut g, "roll"); // 7: seven-out
    assert_eq!(view(&g)["bayar"]["pass"], -10);
    assert!(TurnGame::is_over(&g));
}

#[test]
fn place_field_and_hardways() {
    // Come-out 8 (titik), lalu 6, 5, 4, 4-4 keras, 7.
    let mut g = craps(&[[5, 3], [5, 1], [3, 2], [3, 1], [4, 4], [6, 1]]);
    act(&mut g, "bet pass 10");
    act(&mut g, "bet place-6 30");
    act(&mut g, "bet hard-8 10");
    act(&mut g, "roll");
    let v = view(&g);
    assert_eq!(v["bayar"].get("place-6"), None, "place off di come-out");
    assert!(
        Session::act(&mut g, 0, "bet place-6 20").is_err(),
        "place 6 kelipatan 30"
    );
    act(&mut g, "bet place-5 10");
    act(&mut g, "bet place-4 10");
    act(&mut g, "bet field 10");
    act(&mut g, "roll"); // 6
    let v = view(&g);
    assert_eq!(v["bayar"]["place-6"], 35, "7:6");
    assert_eq!(v["bayar"]["field"], -10);
    assert_eq!(v["taruhan"]["place-6"], 30, "tetap terpasang");
    act(&mut g, "bet field 10");
    act(&mut g, "roll"); // 5
    assert_eq!(view(&g)["bayar"]["place-5"], 14, "7:5");
    act(&mut g, "bet field 10");
    act(&mut g, "roll"); // 4
    let v = view(&g);
    assert_eq!(v["bayar"]["place-4"], 18, "9:5");
    assert_eq!(v["bayar"]["field"], 10);
    act(&mut g, "take place-4");
    act(&mut g, "roll"); // 8 keras: titik tercapai, hard-8 9:1
    let v = view(&g);
    assert_eq!(v["bayar"]["hard-8"], 90);
    assert_eq!(v["bayar"]["pass"], 10);
    act(&mut g, "bet pass 10");
    act(&mut g, "roll"); // come-out 7: pass menang; place/hard off
    let v = view(&g);
    assert_eq!(v["bayar"]["pass"], 10);
    assert_eq!(v["bayar"].get("hard-8"), None);
    assert!(!TurnGame::is_over(&g), "7 di come-out bukan seven-out");
    let mut f = craps(&[[1, 1]]);
    act(&mut f, "bet field 10");
    act(&mut f, "roll");
    assert_eq!(view(&f)["bayar"]["field"], 20, "2 dibayar 2:1");
    let mut f = craps(&[[6, 6]]);
    act(&mut f, "bet field 10");
    act(&mut f, "roll");
    assert_eq!(view(&f)["bayar"]["field"], 30, "12 dibayar 3:1");
    let mut h = craps(&[[3, 3], [5, 1]]);
    act(&mut h, "bet pass 10");
    act(&mut h, "roll");
    act(&mut h, "bet hard-6 10");
    act(&mut h, "roll");
    assert_eq!(view(&h)["bayar"]["hard-6"], -10, "6 lunak");
}

#[test]
fn leaving_and_the_neutral_action() {
    let mut g = craps(&[[2, 2]]);
    assert_eq!(
        view(&g)["netral"],
        Value::Null,
        "tanpa taruhan: boleh berhenti"
    );
    act(&mut g, "bet pass 10");
    act(&mut g, "bet field 10");
    assert_eq!(view(&g)["netral"], "roll");
    act(&mut g, "roll");
    let legal: Vec<String> = TurnGame::legal_actions(&g, 0)
        .iter()
        .map(|a| a.usage())
        .collect();
    assert!(
        !legal.contains(&"leave".to_string()),
        "pass dengan titik: tidak boleh berhenti"
    );
    assert_eq!(view(&g)["netral"], "roll");
    assert_eq!(view(&g)["taruhan_meja"], 10);
    let mut q = craps(&[[3, 3]]);
    act(&mut q, "bet field 10");
    act(&mut q, "roll");
    act(&mut q, "bet place-8 30");
    act(&mut q, "leave");
    let v = view(&q);
    assert_eq!(v["fase"], "selesai");
    assert_eq!(v["alasan"], "berhenti");
    assert_eq!(v["taruhan_meja"], 0, "place ditarik saat berhenti");
    for cmd in ["bet pass 10", "take place-6", "roll", "leave"] {
        let a = q.parse_command(cmd).unwrap();
        assert_eq!(q.format_action(&a), cmd);
    }
}

#[test]
fn announced_rtp_matches_the_exact_odds() {
    let get = |w: &str| RTP.iter().find(|r| r.wager == w).unwrap().percent;
    let near = |a: f64, b: f64, w: &str| assert!((a - b).abs() < 5e-5, "{w}: {a} ≠ {b}");
    near(get("pass"), 100.0 * 2.0 * 244.0 / 495.0, "pass");
    near(get("dont-pass"), 100.0 * (1.0 - 3.0 / 220.0), "dont-pass");
    near(get("place-6"), 100.0 * (1.0 - 1.0 / 66.0), "place-6");
    near(get("place-5"), 96.0, "place-5");
    near(get("place-4"), 100.0 * (1.0 - 1.0 / 15.0), "place-4");
    near(get("field"), 100.0 * 35.0 / 36.0, "field");
    near(get("hard-6"), 100.0 * 10.0 / 11.0, "hard-6");
    near(get("hard-4"), 100.0 * 8.0 / 9.0, "hard-4");
    assert!(RTP.iter().filter(|r| r.manifest).count() == 1);
    for r in RTP {
        assert!(r.percent < 100.0, "{}", r.wager);
    }
}

/// Properti: bersih = jumlah semua bayaran; taruhan di meja tidak pernah
/// negatif; penembak selalu berakhir (SPEC §7 poin 1).
#[test]
fn random_shooters_settle_consistently() {
    use kyusin_core::GameRng;
    for seed in 0..200u8 {
        let mut rng = GameRng::from_seed([seed; 32]);
        let mut g = Craps::new(Default::default(), [seed; 32]).unwrap();
        let mut paid = 0i64;
        let mut steps = 0;
        while !TurnGame::is_over(&g) && steps < 500 {
            for spot in ["pass", "field", "place-6", "come", "hard-8", "dont-pass"] {
                if rng.below(3) == 0 {
                    let n = if spot == "place-6" { 30 } else { 10 };
                    let _ = Session::act(&mut g, 0, &format!("bet {spot} {n}"));
                }
            }
            let _ = Session::act(&mut g, 0, "bet odds-pass 10");
            if view(&g)["taruhan_meja"] == 0 {
                act(&mut g, "bet field 10");
            }
            act(&mut g, "roll");
            let v = view(&g);
            paid += v["bayar"]
                .as_object()
                .unwrap()
                .values()
                .map(|x| x.as_i64().unwrap())
                .sum::<i64>();
            assert_eq!(
                v["bersih"].as_i64().unwrap(),
                paid,
                "sesi {seed} langkah {steps}"
            );
            assert!(v["taruhan_meja"].as_i64().unwrap() >= 0);
            steps += 1;
        }
        assert!(TurnGame::is_over(&g), "sesi {seed}");
    }
}
