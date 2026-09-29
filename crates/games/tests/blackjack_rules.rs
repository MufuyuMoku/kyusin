//! Tes aturan dan pembayaran Blackjack (SPEC §6.3, §7; D-056), ditulis
//! sebelum mesinnya.
//!
//! Aturan meja: 6 dek; kartu dibagi pemain, bandar (terbuka), pemain,
//! bandar (tertutup). Taruhan 10–2.000 chip, kelipatan 10. Blackjack
//! dibayar 3:2. Bandar mengintip hole card bila kartu terbukanya as atau
//! bernilai 10; bila blackjack, ronde langsung selesai. Bila kartu terbuka
//! as, insurance ditawarkan lebih dulu (setengah taruhan, dibayar 2:1).
//! Bandar berdiri di semua 17 termasuk soft 17 (S17). Double di dua kartu
//! pertama mana pun, termasuk setelah split; satu kartu lalu tangan
//! selesai. Split pasangan peringkat sama sampai 4 tangan; tangan hasil
//! split menerima kartu keduanya saat mulai dimainkan. As yang di-split
//! hanya mendapat satu kartu masing-masing dan tidak bisa di-split lagi;
//! 21 setelah split bukan blackjack. Late surrender: hanya aksi pertama
//! pada dua kartu awal (bukan setelah split), mengembalikan setengah
//! taruhan. Bila semua tangan pemain bust atau menyerah, bandar tidak
//! mengambil kartu. Shoe selesai di akhir ronde bila kartu terpakai sudah
//! mencapai titik potong (bawaan 75%), atau saat pemain `leave` di antara
//! ronde.

use kyusin_core::{ActionSpec, GameError, GameRng, Lang, ParamKind, Session, TurnGame};
use kyusin_games::blackjack::{Action, Blackjack, Config};
use serde_json::Value;

fn bj(cards: &[&str], cut: usize) -> Blackjack {
    Blackjack::new(
        Config {
            dek: None,
            shoe: Some(cards.iter().map(|c| c.to_string()).collect()),
            potong: Some(cut),
        },
        [0; 32],
    )
    .unwrap()
}

fn fresh(seed: u8) -> Blackjack {
    Blackjack::new(Config::default(), [seed; 32]).unwrap()
}

fn act(g: &mut Blackjack, cmd: &str) {
    Session::act(g, 0, cmd).unwrap_or_else(|e| panic!("`{cmd}`: {e:?}"));
}

fn view(g: &Blackjack) -> Value {
    Session::view_data(g, 0)
}

fn legal(g: &Blackjack) -> Vec<String> {
    TurnGame::legal_actions(g, 0)
        .iter()
        .map(|a| a.usage())
        .collect()
}

fn net(g: &Blackjack) -> i64 {
    view(g)["bersih"].as_i64().unwrap()
}

#[test]
fn betting_limits_and_legal_actions() {
    let mut g = fresh(1);
    assert_eq!(view(&g)["fase"], "taruhan");
    let specs = TurnGame::legal_actions(&g, 0);
    let bet = specs
        .iter()
        .find(|s| s.verb() == "bet")
        .expect("ada `bet <jumlah>`");
    match bet {
        ActionSpec::Template { params, .. } => assert_eq!(
            params[0].kind,
            ParamKind::Int {
                min: 10,
                max: 2000,
                step: 10
            }
        ),
        other => panic!("{other:?}"),
    }
    assert!(legal(&g).contains(&"leave".to_string()));
    for bad in ["bet 5", "bet 2010", "bet 15", "bet 0", "hit", "stand"] {
        assert!(Session::act(&mut g, 0, bad).is_err(), "{bad}");
    }
    assert_eq!(view(&g)["min_taruhan"], 10);
    assert_eq!(view(&g)["maks_taruhan"], 2000);
    act(&mut g, "bet 2000");
    assert_eq!(view(&g)["tangan"][0]["taruhan"], 2000);
}

#[test]
fn natural_blackjack_pays_three_to_two() {
    let mut g = bj(&["Ah", "9c", "Kd", "7s"], 1000);
    act(&mut g, "bet 100");
    let v = view(&g);
    assert_eq!(v["fase"], "taruhan", "ronde langsung selesai");
    assert_eq!(v["tangan"][0]["blackjack"], true);
    assert_eq!(v["tangan"][0]["hasil"], "menang");
    assert_eq!(v["tangan"][0]["bayar"], 150);
    assert_eq!(net(&g), 150);
    // Bandar tidak mengambil kartu; hole card dibuka.
    assert_eq!(v["bandar"], serde_json::json!(["9c", "7s"]));
    assert_eq!(v["bandar_nilai"], 16);
    assert_eq!(v["ronde"], 1);
}

