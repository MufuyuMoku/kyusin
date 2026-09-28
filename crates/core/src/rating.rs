//! Glicko-2 (SPEC §8), implementasi sendiri mengikuti Mark E. Glickman,
//! "Example of the Glicko-2 system" (2013).
//!
//! Di KyuSin satu pertandingan selesai = satu periode rating. Rating dan RD
//! lawan (bot) tetap, diambil dari kalibrasi. Semua angka ditampilkan UI
//! sebagai "rating lokal", bukan Elo resmi.

use serde::{Deserialize, Serialize};

/// Konstanta sistem τ (membatasi perubahan volatilitas).
pub const TAU: f64 = 0.5;

/// Skala Glicko ↔ Glicko-2.
const SCALE: f64 = 173.7178;
const CONVERGENCE: f64 = 0.000_001;

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Rating {
    pub rating: f64,
    /// Rating deviation (ketidakpastian).
    pub rd: f64,
    /// Volatilitas.
    pub vol: f64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Outcome {
    Win,
    Draw,
    Loss,
}

impl Outcome {
    pub fn score(self) -> f64 {
        match self {
            Outcome::Win => 1.0,
            Outcome::Draw => 0.5,
            Outcome::Loss => 0.0,
        }
    }
}

fn g(phi: f64) -> f64 {
    1.0 / (1.0 + 3.0 * phi * phi / (std::f64::consts::PI * std::f64::consts::PI)).sqrt()
}

fn expected(mu: f64, mu_j: f64, phi_j: f64) -> f64 {
    1.0 / (1.0 + (-g(phi_j) * (mu - mu_j)).exp())
}

impl Rating {
    /// Pemain baru: 1500 / RD 350 / volatilitas 0,06 (bawaan Glicko-2).
    pub fn new_player() -> Self {
        Rating {
            rating: 1500.0,
            rd: 350.0,
            vol: 0.06,
        }
    }

    /// Rating baru setelah satu periode berisi `games` (lawan + hasil).
    /// Periode tanpa pertandingan hanya menaikkan RD.
    pub fn update(&self, games: &[(Rating, Outcome)], tau: f64) -> Rating {
        let mu = (self.rating - 1500.0) / SCALE;
        let phi = self.rd / SCALE;
        let sigma = self.vol;
        if games.is_empty() {
            let phi_star = (phi * phi + sigma * sigma).sqrt();
            return Rating {
                rating: self.rating,
                rd: phi_star * SCALE,
                vol: sigma,
            };
        }

        // Langkah 3–4: varians v dan perbaikan Δ.
        let mut v_inv = 0.0;
        let mut delta_sum = 0.0;
        for (opp, outcome) in games {
            let mu_j = (opp.rating - 1500.0) / SCALE;
            let phi_j = opp.rd / SCALE;
            let e = expected(mu, mu_j, phi_j);
            let gj = g(phi_j);
            v_inv += gj * gj * e * (1.0 - e);
            delta_sum += gj * (outcome.score() - e);
        }
        let v = 1.0 / v_inv;
        let delta = v * delta_sum;

        // Langkah 5: volatilitas baru (metode Illinois).
        let a = (sigma * sigma).ln();
        let f = |x: f64| {
            let ex = x.exp();
            let d = phi * phi + v + ex;
            ex * (delta * delta - phi * phi - v - ex) / (2.0 * d * d) - (x - a) / (tau * tau)
        };
        let mut big_a = a;
        let mut big_b = if delta * delta > phi * phi + v {
            (delta * delta - phi * phi - v).ln()
        } else {
            let mut k = 1.0;
            while f(a - k * tau) < 0.0 {
                k += 1.0;
            }
            a - k * tau
        };
        let mut f_a = f(big_a);
        let mut f_b = f(big_b);
        let mut guard = 0;
        while (big_b - big_a).abs() > CONVERGENCE && guard < 1000 {
            let c = big_a + (big_a - big_b) * f_a / (f_b - f_a);
            let f_c = f(c);
            if f_c * f_b <= 0.0 {
                big_a = big_b;
                f_a = f_b;
            } else {
                f_a /= 2.0;
            }
            big_b = c;
            f_b = f_c;
            guard += 1;
        }
        let sigma_new = (big_a / 2.0).exp();

        // Langkah 6–8.
        let phi_star = (phi * phi + sigma_new * sigma_new).sqrt();
        let phi_new = 1.0 / (1.0 / (phi_star * phi_star) + 1.0 / v).sqrt();
        let mu_new = mu + phi_new * phi_new * delta_sum;
        Rating {
            rating: mu_new * SCALE + 1500.0,
            rd: phi_new * SCALE,
            vol: sigma_new,
        }
    }
}
