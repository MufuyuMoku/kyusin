//! Impor/ekspor PGN (SPEC §6.1).
//!
//! Ekspor: tag tujuh baku (+ FEN/SetUp bila posisi awal kustom, TimeControl
//! bila ada jam) dan langkah SAN. Impor: toleran terhadap komentar `{…}` dan
//! `;`, variasi `(…)`, NAG `$n`, nomor langkah, dan hasil; setiap langkah
//! diperiksa terhadap mesin aturan asli.

use kyusin_core::TurnGame;

use super::{Catur, Config, START_FEN};

pub struct Tags {
    pub white: String,
    pub black: String,
    /// `YYYY.MM.DD`.
    pub date: String,
}

impl Tags {
    pub fn new(white: &str, black: &str, date: &str) -> Self {
        Tags {
            white: white.into(),
            black: black.into(),
            date: date.into(),
        }
    }
}

/// Hasil PGN dari kursi pemenang: `1-0`, `0-1`, `1/2-1/2`, atau `*`.
pub fn result_token(winners: Option<&[u8]>) -> &'static str {
    match winners {
        None => "*",
        Some([0]) => "1-0",
        Some([1]) => "0-1",
        Some(_) => "1/2-1/2",
    }
}

/// PGN dari posisi awal, langkah SAN, dan hasil.
pub fn export_moves(
    start_fen: &str,
    sans: &[String],
    result: &str,
    time_control: Option<String>,
    tags: &Tags,
) -> String {
    let esc = |s: &str| s.replace('\\', "\\\\").replace('"', "\\\"");
    let mut out = String::new();
    for (k, v) in [
        ("Event", "KyuSin".to_string()),
        ("Site", "KyuSin".to_string()),
        ("Date", tags.date.clone()),
        ("Round", "-".to_string()),
        ("White", tags.white.clone()),
        ("Black", tags.black.clone()),
        ("Result", result.to_string()),
    ] {
        out.push_str(&format!("[{k} \"{}\"]\n", esc(&v)));
    }
    if start_fen != START_FEN {
        out.push_str(&format!("[FEN \"{}\"]\n[SetUp \"1\"]\n", esc(start_fen)));
    }
    if let Some(tc) = time_control {
        out.push_str(&format!("[TimeControl \"{tc}\"]\n"));
    }
    out.push('\n');

    // Nomor langkah awal mengikuti FEN (hitam jalan duluan → "1...").
    let mut fields = start_fen.split_whitespace();
    let black_first = fields.nth(1) == Some("b");
    let mut number: u32 = fields.nth(3).and_then(|n| n.parse().ok()).unwrap_or(1);
    let mut tokens = Vec::new();
    for (i, san) in sans.iter().enumerate() {
        let white_turn = (i % 2 == 0) != black_first;
        if white_turn {
            tokens.push(format!("{number}."));
        } else if i == 0 {
            tokens.push(format!("{number}..."));
        }
        tokens.push(san.clone());
        if !white_turn {
            number += 1;
        }
    }
    tokens.push(result.to_string());

    let mut line = String::new();
    for tok in tokens {
        if !line.is_empty() && line.len() + 1 + tok.len() > 80 {
            out.push_str(&line);
            out.push('\n');
            line.clear();
        }
        if !line.is_empty() {
            line.push(' ');
        }
        line.push_str(&tok);
    }
    out.push_str(&line);
    out.push('\n');
    out
}

pub fn export(game: &Catur, tags: &Tags) -> String {
    let winners = TurnGame::result(game).map(|r| r.winners);
    export_moves(
        &game.start_fen,
        &game.history,
        result_token(winners.as_deref()),
        game.jam.as_ref().map(|j| format!("{}+{}", j.menit * 60, j.tambahan_detik)),
        tags,
    )
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Imported {
    pub tags: Vec<(String, String)>,
    /// FEN awal bila ada tag FEN.
    pub fen: Option<String>,
    /// Langkah dalam SAN kanonik.
    pub moves: Vec<String>,
    pub result: String,
}

/// Membaca satu partai PGN. Galat berisi teks yang menjelaskan langkah
/// mana yang tidak sah.
pub fn import(text: &str) -> Result<Imported, String> {
    let mut tags = Vec::new();
    let mut body = String::new();
    for line in text.lines() {
        let l = line.trim();
        if l.starts_with('[') && l.ends_with(']') {
            let inner = &l[1..l.len() - 1];
            if let Some((k, v)) = inner.split_once(' ') {
                tags.push((k.to_string(), v.trim().trim_matches('"').replace("\\\"", "\"")));
            }
        } else if !l.starts_with('%') {
            body.push_str(line);
            body.push('\n');
        }
    }

    // Buang komentar dan variasi (bisa bersarang).
    let mut clean = String::new();
    let (mut paren, mut brace, mut semi) = (0usize, false, false);
    for ch in body.chars() {
        match ch {
            '\n' if semi => {
                semi = false;
                clean.push(' ');
            }
            _ if semi => {}
            '{' if !brace => brace = true,
            '}' if brace => brace = false,
            _ if brace => {}
            ';' => semi = true,
            '(' => paren += 1,
            ')' => paren = paren.saturating_sub(1),
            _ if paren > 0 => {}
            c => clean.push(c),
        }
    }

    let fen = tags.iter().find(|(k, _)| k == "FEN").map(|(_, v)| v.clone());
    let mut game = Catur::new(
        Config {
            fen: fen.clone(),
            jam: None,
        },
        [0; 32],
    )
    .map_err(|e| format!("FEN tidak sah: {e}"))?;
    let mut moves = Vec::new();
    let mut result = "*".to_string();
    for tok in clean.split_whitespace() {
        if ["1-0", "0-1", "1/2-1/2", "*"].contains(&tok) {
            result = tok.to_string();
            continue;
        }
        // Nomor langkah (`12.` atau `12...`), bisa menempel ke langkah (`1.e4`).
        let digits = tok.len() - tok.trim_start_matches(|c: char| c.is_ascii_digit()).len();
        let tok = if digits > 0 && tok[digits..].starts_with('.') {
            tok[digits..].trim_start_matches('.')
        } else {
            tok
        };
        if tok.is_empty() || tok.starts_with('$') {
            continue;
        }
        let seat = TurnGame::pending_players(&game)
            .first()
            .copied()
            .ok_or_else(|| format!("langkah ke-{} `{tok}` setelah permainan selesai", moves.len() + 1))?;
        let canonical = kyusin_core::Session::canonical(&game, tok);
        kyusin_core::Session::act(&mut game, seat, tok)
            .map_err(|_| format!("langkah ke-{} `{tok}` tidak sah", moves.len() + 1))?;
        moves.push(canonical);
    }
    Ok(Imported {
        tags,
        fen,
        moves,
        result,
    })
}
