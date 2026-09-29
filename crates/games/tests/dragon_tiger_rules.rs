//! Tes aturan dan pembayaran Dragon Tiger (SPEC §6.3, §7; M5a), ditulis
//! sebelum mesinnya.
//!
//! Aturan meja (varian paling umum, lihat DECISIONS): 8 dek dalam shoe,
//! titik potong 75%. Satu kartu untuk Naga (dragon) lalu satu untuk Macan
//! (tiger); kartu lebih tinggi menang, as terendah (A < 2 < … < K), jenis
//! tidak berpengaruh. Taruhan: `dragon`, `tiger` (1:1), `tie` (8:1). Bila
//! seri, taruhan dragon/tiger kalah setengah. Tiap tempat taruhan 10–2.000
//! chip, kelipatan 10; taruhan disusun dulu (`bet <tempat> <jumlah>`, boleh
//! beberapa kali sampai batas), `clear` menarik semuanya, `deal` membagi.
//! `leave` di antara ronde mengakhiri shoe (taruhan yang belum dibagi
//! kembali).

use kyusin_core::{GameError, Lang, Session, TurnGame};
use kyusin_games::dragon_tiger::{Config, DragonTiger};
use serde_json::Value;

fn dt(cards: &[&str], cut: usize) -> DragonTiger {
    DragonTiger::new(
        Config {
            dek: None,
            shoe: Some(cards.iter().map(|c| c.to_string()).collect()),
            potong: Some(cut),
        },
        [0; 32],
    )
    .unwrap()
}

fn act(g: &mut DragonTiger, cmd: &str) {
    Session::act(g, 0, cmd).unwrap_or_else(|e| panic!("`{cmd}`: {e:?}"));
}

fn view(g: &DragonTiger) -> Value {
    Session::view_data(g, 0)
}

fn legal(g: &DragonTiger) -> Vec<String> {
    TurnGame::legal_actions(g, 0)
        .iter()
        .map(|a| a.usage())
        .collect()
}

fn net(g: &DragonTiger) -> i64 {
    view(g)["bersih"].as_i64().unwrap()
}

#[test]
fn default_shoe_is_eight_decks_cut_at_75_percent() {
    let g = DragonTiger::new(Config::default(), [7; 32]).unwrap();
    let v = view(&g);
    assert_eq!(v["sisa"], 416);
    assert_eq!(v["potong"], 312);
    assert_eq!(v["fase"], "taruhan");
    assert_eq!(v["min_taruhan"], 10);
    assert_eq!(v["maks_taruhan"], 2000);
    assert_eq!(v["langkah_taruhan"], 10);
    // Seed berbeda → urutan berbeda; seed sama → sama.
    let a = DragonTiger::new(Config::default(), [7; 32]).unwrap();
    let b = DragonTiger::new(Config::default(), [8; 32]).unwrap();
    assert_eq!(Session::state_hash(&g), Session::state_hash(&a));
    assert_ne!(Session::state_hash(&g), Session::state_hash(&b));
}

#[test]
fn higher_card_wins_even_money_ace_is_low() {
    // Naga K, Macan A: naga menang.
    let mut g = dt(&["Kh", "Ac", "2d", "5s", "9h", "9c"], 100);
    act(&mut g, "bet dragon 100");
    act(&mut g, "bet tiger 50");
    act(&mut g, "deal");
    let v = view(&g);
    assert_eq!(v["naga"], "Kh");
    assert_eq!(v["macan"], "Ac");
    assert_eq!(v["pemenang"], "naga");
    assert_eq!(v["bayar"]["dragon"], 100);
    assert_eq!(v["bayar"]["tiger"], -50);
    assert_eq!(net(&g), 50);
    assert_eq!(v["dipertaruhkan"], 150);
    assert_eq!(v["fase"], "taruhan");
    assert_eq!(v["ronde"], 1);
    // Naga 2, Macan 5: macan menang (as terendah terlihat di ronde 1).
    act(&mut g, "bet tiger 10");
    act(&mut g, "deal");
    assert_eq!(view(&g)["pemenang"], "macan");
    assert_eq!(net(&g), 60);
}

