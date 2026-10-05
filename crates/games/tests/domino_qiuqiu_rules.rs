//! Tes aturan Domino QiuQiu (SPEC §6.3, §7; M5b-2), ditulis sebelum
//! mesinnya. Aturan lokal dipilih klien (D-065):
//!
//! - kartu domino 28, 2–6 kursi; ante 10 dari setiap kursi;
//! - bagi 3 kartu (satu per satu mulai kursi setelah dealer), putaran
//!   taruhan; bagi kartu ke-4, putaran taruhan; buka kartu;
//! - taruhan pot-limit, bet paling kecil 20; raise paling kecil sebesar
//!   raise terakhir; yang pertama bertindak kursi setelah dealer;
//! - empat kartu dibagi menjadi dua pasangan; nilai pasangan = jumlah
//!   bulatan mod 10; pembagian terbaik dipilih otomatis (pasangan tertinggi
//!   lalu pasangan kedua);
//! - kartu spesial: Enam Dewa > Balak > Murni Kecil > Murni Besar, lalu
//!   QiuQiu (9-9) dan pasangan biasa;
//! - nilai sama: kartu tunggal tertinggi (balak > non-balak, lalu jumlah
//!   bulatan, lalu angka terbesar); masih sama = pot dibagi.

use std::cmp::Ordering;

use kyusin_core::{ActionSpec, GameRng, Lang, ParamKind, Session, TurnGame};
use kyusin_games::domino::Tile;
use kyusin_games::domino_qiuqiu::{Config, QiuQiu, QqClass, eval};
use serde_json::Value;

fn tiles(text: &str) -> Vec<Tile> {
    text.split_whitespace().map(|t| t.parse().unwrap()).collect()
}

fn qq(seats: u8, dealer: u8, deck: &[&str]) -> QiuQiu {
    QiuQiu::new(
        Config {
            kursi: Some(seats),
            dealer: Some(dealer),
            dek: Some(vec![deck.iter().map(|t| t.to_string()).collect()]),
            ..Config::default()
        },
        [0; 32],
    )
    .unwrap()
}

fn act(g: &mut QiuQiu, seat: u8, cmd: &str) {
    Session::act(g, seat, cmd).unwrap_or_else(|e| panic!("kursi {seat} `{cmd}`: {e:?}"));
}

fn view(g: &QiuQiu, seat: u8) -> Value {
    Session::view_data(g, seat)
}

fn legal(g: &QiuQiu, seat: u8) -> Vec<String> {
    TurnGame::legal_actions(g, seat)
        .iter()
        .map(|a| a.usage())
        .collect()
}

fn stacks(v: &Value) -> Vec<i64> {
    v["kursi"]
        .as_array()
        .unwrap()
        .iter()
        .map(|k| k["tumpukan"].as_i64().unwrap())
        .collect()
}

#[test]
fn special_hands_rank_above_pairs_in_the_chosen_order() {
    let order = [
        ("6-6 6-1 5-3 2-0", QqClass::Pasangan),
        ("6-3 0-0 5-4 5-5", QqClass::QiuQiu),
        ("6-6 6-5 5-5 6-4", QqClass::MurniBesar),
        ("0-0 1-0 2-1 2-0", QqClass::MurniKecil),
        ("0-0 1-1 2-2 6-6", QqClass::Balak),
        ("6-0 5-1 4-2 3-3", QqClass::EnamDewa),
    ];
    for w in order.windows(2) {
        let (lo, hi) = (eval(&tiles(w[0].0)), eval(&tiles(w[1].0)));
        assert_eq!(lo.class(), w[0].1, "{}", w[0].0);
        assert_eq!(hi.class(), w[1].1, "{}", w[1].0);
        assert!(hi > lo, "{} > {}", w[1].0, w[0].0);
    }
    // Empat balak yang jumlahnya besar tetap Balak, bukan Murni Besar.
    assert_eq!(eval(&tiles("6-6 5-5 4-4 3-3")).class(), QqClass::Balak);
    assert_eq!(eval(&tiles("6-0 5-1 4-2 3-3")).key(), "enam_dewa");
    assert_eq!(eval(&tiles("6-3 0-0 5-4 5-5")).key(), "qiuqiu");
}

