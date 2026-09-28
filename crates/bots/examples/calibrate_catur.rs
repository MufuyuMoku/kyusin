//! Kalibrasi level bot catur terhadap Stockfish (SPEC §8, D-041).
//!
//! Hanya dijalankan oleh workflow manual `.github/workflows/calibrate.yml`,
//! yang mengunduh Stockfish saat berjalan. Stockfish tidak masuk repo, tidak
//! ikut dikirim, dan tidak ditautkan: alat ini bicara dengannya lewat UCI.
//!
//! Setiap level bermain melawan Stockfish dengan `UCI_LimitStrength` pada
//! beberapa `UCI_Elo`, dari beberapa pembukaan pendek dengan warna
//! bergantian. Rating tiap level ditaksir dengan kemungkinan maksimum model
//! Elo logistik, beserta selang kepercayaan 95%.
//!
//! cargo run --release -p kyusin-bots --example calibrate_catur -- \
//!   --stockfish PATH --games 16 --elos 1320,1500,1700,1900 \
//!   --levels 1,2,3,4 --movetime 100 --date 2026-09-28 --out data/calibration/catur.json

use std::io::{BufRead, BufReader, Write};
use std::process::{Child, ChildStdin, ChildStdout, Command, Stdio};

use cozy_chess::Board;
use kyusin_bots::catur::ChessBot;
use kyusin_core::rng::derive;
use kyusin_core::{Session, TurnGame};
use kyusin_games::catur::{Catur, Config, START_FEN, san};
use serde_json::json;

const OPENINGS: &[&[&str]] = &[
    &[],
    &["e4", "e5"],
    &["d4", "d5"],
    &["e4", "c5"],
    &["d4", "Nf6", "c4", "e6"],
    &["e4", "e6"],
    &["c4", "e5"],
    &["Nf3", "d5"],
];

struct Stockfish {
    _child: Child,
    stdin: ChildStdin,
    stdout: BufReader<ChildStdout>,
    name: String,
}

impl Stockfish {
    fn start(path: &str) -> Stockfish {
        let mut child = Command::new(path)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .spawn()
            .expect("Stockfish tidak bisa dijalankan");
        let stdin = child.stdin.take().unwrap();
        let stdout = BufReader::new(child.stdout.take().unwrap());
        let mut sf = Stockfish {
            _child: child,
            stdin,
            stdout,
            name: String::new(),
        };
        sf.send("uci");
        loop {
            let line = sf.line();
            if let Some(n) = line.strip_prefix("id name ") {
                sf.name = n.trim().to_string();
            }
            if line.trim() == "uciok" {
                break;
            }
        }
        sf
    }

    fn send(&mut self, cmd: &str) {
        writeln!(self.stdin, "{cmd}").unwrap();
        self.stdin.flush().unwrap();
    }

    fn line(&mut self) -> String {
        let mut s = String::new();
        self.stdout.read_line(&mut s).expect("Stockfish berhenti");
        s
    }

    fn ready(&mut self) {
        self.send("isready");
        while self.line().trim() != "readyok" {}
    }

    fn set_elo(&mut self, elo: u32) {
        self.send("setoption name UCI_LimitStrength value true");
        self.send(&format!("setoption name UCI_Elo value {elo}"));
        self.send("ucinewgame");
        self.ready();
    }

    fn best(&mut self, uci_moves: &[String], movetime: u32) -> String {
        self.send(&format!(
            "position fen {START_FEN} moves {}",
            uci_moves.join(" ")
        ));
        self.send(&format!("go movetime {movetime}"));
        loop {
            let line = self.line();
            if let Some(rest) = line.strip_prefix("bestmove ") {
                return rest.split_whitespace().next().unwrap_or("").to_string();
            }
        }
    }
}

fn uci_of(m: &san::Legal) -> String {
    let promo = m
        .promotion
        .map(|p| san::promotion_char(p).to_string())
        .unwrap_or_default();
    format!("{}{}{promo}", m.from, m.to)
}

