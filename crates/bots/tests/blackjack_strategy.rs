//! Strategi dasar Blackjack dan verifikasi RTP (SPEC §7, D-057).
//!
//! CI setiap push: ≥100.000 ronde; RTP simulasi harus berada dalam
//! 4 × σ/√n dari angka di manifest. Workflow `rtp.yml` menjalankan
//! ≥10.000.000 ronde secara manual/terjadwal.

use kyusin_bots::blackjack::{Options, decide_hand, simulate};
use kyusin_games::blackjack::HandView;

fn hand(cards: &[&str]) -> HandView {
    let parsed: Vec<kyusin_games::cards::Card> = cards.iter().map(|c| c.parse().unwrap()).collect();
    let (nilai, lunak) = kyusin_games::blackjack::value(&parsed);
    HandView {
        kartu: cards.iter().map(|c| c.to_string()).collect(),
        taruhan: 100,
        nilai,
        lunak,
        ganda: false,
        blackjack: false,
        selesai: false,
        hasil: None,
        bayar: None,
    }
}

const ALL: Options = Options {
    double: true,
    split: true,
    surrender: true,
};

#[test]
fn chart_spot_checks() {
    let d = |cards: &[&str], up: u8| decide_hand(&hand(cards), up, ALL);
    assert_eq!(d(&["Th", "6c"], 10), "surrender");
    assert_eq!(d(&["Th", "6c"], 9), "surrender");
    assert_eq!(d(&["Th", "5c"], 10), "surrender");
    assert_eq!(d(&["Th", "5c"], 9), "hit");
    assert_eq!(d(&["8h", "8c"], 11), "split");
    assert_eq!(d(&["8h", "8c"], 10), "split");
    assert_eq!(d(&["Ah", "Ac"], 6), "split");
    assert_eq!(d(&["Th", "Kc"], 6), "stand");
    assert_eq!(d(&["9h", "9c"], 7), "stand");
    assert_eq!(d(&["9h", "9c"], 8), "split");
    assert_eq!(d(&["5h", "5c"], 6), "double");
    assert_eq!(d(&["6h", "5c"], 10), "double");
    assert_eq!(d(&["6h", "5c"], 11), "hit");
    assert_eq!(d(&["Ah", "7c"], 6), "double");
    assert_eq!(d(&["Ah", "7c"], 7), "stand");
    assert_eq!(d(&["Ah", "7c"], 10), "hit");
    assert_eq!(d(&["Ah", "2c"], 4), "hit");
    assert_eq!(d(&["Ah", "2c"], 5), "double");
    assert_eq!(d(&["Th", "2c"], 3), "hit");
    assert_eq!(d(&["Th", "2c"], 4), "stand");
    assert_eq!(d(&["Th", "3c"], 6), "stand");
    assert_eq!(d(&["Th", "3c"], 7), "hit");
    // Double tidak diizinkan: hit, kecuali soft 18 yang berdiri.
    let no_double = Options {
        double: false,
        ..ALL
    };
    assert_eq!(decide_hand(&hand(&["6h", "5c"]), 6, no_double), "hit");
    assert_eq!(decide_hand(&hand(&["Ah", "7c"]), 6, no_double), "stand");
    assert_eq!(
        decide_hand(&hand(&["2h", "Ac", "5d"]), 6, no_double),
        "stand"
    );
}

#[test]
fn rtp_matches_the_manifest_within_four_sigma() {
    let rtp = simulate(100_000, &[42; 32]);
    let registry = kyusin_games::builtin().unwrap();
    let manifest = registry
        .get(kyusin_games::blackjack::ID)
        .unwrap()
        .manifest
        .rtp
        .unwrap();
    eprintln!(
        "RTP {:.4}% ± {:.4}% (manifest {manifest}%, {} ronde)",
        rtp.percent(),
        rtp.tolerance_percent(),
        rtp.rounds
    );
    assert!(rtp.rounds >= 100_000);
    assert!(
        (rtp.percent() - manifest).abs() <= rtp.tolerance_percent(),
        "RTP {:.4}% di luar {manifest}% ± {:.4}%",
        rtp.percent(),
        rtp.tolerance_percent()
    );
}
