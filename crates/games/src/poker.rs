//! Peringkat tangan poker bersama untuk meja casino (M5a): lima kartu
//! (termasuk terbaik 5 dari 6–7 kartu) dan tiga kartu (Three Card Poker).
//!
//! Nilai tangan adalah bilangan yang bisa dibandingkan langsung: kategori
//! di bit atas, lalu peringkat penentu (as = 14) dari yang terpenting.
//! Wheel A-2-3-4-5 adalah straight terendah; Pai Gow Poker, yang memakai
//! aturan wheel berbeda dan joker, punya evaluatornya sendiri.

use crate::cards::{Card, Rank};

/// Nilai peringkat untuk poker: 2–10, J 11, Q 12, K 13, A 14.
pub fn rank_value(r: Rank) -> u8 {
    match r {
        Rank::Ace => 14,
        r => r as u8 + 1,
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Category {
    HighCard,
    Pair,
    TwoPair,
    Trips,
    Straight,
    Flush,
    FullHouse,
    Quads,
    StraightFlush,
}

impl Category {
    const ALL: [Category; 9] = [
        Category::HighCard,
        Category::Pair,
        Category::TwoPair,
        Category::Trips,
        Category::Straight,
        Category::Flush,
        Category::FullHouse,
        Category::Quads,
        Category::StraightFlush,
    ];

    /// Kunci teks (dipakai view dan terjemahan).
    pub fn key(self) -> &'static str {
        match self {
            Category::HighCard => "high_card",
            Category::Pair => "pair",
            Category::TwoPair => "two_pair",
            Category::Trips => "trips",
            Category::Straight => "straight",
            Category::Flush => "flush",
            Category::FullHouse => "full_house",
            Category::Quads => "quads",
            Category::StraightFlush => "straight_flush",
        }
    }
}

/// Nilai tangan lima kartu; lebih besar = lebih kuat.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Value(pub u32);

impl Value {
    fn new(cat: Category, ranks: &[u8]) -> Value {
        let mut v = (cat as u32) << 20;
        for (i, r) in ranks.iter().take(5).enumerate() {
            v |= u32::from(*r) << (16 - 4 * i);
        }
        Value(v)
    }

    pub fn category(self) -> Category {
        Category::ALL[(self.0 >> 20) as usize]
    }

    /// Peringkat penentu ke-`i` (0 = terpenting).
    pub fn rank(self, i: usize) -> u8 {
        ((self.0 >> (16 - 4 * i)) & 0xF) as u8
    }

    /// Straight flush as tinggi.
    pub fn is_royal(self) -> bool {
        self.category() == Category::StraightFlush && self.rank(0) == 14
    }

    /// Kunci teks, dengan royal flush dipisah.
    pub fn key(self) -> &'static str {
        if self.is_royal() {
            "royal_flush"
        } else {
            self.category().key()
        }
    }
}

/// Kartu tertinggi straight dalam bitmask peringkat (bit v = peringkat v,
/// bit 1 = as rendah), bila ada.
fn straight_high(mask: u16) -> Option<u8> {
    let mask = if mask & (1 << 14) != 0 {
        mask | 0b10
    } else {
        mask
    };
    (5..=14u8)
        .rev()
        .find(|&h| (mask >> (h - 4)) & 0b11111 == 0b11111)
}

fn top_bits(mask: u16, n: usize) -> Vec<u8> {
    (2..=14u8)
        .rev()
        .filter(|r| mask & (1 << r) != 0)
        .take(n)
        .collect()
}

/// Nilai tangan terbaik dari 5–7 kartu.
pub fn eval_best(cards: &[Card]) -> Value {
    debug_assert!((5..=7).contains(&cards.len()));
    let mut counts = [0u8; 15];
    let mut suits = [0u16; 4];
    let mut suit_n = [0u8; 4];
    let mut all = 0u16;
    for c in cards {
        let v = rank_value(c.rank);
        counts[v as usize] += 1;
        suits[c.suit as usize] |= 1 << v;
        suit_n[c.suit as usize] += 1;
        all |= 1 << v;
    }
    let flush = (0..4).find(|&s| suit_n[s] >= 5);
    if let Some(s) = flush
        && let Some(h) = straight_high(suits[s])
    {
        return Value::new(Category::StraightFlush, &[h]);
    }
    // Peringkat dikelompokkan menurut jumlah, tertinggi dulu.
    let by = |n: u8| -> Vec<u8> {
        (2..=14u8)
            .rev()
            .filter(|&r| counts[r as usize] == n)
            .collect()
    };
    let quads = by(4);
    let trips = by(3);
    let pairs = by(2);
    let kickers = |exclude: &[u8], n: usize| -> Vec<u8> {
        (2..=14u8)
            .rev()
            .filter(|r| counts[*r as usize] > 0 && !exclude.contains(r))
            .take(n)
            .collect()
    };
    if let Some(&q) = quads.first() {
        let mut r = vec![q];
        r.extend(kickers(&[q], 1));
        return Value::new(Category::Quads, &r);
    }
    if let Some(&t) = trips.first() {
        let pair = trips.iter().skip(1).chain(pairs.iter()).copied().max();
        if let Some(p) = pair {
            return Value::new(Category::FullHouse, &[t, p]);
        }
    }
    if let Some(s) = flush {
        return Value::new(Category::Flush, &top_bits(suits[s], 5));
    }
    if let Some(h) = straight_high(all) {
        return Value::new(Category::Straight, &[h]);
    }
    if let Some(&t) = trips.first() {
        let mut r = vec![t];
        r.extend(kickers(&[t], 2));
        return Value::new(Category::Trips, &r);
    }
    if pairs.len() >= 2 {
        let (a, b) = (pairs[0], pairs[1]);
        let mut r = vec![a, b];
        r.extend(kickers(&[a, b], 1));
        return Value::new(Category::TwoPair, &r);
    }
    if let Some(&p) = pairs.first() {
        let mut r = vec![p];
        r.extend(kickers(&[p], 3));
        return Value::new(Category::Pair, &r);
    }
    Value::new(Category::HighCard, &top_bits(all, 5))
}