#[test]
fn small_is_better_lower_and_big_is_better_higher() {
    // Murni Kecil: total lebih kecil lebih kuat (6 < 8).
    assert!(eval(&tiles("0-0 1-0 2-1 2-0")) > eval(&tiles("0-0 1-0 2-1 3-1")));
    // Murni Besar: total lebih besar lebih kuat (43 > 42).
    assert!(eval(&tiles("6-6 6-5 5-5 6-4")) > eval(&tiles("6-6 6-5 5-4 6-4")));
}

#[test]
fn the_best_split_is_chosen_automatically() {
    // 6-6 3-0 | 5-1 2-1 = 5 dan 9; pembagian lain lebih buruk.
    let v = eval(&tiles("6-6 5-1 3-0 2-1"));
    assert_eq!(v.class(), QqClass::Pasangan);
    assert_eq!(v.pairs(), (9, 5));
    let [a, b] = v.split();
    assert_eq!((a[0].pips() + a[1].pips()) % 10, 9);
    assert_eq!((b[0].pips() + b[1].pips()) % 10, 5);
    // Pasangan tertinggi menentukan lebih dulu: 9-0 > 8-8.
    assert!(eval(&tiles("6-6 6-1 5-3 2-0")) > eval(&tiles("4-4 4-0 6-2 0-0")));
    assert_eq!(eval(&tiles("4-4 4-0 6-2 0-0")).pairs(), (8, 8));
    assert!(eval(&tiles("6-3 0-0 5-4 6-2")) > eval(&tiles("6-3 0-0 5-4 6-1")));
}

#[test]
fn equal_pairs_are_broken_by_the_highest_tile() {
    // Keduanya 9-8: 6-6 (balak, 12) > 4-4 (balak, 8).
    let a = eval(&tiles("6-6 6-1 5-3 0-0"));
    let b = eval(&tiles("4-4 1-0 6-2 0-0"));
    assert_eq!(a.pairs(), (9, 8));
    assert_eq!(b.pairs(), (9, 8));
    assert!(a > b);
    // Balak mengalahkan non-balak yang bulatannya lebih banyak: kartu
    // tertinggi 3-3 (balak, 6) > 6-5 (11); keduanya 9-7.
    let c = eval(&tiles("3-3 2-1 5-0 2-0"));
    let d = eval(&tiles("6-5 6-2 4-1 2-0"));
    assert_eq!(c.pairs(), (9, 7));
    assert_eq!(d.pairs(), (9, 7));
    assert!(c > d);
    // Tanpa balak: jumlah bulatan kartu tertinggi (6-5 = 11 > 6-4 = 10).
    let e = eval(&tiles("6-4 6-3 4-1 2-0")); // 6-4,6-3 = 19 → 9; 4-1,2-0 = 7
    assert_eq!(e.pairs(), (9, 7));
    assert!(d > e);
    // Tangan identik = seri.
    assert_eq!(a.cmp(&eval(&tiles("0-0 5-3 6-1 6-6"))), Ordering::Equal);
}

/// Dua kursi, dealer 0: kartu dibagi ke kursi 1, 0, 1, 0, 1, 0, lalu kartu
/// ke-4 ke kursi 1, 0.
const DECK2: &[&str] = &["6-3", "6-6", "0-0", "6-1", "5-4", "5-3", "5-5", "2-0"];

