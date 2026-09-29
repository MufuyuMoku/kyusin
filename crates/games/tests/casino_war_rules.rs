//! Tes aturan dan pembayaran Casino War (SPEC §6.3, §7; M5a), ditulis
//! sebelum mesinnya.
//!
//! Aturan meja (varian paling umum, lihat DECISIONS): 6 dek dalam shoe,
//! titik potong 75%, as tertinggi. Taruhan `ante` (wajib) dan `tie`
//! (opsional, 10:1 bila kartu pertama seri), masing-masing 10–2.000,
//! kelipatan 10. Satu kartu untuk pemain lalu satu untuk bandar; lebih
//! tinggi menang 1:1. Bila seri, pemain memilih `surrender` (kehilangan
//! setengah ante) atau `war` (menambah raise sebesar ante): bandar membuang
//! tiga kartu, lalu satu kartu untuk pemain dan satu untuk bandar. Kartu
//! pemain lebih tinggi atau sama: raise dibayar 1:1 dan ante kembali (bersih
//! +ante); kartu bandar lebih tinggi: ante dan raise kalah.

use kyusin_core::{Lang, Session, TurnGame};
use kyusin_games::casino_war::{CasinoWar, Config};
use serde_json::Value;

fn war(cards: &[&str], cut: usize) -> CasinoWar {
    CasinoWar::new(
        Config {
            dek: None,
            shoe: Some(cards.iter().map(|c| c.to_string()).collect()),
            potong: Some(cut),
        },
        [0; 32],
    )
    .unwrap()
}

fn act(g: &mut CasinoWar, cmd: &str) {
    Session::act(g, 0, cmd).unwrap_or_else(|e| panic!("`{cmd}`: {e:?}"));
}

fn view(g: &CasinoWar) -> Value {
    Session::view_data(g, 0)
}

fn legal(g: &CasinoWar) -> Vec<String> {
    TurnGame::legal_actions(g, 0)
        .iter()
        .map(|a| a.usage())
        .collect()
}

fn net(g: &CasinoWar) -> i64 {
    view(g)["bersih"].as_i64().unwrap()
}

#[test]
fn default_shoe_is_six_decks() {
    let g = CasinoWar::new(Config::default(), [3; 32]).unwrap();
    let v = view(&g);
    assert_eq!(v["sisa"], 312);
    assert_eq!(v["potong"], 234);
    assert_eq!(v["fase"], "taruhan");
}

#[test]
fn higher_card_wins_even_money_ace_high() {
    let mut g = war(&["Ah", "Kc", "2d", "3s", "9h", "9c"], 100);
    act(&mut g, "bet ante 100");
    act(&mut g, "deal");
    let v = view(&g);
    assert_eq!(v["pemain"], serde_json::json!(["Ah"]));
    assert_eq!(v["bandar"], serde_json::json!(["Kc"]));
    assert_eq!(v["bayar"]["ante"], 100);
    assert_eq!(net(&g), 100);
    assert_eq!(v["fase"], "taruhan");
    assert_eq!(v["dipertaruhkan"], 100);
    act(&mut g, "bet ante 50");
    act(&mut g, "deal");
    assert_eq!(view(&g)["bayar"]["ante"], -50);
    assert_eq!(net(&g), 50);
}

#[test]
fn tie_bet_pays_ten_to_one_on_the_first_tie_only() {
    let mut g = war(&["7h", "7c", "Kd", "2s", "9h", "5c", "Jh", "Qc"], 100);
    act(&mut g, "bet ante 100");
    act(&mut g, "bet tie 10");
    assert_eq!(view(&g)["taruhan_meja"], 110);
    act(&mut g, "deal");
    let v = view(&g);
    assert_eq!(v["fase"], "perang");
    assert_eq!(v["bayar"]["tie"], 100);
    assert_eq!(v["netral"], "surrender");
    assert_eq!(v["biaya"]["war"], 100);
    assert_eq!(v["taruhan_meja"], 100, "ante masih dipertaruhkan");
    assert_eq!(legal(&g), vec!["war", "surrender"]);
    act(&mut g, "surrender");
    let v = view(&g);
    assert_eq!(v["bayar"]["ante"], -50);
    assert_eq!(net(&g), 50);
    assert_eq!(v["dipertaruhkan"], 110);
    // Tie kalah bila bukan seri.
    act(&mut g, "bet ante 10");
    act(&mut g, "bet tie 10");
    act(&mut g, "deal");
    assert_eq!(view(&g)["bayar"]["tie"], -10);
}