/// Nilai tepat lima kartu.
pub fn eval5(cards: &[Card]) -> Value {
    assert_eq!(cards.len(), 5, "eval5 butuh lima kartu");
    eval_best(cards)
}

/// Tangan lima kartu terbaik dari 5–7 kartu, beserta kartunya (untuk
/// ditampilkan; lebih lambat dari [`eval_best`]).
pub fn best5(cards: &[Card]) -> (Value, Vec<Card>) {
    let n = cards.len();
    let mut best: Option<(Value, Vec<Card>)> = None;
    for mask in 0u32..(1 << n) {
        if mask.count_ones() != 5 {
            continue;
        }
        let pick: Vec<Card> = (0..n)
            .filter(|i| mask & (1 << i) != 0)
            .map(|i| cards[i])
            .collect();
        let v = eval_best(&pick);
        if best.as_ref().is_none_or(|(b, _)| v > *b) {
            best = Some((v, pick));
        }
    }
    best.expect("paling sedikit lima kartu")
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Category3 {
    HighCard,
    Pair,
    Flush,
    Straight,
    Trips,
    StraightFlush,
}

impl Category3 {
    const ALL: [Category3; 6] = [
        Category3::HighCard,
        Category3::Pair,
        Category3::Flush,
        Category3::Straight,
        Category3::Trips,
        Category3::StraightFlush,
    ];

    pub fn key(self) -> &'static str {
        match self {
            Category3::HighCard => "high_card",
            Category3::Pair => "pair",
            Category3::Flush => "flush",
            Category3::Straight => "straight",
            Category3::Trips => "trips",
            Category3::StraightFlush => "straight_flush",
        }
    }
}

/// Nilai tangan tiga kartu (Three Card Poker).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Value3(pub u32);

impl Value3 {
    pub fn category(self) -> Category3 {
        Category3::ALL[(self.0 >> 12) as usize]
    }

    /// Peringkat penentu ke-`i` (0 = terpenting).
    pub fn rank(self, i: usize) -> u8 {
        ((self.0 >> (8 - 4 * i)) & 0xF) as u8
    }
}

/// Nilai tepat tiga kartu.
pub fn eval3(cards: &[Card]) -> Value3 {
    assert_eq!(cards.len(), 3, "eval3 butuh tiga kartu");
    let mut r: Vec<u8> = cards.iter().map(|c| rank_value(c.rank)).collect();
    r.sort_unstable_by(|a, b| b.cmp(a));
    let flush = cards.iter().all(|c| c.suit == cards[0].suit);
    let straight = if r[0] == r[1] + 1 && r[1] == r[2] + 1 {
        Some(r[0])
    } else if r == [14, 3, 2] {
        Some(3)
    } else {
        None
    };
    let make = |cat: Category3, ranks: &[u8]| {
        let mut v = (cat as u32) << 12;
        for (i, x) in ranks.iter().enumerate() {
            v |= u32::from(*x) << (8 - 4 * i);
        }
        Value3(v)
    };
    match (straight, flush) {
        (Some(h), true) => make(Category3::StraightFlush, &[h]),
        _ if r[0] == r[2] => make(Category3::Trips, &[r[0]]),
        (Some(h), false) => make(Category3::Straight, &[h]),
        (None, true) => make(Category3::Flush, &r),
        _ if r[0] == r[1] => make(Category3::Pair, &[r[0], r[2]]),
        _ if r[1] == r[2] => make(Category3::Pair, &[r[1], r[0]]),
        _ => make(Category3::HighCard, &r),
    }
}