#[test]
fn dealer_ten_up_peeks_and_blackjack_ends_the_round() {
    let mut g = bj(&["9h", "Kc", "7d", "Ah"], 1000);
    act(&mut g, "bet 100");
    let v = view(&g);
    assert_eq!(v["fase"], "taruhan", "tanpa insurance untuk kartu 10");
    assert_eq!(v["tangan"][0]["hasil"], "kalah");
    assert_eq!(net(&g), -100);
    assert_eq!(v["bandar"], serde_json::json!(["Kc", "Ah"]));
}

#[test]
fn ace_up_offers_insurance_first() {
    let mut g = bj(&["Th", "Ad", "9s", "7c"], 1000);
    act(&mut g, "bet 100");
    let v = view(&g);
    assert_eq!(v["fase"], "asuransi");
    assert_eq!(
        v["bandar"],
        serde_json::json!(["Ad", "??"]),
        "hole card tersembunyi"
    );
    let mut l = legal(&g);
    l.sort();
    assert_eq!(l, vec!["decline", "insure"]);
    assert_eq!(v["biaya"]["insure"], 50);
    assert_eq!(v["taruhan_meja"], 100);
    act(&mut g, "insure");
    // Tidak blackjack: insurance kalah, permainan lanjut.
    let v = view(&g);
    assert_eq!(v["fase"], "giliran");
    assert_eq!(v["asuransi"], 50);
    assert_eq!(v["taruhan_meja"], 150);
    act(&mut g, "stand");
    // Bandar A7 = soft 18, berdiri; pemain 19 menang.
    let v = view(&g);
    assert_eq!(v["tangan"][0]["hasil"], "menang");
    assert_eq!(v["asuransi_bayar"], -50);
    assert_eq!(net(&g), 50);
}

#[test]
fn insurance_pays_two_to_one_against_dealer_blackjack() {
    let mut g = bj(&["Th", "Ad", "9s", "Kc"], 1000);
    act(&mut g, "bet 100");
    act(&mut g, "insure");
    let v = view(&g);
    assert_eq!(v["fase"], "taruhan");
    assert_eq!(v["asuransi_bayar"], 100);
    assert_eq!(v["tangan"][0]["hasil"], "kalah");
    assert_eq!(net(&g), 0);

    let mut g = bj(&["Th", "Ad", "9s", "Kc"], 1000);
    act(&mut g, "bet 100");
    act(&mut g, "decline");
    assert_eq!(net(&g), -100);
}

#[test]
fn both_blackjack_is_a_push() {
    let mut g = bj(&["Ah", "As", "Kd", "Kc"], 1000);
    act(&mut g, "bet 100");
    assert_eq!(view(&g)["fase"], "asuransi");
    act(&mut g, "decline");
    let v = view(&g);
    assert_eq!(v["tangan"][0]["hasil"], "seri");
    assert_eq!(v["tangan"][0]["bayar"], 0);
    assert_eq!(net(&g), 0);
}

#[test]
fn player_blackjack_against_dealer_ace_without_blackjack_pays_after_insurance() {
    let mut g = bj(&["Ah", "As", "Kd", "7c"], 1000);
    act(&mut g, "bet 100");
    act(&mut g, "decline");
    let v = view(&g);
    assert_eq!(v["fase"], "taruhan");
    assert_eq!(v["tangan"][0]["bayar"], 150);
}

#[test]
fn dealer_stands_on_soft_17() {
    let mut g = bj(&["Th", "6d", "9s", "Ac"], 1000);
    act(&mut g, "bet 100");
    act(&mut g, "stand");
    let v = view(&g);
    assert_eq!(v["bandar"].as_array().unwrap().len(), 2);
    assert_eq!(v["bandar_nilai"], 17);
    assert_eq!(net(&g), 100);
}

