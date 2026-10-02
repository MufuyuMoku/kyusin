//! Tes aturan dan pembayaran Pai Gow Poker (SPEC §6.3, §7; M5a), ditulis
//! sebelum mesinnya.
//!
//! Aturan meja (keputusan klien untuk house way, lihat DECISIONS): dek 53
//! kartu (52 + joker `JK`) dikocok per ronde; satu ronde = satu
//! pertandingan. `bet <jumlah>` (20–2.000, kelipatan 20 supaya komisi 5%
//! utuh) membagi tujuh kartu untuk pemain dan tujuh tertutup untuk bandar.
//! Pemain menyusun tangan depan dua kartu (`set <kartu> <kartu>`) dan
//! tangan belakang lima kartu sisanya, atau `houseway` (bandar menyusun
//! untuknya). Tangan belakang harus lebih tinggi dari tangan depan. Joker
//! adalah "bug": di lima kartu ia hanya boleh menjadi as atau melengkapi
//! straight, flush, atau straight flush; di dua kartu ia as. Lima as (empat
//! as + joker) adalah tangan tertinggi. A-2-3-4-5 adalah straight tertinggi
//! kedua. Bandar menyusun dengan house way. Kedua tangan pemain harus lebih
//! tinggi (seri = bandar menang): kedua menang dibayar 0,95:1, satu menang
//! satu kalah seri, kedua kalah kalah.

use kyusin_core::{Lang, Session, TurnGame};
use kyusin_games::pai_gow::{Config, PaiGow, Tile, eval_back, eval_front, house_way, valid};
use serde_json::Value;

fn tiles(text: &str) -> Vec<Tile> {
    text.split_whitespace().map(|c| c.parse().unwrap()).collect()
}

fn pg(cards: &[&str]) -> PaiGow {
    PaiGow::new(
        Config {
            kartu: Some(cards.iter().map(|c| c.to_string()).collect()),
        },
        [0; 32],
    )
    .unwrap()
}

fn act(g: &mut PaiGow, cmd: &str) {
    Session::act(g, 0, cmd).unwrap_or_else(|e| panic!("`{cmd}`: {e:?}"));
}

fn view(g: &PaiGow) -> Value {
    Session::view_data(g, 0)
}

fn back(text: &str) -> kyusin_games::pai_gow::BackValue {
    eval_back(&tiles(text))
}

#[test]
fn joker_is_a_bug_in_the_back_hand() {
    // Joker + as = pair as.
    assert_eq!(back("JK Ah 7c 5d 2s").key(), "pair");
    // Tanpa as: joker = as (kartu tinggi), bukan pair dari kartu lain.
    let v = back("JK Kh 7c 5d 2s");
    assert_eq!(v.key(), "high_card");
    assert!(v > back("Ac Qh 7c 5d 3s"), "joker = as");
    assert!(v < back("Ac Kh 7c 5d 4s"));
    // Melengkapi straight, flush, straight flush.
    assert_eq!(back("JK 9h Tc Jd Qs").key(), "straight");
    assert_eq!(back("JK 2h 7h 9h Kh").key(), "flush");
    assert_eq!(back("JK 9h Th Jh Qh").key(), "straight_flush");
    // Joker tidak menjadi kartu sembarang untuk pair/trips selain as.
    assert_eq!(back("JK 9h 9c 5d 2s").key(), "pair");
    // Lima as tertinggi, di atas royal flush.
    assert_eq!(back("JK Ah Ac Ad As").key(), "five_aces");
    assert!(back("JK Ah Ac Ad As") > back("Th Jh Qh Kh Ah"));
}

#[test]
fn the_wheel_is_the_second_highest_straight() {
    let wheel = back("Ah 2d 3c 4s 5h");
    assert!(wheel > back("9h Td Jc Qs Kh"));
    assert!(wheel < back("Th Jd Qc Ks Ah"));
    assert!(back("Ah 2h 3h 4h 5h") > back("9h Th Jh Qh Kh"));
}

#[test]
fn front_hand_and_fouls() {
    // Joker di depan = as.
    assert!(eval_front(&tiles("JK 2c")) > eval_front(&tiles("Kh Qc")));
    assert!(eval_front(&tiles("2h 2c")) > eval_front(&tiles("Ah Kc")));
    // Belakang harus lebih tinggi dari depan.
    assert!(valid(&tiles("9h 9c 5d 4s 2h"), &tiles("Kh Qc")));
    assert!(!valid(&tiles("Kh 9c 5d 4s 2h"), &tiles("Ah Qc")));
    assert!(!valid(&tiles("9h 9c 5d 4s 2h"), &tiles("Th Tc")));
    assert!(valid(&tiles("Ah Qc 5d 4s 2h"), &tiles("Ac Qd")), "A-Q-… lawan A-Q");
    assert!(!valid(&tiles("Ah Tc 5d 4s 2h"), &tiles("Ac Qd")));
}

