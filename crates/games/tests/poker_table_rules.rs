//! Tes aturan Texas Hold'em (No-Limit) dan Omaha (Pot-Limit) (SPEC §6.3,
//! §7; M5b-1), ditulis sebelum mesinnya.
//!
//! Aturan meja (varian paling umum, lihat DECISIONS): blind 10/20, buy-in
//! bawaan 2.000 per kursi, tanpa rake. Satu pertandingan = satu sesi meja.
//! Kartu dibagi satu per satu mulai dari kursi setelah dealer (Hold'em 2,
//! Omaha 4 kartu), lalu kartu meja berurutan (flop 3, turn, river; tanpa
//! burn). Small blind = kursi setelah dealer, big blind sesudahnya; heads-up
//! dealer memasang small blind dan bertindak lebih dulu sebelum flop.
//! Sebelum flop yang pertama bertindak adalah kursi setelah big blind;
//! sesudah flop kursi aktif pertama setelah dealer. `bet <n>` dan
//! `raise <n>` menyatakan total taruhan di babak itu. Raise minimal sebesar
//! raise terakhir (paling sedikit big blind); all-in yang kurang dari raise
//! penuh tidak membuka lagi hak raise bagi yang sudah bertindak. No-Limit:
//! paling banyak seluruh tumpukan. Pot-Limit: paling banyak taruhan saat
//! ini + pot setelah call. Omaha memakai tepat dua kartu tangan dan tiga
//! kartu meja. Setelah tangan selesai semua kursi menjawab `next` (atau
//! `leave`); sesi selesai bila kursi manusia berdiri atau habis, bila
//! tinggal satu kursi bertumpukan, atau setelah `tangan_maks`.

use kyusin_core::{ActionSpec, GameRng, Lang, ParamKind, Session, TurnGame};
use kyusin_games::omaha::Omaha;
use kyusin_games::poker_meja::Config;
use kyusin_games::texas_holdem::TexasHoldem;
use serde_json::{Value, json};

fn cfg(seats: u8, dealer: u8, decks: &[&[&str]]) -> Config {
    Config {
        kursi: Some(seats),
        dealer: Some(dealer),
        dek: Some(
            decks
                .iter()
                .map(|d| d.iter().map(|c| c.to_string()).collect())
                .collect(),
        ),
        ..Config::default()
    }
}

fn th(seats: u8, dealer: u8, decks: &[&[&str]]) -> TexasHoldem {
    TexasHoldem::new(cfg(seats, dealer, decks), [0; 32]).unwrap()
}

fn act<G: Session>(g: &mut G, seat: u8, cmd: &str) {
    g.act(seat, cmd).unwrap_or_else(|e| panic!("kursi {seat} `{cmd}`: {e:?}"));
}

fn view<G: Session>(g: &G, seat: u8) -> Value {
    g.view_data(seat)
}

fn legal<G: Session>(g: &G, seat: u8) -> Vec<String> {
    g.legal_actions(seat).iter().map(|a| a.usage()).collect()
}

fn stacks(v: &Value) -> Vec<i64> {
    v["kursi"]
        .as_array()
        .unwrap()
        .iter()
        .map(|k| k["tumpukan"].as_i64().unwrap())
        .collect()
}

/// 3 kursi, dealer 0: kartu kursi 1, 2, 0, 1, 2, 0 lalu meja.
const DECK3: &[&str] = &[
    "Ah", "2c", "Kd", "As", "7d", "Ks", // kursi 1: Ah As; kursi 2: 2c 7d; kursi 0: Kd Ks
    "Qh", "8c", "3d", "4s", "9h", // flop Qh 8c 3d, turn 4s, river 9h
];

#[test]
fn blinds_and_preflop_order() {
    let g = th(3, 0, &[DECK3]);
    let v = view(&g, 0);
    assert_eq!(v["fase"], "main");
    assert_eq!(v["dealer"], 0);
    assert_eq!(v["kursi"][1]["taruhan"], 10);
    assert_eq!(v["kursi"][2]["taruhan"], 20);
    assert_eq!(stacks(&v), vec![2000, 1990, 1980]);
    assert_eq!(v["giliran"], 0, "setelah big blind");
    assert_eq!(TurnGame::pending_players(&g), vec![0]);
    assert_eq!(v["pot"], 30);
    // Kartu sendiri terlihat, kartu lawan tertutup.
    assert_eq!(v["kursi"][0]["kartu"], json!(["Kd", "Ks"]));
    assert_eq!(v["kursi"][1]["kartu"], json!(["??", "??"]));
    assert_eq!(view(&g, 1)["kursi"][1]["kartu"], json!(["Ah", "As"]));
    let l = legal(&g, 0);
    assert!(l.contains(&"fold".to_string()));
    assert!(l.contains(&"call".to_string()));
    assert!(!l.contains(&"check".to_string()));
    assert!(l.contains(&"raise <jumlah>".to_string()));
}

