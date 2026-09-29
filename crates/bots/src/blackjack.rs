//! Strategi dasar Blackjack untuk aturan meja KyuSin (D-056, D-057): 6 dek,
//! bandar berdiri di soft 17, double di dua kartu mana pun termasuk setelah
//! split, split sampai 4 tangan, as split satu kartu, late surrender.
//! Dipakai untuk menghitung dan memverifikasi RTP (SPEC §7) dan sebagai
//! rujukan di tutorial; bukan lawan (bandar Blackjack otomatis di mesin).
//!
//! Tabel (kartu bandar 2–10, A):
//! - Surrender: keras 16 lawan 9, 10, A (kecuali 8-8); keras 15 lawan 10.
//! - Split: A-A dan 8-8 selalu; 9-9 lawan 2–6, 8–9; 7-7 lawan 2–7;
//!   6-6 lawan 2–6; 4-4 lawan 5–6; 3-3 dan 2-2 lawan 2–7; 10-10 dan 5-5 tidak.
//! - Lunak: A2–A3 double lawan 5–6; A4–A5 double lawan 4–6; A6 double lawan
//!   3–6; A7 double lawan 3–6, stand lawan 2, 7, 8, hit lawan 9, 10, A;
//!   A8 ke atas stand. Lainnya hit.
//! - Keras: 8 ke bawah hit; 9 double lawan 3–6; 10 double lawan 2–9; 11
//!   double lawan 2–10; 12 stand lawan 4–6; 13–16 stand lawan 2–6; 17 ke
//!   atas stand. Lainnya hit.
//! - Double yang tidak diizinkan (lebih dari dua kartu) menjadi hit, kecuali
//!   A7 yang menjadi stand. Insurance selalu ditolak.

use kyusin_core::{Player, PlayerId, SeatKind, Session, TurnGame};
use kyusin_games::blackjack::{Blackjack, HandView, View};

/// Nilai kartu untuk tabel: 2–10, as = 11.
fn card_value(text: &str) -> u8 {
    match text.as_bytes().first() {
        Some(b'A') => 11,
        Some(b'T' | b'J' | b'Q' | b'K') => 10,
        Some(d @ b'2'..=b'9') => d - b'0',
        _ => 0,
    }
}

/// Aksi yang tersedia bagi tangan aktif.
#[derive(Debug, Clone, Copy, Default)]
pub struct Options {
    pub double: bool,
    pub split: bool,
    pub surrender: bool,
}

/// Keputusan strategi dasar untuk tangan `hand` melawan kartu terbuka
/// bandar `up` (2–11).
pub fn decide_hand(hand: &HandView, up: u8, o: Options) -> &'static str {
    let first = hand.kartu.first().map(|c| card_value(c)).unwrap_or(0);
    let pair = hand.kartu.len() == 2 && hand.kartu[0].as_bytes()[0] == hand.kartu[1].as_bytes()[0];
    let total = hand.nilai;
    let dealer = |range: std::ops::RangeInclusive<u8>| range.contains(&up);

    if o.surrender && !hand.lunak && !(pair && first == 8) {
        if (total == 16 && dealer(9..=11)) || (total == 15 && up == 10) {
            return "surrender";
        }
    }
    if o.split && pair {
        let split = match first {
            11 | 8 => true,
            9 => dealer(2..=6) || dealer(8..=9),
            7 => dealer(2..=7),
            6 => dealer(2..=6),
            4 => dealer(5..=6),
            3 | 2 => dealer(2..=7),
            _ => false,
        };
        if split {
            return "split";
        }
    }
    let double = |yes: bool, otherwise: &'static str| {
        if yes && o.double {
            "double"
        } else if yes {
            otherwise
        } else {
            ""
        }
    };
    if hand.lunak {
        let d = match total {
            13 | 14 => double(dealer(5..=6), "hit"),
            15 | 16 => double(dealer(4..=6), "hit"),
            17 => double(dealer(3..=6), "hit"),
            18 => double(dealer(3..=6), "stand"),
            _ => "",
        };
        if !d.is_empty() {
            return d;
        }
        return match total {
            18 if dealer(9..=11) => "hit",
            18 => "stand",
            t if t >= 19 => "stand",
            _ => "hit",
        };
    }
    let d = match total {
        9 => double(dealer(3..=6), "hit"),
        10 => double(dealer(2..=9), "hit"),
        11 => double(dealer(2..=10), "hit"),
        _ => "",
    };
    if !d.is_empty() {
        return d;
    }
    match total {
        t if t <= 11 => "hit",
        12 if dealer(4..=6) => "stand",
        12 => "hit",
        13..=16 if dealer(2..=6) => "stand",
        13..=16 => "hit",
        _ => "stand",
    }
}