#[test]
fn dealer_hits_16_and_soft_16() {
    let mut g = bj(&["Th", "6d", "9s", "Tc", "5h"], 1000);
    act(&mut g, "bet 100");
    act(&mut g, "stand");
    let v = view(&g);
    assert_eq!(v["bandar"].as_array().unwrap().len(), 3);
    assert_eq!(v["bandar_nilai"], 21);
    assert_eq!(net(&g), -100);

    // 5 + A = soft 16 → ambil 2 → soft 18, berdiri.
    let mut g = bj(&["Th", "5d", "9s", "Ac", "2h", "Kh"], 1000);
    act(&mut g, "bet 100");
    act(&mut g, "stand");
    let v = view(&g);
    assert_eq!(v["bandar"], serde_json::json!(["5d", "Ac", "2h"]));
    assert_eq!(v["bandar_nilai"], 18);
    assert_eq!(v["tangan"][0]["hasil"], "menang");
}

#[test]
fn hand_values_count_aces_as_one_or_eleven() {
    let mut g = bj(&["Ah", "9d", "5s", "8c", "Ac", "Kh"], 1000);
    act(&mut g, "bet 100");
    let v = view(&g);
    assert_eq!(v["tangan"][0]["nilai"], 16);
    assert_eq!(v["tangan"][0]["lunak"], true);
    act(&mut g, "hit"); // A 5 A = soft 17
    let v = view(&g);
    assert_eq!(v["tangan"][0]["nilai"], 17);
    assert_eq!(v["tangan"][0]["lunak"], true);
    act(&mut g, "hit"); // + K = 17 keras
    let v = view(&g);
    assert_eq!(v["tangan"][0]["nilai"], 17);
    assert_eq!(v["tangan"][0]["lunak"], false);
}

#[test]
fn bust_loses_and_dealer_does_not_draw() {
    let mut g = bj(&["Th", "6d", "6s", "Tc", "9h"], 1000);
    act(&mut g, "bet 100");
    act(&mut g, "hit");
    let v = view(&g);
    assert_eq!(v["fase"], "taruhan");
    assert_eq!(v["tangan"][0]["nilai"], 25);
    assert_eq!(v["tangan"][0]["hasil"], "kalah");
    assert_eq!(v["bandar"], serde_json::json!(["6d", "Tc"]));
    assert_eq!(net(&g), -100);
}

#[test]
fn double_doubles_the_bet_and_takes_exactly_one_card() {
    let mut g = bj(&["6h", "9d", "5s", "7c", "Th", "Kd"], 1000);
    act(&mut g, "bet 100");
    assert_eq!(view(&g)["biaya"]["double"], 100);
    act(&mut g, "double");
    let v = view(&g);
    assert_eq!(v["tangan"][0]["taruhan"], 200);
    assert_eq!(v["tangan"][0]["ganda"], true);
    assert_eq!(v["tangan"][0]["kartu"].as_array().unwrap().len(), 3);
    // Bandar 16 + K = bust.
    assert_eq!(v["tangan"][0]["bayar"], 200);
    assert_eq!(net(&g), 200);
}

#[test]
fn double_is_allowed_on_any_first_two_cards_only() {
    let mut g = bj(&["Th", "6d", "8s", "Tc", "2h"], 1000);
    act(&mut g, "bet 100");
    assert!(legal(&g).contains(&"double".to_string()), "hard 18");
    act(&mut g, "hit");
    assert!(!legal(&g).contains(&"double".to_string()), "setelah hit");
}

#[test]
fn split_plays_two_hands_with_double_after_split() {
    let mut g = bj(&["8h", "6d", "8s", "Tc", "3h", "2c", "Td", "9s"], 1000);
    act(&mut g, "bet 100");
    assert!(legal(&g).contains(&"split".to_string()));
    assert_eq!(view(&g)["biaya"]["split"], 100);
    act(&mut g, "split");
    let v = view(&g);
    assert_eq!(v["tangan"].as_array().unwrap().len(), 2);
    assert_eq!(v["aktif"], 0);
    assert_eq!(v["tangan"][0]["kartu"], serde_json::json!(["8h", "3h"]));
    assert_eq!(v["tangan"][1]["kartu"], serde_json::json!(["8s"]));
    assert!(
        !legal(&g).contains(&"surrender".to_string()),
        "bukan setelah split"
    );
    act(&mut g, "double"); // 8 3 + 2 = 13
    let v = view(&g);
    assert_eq!(v["aktif"], 1);
    assert_eq!(v["tangan"][1]["kartu"], serde_json::json!(["8s", "Td"]));
    act(&mut g, "stand");
    // Bandar 16 + 9 = bust: kedua tangan menang.
    let v = view(&g);
    assert_eq!(v["tangan"][0]["bayar"], 200);
    assert_eq!(v["tangan"][1]["bayar"], 100);
    assert_eq!(net(&g), 300);
}

