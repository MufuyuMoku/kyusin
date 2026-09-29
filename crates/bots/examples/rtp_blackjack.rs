//! RTP Blackjack dengan strategi dasar (SPEC §7, D-057).
//!
//! Dipakai workflow `rtp.yml` (≥10.000.000 ronde, manual/terjadwal) dan
//! untuk menghitung angka `rtp` di manifest (ronde sangat banyak).
//!
//! cargo run --release -p kyusin-bots --example rtp_blackjack -- \
//!   --rounds 10000000 --threads 4 [--check]
//!
//! `--check` gagal (kode keluar 1) bila selisih dengan manifest melebihi
//! toleransi 4 × σ/√n.

use kyusin_core::rng::derive;

fn arg(name: &str, default: &str) -> String {
    let args: Vec<String> = std::env::args().collect();
    args.iter()
        .position(|a| a == name)
        .and_then(|i| args.get(i + 1).cloned())
        .unwrap_or_else(|| default.to_string())
}

fn main() {
    let rounds: u64 = arg("--rounds", "10000000").parse().unwrap();
    let threads: u64 = arg("--threads", "4").parse().unwrap();
    let seed_text = arg("--seed", "kyusin-rtp");
    let check = std::env::args().any(|a| a == "--check");
    let base = derive(&[0; 32], &seed_text);
    let start = std::time::Instant::now();
    let per = rounds.div_ceil(threads);
    let parts: Vec<kyusin_bots::blackjack::Rtp> = std::thread::scope(|s| {
        let handles: Vec<_> = (0..threads)
            .map(|t| {
                let seed = derive(&base, &format!("utas:{t}"));
                s.spawn(move || kyusin_bots::blackjack::simulate(per, &seed))
            })
            .collect();
        handles.into_iter().map(|h| h.join().unwrap()).collect()
    });
    // Gabungkan rata-rata dan varians.
    let n: f64 = parts.iter().map(|p| p.rounds as f64).sum();
    let mean = parts.iter().map(|p| p.mean * p.rounds as f64).sum::<f64>() / n;
    let second = parts
        .iter()
        .map(|p| (p.sd * p.sd + p.mean * p.mean) * p.rounds as f64)
        .sum::<f64>()
        / n;
    let sd = (second - mean * mean).max(0.0).sqrt();
    let rtp = 100.0 * (1.0 + mean);
    let tol = 100.0 * 4.0 * sd / n.sqrt();
    let registry = kyusin_games::builtin().unwrap();
    let manifest = registry
        .get(kyusin_games::blackjack::ID)
        .unwrap()
        .manifest
        .rtp
        .unwrap();
    println!(
        "ronde {n:.0} · RTP {rtp:.4}% · σ {sd:.4} · toleransi ±{tol:.4}% · manifest {manifest}% · {:.1} s",
        start.elapsed().as_secs_f64()
    );
    if check && (rtp - manifest).abs() > tol {
        eprintln!("GAGAL: RTP simulasi {rtp:.4}% di luar manifest {manifest}% ± {tol:.4}%");
        std::process::exit(1);
    }
}