#[test]
fn heads_up_dealer_posts_small_blind_and_acts_first_preflop() {
    let mut g = th(2, 1, &[&["Ah", "2c", "As", "7d", "Qh", "8c", "3d", "4s", "9h"]]);
    let v = view(&g, 0);
    assert_eq!(v["kursi"][1]["taruhan"], 10, "dealer = small blind");
    assert_eq!(v["kursi"][0]["taruhan"], 20);
    assert_eq!(v["giliran"], 1);
    act(&mut g, 1, "call");
    act(&mut g, 0, "check");
    let v = view(&g, 0);
    assert_eq!(v["meja_kartu"].as_array().unwrap().len(), 3);
    assert_eq!(v["giliran"], 0, "big blind bertindak lebih dulu setelah flop");
}

#[test]
fn raise_sizes_no_limit() {
    let mut g = th(3, 0, &[DECK3]);
    // Raise minimal ke 40 (big blind 20 + raise 20).
    assert!(Session::act(&mut g, 0, "raise 39").is_err());
    act(&mut g, 0, "raise 60");
    // Raise berikutnya minimal 60 + 40 = 100.
    assert!(Session::act(&mut g, 1, "raise 99").is_err());
    act(&mut g, 1, "raise 100");
    // No-Limit: paling banyak seluruh tumpukan (2.000 total).
    assert!(Session::act(&mut g, 2, "raise 2001").is_err());
    act(&mut g, 2, "raise 2000");
    let v = view(&g, 0);
    assert_eq!(v["kursi"][2]["status"], "allin");
    assert_eq!(v["kursi"][2]["tumpukan"], 0);
    act(&mut g, 0, "fold");
    act(&mut g, 1, "fold");
    let v = view(&g, 0);
    assert_eq!(v["fase"], "antara");
    // Kursi 2 memenangkan pot tanpa showdown: 60 + 100 + 2000.
    assert_eq!(stacks(&v), vec![1940, 1900, 2160]);
    assert_eq!(v["hasil"][2]["menang"], 2160);
    assert_eq!(v["kursi"][2]["kartu"], json!(["??", "??"]), "tanpa showdown kartu tetap tertutup");
}

#[test]
fn betting_round_flow_and_showdown() {
    let mut g = th(3, 0, &[DECK3]);
    act(&mut g, 0, "call");
    act(&mut g, 1, "call");
    act(&mut g, 2, "check");
    let v = view(&g, 0);
    assert_eq!(v["meja_kartu"], json!(["Qh", "8c", "3d"]));
    assert_eq!(v["giliran"], 1, "setelah flop: kursi aktif pertama setelah dealer");
    assert!(Session::act(&mut g, 1, "bet 10").is_err(), "bet minimal big blind");
    act(&mut g, 1, "bet 40");
    act(&mut g, 2, "fold");
    act(&mut g, 0, "call");
    act(&mut g, 1, "check");
    act(&mut g, 0, "check");
    act(&mut g, 1, "check");
    act(&mut g, 0, "check");
    let v = view(&g, 0);
    assert_eq!(v["fase"], "antara");
    assert_eq!(v["meja_kartu"].as_array().unwrap().len(), 5);
    // As berpasangan menang dari pair K.
    assert_eq!(v["kursi"][1]["kartu"], json!(["Ah", "As"]));
    assert_eq!(v["kursi"][1]["tangan"], "pair");
    assert_eq!(v["hasil"][1]["menang"], 140);
    assert_eq!(stacks(&v), vec![1940, 2080, 1980]);
}

#[test]
fn side_pot_with_a_short_all_in() {
    // Kursi 0 hanya 100 chip.
    let mut c = cfg(3, 0, &[DECK3]);
    c.tumpukan = Some(vec![100, 2000, 2000]);
    c.manusia = Some(vec![1]);
    let mut g = TexasHoldem::new(c, [0; 32]).unwrap();
    act(&mut g, 0, "raise 100");
    act(&mut g, 1, "raise 400");
    act(&mut g, 2, "call");
    // Kursi 0 all-in; kursi 1 dan 2 lanjut.
    act(&mut g, 1, "check");
    act(&mut g, 2, "check");
    act(&mut g, 1, "check");
    act(&mut g, 2, "check");
    act(&mut g, 1, "check");
    act(&mut g, 2, "check");
    let v = view(&g, 0);
    assert_eq!(v["fase"], "antara");
    // As kursi 1 menang pot utama (300) dan side pot (600).
    assert_eq!(v["hasil"][1]["menang"], 900);
    assert_eq!(stacks(&v), vec![0, 2500, 1600]);
    assert_eq!(v["pots"].as_array().unwrap().len(), 2);
}

