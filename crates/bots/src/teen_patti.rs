//! Bot Teen Patti (SPEC §7.2, §8; M5b-1, D-063). Bot hanya membaca view
//! kursinya sendiri; selama buta, kartunya sendiri pun tidak terlihat.
//! Kekuatan tangan yang terlihat = persentil tangan itu di antara semua
//! 22.100 tangan tiga kartu.
//!
//! 1. Pemula: logika level 2, tetapi 35% keputusannya diganti aksi sah
//!    acak.
//! 2. Menengah: ambang tetap untuk pack, chaal, raise, dan show.
//! 3. Mahir: nilai harapan show/pack/lanjut dari peluang menang (persentil
//!    tangan melawan tangan acak); sideshow untuk tangan sedang.
//! 4. Kuat: seperti mahir, ditambah model lawan: setiap taruhan lawan yang
//!    sudah melihat kartu menaikkan batas bawah kekuatan tangannya; bermain
//!    buta lebih lama saat stake kecil.

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
        if self.level == 1 {
            return self.play_random(v);
        }
        if self.level <= 2 {
            return self.play_simple(v);
        }
        self.play_ev(v)
    }

    /// Level 1: logika level 2, tetapi sebagian keputusan diganti aksi sah
    /// acak (pemula yang sering salah langkah).
    fn play_random(&mut self, v: &View) -> String {
        const MISTAKES: f64 = 0.35;
        if self.chance() >= MISTAKES {
            return self.play_simple(v);
        }
        let mut options: Vec<&str> = v.meja.biaya.keys().map(String::as_str).collect();
        if v.sideshow.is_some_and(|[_, to]| to == v.kamu) {
            options = vec!["accept", "deny"];
        } else {
            options.push("pack");
            if !v.kursi[v.kamu as usize].terlihat {
                options.push("see");
            }
        }
        let i = self.rng.below(options.len() as u32) as usize;
        options[i].to_string()
    }

    /// Level 2: ambang tetap.
    fn play_simple(&mut self, v: &View) -> String {
        let me = &v.kursi[v.kamu as usize];
        let has = |c: &str| v.meja.biaya.contains_key(c);
        let cards: Vec<Card> = me.kartu.iter().filter_map(|c| c.parse().ok()).collect();
        if v.sideshow.is_some_and(|[_, to]| to == v.kamu) {
            let accept = percentile(&cards) > 0.5;
            return if accept { "accept" } else { "deny" }.into();
        }
        let live = v
            .kursi
            .iter()
            .filter(|k| k.status == "aktif" || k.status == "allin")
            .count();
        if !me.terlihat {
            if me.buta_ke >= 2 || !has("chaal") {
                return "see".into();
            }
            return "chaal".into();
        }
        let p = percentile(&cards);
        let (pack_th, raise_th, show_th) = (0.35, 0.85, 0.6);
        if live == 2 && has("show") {
            if p > raise_th && has("raise") {
                return "raise".into();
            }
            if p > show_th {
                return "show".into();
            }
        }
        if p < pack_th {
            return "pack".into();
        }
        if p > raise_th && has("raise") {
            return "raise".into();
        }
        if has("chaal") { "chaal" } else { "pack" }.into()
    }

    /// Batas bawah persentil tangan lawan yang terus bertaruh setelah
    /// melihat kartu (level 4): setiap taruhan terlihat menaikkan batasnya.
    fn floor_of(&self, v: &View, seat: u8) -> f64 {
        if self.level < 4 || !v.kursi[seat as usize].terlihat {
            return 0.0;
        }
        let mut seen = false;
        let mut bets = 0;
        for (s, cmd) in &v.log {
            if *s != seat {
                continue;
            }
            match cmd.as_str() {
                "see" => seen = true,
                "chaal" | "raise" | "sideshow" if seen => bets += 1,
                _ => {}
            }
        }
        1.0 - 0.8f64.powi(bets)
    }

    /// Peluang menang melawan lawan dengan batas bawah persentil `floor`.
    fn win_vs(p: f64, floor: f64) -> f64 {
        if floor <= 0.0 {
            p
        } else if p <= floor {
            0.0
        } else {
            (p - floor) / (1.0 - floor)
        }
    }

    /// Level 3–4: nilai harapan show/pack/lanjut dari peluang menang.
    fn play_ev(&mut self, v: &View) -> String {
        let me = &v.kursi[v.kamu as usize];
        let has = |c: &str| v.meja.biaya.contains_key(c);
        let cost = |c: &str| v.meja.biaya.get(c).copied().unwrap_or(0) as f64;
        let cards: Vec<Card> = me.kartu.iter().filter_map(|c| c.parse().ok()).collect();
        let opponents: Vec<u8> = (0..v.kursi.len() as u8)
            .filter(|&s| {
                s != v.kamu && matches!(v.kursi[s as usize].status.as_str(), "aktif" | "allin")
            })
            .collect();
        if let Some([from, to]) = v.sideshow
            && to == v.kamu
        {
            let w = Self::win_vs(percentile(&cards), self.floor_of(v, from));
            return if w > 0.5 { "accept" } else { "deny" }.into();
        }
        if !me.terlihat {
            let blind_turns = if self.level >= 4 { 3 } else { 2 };
            let cheap = v.stake <= 4 * BOOT;
            if me.buta_ke >= blind_turns || !cheap || !has("chaal") {
                return "see".into();
            }
            return "chaal".into();
        }
        let p = percentile(&cards);
        let win: f64 = opponents
            .iter()
            .map(|&o| Self::win_vs(p, self.floor_of(v, o)))
            .product();
        let pot = v.pot as f64;
        if opponents.len() == 1 && has("show") {
            if win > 0.8 && has("raise") && v.stake < 16 * BOOT {
                return "raise".into();
            }
            if win > 0.8 && has("chaal") && self.chance() < 0.5 {
                return "chaal".into();
            }
            let c = cost("show");
            if win * (pot + c) - c > 0.0 {
                return "show".into();
            }
            return "pack".into();
        }
        let c = cost("chaal");
        if win > 0.6 && has("raise") {
            return "raise".into();
        }
        if has("sideshow") && win < 0.5 && win * 2.0 > c / (pot + c) {
            return "sideshow".into();
        }
        if has("chaal") && win * (pot + c) > 1.4 * c {
            return "chaal".into();
        }
        "pack".into()
    }
}
