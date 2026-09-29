//! Tes mesin kartu bersama (SPEC §9 M4), ditulis sebelum implementasinya.
//!
//! Kartu remi standar: 4 jenis × 13 peringkat. Notasi teks dua karakter:
//! peringkat `A 2 3 4 5 6 7 8 9 T J Q K` lalu jenis `s h d c`
//! (sekop, hati, wajik, keriting), misalnya `Ah`, `Td`, `9c`. Shoe berisi
//! beberapa dek yang dikocok dengan RNG yang disuntikkan (Fisher–Yates),
//! jadi urutannya ditentukan seluruhnya oleh seed.

use kyusin_core::GameRng;
use kyusin_games::cards::{Card, Rank, Shoe, Suit};

#[test]
fn a_deck_has_52_distinct_cards() {
    let deck = Card::deck();
    assert_eq!(deck.len(), 52);
    let mut names: Vec<String> = deck.iter().map(|c| c.to_string()).collect();
    names.sort();
    names.dedup();
    assert_eq!(names.len(), 52);
    for suit in Suit::ALL {
        assert_eq!(deck.iter().filter(|c| c.suit == suit).count(), 13);
    }
    for rank in Rank::ALL {
        assert_eq!(deck.iter().filter(|c| c.rank == rank).count(), 4);
    }
}

#[test]
fn notation_round_trip() {
    for card in Card::deck() {
        let text = card.to_string();
        assert_eq!(text.len(), 2, "{text}");
        assert_eq!(text.parse::<Card>().unwrap(), card);
    }
    assert_eq!(
        "Ah".parse::<Card>().unwrap(),
        Card::new(Rank::Ace, Suit::Hearts)
    );
    assert_eq!(
        "Td".parse::<Card>().unwrap(),
        Card::new(Rank::Ten, Suit::Diamonds)
    );
    assert_eq!(
        "9c".parse::<Card>().unwrap(),
        Card::new(Rank::Nine, Suit::Clubs)
    );
    assert_eq!(
        "Ks".parse::<Card>().unwrap(),
        Card::new(Rank::King, Suit::Spades)
    );
    for bad in ["", "A", "1h", "Ax", "10h", "ah", "AH", "Ahh"] {
        assert!(bad.parse::<Card>().is_err(), "{bad}");
    }
}

#[test]
fn six_deck_shoe_has_every_card_six_times() {
    let mut rng = GameRng::from_seed([1; 32]);
    let shoe = Shoe::shuffled(6, &mut rng);
    assert_eq!(shoe.len(), 312);
    assert_eq!(shoe.remaining(), 312);
    for card in Card::deck() {
        assert_eq!(
            shoe.cards().iter().filter(|c| **c == card).count(),
            6,
            "{card}"
        );
    }
}

#[test]
fn same_seed_same_order_different_seed_different_order() {
    let order = |seed: u8| {
        let mut rng = GameRng::from_seed([seed; 32]);
        Shoe::shuffled(6, &mut rng).cards().to_vec()
    };
    assert_eq!(order(3), order(3));
    assert_ne!(order(3), order(4));
    // Benar-benar dikocok: tidak sama dengan urutan dek baru.
    let fresh: Vec<Card> = (0..6).flat_map(|_| Card::deck()).collect();
    assert_ne!(order(3), fresh);
}

#[test]
fn drawing_takes_from_the_top_and_counts_dealt_cards() {
    let cards: Vec<Card> = ["Ah", "Kd", "2c"]
        .iter()
        .map(|c| c.parse().unwrap())
        .collect();
    let mut shoe = Shoe::from_cards(cards.clone());
    assert_eq!(shoe.dealt(), 0);
    assert_eq!(shoe.draw(), Some(cards[0]));
    assert_eq!(shoe.draw(), Some(cards[1]));
    assert_eq!(shoe.dealt(), 2);
    assert_eq!(shoe.remaining(), 1);
    assert_eq!(shoe.draw(), Some(cards[2]));
    assert_eq!(shoe.draw(), None);
}

#[test]
fn blackjack_values() {
    assert_eq!(Rank::Ace.blackjack_value(), 1);
    assert_eq!(Rank::Nine.blackjack_value(), 9);
    for r in [Rank::Ten, Rank::Jack, Rank::Queen, Rank::King] {
        assert_eq!(r.blackjack_value(), 10);
    }
}