#[test]
fn incomplete_all_in_raise_does_not_reopen_raising() {
    let mut c = cfg(3, 0, &[DECK3]);
    c.tumpukan = Some(vec![2000, 2000, 70]);
    let mut g = TexasHoldem::new(c, [0; 32]).unwrap();
    act(&mut g, 0, "raise 60");
    act(&mut g, 1, "call");
    // Kursi 2 all-in 70: kurang dari raise penuh (100).
    act(&mut g, 2, "raise 70");
    // Kursi 0 dan 1 sudah bertindak: hanya call atau fold.
    let l = legal(&g, 0);
    assert!(l.contains(&"call".to_string()));
    assert!(!l.iter().any(|a| a.starts_with("raise")), "{l:?}");
}

#[test]
fn hand_end_next_and_session_end() {
    let mut g = th(3, 0, &[DECK3]);
    act(&mut g, 0, "fold");
    act(&mut g, 1, "fold");
    let v = view(&g, 0);
    assert_eq!(v["fase"], "antara");
    let mut pending = TurnGame::pending_players(&g);
    pending.sort();
    assert_eq!(pending, vec![0, 1, 2]);
    act(&mut g, 1, "next");
    act(&mut g, 2, "next");
    assert_eq!(view(&g, 0)["fase"], "antara", "menunggu semua kursi");
    act(&mut g, 0, "next");
    let v = view(&g, 0);
    assert_eq!(v["fase"], "main");
    assert_eq!(v["dealer"], 1, "tombol dealer bergeser");
    assert_eq!(v["tangan_ke"], 2);
    // Dealer 1: small blind kursi 2, big blind kursi 0, kursi 1 bertindak dulu.
    assert_eq!(v["giliran"], 1);
    act(&mut g, 1, "fold");
    act(&mut g, 2, "fold");
    // Manusia (kursi 0) berdiri di antara tangan: sesi selesai.
    act(&mut g, 0, "leave");
    let v = view(&g, 0);
    assert_eq!(v["fase"], "selesai");
    assert!(TurnGame::is_over(&g));
    let r = TurnGame::result(&g).unwrap();
    assert_eq!(r.scores.iter().sum::<i64>(), 0, "chip tidak bertambah atau hilang");
    assert_eq!(v["bersih"], r.scores[0]);
    assert_eq!(v["taruhan_meja"], 0);
}

#[test]
fn chip_contract_for_the_host() {
    let mut c = cfg(3, 0, &[DECK3]);
    c.tumpukan = Some(vec![1500, 2000, 2000]);
    let mut g = TexasHoldem::new(c, [0; 32]).unwrap();
    let v = view(&g, 0);
    // Selama duduk, buy-in kursi ini ada di meja; hasil baru saat berdiri.
    assert_eq!(v["taruhan_meja"], 1500);
    assert_eq!(v["bersih"], 0);
    assert_eq!(v["netral"], "fold");
    act(&mut g, 0, "fold");
    act(&mut g, 1, "fold");
    assert_eq!(view(&g, 0)["netral"], "leave");
    act(&mut g, 0, "leave");
    let v = view(&g, 0);
    assert_eq!(v["taruhan_meja"], 0);
    assert_eq!(v["bersih"], 0, "fold sebelum memasang apa pun");
}

#[test]
fn busted_human_ends_the_session_and_hand_limit_ends_it_too() {
    // Heads-up dealer 0: kartu ke kursi 1, 0, 1, 0 (kursi 0 mendapat as).
    let mut c = cfg(2, 0, &[&["2c", "Ah", "7d", "As", "Qh", "8c", "3d", "4s", "9h"]]);
    c.tumpukan = Some(vec![500, 500]);
    c.manusia = Some(vec![1]);
    let mut g = TexasHoldem::new(c, [0; 32]).unwrap();
    // Heads-up dealer 0: kursi 0 SB.
    act(&mut g, 0, "raise 500");
    act(&mut g, 1, "call");
    let v = view(&g, 1);
    assert_eq!(v["fase"], "selesai", "manusia kursi 1 habis");
    assert_eq!(stacks(&v), vec![1000, 0]);
    assert_eq!(TurnGame::result(&g).unwrap().winners, vec![0]);

    let mut c = cfg(2, 0, &[]);
    c.tangan_maks = Some(1);
    let mut g = TexasHoldem::new(c, [3; 32]).unwrap();
    act(&mut g, 0, "fold");
    assert_eq!(view(&g, 0)["fase"], "selesai");
}

