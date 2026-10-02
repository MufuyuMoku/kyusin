//! Bot Teen Patti (SPEC §7.2, §8; M5b-1, D-063). Bot hanya membaca view
//! kursinya sendiri; selama buta, kartunya sendiri pun tidak terlihat.
//! Kekuatan tangan yang terlihat = persentil tangan itu di antara semua
//! 22.100 tangan tiga kartu.
//!
//! 1. Pemula: chaal terus, melihat kartu setelah dua chaal buta, jarang
//!    pack.
//! 2. Menengah: ambang tetap untuk pack, chaal, raise, dan show.
//! 3. Mahir: ambang naik bersama jumlah lawan dan besar stake; memakai
//!    sideshow untuk tangan sedang.
//! 4. Kuat: seperti mahir, ditambah bermain buta lebih lama saat stake
//!    kecil, menggertak sesekali, dan menolak sideshow dengan tangan kuat.

use std::sync::OnceLock;

use kyusin_core::{GameRng, Player, PlayerId, SeatKind, Seed, Session};
use kyusin_games::cards::Card;
use kyusin_games::teen_patti::{BOOT, TpValue, View, eval};

pub const LEVELS: u8 = 4;

/// Persentil tangan (0–1) di antara semua tangan tiga kartu.
pub fn percentile(cards: &[Card]) -> f64 {
    static ALL: OnceLock<Vec<TpValue>> = OnceLock::new();
    let all = ALL.get_or_init(|| {
        let d = Card::deck();
        let mut v = Vec::with_capacity(22_100);
        for a in 0..52 {
            for b in a + 1..52 {
                for c in b + 1..52 {
                    v.push(eval(&[d[a], d[b], d[c]]));
                }
            }
        }
        v.sort_unstable();
        v
    });
    let me = eval(cards);
    all.partition_point(|x| *x < me) as f64 / all.len() as f64
}

pub struct TeenPattiBot {
    level: u8,
    rng: GameRng,
}

impl TeenPattiBot {
    pub fn new(level: u8, seed: Seed) -> Self {
        TeenPattiBot {
            level,
            rng: GameRng::from_seed(seed),
        }
    }

    fn chance(&mut self) -> f64 {
        f64::from(self.rng.next_u32()) / 4_294_967_296.0
    }
}

impl Player for TeenPattiBot {
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

impl TeenPattiBot {
    fn play(&mut self, v: &View) -> String {
        let me = &v.kursi[v.kamu as usize];
        let has = |c: &str| v.meja.biaya.contains_key(c);
        let r = self.chance();
        let cards: Vec<Card> = me.kartu.iter().filter_map(|c| c.parse().ok()).collect();
        if v.sideshow.is_some_and(|[_, to]| to == v.kamu) {
            let p = percentile(&cards);
            let accept = match self.level {
                1 => r < 0.5,
                4 => p < 0.8 && p > 0.45,
                _ => p > 0.5,
            };
            return if accept { "accept" } else { "deny" }.into();
        }
        let live = v
            .kursi
            .iter()
            .filter(|k| k.status == "aktif" || k.status == "allin")
            .count();
        if !me.terlihat {
            let see_after = match self.level {
                1 | 2 => 2,
                3 => 1,
                _ if v.stake <= 4 * BOOT => 3,
                _ => 1,
            };
            if me.buta_ke >= see_after || !has("chaal") {
                return "see".into();
            }
            if self.level >= 3 && has("raise") && r < 0.1 {
                return "raise".into();
            }
            return "chaal".into();
        }
        let p = percentile(&cards);
        let pressure = (v.stake as f64 / (64 * BOOT) as f64).min(1.0);
        let crowd = live.saturating_sub(2) as f64;
        let (pack_th, raise_th) = match self.level {
            1 => (0.1, 0.95),
            2 => (0.35, 0.85),
            _ => (0.3 + 0.06 * crowd + 0.25 * pressure, 0.8 + 0.03 * crowd),
        };
        if live == 2 && has("show") {
            let show_th = match self.level {
                1 => 0.3,
                2 => 0.6,
                _ => 0.55 + 0.2 * pressure,
            };
            if p > raise_th && has("raise") && pressure < 0.5 {
                return "raise".into();
            }
            if p > show_th {
                return "show".into();
            }
            if p < pack_th && !(self.level >= 4 && r < 0.1) {
                return "pack".into();
            }
            return if has("chaal") { "chaal" } else { "pack" }.into();
        }
        if p < pack_th && !(self.level >= 4 && r < 0.08) {
            return "pack".into();
        }
        if self.level >= 3 && has("sideshow") && p < pack_th + 0.2 {
            return "sideshow".into();
        }
        if p > raise_th && has("raise") {
            return "raise".into();
        }
        if has("chaal") { "chaal" } else { "pack" }.into()
    }
}