#[test]
fn ante_three_tiles_two_rounds_and_showdown() {
    let mut g = qq(2, 0, DECK2);
    let v = view(&g, 1);
    assert_eq!(v["ante"], 10);
    assert_eq!(v["pot"], 20);
    assert_eq!(stacks(&v), vec![1990, 1990]);
    assert_eq!(v["kursi"][1]["kartu"], serde_json::json!(["6-3", "0-0", "5-4"]));
    assert_eq!(v["kursi"][0]["kartu"], serde_json::json!(["??", "??", "??"]));
    assert_eq!(TurnGame::pending_players(&g), vec![1], "kursi setelah dealer");
    assert_eq!(legal(&g, 1), vec!["fold", "check", "bet <jumlah>"]);
    assert_eq!(v["naik_min"], 20);
    assert_eq!(v["naik_maks"], 20, "pot-limit: paling besar sebesar pot");
    act(&mut g, 1, "bet 20");
    act(&mut g, 0, "call");
    let v = view(&g, 1);
    assert_eq!(v["kursi"][1]["kartu"].as_array().unwrap().len(), 4, "kartu ke-4");
    assert_eq!(v["kursi"][0]["kartu"].as_array().unwrap().len(), 4);
    assert_eq!(v["putaran"], 2);
    assert_eq!(TurnGame::pending_players(&g), vec![1]);
    act(&mut g, 1, "check");
    act(&mut g, 0, "check");
    let v = view(&g, 0);
    assert_eq!(v["fase"], "antara");
    assert_eq!(stacks(&v), vec![1970, 2030]);
    assert_eq!(v["kursi"][1]["tangan"], "qiuqiu");
    assert_eq!(v["kursi"][0]["tangan"], "pasangan");
    assert_eq!(v["kursi"][0]["nilai"], serde_json::json!([9, 0]));
    // Kartu yang dibuka disusun per pasangan, pasangan tertinggi dulu.
    assert_eq!(v["kursi"][0]["kartu"], serde_json::json!(["6-6", "6-1", "5-3", "2-0"]));
    assert_eq!(v["kursi"][1]["menang"], 60);
}

#[test]
fn pot_limit_raise_bounds() {
    // Tiga kursi, dealer 0: kursi 1 bertindak dulu; pot 30 dari ante.
    let deck: Vec<String> = Tile::set().iter().map(Tile::to_string).collect();
    let deck: Vec<&str> = deck.iter().map(String::as_str).collect();
    let mut g = qq(3, 0, &deck);
    let v = view(&g, 1);
    assert_eq!(v["pot"], 30);
    assert_eq!((v["naik_min"].as_i64(), v["naik_maks"].as_i64()), (Some(20), Some(30)));
    act(&mut g, 1, "bet 20");
    let v = view(&g, 2);
    assert_eq!(v["panggil"], 20);
    assert_eq!(legal(&g, 2), vec!["fold", "call", "raise <jumlah>"]);
    // Raise paling kecil 20 + 20; paling besar 20 + (50 + 20).
    assert_eq!((v["naik_min"].as_i64(), v["naik_maks"].as_i64()), (Some(40), Some(90)));
    assert!(Session::act(&mut g, 2, "raise 91").is_err());
    act(&mut g, 2, "raise 90");
    assert_eq!(view(&g, 0)["panggil"], 90);
}

#[test]
fn folding_down_to_one_wins_without_showing() {
    let mut g = qq(2, 0, DECK2);
    act(&mut g, 1, "bet 20");
    act(&mut g, 0, "fold");
    let v = view(&g, 0);
    assert_eq!(v["fase"], "antara");
    assert_eq!(stacks(&v), vec![1990, 2010]);
    assert_eq!(v["kursi"][1]["kartu"], serde_json::json!(["??", "??", "??"]), "tidak dibuka");
    assert_eq!(v["kursi"][1]["tangan"], Value::Null);
}