#[test]
fn tie_pays_eight_to_one_and_costs_half_of_dragon_and_tiger() {
    // Jenis tidak berpengaruh: 9h lawan 9c seri.
    let mut g = dt(&["9h", "9c", "2d", "3s"], 100);
    act(&mut g, "bet dragon 100");
    act(&mut g, "bet tiger 20");
    act(&mut g, "bet tie 10");
    act(&mut g, "deal");
    let v = view(&g);
    assert_eq!(v["pemenang"], "seri");
    assert_eq!(v["bayar"]["dragon"], -50);
    assert_eq!(v["bayar"]["tiger"], -10);
    assert_eq!(v["bayar"]["tie"], 80);
    assert_eq!(net(&g), 20);
    // Tie kalah bila bukan seri.
    act(&mut g, "bet tie 50");
    act(&mut g, "deal");
    assert_eq!(view(&g)["bayar"]["tie"], -50);
    assert_eq!(net(&g), -30);
}

#[test]
fn bets_are_built_up_and_limited_per_spot() {
    let mut g = dt(&["9h", "8c", "2d", "3s"], 100);
    assert!(legal(&g).contains(&"leave".to_string()));
    assert!(
        !legal(&g).contains(&"deal".to_string()),
        "deal butuh taruhan"
    );
    act(&mut g, "bet dragon 1500");
    act(&mut g, "bet dragon 500");
    // Tempat dragon penuh (2.000): tidak bisa ditambah.
    let err = Session::act(&mut g, 0, "bet dragon 10").unwrap_err();
    assert!(matches!(err, GameError::Illegal(_)), "{err:?}");
    for bad in [
        "bet tiger 5",
        "bet tiger 15",
        "bet tiger 2010",
        "bet naga 10",
        "bet 10",
    ] {
        assert!(Session::act(&mut g, 0, bad).is_err(), "{bad}");
    }
    let v = view(&g);
    assert_eq!(v["taruhan"]["dragon"], 2000);
    assert_eq!(
        v["taruhan_meja"], 2000,
        "taruhan yang dipasang ikut dipertaruhkan"
    );
    act(&mut g, "clear");
    assert_eq!(view(&g)["taruhan_meja"], 0);
    assert!(!legal(&g).contains(&"clear".to_string()));
    act(&mut g, "bet tie 10");
    assert!(legal(&g).contains(&"deal".to_string()));
    assert_eq!(view(&g)["ronde"], 0, "ronde baru dihitung saat dibagi");
}

#[test]
fn shoe_ends_at_the_cut_and_leave_returns_undealt_bets() {
    let mut g = dt(&["Kh", "2c", "Qh", "3c", "4h", "5c"], 3);
    act(&mut g, "bet dragon 10");
    act(&mut g, "deal");
    assert_eq!(view(&g)["fase"], "taruhan", "2 kartu < potong 3");
    act(&mut g, "bet dragon 10");
    act(&mut g, "deal");
    let v = view(&g);
    assert_eq!(v["fase"], "selesai");
    assert_eq!(v["alasan"], "shoe_habis");
    assert!(TurnGame::is_over(&g));
    assert_eq!(TurnGame::result(&g).unwrap().scores, vec![20]);

    let mut g = dt(&["Kh", "2c"], 100);
    act(&mut g, "bet dragon 100");
    act(&mut g, "leave");
    let v = view(&g);
    assert_eq!(v["alasan"], "berhenti");
    assert_eq!(v["taruhan_meja"], 0);
    assert_eq!(v["bersih"], 0);
    assert!(TurnGame::pending_players(&g).is_empty());
}

#[test]
fn commands_round_trip_and_text_view() {
    let g = dt(&["Kh", "2c"], 100);
    for cmd in [
        "bet dragon 100",
        "bet tiger 20",
        "bet tie 10",
        "clear",
        "deal",
        "leave",
    ] {
        let a = g.parse_command(cmd).unwrap();
        assert_eq!(g.format_action(&a), cmd);
    }
    assert!(g.parse_command("bet dragon").is_err());
    assert!(g.parse_command("bet dragon x").is_err());
    let text = Session::view_text(&g, 0, Lang::En);
    assert!(text.contains("dragon"), "{text}");
    assert!(Session::view_text(&g, 0, Lang::Id).contains("Naga"));
}
