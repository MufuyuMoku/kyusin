//! Rating lokal level bot (SPEC §8, D-050): data kalibrasi di
//! `data/calibration/<game>.json` dan penaksirnya.
//!
//! Catur dikalibrasi terhadap Stockfish (`calibrate_catur`). Game tanpa
//! kalibrasi eksternal dikalibrasi antar-bot (`calibrate_internal`) dengan
//! level 1 dipatok 1000.

use kyusin_core::rating::Rating;

/// Rating level 1 untuk game yang dikalibrasi antar-bot.
pub const ANCHOR: f64 = 1000.0;

/// Selisih rating terbesar antara dua level berurutan (SPEC §8, Rev. 10).
pub const MAX_STEP: f64 = 400.0;

/// RD minimum untuk lawan bot: level dengan selang sangat sempit (atau
/// jangkar) tetap dianggap punya sedikit ketidakpastian.
pub const MIN_BOT_RD: f64 = 30.0;

/// Skor harapan model Elo logistik untuk rating `r` melawan `e`.
pub fn expected(r: f64, e: f64) -> f64 {
    1.0 / (1.0 + 10f64.powf((e - r) / 400.0))
}

/// Taksiran kemungkinan maksimum rating dari hasil `(rating lawan, poin,
/// jumlah partai)` + setengah lebar selang 95% (informasi Fisher). Bila
/// semua kalah atau semua menang, ditambah satu remis semu per lawan
/// supaya taksirannya berhingga; hasilnya ditandai ekstrapolasi.
pub fn estimate(results: &[(f64, f64, f64)]) -> (f64, f64, bool) {
    let total: f64 = results.iter().map(|r| r.1).sum();
    let games: f64 = results.iter().map(|r| r.2).sum();
    let extrapolated = total == 0.0 || total == games;
    let data: Vec<(f64, f64, f64)> = if extrapolated {
        results
            .iter()
            .map(|&(e, p, n)| (e, p + 0.5, n + 1.0))
            .collect()
    } else {
        results.to_vec()
    };
    let grad = |r: f64| {
        data.iter()
            .map(|&(e, p, n)| p - n * expected(r, e))
            .sum::<f64>()
    };
    let (mut lo, mut hi) = (-1000.0, 4000.0);
    for _ in 0..100 {
        let mid = (lo + hi) / 2.0;
        if grad(mid) > 0.0 { lo = mid } else { hi = mid }
    }
    let r = (lo + hi) / 2.0;
    let k = std::f64::consts::LN_10 / 400.0;
    let info: f64 = data
        .iter()
        .map(|&(e, _, n)| n * expected(r, e) * (1.0 - expected(r, e)) * k * k)
        .sum();
    (r, 1.96 / info.sqrt(), extrapolated)
}

/// Data kalibrasi yang dibundel, per game.
fn data(game: &str) -> Option<&'static str> {
    Some(match game {
        kyusin_games::catur::ID => include_str!("../../../data/calibration/catur.json"),
        kyusin_games::reversi::ID => include_str!("../../../data/calibration/reversi.json"),
        _ => return None,
    })
}

/// Satu level dari berkas kalibrasi.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct LevelRating {
    pub elo: f64,
    /// Setengah lebar selang 95%.
    pub half_ci: f64,
    pub extrapolated: bool,
}

pub fn level(game: &str, level: u8) -> Option<LevelRating> {
    let doc: serde_json::Value = serde_json::from_str(data(game)?).ok()?;
    let l = doc["levels"]
        .as_array()?
        .iter()
        .find(|l| l["level"] == level)?;
    let elo = l["elo"].as_f64()?;
    let ci = l["ci95"].as_array();
    let half_ci = ci
        .and_then(|c| Some((c.get(1)?.as_f64()? - c.first()?.as_f64()?) / 2.0))
        .unwrap_or(0.0);
    Some(LevelRating {
        elo,
        half_ci,
        extrapolated: l["extrapolated"].as_bool().unwrap_or(false),
    })
}

/// Rating lawan bot untuk Glicko-2: taksiran kalibrasi, RD dari selang 95%
/// (minimal [`MIN_BOT_RD`]). Taksiran ekstrapolasi tetap dipakai dengan RD
/// lebarnya (tidak ditampilkan di UI, D-047).
pub fn bot_rating(game: &str, lvl: u8) -> Option<Rating> {
    let l = level(game, lvl)?;
    Some(Rating {
        rating: l.elo,
        rd: (l.half_ci / 1.96).max(MIN_BOT_RD),
        vol: 0.06,
    })
}