#[test]
fn session_and_contract() {
    let mut g = qq(2, 0, DECK2);
    let v = view(&g, 0);
    assert_eq!(v["taruhan_meja"], 2000);
    assert_eq!(v["netral"], Value::Null, "bukan giliran kursi 0");
    assert_eq!(view(&g, 1)["netral"], "check");
    act(&mut g, 1, "bet 20");
    assert_eq!(view(&g, 0)["netral"], "fold");
    act(&mut g, 0, "fold");
    let mut p = TurnGame::pending_players(&g);
    p.sort();
    assert_eq!(p, vec![0, 1]);
    assert_eq!(view(&g, 0)["netral"], "leave");
    act(&mut g, 0, "leave");
    let v = view(&g, 0);
    assert_eq!(v["fase"], "selesai");
    assert_eq!(v["bersih"], -10);
    let r = TurnGame::result(&g).unwrap();
    assert_eq!(r.scores.iter().sum::<i64>(), 0);
    for cmd in ["fold", "check", "call", "bet 20", "raise 40", "next", "leave"] {
        let a = g.parse_command(cmd).unwrap();
        assert_eq!(g.format_action(&a), cmd);
    }
    assert!(Session::view_text(&g, 0, Lang::Id).contains("6-6"));
}

#[test]
fn a_new_hand_is_dealt_from_the_session_seed() {
    let mk = |seed: u8| {
        QiuQiu::new(
            Config {
                kursi: Some(4),
                dealer: Some(0),
                ..Config::default()
            },
            [seed; 32],
        )
        .unwrap()
    };
    let (a, b, c) = (mk(1), mk(1), mk(2));
    assert_eq!(view(&a, 1)["kursi"][1]["kartu"], view(&b, 1)["kursi"][1]["kartu"]);
    assert_ne!(view(&a, 1)["kursi"][1]["kartu"], view(&c, 1)["kursi"][1]["kartu"]);
    assert!(QiuQiu::new(Config { kursi: Some(7), ..Config::default() }, [0; 32]).is_err());
}

/// Properti: total chip meja tidak pernah berubah (SPEC §7 poin 1).
#[test]
fn chips_are_conserved_under_random_play() {
    for seed in 0..60u8 {
        let mut rng = GameRng::from_seed([seed; 32]);
        let n = 2 + seed % 5;
        let stacks: Vec<i64> = (0..n).map(|s| 15 + 400 * i64::from(s)).collect();
        let total: i64 = stacks.iter().sum();
        let mut g = QiuQiu::new(
            Config {
                kursi: Some(n),
                tumpukan: Some(stacks),
                tangan_maks: Some(12),
                ..Config::default()
            },
            [seed; 32],
        )
        .unwrap();
        let mut steps = 0;
        while !TurnGame::is_over(&g) && steps < 5000 {
            let seat = TurnGame::pending_players(&g)[0];
            let options: Vec<String> = TurnGame::legal_actions(&g, seat)
                .iter()
                .filter(|a| a.usage() != "leave")
                .map(|a| match a {
                    ActionSpec::Fixed { command } => command.clone(),
                    ActionSpec::Template { verb, params } => match &params[0].kind {
                        ParamKind::Int { min, max, .. } => {
                            let pick = if rng.below(2) == 0 { *min } else { *max };
                            format!("{verb} {pick}")
                        }
                        ParamKind::Choice { options } => format!("{verb} {}", options[0]),
                    },
                })
                .collect();
            let cmd = options[rng.below(options.len() as u32) as usize].clone();
            act(&mut g, seat, &cmd);
            let v = view(&g, 0);
            let on_table: i64 = v["kursi"]
                .as_array()
                .unwrap()
                .iter()
                .map(|k| k["tumpukan"].as_i64().unwrap() + k["taruhan_tangan"].as_i64().unwrap())
                .sum();
            assert_eq!(on_table, total, "sesi {seed} langkah {steps}: {cmd}");
            steps += 1;
        }
        assert!(TurnGame::is_over(&g), "sesi {seed}");
        assert_eq!(TurnGame::result(&g).unwrap().scores.iter().sum::<i64>(), 0);
    }
}
