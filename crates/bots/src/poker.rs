//! Bot meja poker antar-pemain: Texas Hold'em dan Omaha (SPEC §7.2, §8;
//! M5b-1, D-063). Bot hanya membaca view kursinya sendiri (kartunya, kartu
//! meja, taruhan yang terlihat), jadi tidak pernah melihat kartu lawan.
//!
//! 1. Pemula: longgar dan pasif; call murah, jarang bet.
//! 2. Menengah: aturan dari kategori tangan (sebelum flop: pair dan kartu
//!    tinggi), tanpa menghitung peluang.
//! 3. Mahir: peluang menang (equity) Monte Carlo 150 kali (Omaha 50)
//!    melawan tangan acak, dibandingkan dengan pot odds.
//! 4. Kuat: equity 400 kali (Omaha 133), besar taruhan menurut equity, sesekali
//!    menggertak saat lawan check, dan lebih hati-hati melawan all-in.

use kyusin_core::{GameRng, Player, PlayerId, SeatKind, Seed, Session};
use kyusin_games::cards::Card;
use kyusin_games::poker::{Category, Value, rank_value};
use kyusin_games::poker_meja::{BIG_BLIND, Variant, View};

pub const LEVELS: u8 = 4;

/// Aturan varian yang dibutuhkan bot.
#[derive(Debug, Clone, Copy)]
pub struct Rules {
    pub hole: usize,
    pub best: fn(&[Card], &[Card]) -> Value,
}

impl Rules {
    pub fn of<V: Variant>() -> Rules {
        Rules {
            hole: V::HOLE,
            best: V::best,
        }
    }
}

pub struct PokerBot {
    level: u8,
    rules: Rules,
    rng: GameRng,
}

impl PokerBot {
    pub fn new(level: u8, rules: Rules, seed: Seed) -> Self {
        PokerBot {
            level,
            rules,
            rng: GameRng::from_seed(seed),
        }
    }

    fn chance(&mut self) -> f64 {
        f64::from(self.rng.next_u32()) / 4_294_967_296.0
    }
}

fn parse(cards: &[String]) -> Vec<Card> {
    cards.iter().filter_map(|c| c.parse().ok()).collect()
}

/// Peluang menang (seri dihitung sebagian) melawan `opponents` tangan acak,
/// dengan `sims` kali simulasi sisa kartu meja.
pub fn equity(
    hole: &[Card],
    board: &[Card],
    opponents: usize,
    sims: u32,
    rules: Rules,
    rng: &mut GameRng,
) -> f64 {
    let known: Vec<Card> = hole.iter().chain(board.iter()).copied().collect();
    let mut deck: Vec<Card> = Card::deck()
        .into_iter()
        .filter(|c| !known.contains(c))
        .collect();
    let need = opponents * rules.hole + (5 - board.len());
    let mut score = 0.0;
    for _ in 0..sims {
        for i in 0..need.min(deck.len()) {
            let j = i + rng.below((deck.len() - i) as u32) as usize;
            deck.swap(i, j);
        }
        let mut full = board.to_vec();
        full.extend_from_slice(&deck[opponents * rules.hole..need]);
        let mine = (rules.best)(hole, &full);
        let mut best_other = Value(0);
        let mut ties = 0;
        for o in 0..opponents {
            let theirs = (rules.best)(&deck[o * rules.hole..(o + 1) * rules.hole], &full);
            match theirs.cmp(&best_other) {
                std::cmp::Ordering::Greater => {
                    best_other = theirs;
                    ties = usize::from(theirs == mine);
                }
                std::cmp::Ordering::Equal if theirs == mine => ties += 1,
                _ => {}
            }
        }
        if mine > best_other {
            score += 1.0;
        } else if mine == best_other {
            score += 1.0 / (ties + 1) as f64;
        }
    }
    score / f64::from(sims)
}

