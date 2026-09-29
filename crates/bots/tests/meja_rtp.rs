//! Verifikasi RTP meja casino M5a (SPEC §7): setiap jenis taruhan yang
//! diumumkan di modul game disimulasikan ≥100.000 ronde di CI setiap push;
//! hasilnya harus berada dalam 4 × σ/√n dari angka yang diumumkan. Angka
//! manifest harus sama dengan taruhan yang ditandai `manifest`. Workflow
//! `rtp.yml` menjalankan ≥10.000.000 ronde per taruhan.

use kyusin_bots::meja::wagers;

#[test]
fn manifest_rtp_is_the_marked_wager() {
    let registry = kyusin_games::builtin().unwrap();
    let all = wagers();
    for m in registry.manifests() {
        if !m.is_against_house() || m.id == kyusin_games::blackjack::ID {
            continue;
        }
        let marked: Vec<_> = all
            .iter()
            .filter(|w| w.game == m.id && w.rtp.manifest)
            .collect();
        assert_eq!(marked.len(), 1, "{}: tepat satu taruhan manifest", m.id);
        let want = (marked[0].rtp.percent * 100.0).round() / 100.0;
        assert_eq!(m.rtp, Some(want), "{}", m.id);
        // Angka manifest adalah taruhan utama dengan RTP terendah (D-060).
        let lowest_main = all
            .iter()
            .filter(|w| w.game == m.id)
            .map(|w| w.rtp.percent)
            .fold(f64::INFINITY, f64::min);
        assert!(marked[0].rtp.percent >= lowest_main);
    }
}

#[test]
fn every_wager_matches_its_rtp_within_four_sigma() {
    let seed = [42u8; 32];
    let mut failures = Vec::new();
    for w in wagers() {
        let r = (w.run)(100_000, &seed);
        eprintln!(
            "{} {}: RTP {:.4}% ± {:.4}% (diumumkan {}%, {} ronde)",
            w.game,
            w.rtp.wager,
            r.percent(),
            r.tolerance_percent(),
            w.rtp.percent,
            r.rounds
        );
        assert!(r.rounds >= 100_000);
        if (r.percent() - w.rtp.percent).abs() > r.tolerance_percent() {
            failures.push(format!("{} {}", w.game, w.rtp.wager));
        }
    }
    assert!(failures.is_empty(), "di luar 4σ: {failures:?}");
}
