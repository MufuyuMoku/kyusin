//! Bot Capsa Susun (SPEC §7.2, §8; M5b-2). Bot hanya membaca view kursinya
//! sendiri (13 kartunya).
//!
//! Strategi dasar ([`best`]): menilai semua susunan sah dengan peluang
//! tiap baris mengalahkan baris yang sama dari tangan lawan biasa (tabel
//! dari 2.000 tangan acak yang disusun dengan saran), ditambah royalti dan
//! sapu bersih, lalu memilih yang nilai harapannya terbesar. Level lebih
//! rendah memakai susunan saran (belakang lima kartu terbaik, tengah lima
//! terbaik dari sisanya) pada sebagian tangan ([`MIX`]):
//!
//! 1. Pemula: susunan terbaik 60%, susunan saran 40%.
//! 2. Menengah: susunan terbaik 85%, susunan saran 15%.
//! 3. Kuat: selalu susunan terbaik.
//!
//! Dalam sesi 60 tangan, selisih kecil per tangan sudah berarti ratusan
//! poin rating; campuran ini menjaga tangga level 100–400 (SPEC §8).

use std::sync::OnceLock;

use kyusin_core::{GameRng, Player, PlayerId, SeatKind, Seed, Session};
use kyusin_games::capsa_susun::{View, front_value, row_value, royalty, suggest, valid};
use kyusin_games::cards::Card;
use kyusin_games::poker::Value;

pub const LEVELS: u8 = 3;

/// Peluang tiap level memakai susunan terbaik dan susunan saran per
/// tangan; sisanya susunan acak.
pub const MIX: [[f64; 2]; 3] = [[0.6, 0.4], [0.85, 0.15], [1.0, 0.0]];

pub struct CapsaBot {
    level: u8,
    rng: GameRng,
}

impl CapsaBot {
    pub fn new(level: u8, seed: Seed) -> Self {
        CapsaBot {
            level,
            rng: GameRng::from_seed(seed),
        }
    }
}

/// Nilai baris depan, tengah, dan belakang dari tangan lawan biasa,
/// terurut, untuk menaksir peluang menang tiap baris.
fn tables() -> &'static [Vec<Value>; 3] {
    static TABLES: OnceLock<[Vec<Value>; 3]> = OnceLock::new();
    TABLES.get_or_init(|| {
        let mut rng = GameRng::from_seed([0x5c; 32]);
        let mut rows: [Vec<Value>; 3] = Default::default();
        for _ in 0..2000 {
            let mut deck = Card::deck();
            rng.shuffle(&mut deck);
            let s = suggest(&deck[..13]);
            rows[0].push(front_value(&s[..3]));
            rows[1].push(row_value(&s[3..8]));
            rows[2].push(row_value(&s[8..]));
        }
        for r in &mut rows {
            r.sort();
        }
        rows
    })
}

/// Peluang baris bernilai `v` di posisi `row` mengalahkan lawan biasa
/// (seri dihitung setengah).
fn win_chance(row: usize, v: Value) -> f64 {
    let t = &tables()[row];
    let below = t.partition_point(|x| *x < v);
    let upto = t.partition_point(|x| *x <= v);
    (below as f64 + (upto - below) as f64 / 2.0) / t.len() as f64
}

/// Nilai harapan poin susunan (depan 3, tengah 5, belakang 5) melawan satu
/// lawan biasa.
pub fn expected(rows: &[Card]) -> f64 {
    let v = [
        front_value(&rows[..3]),
        row_value(&rows[3..8]),
        row_value(&rows[8..]),
    ];
    let p: Vec<f64> = (0..3).map(|i| win_chance(i, v[i])).collect();
    let base: f64 = p.iter().map(|x| 2.0 * x - 1.0).sum();
    let scoop = 3.0 * (p[0] * p[1] * p[2] - (1.0 - p[0]) * (1.0 - p[1]) * (1.0 - p[2]));
    let bonus: f64 = (0..3).map(|i| royalty(i, v[i]) as f64 * p[i]).sum();
    base + scoop + bonus
}

