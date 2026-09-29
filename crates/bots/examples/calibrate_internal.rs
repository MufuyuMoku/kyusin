//! Kalibrasi antar-bot untuk game tanpa kalibrasi eksternal (SPEC §8,
//! D-050). Level 1 dipatok 1000; level berikutnya ditaksir berurutan dari
//! hasilnya melawan semua level di bawahnya (taksiran kemungkinan
//! maksimum model Elo logistik, sama dengan kalibrasi catur).
//!
//! Setiap partai dibuka dengan beberapa langkah acak (RNG ber-seed) supaya
//! partai antarbot tidak berulang persis; kursi bergantian.
//!
//! ```text
//! cargo run --release -p kyusin-bots --example calibrate_internal -- \
//!     --game reversi --games 200 --random-plies 4
//! ```

use kyusin_bots::calibration::{ANCHOR, estimate};
use kyusin_core::rng::derive;
use kyusin_core::{GameRng, Player};
use serde_json::json;

fn arg(name: &str, default: &str) -> String {
    let args: Vec<String> = std::env::args().collect();
    args.iter()
        .position(|a| a == name)
        .and_then(|i| args.get(i + 1).cloned())
        .unwrap_or_else(|| default.to_string())
}

/// Satu partai `a` (kursi `a_seat`) melawan `b`; poin untuk `a`.
fn game(id: &str, a: u8, b: u8, a_seat: u8, index: u32, random_plies: u32) -> f64 {
    let registry = kyusin_games::builtin().unwrap();
    let cartridge = registry.get(id).unwrap();
    let seed = derive(&[a ^ (b << 4); 32], &format!("kalibrasi:{a}:{b}:{index}"));
    let mut session = (cartridge.create)(&serde_json::Value::Null, seed).unwrap();
    let mut rng = GameRng::from_seed(derive(&seed, "pembukaan"));
    let mut players: Vec<Box<dyn Player>> = (0..2u8)
        .map(|s| {
            let level = if s == a_seat { a } else { b };
            kyusin_bots::create(id, level, &seed, s).unwrap()
        })
        .collect();
    let mut plies = 0;
    while !session.is_over() && plies < 1000 {
        let seat = session.pending_players()[0];
        let command = if plies < random_plies {
            let moves: Vec<String> = session
                .legal_actions(seat)
                .iter()
                .flat_map(|a| a.concrete(1000).unwrap_or_default())
                .filter(|c| c != "resign")
                .collect();
            moves[rng.below(moves.len() as u32) as usize].clone()
        } else {
            players[seat as usize]
                .decide(session.as_ref(), seat)
                .expect("bot memutuskan")
        };
        session.act(seat, &command).expect("langkah bot sah");
        plies += 1;
    }
    match session.result() {
        Some(r) if r.winners == vec![a_seat] => 1.0,
        Some(r) if r.winners.is_empty() => 0.5,
        Some(_) => 0.0,
        None => 0.5,
    }
}

fn main() {
    let id = arg("--game", "reversi");
    let games: u32 = arg("--games", "200").parse().unwrap();
    let random_plies: u32 = arg("--random-plies", "4").parse().unwrap();
    let out = arg("--out", &format!("data/calibration/{id}.json"));
    let date = arg("--date", "");
    let levels = kyusin_bots::levels(&id);
    assert!(levels > 0, "game `{id}` tidak punya bot");

    // `--pair A,B`: hanya adu level A lawan B (alat bantu menyetel level),
    // tanpa menulis berkas.
    let pair = arg("--pair", "");
    if !pair.is_empty() {
        let ab: Vec<u8> = pair.split(',').map(|x| x.trim().parse().unwrap()).collect();
        let start = std::time::Instant::now();
        let points: f64 = (0..games)
            .map(|i| game(&id, ab[0], ab[1], (i % 2) as u8, i, random_plies))
            .sum();
        let score = points / f64::from(games);
        let diff = if score > 0.0 && score < 1.0 {
            format!("{:+.0}", -400.0 * (1.0 / score - 1.0).log10())
        } else {
            "tak berhingga".into()
        };
        eprintln!(
            "level {} vs level {}: {points}/{games} (selisih ≈{diff}), {:.1} s",
            ab[0],
            ab[1],
            start.elapsed().as_secs_f64()
        );
        return;
    }

    let mut elos: Vec<f64> = vec![ANCHOR];
    let mut report = vec![json!({
        "level": 1,
        "elo": ANCHOR,
        "ci95": [ANCHOR, ANCHOR],
        "extrapolated": false,
        "anchor": true,
        "results": [],
    })];
    for level in 2..=levels {
        let mut rows = Vec::new();
        let mut per_opponent = Vec::new();
        for lower in 1..level {
            let points: f64 = (0..games)
                .map(|i| game(&id, level, lower, (i % 2) as u8, i, random_plies))
                .sum();
            eprintln!("level {level} vs level {lower}: {points}/{games}");
            rows.push((elos[lower as usize - 1], points, f64::from(games)));
            per_opponent.push(json!({
                "opponent_level": lower,
                "points": points,
                "games": games,
            }));
        }
        let (elo, half, extrapolated) = estimate(&rows);
        eprintln!(
            "level {level}: ~{elo:.0} ± {half:.0}{}",
            if extrapolated { " (ekstrapolasi)" } else { "" }
        );
        elos.push(elo.round());
        report.push(json!({
            "level": level,
            "elo": elo.round(),
            "ci95": [(elo - half).round(), (elo + half).round()],
            "extrapolated": extrapolated,
            "results": per_opponent,
        }));
    }
    let doc = json!({
        "game": id,
        "date": date,
        "opponent": "antar-bot",
        "method": format!(
            "level 1 dipatok {ANCHOR}; level berikutnya ditaksir berurutan dari hasil melawan semua level di bawahnya (kemungkinan maksimum model Elo logistik, selang 95% dari informasi Fisher); {random_plies} langkah pembuka acak, kursi bergantian"
        ),
        "games_per_pairing": games,
        "random_plies": random_plies,
        "levels": report,
    });
    std::fs::create_dir_all(std::path::Path::new(&out).parent().unwrap()).unwrap();
    std::fs::write(&out, serde_json::to_string_pretty(&doc).unwrap() + "\n").unwrap();
    eprintln!("ditulis ke {out}");
}