#[test]
fn resplit_up_to_four_hands() {
    let mut g = bj(
        &[
            "8h", "6d", "8s", "Tc", "8d", "8c", "8h", "2s", "3s", "4s", "9s",
        ],
        1000,
    );
    act(&mut g, "bet 100");
    act(&mut g, "split");
    act(&mut g, "split");
    act(&mut g, "split");
    let v = view(&g);
    assert_eq!(v["tangan"].as_array().unwrap().len(), 4);
    assert_eq!(v["tangan"][0]["kartu"], serde_json::json!(["8h", "8h"]));
    assert!(
        !legal(&g).contains(&"split".to_string()),
        "paling banyak 4 tangan"
    );
    assert_eq!(v["taruhan_meja"], 400);
    for _ in 0..4 {
        act(&mut g, "stand");
    }
    let v = view(&g);
    let order: Vec<Value> = v["tangan"]
        .as_array()
        .unwrap()
        .iter()
        .map(|h| h["kartu"].clone())
        .collect();
    assert_eq!(
        order,
        vec![
            serde_json::json!(["8h", "8h"]),
            serde_json::json!(["8c", "2s"]),
            serde_json::json!(["8d", "3s"]),
            serde_json::json!(["8s", "4s"]),
        ]
    );
    assert_eq!(net(&g), 400);
}

#[test]
fn split_aces_get_one_card_each_and_21_is_not_blackjack() {
    let mut g = bj(&["Ah", "6d", "As", "Tc", "Kh", "Ad", "9s"], 1000);
    act(&mut g, "bet 100");
    act(&mut g, "split");
    let v = view(&g);
    assert_eq!(v["fase"], "taruhan", "tangan as selesai sendiri");
    assert_eq!(v["tangan"][0]["kartu"], serde_json::json!(["Ah", "Kh"]));
    assert_eq!(v["tangan"][0]["blackjack"], false);
    assert_eq!(v["tangan"][0]["bayar"], 100, "21 setelah split dibayar 1:1");
    assert_eq!(v["tangan"][1]["kartu"], serde_json::json!(["As", "Ad"]));
    assert_eq!(v["tangan"][1]["bayar"], 100);
    assert_eq!(net(&g), 200);
}

#[test]
fn split_aces_cannot_be_resplit() {
    // Hand pertama A + A: tidak ada aksi lagi (tidak ada `split`).
    let mut g = bj(&["Ah", "6d", "As", "Tc", "Ad", "Kh", "9s"], 1000);
    act(&mut g, "bet 100");
    act(&mut g, "split");
    let v = view(&g);
    assert_eq!(v["fase"], "taruhan");
    assert_eq!(v["tangan"].as_array().unwrap().len(), 2);
}

#[test]
fn split_needs_the_same_rank() {
    let mut g = bj(&["Kh", "6d", "Qs", "Tc"], 1000);
    act(&mut g, "bet 100");
    assert!(!legal(&g).contains(&"split".to_string()));
    let mut g = bj(&["Kh", "6d", "Ks", "Tc"], 1000);
    act(&mut g, "bet 100");
    assert!(legal(&g).contains(&"split".to_string()));
}

#[test]
fn late_surrender_returns_half_and_only_as_first_action() {
    let mut g = bj(&["Th", "Td", "6s", "7c"], 1000);
    act(&mut g, "bet 100");
    assert!(legal(&g).contains(&"surrender".to_string()));
    act(&mut g, "surrender");
    let v = view(&g);
    assert_eq!(v["tangan"][0]["hasil"], "menyerah");
    assert_eq!(v["tangan"][0]["bayar"], -50);
    assert_eq!(
        v["bandar"].as_array().unwrap().len(),
        2,
        "bandar tidak mengambil kartu"
    );
    assert_eq!(net(&g), -50);

    let mut g = bj(&["Th", "Td", "2s", "7c", "3h"], 1000);
    act(&mut g, "bet 100");
    act(&mut g, "hit");
    assert!(!legal(&g).contains(&"surrender".to_string()));
}