fn combos(n: usize, k: usize) -> Vec<Vec<usize>> {
    let mut out = Vec::new();
    let mut idx: Vec<usize> = (0..k).collect();
    loop {
        out.push(idx.clone());
        let Some(i) = (0..k).rev().find(|&i| idx[i] != i + n - k) else {
            return out;
        };
        idx[i] += 1;
        for j in i + 1..k {
            idx[j] = idx[j - 1] + 1;
        }
    }
}

/// Susunan sah dengan nilai harapan terbesar (level 3).
pub fn best(cards: &[Card]) -> Vec<Card> {
    let mut top: Option<(f64, Vec<Card>)> = None;
    let mids = combos(8, 5);
    for back_idx in combos(13, 5) {
        let back: Vec<Card> = back_idx.iter().map(|&i| cards[i]).collect();
        let bv = row_value(&back);
        let rest: Vec<Card> = (0..13)
            .filter(|i| !back_idx.contains(i))
            .map(|i| cards[i])
            .collect();
        for mid_idx in &mids {
            let mid: Vec<Card> = mid_idx.iter().map(|&i| rest[i]).collect();
            let mv = row_value(&mid);
            if mv > bv {
                continue;
            }
            let front: Vec<Card> = (0..8)
                .filter(|i| !mid_idx.contains(i))
                .map(|i| rest[i])
                .collect();
            if front_value(&front) > mv {
                continue;
            }
            let rows: Vec<Card> = front
                .into_iter()
                .chain(mid)
                .chain(back.iter().copied())
                .collect();
            let e = expected(&rows);
            if top.as_ref().is_none_or(|t| e > t.0) {
                top = Some((e, rows));
            }
        }
    }
    top.map(|t| t.1).unwrap_or_else(|| suggest(cards))
}

/// Susunan sah acak (level 1): kartu dikocok lalu dibagi ke tiga baris;
/// bila tidak sah, baris ditukar menurut kekuatannya, dan bila tetap tidak
/// sah dicoba lagi (paling banyak 50 kali) sebelum memakai saran.
pub fn random(cards: &[Card], rng: &mut GameRng) -> Vec<Card> {
    for _ in 0..50 {
        let mut c = cards.to_vec();
        rng.shuffle(&mut c);
        let (mut a, mut b) = (c[3..8].to_vec(), c[8..].to_vec());
        if row_value(&a) > row_value(&b) {
            std::mem::swap(&mut a, &mut b);
        }
        let rows: Vec<Card> = c[..3].iter().copied().chain(a).chain(b).collect();
        if valid(&rows[..3], &rows[3..8], &rows[8..]) {
            return rows;
        }
    }
    suggest(cards)
}

impl Player for CapsaBot {
    fn kind(&self) -> SeatKind {
        SeatKind::Bot { level: self.level }
    }

    fn decide(&mut self, session: &dyn Session, seat: PlayerId) -> Option<String> {
        if !session.pending_players().contains(&seat) {
            return None;
        }
        let v: View = serde_json::from_value(session.view_data(seat)).ok()?;
        if v.meja.fase != "susun" {
            return Some("next".into());
        }
        let cards: Vec<Card> = v.kursi[v.kamu as usize]
            .kartu
            .iter()
            .filter_map(|c| c.parse().ok())
            .collect();
        if cards.len() != 13 {
            return Some("auto".into());
        }
        // Per tangan: susunan terbaik, susunan saran, atau susunan acak,
        // dengan peluang menurut level.
        let [p_best, p_suggest] = MIX[usize::from(self.level.clamp(1, LEVELS)) - 1];
        let r = f64::from(self.rng.next_u32()) / 4_294_967_296.0;
        let rows = if r < p_best {
            best(&cards)
        } else if r < p_best + p_suggest {
            suggest(&cards)
        } else {
            random(&cards, &mut self.rng)
        };
        Some(format!(
            "arrange {}",
            rows.iter()
                .map(Card::to_string)
                .collect::<Vec<_>>()
                .join(" ")
        ))
    }
}
