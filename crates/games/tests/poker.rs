//! Tes peringkat tangan poker bersama (M5a), ditulis sebelum evaluatornya.
//!
//! Lima kartu (Caribbean Stud, Let It Ride, Casino Hold'em terbaik 5 dari
//! 7, Pai Gow Poker), dari yang tertinggi: straight flush, four of a kind,
//! full house, flush, straight, three of a kind, two pair, one pair, high
//! card. As tinggi, dan A-2-3-4-5 (wheel) adalah straight terendah (Pai
//! Gow punya aturan wheel sendiri di modulnya). Royal flush = straight
//! flush as tinggi. Tiga kartu (Three Card Poker), dari yang tertinggi:
//! straight flush, three of a kind, straight, flush, pair, high card; A-2-3
//! straight terendah, Q-K-A tertinggi.

use kyusin_games::cards::Card;
use kyusin_games::poker::{Category, Category3, best5, eval3, eval5};

fn cards(text: &str) -> Vec<Card> {
    text.split_whitespace()
        .map(|c| c.parse().unwrap())
        .collect()
}

fn five(text: &str) -> kyusin_games::poker::Value {
    eval5(&cards(text))
}

#[test]
fn categories_rank_in_poker_order() {
    let order = [
        ("2h 7d 9c Js Kh", Category::HighCard),
        ("2h 2d 9c Js Kh", Category::Pair),
        ("2h 2d 9c 9s Kh", Category::TwoPair),
        ("2h 2d 2c 9s Kh", Category::Trips),
        ("5h 6d 7c 8s 9h", Category::Straight),
        ("2h 7h 9h Jh Kh", Category::Flush),
        ("2h 2d 2c 9s 9h", Category::FullHouse),
        ("2h 2d 2c 2s 9h", Category::Quads),
        ("5h 6h 7h 8h 9h", Category::StraightFlush),
    ];
    for w in order.windows(2) {
        let (lo, hi) = (five(w[0].0), five(w[1].0));
        assert_eq!(lo.category(), w[0].1, "{}", w[0].0);
        assert_eq!(hi.category(), w[1].1, "{}", w[1].0);
        assert!(lo < hi, "{} < {}", w[0].0, w[1].0);
    }
}

#[test]
fn kickers_and_pairs_break_ties() {
    assert!(five("Ah Kd 9c 7s 3h") > five("Ah Kd 9c 7s 2h"));
    assert!(five("Ah Kd 9c 7s 3h") < five("Ah Kd Tc 2s 3h"));
    assert!(five("8h 8d Ac 4s 3h") > five("8c 8s Kc Qs Jh"));
    assert!(five("9h 9d 2c 2s 3h") > five("8h 8d 7c 7s Ah"));
    assert!(five("9h 9d 4c 4s 3h") > five("9c 9s 3c 3s Ah"));
    assert!(five("9h 9d 4c 4s 5h") > five("9c 9s 4d 4h 3h"));
    assert!(five("3h 3d 3c 2s 2h") > five("2c 2d 2s As Ah"));
    assert!(five("Ah Qh 9h 7h 3h") > five("Ad Jd 9d 7d 6d"));
    // Setara persis: sama.
    assert_eq!(five("Ah Kd 9c 7s 3h"), five("Ac Kh 9d 7h 3s"));
}

#[test]
fn ace_plays_high_and_low_in_straights() {
    let wheel = five("Ah 2d 3c 4s 5h");
    let six = five("2d 3c 4s 5h 6d");
    let broadway = five("Th Jd Qc Ks Ah");
    assert_eq!(wheel.category(), Category::Straight);
    assert!(wheel < six, "wheel adalah straight terendah");
    assert!(six < broadway);
    // Q-K-A-2-3 bukan straight.
    assert_eq!(five("Qh Kd Ac 2s 3h").category(), Category::HighCard);
    let royal = five("Th Jh Qh Kh Ah");
    assert!(royal.is_royal());
    assert!(!five("9h Th Jh Qh Kh").is_royal());
    assert!(royal > five("9h Th Jh Qh Kh"));
    assert!(five("Ah 2h 3h 4h 5h") < five("2h 3h 4h 5h 6h"));
}

