//! Tes aturan dan pembayaran Andar Bahar (SPEC §6.3, §7; M5a), ditulis
//! sebelum mesinnya.
//!
//! Aturan meja (keputusan klien, lihat DECISIONS): satu dek dikocok per
//! ronde; satu ronde = satu pertandingan, jadi commit-reveal berlaku per
//! ronde (SPEC §5.4). Taruhan `andar` dan/atau `bahar`, masing-masing
//! 10–2.000, kelipatan 10. Kartu tengah dibuka, lalu kartu dibagi
//! bergantian mulai dari Andar sampai muncul kartu berperingkat sama dengan
//! kartu tengah; sisi yang menerimanya menang. Andar (sisi pertama) dibayar
//! 0,9:1, Bahar 1:1.

use kyusin_core::{Lang, Session, TurnGame};
use kyusin_games::andar_bahar::{AndarBahar, Config};
use serde_json::{Value, json};

fn ab(cards: &[&str]) -> AndarBahar {
    AndarBahar::new(
        Config {
            kartu: Some(cards.iter().map(|c| c.to_string()).collect()),
        },
        [0; 32],
    )
    .unwrap()
}

fn act(g: &mut AndarBahar, cmd: &str) {
    Session::act(g, 0, cmd).unwrap_or_else(|e| panic!("`{cmd}`: {e:?}"));
}

fn view(g: &AndarBahar) -> Value {
    Session::view_data(g, 0)
}

fn legal(g: &AndarBahar) -> Vec<String> {
    TurnGame::legal_actions(g, 0)
        .iter()
        .map(|a| a.usage())
        .collect()
}

#[test]
fn cards_alternate_from_andar_until_the_rank_matches() {
    // Tengah 7h. Andar 2c, Bahar Kd, Andar 9s, Bahar 7c → Bahar menang.
    let mut g = ab(&["7h", "2c", "Kd", "9s", "7c", "7d"]);
    act(&mut g, "bet andar 100");
    act(&mut g, "bet bahar 50");
    act(&mut g, "deal");
    let v = view(&g);
    assert_eq!(v["tengah"], "7h");
    assert_eq!(v["andar"], json!(["2c", "9s"]));
    assert_eq!(v["bahar"], json!(["Kd", "7c"]));
    assert_eq!(v["pemenang"], "bahar");
    assert_eq!(v["bayar"]["andar"], -100);
    assert_eq!(v["bayar"]["bahar"], 50);
    assert_eq!(v["bersih"], -50);
    assert_eq!(v["dipertaruhkan"], 150);
    assert_eq!(v["ronde"], 1);
    assert_eq!(v["fase"], "selesai");
    assert_eq!(v["alasan"], "ronde_selesai");
    assert!(TurnGame::is_over(&g));
    assert_eq!(TurnGame::result(&g).unwrap().scores, vec![-50]);
}

#[test]
fn andar_pays_nine_to_ten() {
    // Kartu pertama langsung cocok: Andar menang.
    let mut g = ab(&["Qs", "Qh", "2c"]);
    act(&mut g, "bet andar 100");
    act(&mut g, "deal");
    let v = view(&g);
    assert_eq!(v["andar"], json!(["Qh"]));
    assert_eq!(v["bahar"], json!([]));
    assert_eq!(v["pemenang"], "andar");
    assert_eq!(v["bayar"]["andar"], 90);
    assert_eq!(v["bersih"], 90);
    // 10 chip: 9.
    let mut g = ab(&["Qs", "3d", "4d", "Qc"]);
    act(&mut g, "bet andar 10");
    act(&mut g, "deal");
    assert_eq!(view(&g)["pemenang"], "andar");
    assert_eq!(view(&g)["bayar"]["andar"], 9);
}

#[test]
fn bets_before_the_deal() {
    let mut g = ab(&["Qs", "Qh"]);
    assert!(!legal(&g).contains(&"deal".to_string()));
    assert!(legal(&g).contains(&"leave".to_string()));
    act(&mut g, "bet bahar 1990");
    assert!(Session::act(&mut g, 0, "bet bahar 20").is_err(), "lebih dari 2.000");
    act(&mut g, "bet bahar 10");
    assert_eq!(view(&g)["taruhan_meja"], 2000);
    for bad in ["bet andar 5", "bet tengah 10", "bet andar", "hit"] {
        assert!(Session::act(&mut g, 0, bad).is_err(), "{bad}");
    }
    act(&mut g, "clear");
    assert_eq!(view(&g)["taruhan_meja"], 0);
    act(&mut g, "leave");
    let v = view(&g);
    assert_eq!(v["fase"], "selesai");
    assert_eq!(v["alasan"], "berhenti");
    assert_eq!(v["ronde"], 0);
    assert_eq!(v["bersih"], 0);
}

#[test]
fn shuffled_deck_comes_from_the_round_seed() {
    let a = AndarBahar::new(Config::default(), [1; 32]).unwrap();
    let b = AndarBahar::new(Config::default(), [1; 32]).unwrap();
    let c = AndarBahar::new(Config::default(), [2; 32]).unwrap();
    assert_eq!(Session::state_hash(&a), Session::state_hash(&b));
    assert_ne!(Session::state_hash(&a), Session::state_hash(&c));
    // Satu dek: permainan selalu berakhir sebelum kartu habis.
    for seed in 0..50u8 {
        let mut g = AndarBahar::new(Config::default(), [seed; 32]).unwrap();
        act(&mut g, "bet andar 10");
        act(&mut g, "deal");
        let v = view(&g);
        let dealt = v["andar"].as_array().unwrap().len() + v["bahar"].as_array().unwrap().len();
        assert!((1..=49).contains(&dealt));
        assert!(v["pemenang"].is_string());
    }
}

#[test]
fn commands_round_trip_and_text_view() {
    let g = ab(&["Qs", "Qh"]);
    for cmd in ["bet andar 100", "bet bahar 20", "clear", "deal", "leave"] {
        let a = g.parse_command(cmd).unwrap();
        assert_eq!(g.format_action(&a), cmd);
    }
    assert!(Session::view_text(&g, 0, Lang::Id).contains("andar"));
    assert!(Session::view_text(&g, 0, Lang::En).contains("andar"));
}