/// Pelanggaran aturan tangga level merata (SPEC §8): selisih dua level
/// berurutan lebih dari [`MAX_STEP`]. Level terbawah yang taksirannya
/// ekstrapolasi (hanya punya batas atas) dikecualikan.
pub fn ladder_violations(levels: &[LevelRating]) -> Vec<String> {
    let skip = usize::from(levels.first().is_some_and(|l| l.extrapolated));
    levels
        .windows(2)
        .enumerate()
        .skip(skip)
        .filter(|(_, w)| w[1].elo - w[0].elo > MAX_STEP)
        .map(|(i, w)| {
            format!(
                "level {} → {}: {:.0} → {:.0} (selisih {:.0} > {MAX_STEP})",
                i + 1,
                i + 2,
                w[0].elo,
                w[1].elo,
                w[1].elo - w[0].elo
            )
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn estimate_recovers_the_rating_of_even_results() {
        // Setengah poin melawan 1200 → sekitar 1200.
        let (r, half, ex) = estimate(&[(1200.0, 20.0, 40.0)]);
        assert!((r - 1200.0).abs() < 0.5, "{r}");
        assert!(half > 0.0 && !ex);
        // 75% melawan 1000 → sekitar 1191 (400·log10(3)).
        let (r, _, _) = estimate(&[(1000.0, 30.0, 40.0)]);
        assert!((r - 1190.8).abs() < 1.0, "{r}");
    }

    #[test]
    fn perfect_scores_are_finite_and_flagged() {
        let (r, _, ex) = estimate(&[(1000.0, 10.0, 10.0)]);
        assert!(ex && r.is_finite() && r > 1300.0, "{r}");
        let (r, _, ex) = estimate(&[(1000.0, 0.0, 10.0)]);
        assert!(ex && r.is_finite() && r < 700.0, "{r}");
    }

    #[test]
    fn every_competitive_game_with_bots_has_calibration_for_each_level() {
        let registry = kyusin_games::builtin().unwrap();
        for m in registry.manifests() {
            if !m.competitive || crate::levels(&m.id) == 0 {
                continue;
            }
            let mut last = f64::MIN;
            for l in 1..=crate::levels(&m.id) {
                let r = bot_rating(&m.id, l)
                    .unwrap_or_else(|| panic!("{} level {l} tanpa data kalibrasi", m.id));
                assert!(r.rating > last, "{}: rating level {l} tidak naik", m.id);
                assert!(r.rd >= MIN_BOT_RD);
                last = r.rating;
            }
        }
    }

    fn lr(elo: f64, extrapolated: bool) -> LevelRating {
        LevelRating {
            elo,
            half_ci: 50.0,
            extrapolated,
        }
    }

    #[test]
    fn ladder_rule_flags_wide_steps_except_an_upper_bound_bottom() {
        assert!(
            ladder_violations(&[lr(1000.0, false), lr(1400.0, false), lr(1790.0, false)])
                .is_empty()
        );
        let v = ladder_violations(&[lr(1000.0, false), lr(1344.0, false), lr(2133.0, false)]);
        assert_eq!(v.len(), 1, "{v:?}");
        assert!(v[0].starts_with("level 2 → 3"), "{v:?}");
        // Level terbawah ekstrapolasi (mis. catur level 1, 0/64) dikecualikan…
        assert!(ladder_violations(&[lr(888.0, true), lr(1400.0, false)]).is_empty());
        // …tetapi hanya yang terbawah.
        assert_eq!(
            ladder_violations(&[lr(1000.0, false), lr(1500.0, true)]).len(),
            1
        );
    }

    /// SPEC §8 Rev. 10: data kalibrasi setiap game kompetitif memenuhi
    /// aturan tangga level merata.
    #[test]
    fn every_competitive_game_has_an_even_level_ladder() {
        let registry = kyusin_games::builtin().unwrap();
        let mut problems = Vec::new();
        for m in registry.manifests() {
            if !m.competitive || crate::levels(&m.id) == 0 {
                continue;
            }
            let levels: Vec<LevelRating> = (1..=crate::levels(&m.id))
                .filter_map(|l| level(&m.id, l))
                .collect();
            problems.extend(
                ladder_violations(&levels)
                    .into_iter()
                    .map(|p| format!("{}: {p}", m.id)),
            );
        }
        assert!(
            problems.is_empty(),
            "tangga level tidak merata:
{}",
            problems.join(
                "
"
            )
        );
    }

    #[test]
    fn internal_calibration_is_anchored_at_1000() {
        let l1 = level(kyusin_games::reversi::ID, 1).unwrap();
        assert_eq!(l1.elo, ANCHOR);
    }
}
