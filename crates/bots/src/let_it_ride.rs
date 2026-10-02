//! Strategi optimal Let It Ride (D-060). Tiap tempat dibayar menurut tangan
//! akhir yang sama, jadi keputusan tempat 1 dan tempat 2 tidak saling
//! bergantung: sebuah tempat dibiarkan (ride) bila rata-rata pembayarannya
//! atas semua kartu yang mungkin positif. Rata-rata itu dihitung tepat
//! sekali untuk semua 22.100 tangan tiga kartu (×1.176 pasangan kartu
//! bersama) dan 270.725 tangan empat kartu (×48 kartu terakhir), lalu
//! disimpan sebagai tabel. Dipakai untuk menghitung dan memverifikasi RTP
//! (SPEC §7).

use std::sync::OnceLock;

use kyusin_core::{Seed, TurnGame};
use kyusin_games::cards::Card;
use kyusin_games::let_it_ride::{self, LetItRide, View, pays};
use kyusin_games::poker::eval5;

use crate::meja::{Wager, simulate};

fn index(c: &Card) -> usize {
    c.suit as usize * 13 + c.rank as usize
}

fn deck() -> Vec<Card> {
    let mut d = vec![Card::deck()[0]; 52];
    for c in Card::deck() {
        d[index(&c)] = c;
    }
    d
}

fn key(cards: &mut [usize]) -> usize {
    cards.sort_unstable();
    cards.iter().fold(0, |k, &c| k * 52 + c)
}

/// Tabel ride untuk tiga kartu, berindeks kunci kartu terurut.
fn three() -> &'static Vec<bool> {
    static T: OnceLock<Vec<bool>> = OnceLock::new();
    T.get_or_init(|| {
        let d = deck();
        let mut t = vec![false; 52 * 52 * 52];
        for a in 0..52 {
            for b in a + 1..52 {
                for c in b + 1..52 {
                    let mut sum = 0i64;
                    for x in (0..52).filter(|x| ![a, b, c].contains(x)) {
                        for y in (x + 1..52).filter(|y| ![a, b, c].contains(y)) {
                            sum += pays(eval5(&[d[a], d[b], d[c], d[x], d[y]]));
                        }
                    }
                    t[key(&mut [a, b, c])] = sum > 0;
                }
            }
        }
        t
    })
}

/// Tabel ride untuk empat kartu.
fn four() -> &'static Vec<bool> {
    static T: OnceLock<Vec<bool>> = OnceLock::new();
    T.get_or_init(|| {
        let d = deck();
        let mut t = vec![false; 52 * 52 * 52 * 52];
        for a in 0..52 {
            for b in a + 1..52 {
                for c in b + 1..52 {
                    for e in c + 1..52 {
                        let sum: i64 = (0..52)
                            .filter(|x| ![a, b, c, e].contains(x))
                            .map(|x| pays(eval5(&[d[a], d[b], d[c], d[e], d[x]])))
                            .sum();
                        t[key(&mut [a, b, c, e])] = sum > 0;
                    }
                }
            }
        }
        t
    })
}

/// Apakah tempat dibiarkan dengan kartu yang terlihat (3 atau 4 kartu).
pub fn rides(cards: &[Card]) -> bool {
    let mut idx: Vec<usize> = cards.iter().map(index).collect();
    match idx.len() {
        3 => three()[key(&mut idx)],
        4 => four()[key(&mut idx)],
        _ => true,
    }
}

pub fn decide(v: &View, bet: i64) -> String {
    match v.meja.fase.as_str() {
        "keputusan" => {
            let seen: Vec<Card> = v
                .pemain
                .iter()
                .chain(v.bersama.iter())
                .filter_map(|c| c.parse().ok())
                .collect();
            if rides(&seen) { "ride" } else { "pull" }.into()
        }
        _ => format!("bet {bet}"),
    }
}

pub fn wagers() -> Vec<Wager> {
    let_it_ride::RTP
        .iter()
        .map(|&rtp| Wager {
            game: let_it_ride::ID,
            rtp,
            run: |n, s: &Seed| {
                simulate(
                    n,
                    s,
                    100,
                    |seed| LetItRide::new(Default::default(), seed).expect("dek"),
                    |_, v| decide(v, 100),
                )
            },
        })
        .collect()
}

/// RTP tepat per tempat: semua tangan tiga kartu × semua urutan dua kartu
/// bersama (52 juta tangan). Dijalankan di workflow `rtp.yml`.
pub fn exact() -> f64 {
    let d = deck();
    let (t3, t4) = (three(), four());
    let (mut total, mut n) = (0f64, 0f64);
    for a in 0..52 {
        for b in a + 1..52 {
            for c in b + 1..52 {
                let r1 = f64::from(u8::from(t3[key(&mut [a, b, c])]));
                for x in (0..52).filter(|x| ![a, b, c].contains(x)) {
                    let r2 = f64::from(u8::from(t4[key(&mut [a, b, c, x])]));
                    for y in (0..52).filter(|y| ![a, b, c, x].contains(y)) {
                        let p = pays(eval5(&[d[a], d[b], d[c], d[x], d[y]])) as f64;
                        total += p * (r1 + r2 + 1.0);
                        n += 1.0;
                    }
                }
            }
        }
    }
    100.0 * (1.0 + total / n)
}