/// Perintah strategi dasar untuk seluruh meja (taruhan tetap `bet`).
pub fn decide(v: &View, game: &Blackjack, bet: i64) -> String {
    match v.fase.as_str() {
        "taruhan" => format!("bet {bet}"),
        "asuransi" => "decline".into(),
        "giliran" => {
            let Some(i) = v.aktif else {
                return "stand".into();
            };
            let legal: Vec<String> = TurnGame::legal_actions(game, 0)
                .iter()
                .map(|a| a.usage())
                .collect();
            let has = |c: &str| legal.iter().any(|l| l == c);
            let up = v.bandar.first().map(|c| card_value(c)).unwrap_or(10);
            decide_hand(
                &v.tangan[i],
                up,
                Options {
                    double: has("double"),
                    split: has("split"),
                    surrender: has("surrender"),
                },
            )
            .into()
        }
        _ => "leave".into(),
    }
}

/// Pemain otomatis yang memainkan strategi dasar dengan taruhan tetap.
pub struct BasicStrategy {
    pub bet: i64,
}

impl Player for BasicStrategy {
    fn kind(&self) -> SeatKind {
        SeatKind::Bot { level: 1 }
    }

    fn decide(&mut self, session: &dyn Session, seat: PlayerId) -> Option<String> {
        if !session.pending_players().contains(&seat) {
            return None;
        }
        let view: View = serde_json::from_value(session.view_data(seat)).ok()?;
        // Aksi yang tersedia dibaca dari daftar aksi sah sesi.
        let legal: Vec<String> = session
            .legal_actions(seat)
            .iter()
            .map(|a| a.usage())
            .collect();
        let has = |c: &str| legal.iter().any(|l| l == c);
        Some(match view.fase.as_str() {
            "taruhan" => format!("bet {}", self.bet),
            "asuransi" => "decline".into(),
            "giliran" => {
                let i = view.aktif?;
                let up = view.bandar.first().map(|c| card_value(c)).unwrap_or(10);
                decide_hand(
                    &view.tangan[i],
                    up,
                    Options {
                        double: has("double"),
                        split: has("split"),
                        surrender: has("surrender"),
                    },
                )
                .into()
            }
            _ => return None,
        })
    }
}

/// Hasil simulasi RTP.
#[derive(Debug, Clone, Copy)]
pub struct Rtp {
    pub rounds: u64,
    /// Rata-rata hasil bersih per ronde, dalam satuan taruhan awal.
    pub mean: f64,
    /// Simpangan baku hasil bersih per ronde (satuan taruhan awal).
    pub sd: f64,
}

impl Rtp {
    /// RTP dalam persen.
    pub fn percent(&self) -> f64 {
        100.0 * (1.0 + self.mean)
    }

    /// Toleransi SPEC §7: 4 × σ/√n, dalam persen.
    pub fn tolerance_percent(&self) -> f64 {
        100.0 * 4.0 * self.sd / (self.rounds as f64).sqrt()
    }
}

/// Memainkan strategi dasar sampai `rounds` ronde, shoe demi shoe; setiap
/// shoe memakai seed turunan `seed` dan nomor shoe.
pub fn simulate(rounds: u64, seed: &[u8; 32]) -> Rtp {
    use kyusin_core::rng::derive;
    const BET: i64 = 100;
    let mut n = 0u64;
    let mut sum = 0f64;
    let mut sum_sq = 0f64;
    let mut shoe = 0u64;
    while n < rounds {
        let s = derive(seed, &format!("rtp:{shoe}"));
        shoe += 1;
        let mut g = Blackjack::new(Default::default(), s).expect("shoe");
        let mut before = 0i64;
        while !TurnGame::is_over(&g) && n < rounds {
            let v = TurnGame::view_for(&g, 0);
            let cmd = decide(&v, &g, BET);
            let action = g.parse_command(&cmd).expect("perintah strategi");
            TurnGame::apply(&mut g, 0, action).expect("aksi strategi sah");
            let after = TurnGame::view_for(&g, 0);
            if after.fase == "taruhan" || after.fase == "selesai" {
                let x = (after.bersih - before) as f64 / BET as f64;
                before = after.bersih;
                sum += x;
                sum_sq += x * x;
                n += 1;
            }
        }
    }
    let mean = sum / n as f64;
    let var = (sum_sq / n as f64 - mean * mean).max(0.0);
    Rtp {
        rounds: n,
        mean,
        sd: var.sqrt(),
    }
}
