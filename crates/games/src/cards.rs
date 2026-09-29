//! Mesin kartu bersama (SPEC §9 M4): kartu remi standar, dek, dan shoe
//! berisi beberapa dek yang dikocok dengan RNG yang disuntikkan.
//!
//! Notasi teks dua karakter: peringkat `A 2 3 4 5 6 7 8 9 T J Q K` lalu
//! jenis `s h d c` (sekop, hati, wajik, keriting), misalnya `Ah`, `Td`.
//! Dipakai game kartu casino (Blackjack di M4, meja kartu lain di M5).

use std::fmt;
use std::str::FromStr;

use kyusin_core::GameRng;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
pub enum Suit {
    Spades,
    Hearts,
    Diamonds,
    Clubs,
}

impl Suit {
    pub const ALL: [Suit; 4] = [Suit::Spades, Suit::Hearts, Suit::Diamonds, Suit::Clubs];

    pub fn letter(self) -> char {
        match self {
            Suit::Spades => 's',
            Suit::Hearts => 'h',
            Suit::Diamonds => 'd',
            Suit::Clubs => 'c',
        }
    }

    fn from_letter(c: char) -> Option<Suit> {
        Suit::ALL.into_iter().find(|s| s.letter() == c)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
pub enum Rank {
    Ace,
    Two,
    Three,
    Four,
    Five,
    Six,
    Seven,
    Eight,
    Nine,
    Ten,
    Jack,
    Queen,
    King,
}

impl Rank {
    pub const ALL: [Rank; 13] = [
        Rank::Ace,
        Rank::Two,
        Rank::Three,
        Rank::Four,
        Rank::Five,
        Rank::Six,
        Rank::Seven,
        Rank::Eight,
        Rank::Nine,
        Rank::Ten,
        Rank::Jack,
        Rank::Queen,
        Rank::King,
    ];

    pub fn letter(self) -> char {
        b"A23456789TJQK"[self as usize] as char
    }

    fn from_letter(c: char) -> Option<Rank> {
        Rank::ALL.into_iter().find(|r| r.letter() == c)
    }

    /// Nilai Blackjack: as 1 (bisa dihitung 11 oleh tangan), gambar 10.
    pub fn blackjack_value(self) -> u8 {
        match self {
            Rank::Ace => 1,
            Rank::Jack | Rank::Queen | Rank::King => 10,
            r => r as u8 + 1,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct Card {
    pub rank: Rank,
    pub suit: Suit,
}

impl Card {
    pub fn new(rank: Rank, suit: Suit) -> Self {
        Card { rank, suit }
    }

    /// Satu dek baru berurutan: sekop A–K, hati, wajik, keriting.
    pub fn deck() -> Vec<Card> {
        Suit::ALL
            .into_iter()
            .flat_map(|s| Rank::ALL.into_iter().map(move |r| Card::new(r, s)))
            .collect()
    }
}

impl fmt::Display for Card {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}{}", self.rank.letter(), self.suit.letter())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParseCardError(pub String);

impl FromStr for Card {
    type Err = ParseCardError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut chars = s.chars();
        let (Some(r), Some(su), None) = (chars.next(), chars.next(), chars.next()) else {
            return Err(ParseCardError(s.into()));
        };
        match (Rank::from_letter(r), Suit::from_letter(su)) {
            (Some(rank), Some(suit)) => Ok(Card::new(rank, suit)),
            _ => Err(ParseCardError(s.into())),
        }
    }
}

impl Serialize for Card {
    fn serialize<S: serde::Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
        s.collect_str(self)
    }
}

impl<'de> Deserialize<'de> for Card {
    fn deserialize<D: serde::Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        let text = String::deserialize(d)?;
        text.parse()
            .map_err(|_| serde::de::Error::custom(format!("kartu `{text}`")))
    }
}

/// Tumpukan kartu yang dibagikan dari atas.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Shoe {
    cards: Vec<Card>,
    next: usize,
}

impl Shoe {
    /// `decks` dek yang dikocok Fisher–Yates dengan `rng`.
    pub fn shuffled(decks: u8, rng: &mut GameRng) -> Self {
        let mut cards: Vec<Card> = (0..decks).flat_map(|_| Card::deck()).collect();
        rng.shuffle(&mut cards);
        Shoe { cards, next: 0 }
    }

    /// Urutan tetap (kartu pertama dibagi lebih dulu); untuk tes dan tutorial.
    pub fn from_cards(cards: Vec<Card>) -> Self {
        Shoe { cards, next: 0 }
    }

    pub fn draw(&mut self) -> Option<Card> {
        let card = self.cards.get(self.next).copied();
        if card.is_some() {
            self.next += 1;
        }
        card
    }

    pub fn len(&self) -> usize {
        self.cards.len()
    }

    pub fn is_empty(&self) -> bool {
        self.cards.is_empty()
    }

    pub fn dealt(&self) -> usize {
        self.next
    }

    pub fn remaining(&self) -> usize {
        self.cards.len() - self.next
    }

    pub fn cards(&self) -> &[Card] {
        &self.cards
    }
}
