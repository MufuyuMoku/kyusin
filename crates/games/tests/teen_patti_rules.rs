//! Tes aturan Teen Patti (SPEC §6.3, §7; M5b-1), ditulis sebelum mesinnya.
//!
//! Aturan meja (varian paling umum, lihat DECISIONS): boot 10 dari setiap
//! kursi, taruhan dasar (stake) mula-mula = boot. Kartu tiga per kursi,
//! dibagi satu per satu mulai dari kursi setelah dealer; yang pertama
//! bertindak kursi setelah dealer. Pemain buta (belum melihat kartu)
//! `chaal` sebesar stake atau `raise` dua kali stake; pemain yang sudah
//! `see` membayar dua kali dan empat kali stake. Raise menggandakan stake;
//! stake paling besar 64 × boot. Paling banyak 4 kali chaal buta, sesudah
//! itu harus melihat kartu. `show` hanya bila tinggal dua pemain (biaya =
//! chaal pemintanya); `sideshow` hanya antara dua pemain yang sudah melihat
//! (peminta dan pemain aktif sebelumnya) dengan paling sedikit tiga pemain
//! tersisa; pemain yang diminta `accept` atau `deny`; tangan yang lebih
//! rendah pack, seri = peminta pack. Pot mencapai 1.024 × boot: semua yang
//! tersisa dibandingkan. Urutan tangan: trail, pure sequence, sequence,
//! color, pair, kartu tinggi; A-K-Q sequence tertinggi, lalu A-2-3, lalu
//! K-Q-J. Seri dibagi rata. All-in memakai pot utama dan side pot.

use kyusin_core::{ActionSpec, GameRng, Lang, ParamKind, Session, TurnGame};
use kyusin_games::cards::Card;
use kyusin_games::teen_patti::{Config, TeenPatti, TpCategory, eval};
use serde_json::{Value, json};

fn cards(text: &str) -> Vec<Card> {
    text.split_whitespace()
        .map(|c| c.parse().unwrap())
        .collect()
}

fn tp(seats: u8, dealer: u8, deck: &[&str]) -> TeenPatti {
    TeenPatti::new(
        Config {
            kursi: Some(seats),
            dealer: Some(dealer),
            dek: Some(vec![deck.iter().map(|c| c.to_string()).collect()]),
            ..Config::default()
        },
        [0; 32],
    )
    .unwrap()
}

fn act(g: &mut TeenPatti, seat: u8, cmd: &str) {
    Session::act(g, seat, cmd).unwrap_or_else(|e| panic!("kursi {seat} `{cmd}`: {e:?}"));
}

fn view(g: &TeenPatti, seat: u8) -> Value {
    Session::view_data(g, seat)
}