#[test]
fn going_to_war() {
    // Seri 8-8, buang 3, pemain K lawan bandar 4: bersih +ante.
    let mut g = war(&["8h", "8c", "2d", "3d", "4d", "Kh", "4c", "5h", "5c", "2h", "2s", "6s", "Ah", "3c", "Td", "Th", "Tc", "9s", "9c", "Ac", "Jd"], 100);
    act(&mut g, "bet ante 100");
    act(&mut g, "deal");
    act(&mut g, "war");
    let v = view(&g);
    assert_eq!(v["pemain"], serde_json::json!(["8h", "Kh"]));
    assert_eq!(v["bandar"], serde_json::json!(["8c", "4c"]));
    assert_eq!(v["dibuang"], 3);
    assert_eq!(v["bayar"]["ante"], 0);
    assert_eq!(v["bayar"]["raise"], 100);
    assert_eq!(net(&g), 100);
    assert_eq!(v["dipertaruhkan"], 200);
    // Seri 5-5, buang 3 (2h 2s 6s), pemain A lawan bandar 3: menang lagi.
    act(&mut g, "bet ante 10");
    act(&mut g, "deal");
    act(&mut g, "war");
    assert_eq!(net(&g), 110);
    // Seri T-T, buang 3 (Tc 9s 9c), pemain A lawan bandar J.
    act(&mut g, "bet ante 20");
    act(&mut g, "deal");
    assert_eq!(view(&g)["fase"], "perang");
    act(&mut g, "war");
    assert_eq!(net(&g), 130);
}

#[test]
fn war_tie_wins_the_raise_and_dealer_high_loses_both() {
    // Seri 6-6, buang 3, 9 lawan 9: seri lagi → bersih +ante.
    let mut g = war(&["6h", "6c", "2d", "3d", "4d", "9h", "9c", "Qh", "Qc", "2h", "3h", "4h", "5h", "Kc"], 100);
    act(&mut g, "bet ante 100");
    act(&mut g, "deal");
    act(&mut g, "war");
    let v = view(&g);
    assert_eq!(v["bayar"]["ante"], 0);
    assert_eq!(v["bayar"]["raise"], 100);
    assert_eq!(net(&g), 100);
    // Seri Q-Q, buang 3, 5 lawan K: kalah ante + raise.
    act(&mut g, "bet ante 100");
    act(&mut g, "deal");
    act(&mut g, "war");
    let v = view(&g);
    assert_eq!(v["bayar"]["ante"], -100);
    assert_eq!(v["bayar"]["raise"], -100);
    assert_eq!(net(&g), -100);
}

#[test]
fn betting_rules_and_leave() {
    let mut g = war(&["Kh", "2c", "3h", "4c"], 100);
    assert!(!legal(&g).contains(&"deal".to_string()));
    act(&mut g, "bet tie 10");
    assert!(!legal(&g).contains(&"deal".to_string()), "deal butuh ante");
    act(&mut g, "bet ante 10");
    assert!(legal(&g).contains(&"deal".to_string()));
    act(&mut g, "clear");
    assert_eq!(view(&g)["taruhan_meja"], 0);
    for bad in ["bet ante 5", "bet ante 2010", "bet raise 10", "war"] {
        assert!(Session::act(&mut g, 0, bad).is_err(), "{bad}");
    }
    act(&mut g, "bet ante 100");
    act(&mut g, "leave");
    let v = view(&g);
    assert_eq!(v["fase"], "selesai");
    assert_eq!(v["alasan"], "berhenti");
    assert_eq!(v["taruhan_meja"], 0);
    assert_eq!(v["bersih"], 0);
}

#[test]
fn shoe_ends_at_the_cut_after_the_round() {
    // Potong 3: ronde pertama seri lalu perang memakai 7 kartu.
    let mut g = war(&["8h", "8c", "2d", "3d", "4d", "Kh", "4c", "5h", "5c"], 3);
    act(&mut g, "bet ante 10");
    act(&mut g, "deal");
    assert_eq!(view(&g)["fase"], "perang");
    act(&mut g, "war");
    let v = view(&g);
    assert_eq!(v["fase"], "selesai");
    assert_eq!(v["alasan"], "shoe_habis");
    assert_eq!(TurnGame::result(&g).unwrap().scores, vec![10]);
}

#[test]
fn commands_round_trip_and_text_view() {
    let g = war(&["Kh", "2c"], 100);
    for cmd in ["bet ante 100", "bet tie 20", "clear", "deal", "war", "surrender", "leave"] {
        let a = g.parse_command(cmd).unwrap();
        assert_eq!(g.format_action(&a), cmd);
    }
    assert!(g.parse_command("war now").is_err());
    assert!(Session::view_text(&g, 0, Lang::Id).contains("ante"));
    assert!(Session::view_text(&g, 0, Lang::En).contains("ante"));
}
