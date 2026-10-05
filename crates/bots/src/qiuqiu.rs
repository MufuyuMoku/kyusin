//! Bot Domino QiuQiu (SPEC §7.2, §8; M5b-2). Bot hanya membaca view
//! kursinya sendiri (kartunya dan taruhan yang terlihat).
//!
//! Strategi dasar: peluang menang (equity) Monte Carlo 400 kali dengan
//! model lawan (lawan yang bet/raise dianggap memegang tangan layak),
//! dibandingkan dengan pot odds; bet saat equity jelas di atas rata-rata,
//! raise dengan tangan kuat, sesekali menggertak saat semua check. Level
//! lebih rendah memakai strategi yang sama dengan salah langkah acak
//! ([`MISTAKES`]):
//!
//! 1. Pemula: 90% keputusan diganti aksi sah acak.
//! 2. Menengah: 45% keputusan diganti aksi sah acak.
//! 3. Kuat: tanpa salah langkah.
//!
//! Versi awal (level 1 selalu ikut, level 2 ambang tetap) membuat level 3
//! tidak lebih kuat dari level 2, karena permainan ini sempit ruang
//! strateginya; salah langkah acak memberi tangga yang rata.

use kyusin_core::{GameRng, Player, PlayerId, SeatKind, Seed, Session};
use kyusin_games::domino::Tile;
use kyusin_games::domino_qiuqiu::{QqClass, QqValue, View, eval, pair_value};

pub const LEVELS: u8 = 3;

/// Peluang salah langkah acak level 1 dan 2.
pub const MISTAKES: [f64; 2] = [0.9, 0.45];

pub struct QiuQiuBot {
    level: u8,
    rng: GameRng,
}

impl QiuQiuBot {
    pub fn new(level: u8, seed: Seed) -> Self {
        QiuQiuBot {
            level,
            rng: GameRng::from_seed(seed),
        }
    }

    fn chance(&mut self) -> f64 {
        f64::from(self.rng.next_u32()) / 4_294_967_296.0
    }
}

fn parse(tiles: &[String]) -> Vec<Tile> {
    tiles.iter().filter_map(|t| t.parse().ok()).collect()
}

/// Kekuatan kasar tiga atau empat kartu tanpa simulasi, 0–1: pasangan
/// terbaik yang sudah terbentuk.
pub fn rough(tiles: &[Tile]) -> f64 {
    if tiles.len() == 4 {
        let v = eval(tiles);
        let (a, b) = v.pairs();
        return if v.class() > QqClass::Pasangan {
            0.97
        } else {
            (f64::from(a) * 10.0 + f64::from(b)) / 100.0
        };
    }
    let best = (0..tiles.len())
        .flat_map(|i| (i + 1..tiles.len()).map(move |j| (i, j)))
        .map(|(i, j)| pair_value(tiles[i], tiles[j]))
        .max()
        .unwrap_or(0);
    f64::from(best) / 10.0
}

/// Peluang menang (seri dihitung sebagian) melawan `opponents` tangan acak;
/// `strong` lawan pertama dianggap memegang tangan layak (tangan lemah
/// diundi ulang paling banyak empat kali).
pub fn equity(mine: &[Tile], opponents: usize, strong: usize, sims: u32, rng: &mut GameRng) -> f64 {
    let mut pool: Vec<Tile> = Tile::set()
        .into_iter()
        .filter(|t| !mine.contains(t))
        .collect();
    let own_missing = 4 - mine.len();
    let need = opponents * 4 + own_missing;
    let mut score = 0.0;
    for _ in 0..sims {
        for i in 0..need.min(pool.len()) {
            let j = i + rng.below((pool.len() - i) as u32) as usize;
            pool.swap(i, j);
        }
        for o in 0..strong.min(opponents) {
            for _ in 0..4 {
                let h = &pool[o * 4..o * 4 + mine.len().min(4)];
                if rough(h) >= 0.7 || pool.len() <= need {
                    break;
                }
                for k in 0..4 {
                    let j = need + rng.below((pool.len() - need) as u32) as usize;
                    pool.swap(o * 4 + k, j);
                }
            }
        }
        let mut own: Vec<Tile> = mine.to_vec();
        own.extend_from_slice(&pool[opponents * 4..need]);
        let me: QqValue = eval(&own);
        let mut best: Option<QqValue> = None;
        let mut ties = 0usize;
        for o in 0..opponents {
            let theirs = eval(&pool[o * 4..o * 4 + 4]);
            match best.map(|b| theirs.cmp(&b)) {
                None | Some(std::cmp::Ordering::Greater) => {
                    best = Some(theirs);
                    ties = usize::from(theirs == me);
                }
                Some(std::cmp::Ordering::Equal) if theirs == me => ties += 1,
                _ => {}
            }
        }
        match best.map(|b| me.cmp(&b)) {
            Some(std::cmp::Ordering::Greater) | None => score += 1.0,
            Some(std::cmp::Ordering::Equal) => score += 1.0 / (ties + 1) as f64,
            _ => {}
        }
    }
    score / f64::from(sims)
}

impl Player for QiuQiuBot {
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

impl QiuQiuBot {
    fn play(&mut self, v: &View) -> String {
        let me = &v.kursi[v.kamu as usize];
        let mine = parse(&me.kartu);
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
        let stack = me.tumpukan;
        // Level rendah: strategi yang sama dengan salah langkah acak.
        let mistakes = match self.level {
            1 => MISTAKES[0],
            2 => MISTAKES[1],
            _ => 0.0,
        };
        if self.chance() < mistakes {
            let mut options = vec![if call == 0 {
                "check".to_string()
            } else {
                "fold".to_string()
            }];
            if call > 0 {
                options.push("call".into());
            }
            if let Some(min) = v.naik_min {
                options.push(format!("{verb} {min}"));
            }
            return options[self.rng.below(options.len() as u32) as usize].clone();
        }
        let strong = {
            let mut aggressors: Vec<u8> = v
                .log
                .iter()
                .filter(|(seat, cmd)| {
                    *seat != v.kamu && (cmd.starts_with("bet") || cmd.starts_with("raise"))
                })
                .map(|(seat, _)| *seat)
                .collect();
            aggressors.sort_unstable();
            aggressors.dedup();
            aggressors.len()
        };
        let eq = equity(&mine, opponents, strong, 400, &mut self.rng);
        let avg = 1.0 / (opponents + 1) as f64;
        let odds = call as f64 / (pot + call).max(1) as f64;
        let r = self.chance();
        if call == 0 {
            if eq > avg + 0.2 {
                return size(0.6).unwrap_or_else(|| "check".into());
            }
            if r < 0.05 {
                return size(0.5).unwrap_or_else(|| "check".into());
            }
            return "check".into();
        }
        let margin = if call >= stack / 2 { 0.1 } else { 0.05 };
        if eq > avg + 0.3 {
            size(0.7).unwrap_or_else(|| "call".into())
        } else if eq >= odds + margin {
            "call".into()
        } else {
            "fold".into()
        }
    }
}
