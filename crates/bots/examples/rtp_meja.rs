//! Simulasi RTP meja casino M5a (SPEC §7): setiap jenis taruhan yang
//! diumumkan modul game, ≥10.000.000 ronde per taruhan di workflow
//! `rtp.yml` (manual/terjadwal; bukan di laptop).
//!
//! cargo run --release -p kyusin-bots --example rtp_meja -- \
//!   --rounds 10000000 --threads 4 [--game <id>] [--check]
//!
//! `--check` gagal (kode keluar 1) bila ada taruhan yang RTP simulasinya
//! berselisih lebih dari 4 × σ/√n dengan angka yang diumumkan.

use kyusin_bots::meja::{Rtp, wagers};
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
    let game = arg("--game", "all");
    let seed_text = arg("--seed", "kyusin-rtp-meja");
    let check = std::env::args().any(|a| a == "--check");
    let base = derive(&[0; 32], &seed_text);
    let mut failed = Vec::new();
    for w in wagers()
        .into_iter()
        .filter(|w| game == "all" || w.game == game)
    {
        let start = std::time::Instant::now();
        let per = rounds.div_ceil(threads);
        let seed = derive(&base, &format!("{}:{}", w.game, w.rtp.wager));
        let parts: Vec<Rtp> = std::thread::scope(|s| {
            let handles: Vec<_> = (0..threads)
                .map(|t| {
                    let seed = derive(&seed, &format!("utas:{t}"));
                    s.spawn(move || (w.run)(per, &seed))
                })
                .collect();
            handles.into_iter().map(|h| h.join().unwrap()).collect()
        });
        let r = Rtp::merge(&parts);
        let (rtp, tol) = (r.percent(), r.tolerance_percent());
        let ok = (rtp - w.rtp.percent).abs() <= tol;
        println!(
            "{} {}: ronde {} · RTP {rtp:.4}% · σ {:.4} · toleransi ±{tol:.4}% · diumumkan {}% · {} · {:.1} s",
            w.game,
            w.rtp.wager,
            r.rounds,
            r.sd,
            w.rtp.percent,
            if ok { "cocok" } else { "DI LUAR TOLERANSI" },
            start.elapsed().as_secs_f64()
        );
        if !ok {
            failed.push(format!("{} {}", w.game, w.rtp.wager));
        }
    }
    if check && !failed.is_empty() {
        eprintln!("GAGAL: {failed:?}");
        std::process::exit(1);
    }
}
