//! Tes aturan dan pembayaran Red Dog (SPEC §6.3, §7; M5a), ditulis sebelum
//! mesinnya.
//!
//! Aturan meja (varian paling umum, lihat DECISIONS): 8 dek dalam shoe,
//! titik potong 75%, as tertinggi, jenis tidak berpengaruh. `bet <jumlah>`
//! (10–2.000, kelipatan 10) langsung membagi dua kartu. Berurutan (mis. 7-8):
//! seri, taruhan kembali. Sepasang: kartu ketiga dibagi; bila seperingkat
//! (three of a kind) dibayar 11:1, bila tidak seri. Selain itu jarak
//! (spread) = jumlah peringkat di antara dua kartu; pemain boleh `raise`
//! (menggandakan taruhan) atau `call`, lalu kartu ketiga dibagi. Kartu
//! ketiga di antara keduanya menang: spread 1 dibayar 5:1, 2 dibayar 4:1,
//! 3 dibayar 2:1, 4–11 dibayar 1:1; selain itu kalah.

use kyusin_core::{Lang, Session, TurnGame};
use kyusin_games::red_dog::{Config, RedDog};
use serde_json::{Value, json};

fn rd(cards: &[&str], cut: usize) -> RedDog {
    RedDog::new(
        Config {
            dek: None,
            shoe: Some(cards.iter().map(|c| c.to_string()).collect()),
            potong: Some(cut),
        },
        [0; 32],
    )
    .unwrap()
}

fn act(g: &mut RedDog, cmd: &str) {
    Session::act(g, 0, cmd).unwrap_or_else(|e| panic!("`{cmd}`: {e:?}"));
}

fn view(g: &RedDog) -> Value {
    Session::view_data(g, 0)
}

fn legal(g: &RedDog) -> Vec<String> {
    TurnGame::legal_actions(g, 0)
        .iter()
        .map(|a| a.usage())
        .collect()
}

#[test]
fn default_shoe_is_eight_decks() {
    let g = RedDog::new(Config::default(), [9; 32]).unwrap();
    assert_eq!(view(&g)["sisa"], 416);
    assert_eq!(view(&g)["potong"], 312);
}

#[test]
fn consecutive_cards_push_at_once() {
    let mut g = rd(&["7h", "8c", "2d", "3s"], 100);
    act(&mut g, "bet 100");
    let v = view(&g);
    assert_eq!(v["kartu"], json!(["7h", "8c"]));
    assert_eq!(v["hasil"], "seri");
    assert_eq!(v["bayar"], 0);
    assert_eq!(v["fase"], "taruhan");
    assert_eq!(v["dipertaruhkan"], 100);
    assert_eq!(v["ronde"], 1);
}

#[test]
fn pairs_draw_a_third_card_trips_pay_eleven() {
    let mut g = rd(&["9h", "9c", "9d", "Kh", "Kc", "2s"], 100);
    act(&mut g, "bet 50");
    let v = view(&g);
    assert_eq!(v["kartu"], json!(["9h", "9c", "9d"]));
    assert_eq!(v["hasil"], "tiga");
    assert_eq!(v["bayar"], 550);
    act(&mut g, "bet 50");
    let v = view(&g);
    assert_eq!(v["kartu"], json!(["Kh", "Kc", "2s"]));
    assert_eq!(v["hasil"], "seri");
    assert_eq!(v["bayar"], 0);
    assert_eq!(v["bersih"], 550);
}

#[test]
fn spread_payouts_and_raise() {
    // (dua kartu, kartu ketiga, raise?, bayar untuk taruhan 100)
    let cases: [(&str, &str, &str, bool, i64); 8] = [
        ("5h", "7c", "6d", false, 500),  // spread 1
        ("5h", "8c", "6d", false, 400),  // spread 2
        ("5h", "9c", "8d", false, 200),  // spread 3
        ("5h", "Tc", "7d", false, 100),  // spread 4
        ("2h", "Ac", "Kd", true, 200),   // spread 11, raise
        ("5h", "7c", "7d", false, -100), // sama dengan tepi: kalah
        ("5h", "Tc", "Jd", true, -200),  // di luar, raise
        ("Kh", "4c", "Qd", false, 100),  // urutan kartu tidak berpengaruh
    ];
    for (a, b, third, raise, pay) in cases {
        let mut g = rd(&[a, b, third, "2c", "2d"], 100);
        act(&mut g, "bet 100");
        let v = view(&g);
        assert_eq!(v["fase"], "naikkan", "{a} {b}");
        assert_eq!(v["biaya"]["raise"], 100);
        assert_eq!(v["netral"], "call");
        assert_eq!(v["taruhan_meja"], 100);
        assert_eq!(legal(&g), vec!["raise", "call"]);
        act(&mut g, if raise { "raise" } else { "call" });
        let v = view(&g);
        assert_eq!(v["bayar"], pay, "{a} {b} {third}");
        assert_eq!(v["kartu"].as_array().unwrap().len(), 3);
        assert_eq!(v["dipertaruhkan"], if raise { 200 } else { 100 });
    }
    let v = {
        let mut g = rd(&["5h", "7c", "6d"], 100);
        act(&mut g, "bet 100");
        view(&g)
    };
    assert_eq!(v["jarak"], 1);
    assert_eq!(v["kali"], 5);
}

#[test]
fn bets_limits_leave_and_shoe_end() {
    let mut g = rd(&["7h", "8c", "2d", "3s"], 2);
    for bad in ["bet 5", "bet 2010", "raise", "call", "bet dragon 10"] {
        assert!(Session::act(&mut g, 0, bad).is_err(), "{bad}");
    }
    act(&mut g, "bet 10");
    let v = view(&g);
    assert_eq!(v["fase"], "selesai");
    assert_eq!(v["alasan"], "shoe_habis");
    let mut g = rd(&["7h", "8c"], 100);
    act(&mut g, "leave");
    assert!(TurnGame::is_over(&g));
    assert_eq!(view(&g)["alasan"], "berhenti");
}

#[test]
fn commands_round_trip_and_text_view() {
    let g = rd(&["7h", "8c"], 100);
    for cmd in ["bet 100", "raise", "call", "leave"] {
        let a = g.parse_command(cmd).unwrap();
        assert_eq!(g.format_action(&a), cmd);
    }
    assert!(Session::view_text(&g, 0, Lang::Id).contains("bet"));
    assert!(Session::view_text(&g, 0, Lang::En).contains("bet"));
}