#[test]
fn omaha_uses_exactly_two_hole_cards_and_pot_limit() {
    // 3 kursi, dealer 0: kursi 1, 2, 0 masing-masing 4 kartu.
    let deck: &[&str] = &[
        "Ah", "2c", "Kd", //
        "3s", "7d", "Ks", //
        "5c", "8d", "2d", //
        "6c", "9s", "3d", //
        "Th", "Jh", "Qh", "4h", "5d", // meja: empat hati
    ];
    let mut g = Omaha::new(cfg(3, 0, &[deck]), [0; 32]).unwrap();
    // Pot-limit: raise paling besar = 20 + (30 + 20) = 70.
    assert!(Session::act(&mut g, 0, "raise 71").is_err());
    act(&mut g, 0, "raise 70");
    act(&mut g, 1, "call");
    act(&mut g, 2, "call");
    for _ in 0..3 {
        act(&mut g, 1, "check");
        act(&mut g, 2, "check");
        act(&mut g, 0, "check");
    }
    let v = view(&g, 0);
    assert_eq!(v["fase"], "antara");
    // Empat hati di meja, tetapi tidak ada kursi dengan dua hati di tangan:
    // di Omaha tidak ada yang flush (di Hold'em kursi 1 dengan Ah akan flush).
    for s in 0..3 {
        assert_ne!(v["kursi"][s]["tangan"], "flush", "kursi {s}");
    }
}

#[test]
fn view_and_commands() {
    let g = th(3, 0, &[DECK3]);
    for cmd in ["fold", "check", "call", "bet 40", "raise 100", "next", "leave"] {
        let a = g.parse_command(cmd).unwrap();
        assert_eq!(g.format_action(&a), cmd);
    }
    assert!(g.parse_command("raise").is_err());
    assert!(Session::view_text(&g, 0, Lang::Id).contains("Kd"));
    assert!(!Session::view_text(&g, 0, Lang::En).contains("Ah"), "kartu lawan tersembunyi");
}

/// Properti: total chip meja tidak pernah berubah, untuk aksi sah acak
/// pada banyak sesi (SPEC §7 poin 1).
#[test]
fn chips_are_conserved_under_random_play() {
    for (variant, seed) in (0..60u8).map(|i| (i % 2, i)) {
        let mut rng = GameRng::from_seed([seed; 32]);
        let mut c = Config {
            kursi: Some(2 + seed % 5),
            tangan_maks: Some(15),
            ..Config::default()
        };
        c.tumpukan = Some((0..c.kursi.unwrap()).map(|s| 200 + 300 * i64::from(s)).collect());
        let total: i64 = c.tumpukan.as_ref().unwrap().iter().sum();
        let mut g: Box<dyn Session> = if variant == 0 {
            Box::new(TexasHoldem::new(c, [seed; 32]).unwrap())
        } else {
            Box::new(Omaha::new(c, [seed; 32]).unwrap())
        };
        let mut steps = 0;
        while !g.is_over() && steps < 5000 {
            let seat = g.pending_players()[0];
            let options: Vec<String> = g
                .legal_actions(seat)
                .iter()
                .filter(|a| a.usage() != "leave")
                .map(|a| match a {
                    ActionSpec::Fixed { command } => command.clone(),
                    ActionSpec::Template { verb, params } => match &params[0].kind {
                        ParamKind::Int { min, max, .. } => {
                            let span = (max - min) as u32;
                            format!("{verb} {}", min + i64::from(rng.below(span + 1)))
                        }
                        ParamKind::Choice { options } => format!("{verb} {}", options[0]),
                    },
                })
                .collect();
            let cmd = options[rng.below(options.len() as u32) as usize].clone();
            g.act(seat, &cmd).unwrap();
            let v = g.view_data(0);
            let on_table: i64 = v["kursi"]
                .as_array()
                .unwrap()
                .iter()
                .map(|k| k["tumpukan"].as_i64().unwrap() + k["taruhan_tangan"].as_i64().unwrap())
                .sum();
            assert_eq!(on_table, total, "sesi {seed} langkah {steps}: {cmd}");
            steps += 1;
        }
        assert!(g.is_over(), "sesi {seed} selesai");
        assert_eq!(g.result().unwrap().scores.iter().sum::<i64>(), 0);
    }
}