fn legal(g: &TeenPatti, seat: u8) -> Vec<String> {
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
fn hand_ranking() {
    let order = [
        ("2h 7d 9c", TpCategory::HighCard),
        ("2h 2d 9c", TpCategory::Pair),
        ("2h 7h 9h", TpCategory::Color),
        ("4h 5d 6c", TpCategory::Sequence),
        ("4h 5h 6h", TpCategory::PureSequence),
        ("2h 2d 2c", TpCategory::Trail),
    ];
    for w in order.windows(2) {
        let (lo, hi) = (eval(&cards(w[0].0)), eval(&cards(w[1].0)));
        assert_eq!(lo.category(), w[0].1);
        assert_eq!(hi.category(), w[1].1);
        assert!(lo < hi, "{} < {}", w[0].0, w[1].0);
    }
    // Sequence: A-K-Q > A-2-3 > K-Q-J > … > 2-3-4.
    let akq = eval(&cards("Ah Kd Qc"));
    let a23 = eval(&cards("Ah 2d 3c"));
    let kqj = eval(&cards("Kh Qd Jc"));
    let low = eval(&cards("2h 3d 4c"));
    assert!(akq > a23 && a23 > kqj && kqj > low);
    assert_eq!(eval(&cards("Kh Ad 2c")).category(), TpCategory::HighCard);
    assert!(eval(&cards("Ah Ad Ac")) > eval(&cards("Kh Kd Kc")));
    assert!(eval(&cards("9h 9d Ac")) > eval(&cards("9c 9s Kd")));
    assert_eq!(eval(&cards("Ah Kd 9c")), eval(&cards("Ac Ks 9d")));
}

/// 3 kursi, dealer 0: kartu ke kursi 1, 2, 0, tiga putaran.
const DECK3: &[&str] = &[
    "Ah", "2c", "Kd", //
    "As", "5d", "Ks", //
    "Ad", "9h", "4c", // kursi 1: trail as; kursi 2: 2 5 9; kursi 0: K K 4
];

#[test]
fn boot_order_and_blind_cards() {
    let g = tp(3, 0, DECK3);
    let v = view(&g, 1);
    assert_eq!(v["fase"], "main");
    assert_eq!(v["pot"], 30);
    assert_eq!(v["stake"], 10);
    assert_eq!(stacks(&v), vec![1990, 1990, 1990]);
    assert_eq!(v["giliran"], 1, "kursi setelah dealer");
    // Pemain buta tidak melihat kartunya sendiri.
    assert_eq!(v["kursi"][1]["kartu"], json!(["??", "??", "??"]));
    assert_eq!(v["kursi"][1]["terlihat"], false);
    let l = legal(&g, 1);
    for a in ["see", "pack", "chaal", "raise"] {
        assert!(l.contains(&a.to_string()), "{a}: {l:?}");
    }
    assert!(
        !l.contains(&"show".to_string()),
        "tiga pemain: tidak ada show"
    );
    assert!(
        !l.contains(&"sideshow".to_string()),
        "buta: tidak ada sideshow"
    );
    assert_eq!(v["biaya"]["chaal"], 10);
    assert_eq!(v["biaya"]["raise"], 20);
}

#[test]
fn blind_and_seen_amounts_and_raises() {
    let mut g = tp(3, 0, DECK3);
    act(&mut g, 1, "chaal");
    let v = view(&g, 2);
    assert_eq!(v["pot"], 40);
    assert_eq!(v["kursi"][1]["buta_ke"], 1);
    act(&mut g, 2, "see");
    let v = view(&g, 2);
    assert_eq!(v["kursi"][2]["kartu"], json!(["2c", "5d", "9h"]));
    assert_eq!(v["giliran"], 2, "melihat kartu tidak memakai giliran");
    assert_eq!(
        v["biaya"]["chaal"], 20,
        "yang sudah melihat membayar dua kali stake"
    );
    assert_eq!(v["biaya"]["raise"], 40);
    act(&mut g, 2, "raise");
    let v = view(&g, 0);
    assert_eq!(v["stake"], 20);
    assert_eq!(v["pot"], 80);
    assert_eq!(v["biaya"]["chaal"], 20, "buta: stake");
    // Lawan buta (kursi 1, belum melihat) tetap tidak terlihat kursi lain.
    assert_eq!(v["kursi"][1]["kartu"], json!(["??", "??", "??"]));
    assert_eq!(v["kursi"][2]["kartu"], json!(["??", "??", "??"]));
}

#[test]
fn stake_cap_and_blind_limit() {
    let mut g = tp(2, 1, &["Ah", "2c", "As", "5d", "Ad", "9h"]);
    // Heads-up dealer 1: kursi 0 bertindak dulu. Raise buta sampai batas.
    let mut seat = 0u8;
    let mut raises = 0;
    while legal(&g, seat).contains(&"raise".to_string()) {
        act(&mut g, seat, "raise");
        raises += 1;
        seat = 1 - seat;
        if raises > 20 {
            break;
        }
    }
    let v = view(&g, 0);
    assert_eq!(v["stake"], 640, "stake paling besar 64 × boot");
    assert!(raises <= 6);
    // Chaal buta paling banyak 4 kali per pemain.
    let s0 = v["kursi"][seat as usize]["buta_ke"].as_i64().unwrap();
    assert!(s0 <= 4);
    let mut g = tp(2, 1, &["Ah", "2c", "As", "5d", "Ad", "9h"]);
    for _ in 0..4 {
        act(&mut g, 0, "chaal");
        act(&mut g, 1, "chaal");
    }
    let l = legal(&g, 0);
    assert!(
        !l.contains(&"chaal".to_string()),
        "setelah 4 chaal buta harus see: {l:?}"
    );
    assert!(l.contains(&"see".to_string()));
    act(&mut g, 0, "see");
    assert!(legal(&g, 0).contains(&"chaal".to_string()));
}

#[test]
fn show_with_two_players() {
    let mut g = tp(3, 0, DECK3);
    act(&mut g, 1, "chaal");
    act(&mut g, 2, "pack");
    let l = legal(&g, 0);
    assert!(l.contains(&"show".to_string()));
    act(&mut g, 0, "see");
    assert_eq!(view(&g, 0)["biaya"]["show"], 20);
    act(&mut g, 0, "show");
    let v = view(&g, 0);
    assert_eq!(v["fase"], "antara");
    // Trail as kursi 1 menang dari pair K kursi 0.
    assert_eq!(v["kursi"][1]["kartu"], json!(["Ah", "As", "Ad"]));
    assert_eq!(v["kursi"][1]["tangan"], "trail");
    assert_eq!(v["hasil"][1]["menang"], 60);
    assert_eq!(stacks(&v), vec![1970, 2040, 1990]);
}

#[test]
fn sideshow_accept_deny_and_tie() {
    // 4 kursi, dealer 0: kursi 1, 2, 3, 0.
    let deck = [
        "2h", "Kh", "Qc", "7c", //
        "3d", "Kd", "Qd", "8c", //
        "9s", "4c", "4d",
        "Jc", // kursi 1: 2 3 9; kursi 2: K K 4; kursi 3: Q Q 4; kursi 0: 7 8 J
    ];
    let mut g = tp(4, 0, &deck);
    act(&mut g, 1, "chaal");
    act(&mut g, 2, "see");
    act(&mut g, 2, "chaal");
    assert!(
        !legal(&g, 3).contains(&"sideshow".to_string()),
        "kursi 3 buta"
    );
    act(&mut g, 3, "see");
    // Pemain aktif sebelumnya (kursi 2) sudah melihat: sideshow boleh.
    assert!(legal(&g, 3).contains(&"sideshow".to_string()));
    act(&mut g, 3, "sideshow");
    let v = view(&g, 2);
    assert_eq!(v["sideshow"], json!([3, 2]));
    assert_eq!(TurnGame::pending_players(&g), vec![2]);
    assert_eq!(legal(&g, 2), vec!["accept", "deny"]);
    act(&mut g, 2, "accept");
    // Q Q (kursi 3) kalah dari K K (kursi 2): kursi 3 pack.
    let v = view(&g, 0);
    assert_eq!(v["kursi"][3]["status"], "pack");
    assert_eq!(v["giliran"], 0);
    // Deny: giliran berlanjut.
    let mut g = tp(4, 0, &deck);
    act(&mut g, 1, "chaal");
    act(&mut g, 2, "see");
    act(&mut g, 2, "chaal");
    act(&mut g, 3, "see");
    act(&mut g, 3, "sideshow");
    act(&mut g, 2, "deny");
    let v = view(&g, 0);
    assert_eq!(v["kursi"][3]["status"], "aktif");
    assert_eq!(v["giliran"], 0);
    // Seri: peminta pack.
    let tie = [
        "2h", "Kh", "Kc", "7c", //
        "3d", "9d", "9s", "8c", //
        "9h", "4c", "4d", "Jc", // kursi 2: K 9 4; kursi 3: K 9 4
    ];
    let mut g = tp(4, 0, &tie);
    act(&mut g, 1, "chaal");
    act(&mut g, 2, "see");
    act(&mut g, 2, "chaal");
    act(&mut g, 3, "see");
    act(&mut g, 3, "sideshow");
    act(&mut g, 2, "accept");
    assert_eq!(view(&g, 0)["kursi"][3]["status"], "pack");
}

#[test]
fn packing_down_to_one_wins_the_pot() {
    let mut g = tp(3, 0, DECK3);
    act(&mut g, 1, "raise");
    act(&mut g, 2, "pack");
    act(&mut g, 0, "pack");
    let v = view(&g, 1);
    assert_eq!(v["fase"], "antara");
    assert_eq!(v["hasil"][1]["menang"], 50);
    assert_eq!(stacks(&v), vec![1990, 2020, 1990]);
    // Tanpa show kartu pemenang tetap tertutup bagi kursi lain.
    assert_eq!(view(&g, 0)["kursi"][1]["kartu"], json!(["??", "??", "??"]));
}

#[test]
fn short_stack_all_in_and_side_pot() {
    let mut c = Config {
        kursi: Some(3),
        dealer: Some(0),
        dek: Some(vec![DECK3.iter().map(|c| c.to_string()).collect()]),
        tumpukan: Some(vec![2000, 2000, 25]),
        manusia: Some(vec![0]),
        ..Config::default()
    };
    c.tangan_maks = None;
    let mut g = TeenPatti::new(c, [0; 32]).unwrap();
    act(&mut g, 1, "see");
    act(&mut g, 1, "raise");
    // Kursi 2 punya 15: chaal buta (20) menjadi all-in 15.
    act(&mut g, 2, "chaal");
    let v = view(&g, 0);
    assert_eq!(v["kursi"][2]["status"], "allin");
    act(&mut g, 0, "see");
    act(&mut g, 0, "pack");
    // Tinggal satu pemain yang bisa bertindak, sisanya all-in: langsung
    // dibandingkan tanpa babak taruhan lagi.
    let v = view(&g, 0);
    assert_eq!(v["fase"], "antara");
    // Trail kursi 1 memenangkan pot utama dan side pot.
    let total: i64 = stacks(&v).iter().sum();
    assert_eq!(total, 4025);
    assert_eq!(stacks(&v)[2], 0);
}

#[test]
fn pot_limit_forces_a_show() {
    let mut c = Config {
        kursi: Some(2),
        dealer: Some(1),
        dek: Some(vec![
            ["Ah", "2c", "As", "5d", "Ad", "9h"]
                .iter()
                .map(|c| c.to_string())
                .collect(),
        ]),
        tumpukan: Some(vec![20000, 20000]),
        ..Config::default()
    };
    c.manusia = Some(vec![0]);
    let mut g = TeenPatti::new(c, [0; 32]).unwrap();
    let mut seat = 0u8;
    for _ in 0..200 {
        if view(&g, 0)["fase"] != "main" {
            break;
        }
        if view(&g, seat)["kursi"][seat as usize]["terlihat"] == false {
            act(&mut g, seat, "see");
        }
        let cmd = if legal(&g, seat).contains(&"raise".to_string()) {
            "raise"
        } else {
            "chaal"
        };
        act(&mut g, seat, cmd);
        seat = 1 - seat;
    }
    let v = view(&g, 0);
    assert_eq!(v["fase"], "antara", "pot limit memaksa show");
    assert_eq!(v["alasan_tangan"], "batas_pot");
    assert_eq!(v["kursi"][0]["tangan"], "trail");
}

#[test]
fn session_and_contract() {
    let mut g = tp(3, 0, DECK3);
    let v = view(&g, 0);
    assert_eq!(v["taruhan_meja"], 2000);
    assert_eq!(
        v["netral"],
        serde_json::Value::Null,
        "bukan giliran kursi 0"
    );
    act(&mut g, 1, "pack");
    act(&mut g, 2, "pack");
    let mut p = TurnGame::pending_players(&g);
    p.sort();
    assert_eq!(p, vec![0, 1, 2]);
    assert_eq!(view(&g, 0)["netral"], "leave");
    act(&mut g, 0, "leave");
    let v = view(&g, 0);
    assert_eq!(v["fase"], "selesai");
    assert_eq!(v["bersih"], 20);
    let r = TurnGame::result(&g).unwrap();
    assert_eq!(r.scores.iter().sum::<i64>(), 0);
    for cmd in [
        "see", "pack", "chaal", "raise", "show", "sideshow", "accept", "deny", "next", "leave",
    ] {
        let a = g.parse_command(cmd).unwrap();
        assert_eq!(g.format_action(&a), cmd);
    }
    assert!(Session::view_text(&g, 0, Lang::Id).contains("Kd"));
}

/// Properti: total chip meja tidak pernah berubah (SPEC §7 poin 1).
#[test]
fn chips_are_conserved_under_random_play() {
    for seed in 0..60u8 {
        let mut rng = GameRng::from_seed([seed; 32]);
        let n = 2 + seed % 5;
        let stacks: Vec<i64> = (0..n).map(|s| 50 + 400 * i64::from(s)).collect();
        let total: i64 = stacks.iter().sum();
        let mut g = TeenPatti::new(
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
                        ParamKind::Int { min, .. } => format!("{verb} {min}"),
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