#[test]
fn house_way_examples() {
    // Tanpa pair: kartu tertinggi di belakang, dua berikutnya di depan.
    let (front, _) = house_way(&tiles("Ah Kd 9c 7s 5h 3d 2c"));
    assert_eq!(front, tiles("Kd 9c"));
    // Satu pair: pair di belakang, dua kartu tertinggi di depan.
    let (front, _) = house_way(&tiles("8h 8d Ac Ks 5h 3d 2c"));
    assert_eq!(front, tiles("Ac Ks"));
    // Two pair J ke atas: dipisah, pair rendah di depan.
    let (front, _) = house_way(&tiles("Jh Jd 4c 4s 9h 7d 2c"));
    assert_eq!(front, tiles("4c 4s"));
    // Two pair rendah dengan as tunggal: tetap di belakang, as di depan.
    let (front, _) = house_way(&tiles("6h 6d 4c 4s Ah 7d 2c"));
    assert_eq!(front, tiles("Ah 7d"));
    // Three of a kind as: pair as di belakang, as + kartu tertinggi di depan.
    let (front, back) = house_way(&tiles("Ah Ad Ac 9s 7h 5d 2c"));
    assert_eq!(front.len(), 2);
    assert_eq!(eval_front(&front), eval_front(&tiles("As 9s")));
    assert_eq!(eval_back(&back).key(), "pair");
    // Full house: pair di depan, three of a kind di belakang.
    let (front, back) = house_way(&tiles("9h 9d 9c Ks Kh 5d 2c"));
    assert_eq!(front, tiles("Ks Kh"));
    assert_eq!(eval_back(&back).key(), "trips");
    // Selalu sah.
    for hand in [
        "JK Ah Kd 9c 7s 5h 3d",
        "2h 3d 4c 5s 6h 8d 8c",
        "Th Ts Tc Td 2h 3h 4h",
        "Ah 2h 3h 4h 9h Kc Kd",
    ] {
        let (f, b) = house_way(&tiles(hand));
        assert!(valid(&b, &f), "{hand}");
    }
}

/// Pemain menyusun `front`; tujuh kartu pemain lalu tujuh bandar.
fn played(player: &str, dealer: &str, cmd: &str) -> Value {
    let cards: Vec<&str> = player.split(' ').chain(dealer.split(' ')).collect();
    let mut g = pg(&cards);
    act(&mut g, "bet 100");
    act(&mut g, cmd);
    view(&g)
}

#[test]
fn deal_hides_the_dealer_and_rejects_fouls() {
    let mut g = pg(&[
        "Ah", "Kd", "9c", "7s", "5h", "3d", "2c", "Qh", "Jd", "8c", "6s", "4h", "3c", "2d",
    ]);
    act(&mut g, "bet 100");
    let v = view(&g);
    assert_eq!(v["fase"], "susun");
    assert_eq!(v["bandar"].as_array().unwrap().len(), 7);
    assert_eq!(v["bandar"][0], "??");
    assert_eq!(v["netral"], "houseway");
    assert_eq!(v["taruhan_meja"], 100);
    // A-K di depan membuat depan lebih tinggi dari belakang: ditolak.
    assert!(Session::act(&mut g, 0, "set Ah Kd").is_err());
    // Kartu yang bukan milik pemain ditolak.
    assert!(Session::act(&mut g, 0, "set Qh Jd").is_err());
    act(&mut g, "set Kd 9c");
    assert_eq!(view(&g)["fase"], "selesai");
}

#[test]
fn payouts_commission_and_copies() {
    // Pemain: belakang pair K, depan Q-J. Bandar (house way) lebih lemah di
    // kedua tangan: kedua menang, 0,95:1.
    let v = played("Kh Kd Qc Js 9h 5d 2c", "Th 8d 7c 6s 4h 3c 2d", "set Qc Js");
    assert_eq!(v["hasil_depan"], "menang");
    assert_eq!(v["hasil_belakang"], "menang");
    assert_eq!(v["bayar"], 95);
    assert_eq!(v["bersih"], 95);
    // Seri persis di tangan depan (copy) = bandar menang; belakang menang →
    // seri keseluruhan.
    let v = played("Kh Kd Qc Js 9h 5d 2c", "Ad Qd Jc 7c 6s 4h 3c", "set Qc Js");
    assert_eq!(v["hasil_depan"], "kalah");
    assert_eq!(v["hasil_belakang"], "menang");
    assert_eq!(v["bayar"], 0);
    // Kedua kalah.
    let v = played("Th 8d 7c 6s 4h 3c 2d", "Kh Kd Qc Js 9h 5d 2c", "houseway");
    assert_eq!(v["bayar"], -100);
    assert_eq!(v["dipertaruhkan"], 100);
}

#[test]
fn bets_and_commands() {
    let mut g = pg(&["Ah", "Kd", "9c", "7s", "5h", "3d", "2c", "Qh", "Jd", "8c", "6s", "4h", "3c", "2d"]);
    for bad in ["bet 10", "bet 30", "bet 2020", "houseway", "set Ah Kd"] {
        assert!(Session::act(&mut g, 0, bad).is_err(), "{bad}");
    }
    for cmd in ["bet 100", "set Ah Kd", "houseway", "leave"] {
        let a = g.parse_command(cmd).unwrap();
        assert_eq!(g.format_action(&a), cmd);
    }
    assert!(Session::view_text(&g, 0, Lang::Id).contains("bet"));
    assert!(Session::view_text(&g, 0, Lang::En).contains("bet"));
    act(&mut g, "leave");
    assert!(TurnGame::is_over(&g));
    // Dek 53 kartu dari seed.
    let a = PaiGow::new(Config::default(), [4; 32]).unwrap();
    let b = PaiGow::new(Config::default(), [5; 32]).unwrap();
    assert_ne!(Session::state_hash(&a), Session::state_hash(&b));
}