#[test]
fn best_five_of_seven() {
    // Flush lebih baik dari straight yang juga ada.
    let (v, used) = best5(&cards("4h 5d 6h 7h 8c Kh 2h"));
    assert_eq!(v.category(), Category::Flush);
    assert_eq!(used.len(), 5);
    assert!(
        used.iter()
            .all(|c| c.suit == kyusin_games::cards::Suit::Hearts)
    );
    // Full house dari dua trips: trips tertinggi + pasangan dari yang lain.
    let (v, _) = best5(&cards("9h 9d 9c 4s 4h 4d Ac"));
    assert_eq!(v, five("9h 9d 9c 4s 4h"));
    // Two pair terbaik dari tiga pasangan, kicker tertinggi.
    let (v, _) = best5(&cards("9h 9d 5c 5s 2h 2d Kc"));
    assert_eq!(v, five("9h 9d 5c 5s Kc"));
    // Lima kartu: sama dengan eval5.
    assert_eq!(best5(&cards("Ah 2d 3c 4s 5h")).0, five("Ah 2d 3c 4s 5h"));
    // Enam kartu.
    assert_eq!(
        best5(&cards("Ah Ad 3c 4s 5h Ac")).0.category(),
        Category::Trips
    );
}

#[test]
fn three_card_hands() {
    let order = [
        ("2h 7d 9c", Category3::HighCard),
        ("2h 2d 9c", Category3::Pair),
        ("2h 7h 9h", Category3::Flush),
        ("5h 6d 7c", Category3::Straight),
        ("2h 2d 2c", Category3::Trips),
        ("5h 6h 7h", Category3::StraightFlush),
    ];
    for w in order.windows(2) {
        let (lo, hi) = (eval3(&cards(w[0].0)), eval3(&cards(w[1].0)));
        assert_eq!(lo.category(), w[0].1, "{}", w[0].0);
        assert_eq!(hi.category(), w[1].1, "{}", w[1].0);
        assert!(lo < hi, "{} < {}", w[0].0, w[1].0);
    }
    let a23 = eval3(&cards("Ah 2d 3c"));
    assert_eq!(a23.category(), Category3::Straight);
    assert!(a23 < eval3(&cards("2h 3d 4c")));
    assert!(eval3(&cards("Qh Kd Ac")) > eval3(&cards("Jh Qd Kc")));
    assert_eq!(eval3(&cards("Kh Ad 2c")).category(), Category3::HighCard);
    // Q-6-4 dibandingkan dengan Q-6-3 dan Q-7-2.
    assert!(eval3(&cards("Qh 6d 4c")) > eval3(&cards("Qd 6c 3h")));
    assert!(eval3(&cards("Qh 6d 4c")) < eval3(&cards("Qd 7c 2h")));
    assert!(eval3(&cards("9h 9d 3c")) > eval3(&cards("9c 9s 2h")));
    assert!(eval3(&cards("Th 9d 3c")) < eval3(&cards("2c 2s 3h")));
}

/// Frekuensi kategori untuk semua tangan dari satu dek (angka baku).
#[test]
fn full_deck_frequencies() {
    let deck = Card::deck();
    let mut five_counts = [0u32; 9];
    let mut royal = 0;
    for a in 0..52 {
        for b in a + 1..52 {
            for c in b + 1..52 {
                for d in c + 1..52 {
                    for e in d + 1..52 {
                        let hand = [deck[a], deck[b], deck[c], deck[d], deck[e]];
                        let v = eval5(&hand);
                        five_counts[v.category() as usize] += 1;
                        royal += v.is_royal() as u32;
                    }
                }
            }
        }
    }
    assert_eq!(
        five_counts,
        [
            1_302_540, 1_098_240, 123_552, 54_912, 10_200, 5_108, 3_744, 624, 40
        ]
    );
    assert_eq!(royal, 4);

    let mut three = [0u32; 6];
    for a in 0..52 {
        for b in a + 1..52 {
            for c in b + 1..52 {
                three[eval3(&[deck[a], deck[b], deck[c]]).category() as usize] += 1;
            }
        }
    }
    assert_eq!(three, [16_440, 3_744, 1_096, 720, 52, 48]);
}