#[test]
fn shoe_ends_at_the_cut_card_after_the_round() {
    let mut g = bj(&["Th", "6d", "9s", "Ac", "Kh", "Kd", "Ks", "Kc"], 4);
    act(&mut g, "bet 100");
    act(&mut g, "stand");
    assert!(TurnGame::is_over(&g));
    assert!(TurnGame::pending_players(&g).is_empty());
    let v = view(&g);
    assert_eq!(v["fase"], "selesai");
    assert_eq!(v["alasan"], "shoe_habis");
    let r = TurnGame::result(&g).unwrap();
    assert_eq!(r.scores, vec![100]);
    assert_eq!(Session::act(&mut g, 0, "bet 10"), Err(GameError::Over));
}

#[test]
fn leaving_between_rounds_ends_the_shoe() {
    let mut g = bj(&["Th", "6d", "9s", "Ac"], 1000);
    assert!(legal(&g).contains(&"leave".to_string()));
    act(&mut g, "bet 100");
    assert!(
        !legal(&g).contains(&"leave".to_string()),
        "bukan di tengah ronde"
    );
    act(&mut g, "stand");
    act(&mut g, "leave");
    assert!(TurnGame::is_over(&g));
    assert_eq!(view(&g)["alasan"], "berhenti");
    assert_eq!(TurnGame::result(&g).unwrap().scores, vec![100]);
}

#[test]
fn default_shoe_is_six_decks_cut_at_75_percent() {
    let g = fresh(2);
    let v = view(&g);
    assert_eq!(v["sisa"], 312);
    assert_eq!(v["potong"], 234);
}

#[test]
fn command_round_trip() {
    let g = fresh(1);
    for cmd in [
        "hit",
        "stand",
        "double",
        "split",
        "surrender",
        "insure",
        "decline",
        "leave",
        "bet 10",
        "bet 2000",
    ] {
        let a: Action = g.parse_command(cmd).unwrap();
        assert_eq!(g.format_action(&a), cmd);
    }
    for bad in ["", "bet", "bet x", "HIT", "hit 2", "bet -10"] {
        assert!(g.parse_command(bad).is_err(), "{bad}");
    }
}

#[test]
fn random_playouts_settle_every_chip_and_end_at_the_cut() {
    let mut rng = GameRng::from_seed([7; 32]);
    for seed in 0..40u8 {
        let mut g = fresh(seed);
        let mut rounds = 0;
        let mut expected_net = 0;
        while !TurnGame::is_over(&g) {
            let v = view(&g);
            let cmd = if v["fase"] == "taruhan" {
                rounds += 1;
                format!("bet {}", 10 * (1 + rng.below(20)))
            } else {
                let options: Vec<String> = legal(&g);
                options[rng.below(options.len() as u32) as usize].clone()
            };
            let before = net(&g);
            act(&mut g, &cmd);
            let v = view(&g);
            if v["fase"] == "taruhan" || v["fase"] == "selesai" {
                // Ronde selesai: bersih bertambah tepat sebesar jumlah bayaran.
                let paid: i64 = v["tangan"]
                    .as_array()
                    .unwrap()
                    .iter()
                    .map(|h| h["bayar"].as_i64().unwrap())
                    .sum::<i64>()
                    + v["asuransi_bayar"].as_i64().unwrap_or(0);
                if before != net(&g) || paid != 0 {
                    expected_net += paid;
                }
                assert_eq!(v["taruhan_meja"], 0);
                for h in v["tangan"].as_array().unwrap() {
                    assert!(h["hasil"].is_string());
                }
            }
            assert!(rounds <= 200);
        }
        let v = view(&g);
        assert_eq!(net(&g), expected_net, "seed {seed}");
        assert!(v["kartu_terpakai"].as_u64().unwrap() >= 234);
        assert_eq!(TurnGame::result(&g).unwrap().scores, vec![expected_net]);
    }
}

#[test]
fn same_seed_same_game_and_text_view() {
    let play = |seed: u8| {
        let mut g = fresh(seed);
        for cmd in ["bet 100"] {
            act(&mut g, cmd);
        }
        (Session::state_hash(&g), view(&g))
    };
    assert_eq!(play(5), play(5));
    assert_ne!(play(5).0, play(6).0);
    let g = fresh(5);
    let id = Session::view_text(&g, 0, Lang::Id);
    let en = Session::view_text(&g, 0, Lang::En);
    assert!(id.contains("taruhan") || id.contains("Taruhan"), "{id}");
    assert!(en.contains("bet") || en.contains("Bet"), "{en}");
}
