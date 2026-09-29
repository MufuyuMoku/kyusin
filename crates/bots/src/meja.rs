//! Simulasi RTP meja casino melawan bandar (SPEC §7; M5a).
//!
//! Satu simulator untuk semua meja: sesi demi sesi dengan seed turunan,
//! pemain otomatis memilih perintah teks dari view, dan setiap ronde yang
//! selesai menyumbang satu sampel hasil bersih dibagi taruhan awal ronde
//! itu (taruhan yang dipasang sebelum kartu terlihat; D-060). Toleransi
//! SPEC §7: 4 × σ/√n dari simulasi itu sendiri.

use kyusin_core::rng::derive;
use kyusin_core::{Seed, TurnGame};
use kyusin_games::meja::{MejaView, WagerRtp};

/// Hasil simulasi RTP.
#[derive(Debug, Clone, Copy, Default)]
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

    /// Menggabungkan beberapa simulasi (misalnya per utas).
    pub fn merge(parts: &[Rtp]) -> Rtp {
        let n: f64 = parts.iter().map(|p| p.rounds as f64).sum();
        if n == 0.0 {
            return Rtp::default();
        }
        let mean = parts.iter().map(|p| p.mean * p.rounds as f64).sum::<f64>() / n;
        let second = parts
            .iter()
            .map(|p| (p.sd * p.sd + p.mean * p.mean) * p.rounds as f64)
            .sum::<f64>()
            / n;
        Rtp {
            rounds: n as u64,
            mean,
            sd: (second - mean * mean).max(0.0).sqrt(),
        }
    }
}

/// Penjumlah sampel.
#[derive(Debug, Clone, Copy, Default)]
pub struct Acc {
    n: u64,
    sum: f64,
    sum_sq: f64,
}

impl Acc {
    pub fn push(&mut self, x: f64) {
        self.n += 1;
        self.sum += x;
        self.sum_sq += x * x;
    }

    pub fn len(&self) -> u64 {
        self.n
    }

    pub fn is_empty(&self) -> bool {
        self.n == 0
    }

    pub fn finish(&self) -> Rtp {
        if self.n == 0 {
            return Rtp::default();
        }
        let mean = self.sum / self.n as f64;
        Rtp {
            rounds: self.n,
            mean,
            sd: (self.sum_sq / self.n as f64 - mean * mean).max(0.0).sqrt(),
        }
    }
}

/// Memainkan `rounds` ronde dengan `decide`, sesi demi sesi (`new` dengan
/// seed turunan `seed`), dan mengukur hasil per `unit` chip taruhan awal.
pub fn simulate<G>(
    rounds: u64,
    seed: &Seed,
    unit: i64,
    new: impl Fn(Seed) -> G,
    mut decide: impl FnMut(&G, &G::View) -> String,
) -> Rtp
where
    G: TurnGame,
    G::View: MejaView,
{
    let mut acc = Acc::default();
    let mut session = 0u64;
    while acc.len() < rounds {
        let mut g = new(derive(seed, &format!("rtp:{session}")));
        session += 1;
        let mut settled = 0;
        let mut net = 0;
        while !g.is_over() && acc.len() < rounds {
            let v = g.view_for(0);
            let cmd = decide(&g, &v);
            let action = g
                .parse_command(&cmd)
                .unwrap_or_else(|e| panic!("perintah strategi `{cmd}`: {e}"));
            g.apply(0, action)
                .unwrap_or_else(|e| panic!("aksi strategi `{cmd}`: {e}"));
            let after = g.view_for(0);
            if after.settled() > settled {
                settled = after.settled();
                let now = after.meja().bersih;
                acc.push((now - net) as f64 / unit as f64);
                net = now;
            }
        }
    }
    acc.finish()
}

/// Satu jenis taruhan yang bisa disimulasikan: angka yang dijanjikan
/// (dari modul game) dan simulatornya.
#[derive(Clone, Copy)]
pub struct Wager {
    pub game: &'static str,
    pub rtp: WagerRtp,
    pub run: fn(u64, &Seed) -> Rtp,
}

/// Semua taruhan meja casino M5a yang diverifikasi RTP-nya.
pub fn wagers() -> Vec<Wager> {
    let mut out = Vec::new();
    out.extend(crate::dragon_tiger::wagers());
    out.extend(crate::casino_war::wagers());
    out
}