/// Satu partai; poin bot (1, ½, 0).
fn game(
    sf: &mut Stockfish,
    level: u8,
    bot_white: bool,
    opening: &[&str],
    seed: u64,
    movetime: u32,
) -> f64 {
    let mut g = Catur::new(Config::default(), [0; 32]).unwrap();
    let mut uci = Vec::new();
    let mut bot = ChessBot::new(level, derive(&[level; 32], &format!("kalibrasi:{seed}")));
    let apply = |g: &mut Catur, uci: &mut Vec<String>, san_text: &str| {
        let legal = g.legal().to_vec();
        let i = san::find(&legal, san_text).expect("langkah sah");
        uci.push(uci_of(&legal[i]));
        let seat = TurnGame::pending_players(g)[0];
        Session::act(g, seat, &legal[i].san).unwrap();
    };
    for m in opening {
        apply(&mut g, &mut uci, m);
    }
    let bot_seat = if bot_white { 0 } else { 1 };
    let mut plies = 0;
    while !TurnGame::is_over(&g) && plies < 400 {
        let seat = TurnGame::pending_players(&g)[0];
        let text = if seat == bot_seat {
            let board: &Board = g.board();
            let mv = bot.choose(board).expect("ada langkah");
            san::legal_moves(board)
                .into_iter()
                .find(|m| m.raw == mv)
                .unwrap()
                .san
        } else {
            sf.best(&uci, movetime)
        };
        apply(&mut g, &mut uci, &text);
        plies += 1;
    }
    match TurnGame::result(&g) {
        Some(r) if r.winners == vec![bot_seat] => 1.0,
        Some(r) if r.winners.is_empty() => 0.5,
        Some(_) => 0.0,
        None => 0.5,
    }
}

fn expected(r: f64, e: f64) -> f64 {
    1.0 / (1.0 + 10f64.powf((e - r) / 400.0))
}

/// Taksiran kemungkinan maksimum + selang 95%. Bila semua kalah atau semua
/// menang, ditambah satu remis semu per lawan supaya taksirannya berhingga
/// (ditandai ekstrapolasi).
fn estimate(results: &[(f64, f64, f64)]) -> (f64, f64, bool) {
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

fn arg(name: &str, default: &str) -> String {
    let args: Vec<String> = std::env::args().collect();
    args.iter()
        .position(|a| a == name)
        .and_then(|i| args.get(i + 1).cloned())
        .unwrap_or_else(|| default.to_string())
}

fn list(text: &str) -> Vec<u32> {
    text.split(',')
        .map(|x| x.trim().parse().expect("angka"))
        .collect()
}

fn main() {
    let path = arg("--stockfish", "stockfish");
    let games: u32 = arg("--games", "16").parse().unwrap();
    let elos = list(&arg("--elos", "1320,1500,1700,1900"));
    let levels = list(&arg("--levels", "1,2,3,4"));
    let movetime: u32 = arg("--movetime", "100").parse().unwrap();
    let out = arg("--out", "data/calibration/catur.json");
    let date = arg("--date", "");

    let mut sf = Stockfish::start(&path);
    eprintln!("mesin lawan: {}", sf.name);
    let mut report = Vec::new();
    for &level in &levels {
        let mut rows = Vec::new();
        let mut per_opponent = Vec::new();
        for &elo in &elos {
            sf.set_elo(elo);
            let mut points = 0.0;
            for i in 0..games {
                let opening = OPENINGS[(i as usize / 2) % OPENINGS.len()];
                let seed = u64::from(level) * 1_000_000 + u64::from(elo) * 100 + u64::from(i);
                points += game(&mut sf, level as u8, i % 2 == 0, opening, seed, movetime);
            }
            eprintln!("level {level} vs UCI_Elo {elo}: {points}/{games}");
            rows.push((f64::from(elo), points, f64::from(games)));
            per_opponent.push(json!({ "opponent_elo": elo, "points": points, "games": games }));
        }
        let (elo, half, extrapolated) = estimate(&rows);
        eprintln!(
            "level {level}: ~{elo:.0} ± {half:.0}{}",
            if extrapolated { " (ekstrapolasi)" } else { "" }
        );
        report.push(json!({
            "level": level,
            "elo": elo.round(),
            "ci95": [(elo - half).round(), (elo + half).round()],
            "extrapolated": extrapolated,
            "results": per_opponent,
        }));
    }
    let doc = json!({
        "game": "catur",
        "date": date,
        "opponent": sf.name,
        "method": "UCI_LimitStrength + UCI_Elo; taksiran kemungkinan maksimum model Elo logistik; selang 95% dari informasi Fisher",
        "movetime_ms": movetime,
        "games_per_pairing": games,
        "openings": OPENINGS.len(),
        "levels": report,
    });
    std::fs::create_dir_all(std::path::Path::new(&out).parent().unwrap()).unwrap();
    std::fs::write(&out, serde_json::to_string_pretty(&doc).unwrap() + "\n").unwrap();
    eprintln!("ditulis ke {out}");
}
