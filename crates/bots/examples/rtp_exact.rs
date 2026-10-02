//! Perhitungan RTP tepat yang berat (SPEC §7: enumerasi bila memungkinkan).
//! Dijalankan di workflow `rtp.yml`, bukan di laptop dan bukan di tes setiap
//! push. Hitungan ringan ada di `tests/meja_exact.rs`.
//!
//! cargo run --release -p kyusin-bots --example rtp_exact -- [--check]
//!
//! `--check` gagal (kode keluar 1) bila hasilnya berbeda dari angka yang
//! diumumkan modul game (4 desimal).

use kyusin_games::meja::WagerRtp;

fn announced(list: &[WagerRtp], wager: &str) -> f64 {
    list.iter().find(|w| w.wager == wager).unwrap().percent
}

fn main() {
    let check = std::env::args().any(|a| a == "--check");
    let mut failed = Vec::new();
    let mut report = |game: &str, wager: &str, list: &[WagerRtp], value: f64, secs: f64| {
        let a = announced(list, wager);
        let ok = (value - a).abs() < 0.00006;
        println!(
            "{game} {wager}: tepat {value:.6}% · diumumkan {a}% · {} · {secs:.1} s",
            if ok { "cocok" } else { "BERBEDA" }
        );
        if !ok {
            failed.push(format!("{game} {wager}"));
        }
    };

    let t = std::time::Instant::now();
    let v = kyusin_bots::three_card_poker::exact_ante_play();
    report(
        kyusin_games::three_card_poker::ID,
        "ante",
        kyusin_games::three_card_poker::RTP,
        v,
        t.elapsed().as_secs_f64(),
    );

    let t = std::time::Instant::now();
    let v = kyusin_bots::let_it_ride::exact();
    report(
        kyusin_games::let_it_ride::ID,
        "bet",
        kyusin_games::let_it_ride::RTP,
        v,
        t.elapsed().as_secs_f64(),
    );

    if check && !failed.is_empty() {
        eprintln!("GAGAL: {failed:?}");
        std::process::exit(1);
    }
}