/// Kekuatan kasar tanpa simulasi (level 2), 0–1.
fn rough_strength(hole: &[Card], board: &[Card], rules: Rules) -> f64 {
    if board.is_empty() {
        let mut r: Vec<u8> = hole.iter().map(|c| rank_value(c.rank)).collect();
        r.sort_unstable_by(|a, b| b.cmp(a));
        let pair = r.windows(2).any(|w| w[0] == w[1]);
        let suited = hole
            .iter()
            .any(|a| hole.iter().filter(|b| b.suit == a.suit).count() >= 2);
        let high = r.iter().filter(|&&x| x >= 10).count();
        let mut s = 0.2 + 0.07 * high as f64;
        if pair {
            s += 0.25 + f64::from(r[0]) / 60.0;
        }
        if suited {
            s += 0.05;
        }
        if r[0] == 14 {
            s += 0.05;
        }
        return s.min(0.95);
    }
    let v = (rules.best)(hole, board);
    match v.category() {
        Category::HighCard => 0.15 + f64::from(v.rank(0)) / 100.0,
        Category::Pair => 0.4 + f64::from(v.rank(0)) / 50.0,
        Category::TwoPair => 0.7,
        Category::Trips => 0.8,
        _ => 0.92,
    }
}

impl Player for PokerBot {
    fn kind(&self) -> SeatKind {
        SeatKind::Bot { level: self.level }
    }

    fn decide(&mut self, session: &dyn Session, seat: PlayerId) -> Option<String> {
        if !session.pending_players().contains(&seat) {
            return None;
        }
        let v: View = serde_json::from_value(session.view_data(seat)).ok()?;
        if v.meja.fase != "main" {
            return Some("next".into());
        }
        Some(self.play(&v))
    }
}

impl PokerBot {
    fn play(&mut self, v: &View) -> String {
        let me = &v.kursi[v.kamu as usize];
        let hole = parse(&me.kartu);
        let board = parse(&v.meja_kartu);
        let call = v.panggil;
        let pot = v.pot;
        let current = v.kursi.iter().map(|k| k.taruhan).max().unwrap_or(0);
        let verb = if current == 0 { "bet" } else { "raise" };
        let opponents = v
            .kursi
            .iter()
            .enumerate()
            .filter(|(i, k)| *i as u8 != v.kamu && (k.status == "aktif" || k.status == "allin"))
            .count()
            .max(1);
        let size = |frac: f64| -> Option<String> {
            let (min, max) = (v.naik_min?, v.naik_maks?);
            let target = current + (frac * (pot + call) as f64) as i64;
            Some(format!("{verb} {}", target.clamp(min, max)))
        };
        let passive = if call == 0 { "check" } else { "fold" };
        let stack = me.tumpukan;
        match self.level {
            1 => {
                let r = self.chance();
                if call == 0 {
                    if r < 0.1 {
                        return size(0.0).unwrap_or_else(|| "check".into());
                    }
                    return "check".into();
                }
                let cheap = call <= stack / 5;
                if (cheap && r < 0.85) || r < 0.3 {
                    "call".into()
                } else {
                    "fold".into()
                }
            }
            2 => {
                let s = rough_strength(&hole, &board, self.rules);
                if call == 0 {
                    if s > 0.65 {
                        return size(0.5).unwrap_or_else(|| "check".into());
                    }
                    return "check".into();
                }
                if s > 0.78 {
                    size(0.0).unwrap_or_else(|| "call".into())
                } else if s > 0.42 || call * 4 <= pot {
                    "call".into()
                } else {
                    passive.into()
                }
            }
            level => {
                // Omaha menilai 60 kombinasi per tangan: simulasi lebih sedikit.
                let base = if level == 3 { 150 } else { 400 };
                let sims = if self.rules.hole > 2 { base / 3 } else { base };
                let eq = equity(&hole, &board, opponents, sims, self.rules, &mut self.rng);
                let odds = call as f64 / (pot + call).max(1) as f64;
                let r = self.chance();
                if call == 0 {
                    if eq > 0.62 {
                        let frac = if level == 3 { 0.6 } else { 0.4 + eq / 2.0 };
                        return size(frac).unwrap_or_else(|| "check".into());
                    }
                    let bluff = level >= 4 && !board.is_empty() && r < 0.1;
                    if bluff || (eq > 0.5 && r < 0.3) {
                        return size(0.5).unwrap_or_else(|| "check".into());
                    }
                    return "check".into();
                }
                let margin = if level >= 4 && call >= stack / 2 {
                    0.08
                } else {
                    0.0
                };
                if eq > 0.72 {
                    let frac = if level == 3 { 0.75 } else { eq };
                    size(frac).unwrap_or_else(|| "call".into())
                } else if eq >= odds + margin || (call <= BIG_BLIND && eq > 0.25) {
                    "call".into()
                } else {
                    "fold".into()
                }
            }
        }
    }
}
