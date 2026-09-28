//! Langkah catur dalam notasi standar: SAN (kanonik) dan koordinat (alias).
//!
//! cozy-chess menyandikan rokade sebagai "raja ke petak benteng" (e1h1);
//! di sini diterjemahkan ke bentuk standar (raja e1→g1, SAN `O-O`).

use cozy_chess::{Board, Color, File, Move, Piece, Square};

/// Satu langkah sah beserta semua bentuk tampilannya.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Legal {
    /// Langkah dalam sandi cozy-chess.
    pub raw: Move,
    /// Petak asal dan tujuan standar (rokade: tujuan raja g1/c1).
    pub from: Square,
    pub to: Square,
    pub promotion: Option<Piece>,
    pub castle: Option<Castle>,
    pub en_passant: bool,
    pub capture: bool,
    /// SAN lengkap dengan `+`/`#`.
    pub san: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Castle {
    King,
    Queen,
}

pub fn piece_letter(p: Piece) -> char {
    match p {
        Piece::Pawn => 'P',
        Piece::Knight => 'N',
        Piece::Bishop => 'B',
        Piece::Rook => 'R',
        Piece::Queen => 'Q',
        Piece::King => 'K',
    }
}

pub fn promotion_char(p: Piece) -> char {
    piece_letter(p).to_ascii_lowercase()
}

fn raw_moves(board: &Board) -> Vec<Move> {
    let mut out = Vec::new();
    board.generate_moves(|pm| {
        out.extend(pm);
        false
    });
    out
}

/// Terjemahan satu langkah cozy-chess ke bentuk standar (tanpa SAN).
fn standard(board: &Board, mv: Move) -> Legal {
    let me = board.side_to_move();
    let piece = board.piece_on(mv.from).expect("ada bidak di petak asal");
    let castle = (piece == Piece::King && board.color_on(mv.to) == Some(me)).then(|| {
        if (mv.to.file() as usize) > (mv.from.file() as usize) {
            Castle::King
        } else {
            Castle::Queen
        }
    });
    let to = match castle {
        Some(Castle::King) => Square::new(File::G, mv.from.rank()),
        Some(Castle::Queen) => Square::new(File::C, mv.from.rank()),
        None => mv.to,
    };
    let en_passant =
        piece == Piece::Pawn && mv.from.file() != mv.to.file() && board.piece_on(mv.to).is_none();
    let capture = castle.is_none() && (board.color_on(mv.to) == Some(!me) || en_passant);
    Legal {
        raw: mv,
        from: mv.from,
        to,
        promotion: mv.promotion,
        castle,
        en_passant,
        capture,
        san: String::new(),
    }
}

/// Semua langkah sah di posisi ini, lengkap dengan SAN.
pub fn legal_moves(board: &Board) -> Vec<Legal> {
    let raw = raw_moves(board);
    let std: Vec<Legal> = raw.iter().map(|m| standard(board, *m)).collect();
    std.iter()
        .map(|m| {
            let mut m = m.clone();
            m.san = san(board, &m, &std);
            m
        })
        .collect()
}

fn san(board: &Board, m: &Legal, all: &[Legal]) -> String {
    let mut s = match m.castle {
        Some(Castle::King) => "O-O".to_string(),
        Some(Castle::Queen) => "O-O-O".to_string(),
        None => {
            let piece = board.piece_on(m.from).unwrap();
            let mut s = String::new();
            if piece == Piece::Pawn {
                if m.capture {
                    s.push(file_char(m.from));
                }
            } else {
                s.push(piece_letter(piece));
                let rivals: Vec<&Legal> = all
                    .iter()
                    .filter(|o| {
                        o.to == m.to
                            && o.from != m.from
                            && o.castle.is_none()
                            && board.piece_on(o.from) == Some(piece)
                    })
                    .collect();
                if !rivals.is_empty() {
                    let same_file = rivals.iter().any(|o| o.from.file() == m.from.file());
                    let same_rank = rivals.iter().any(|o| o.from.rank() == m.from.rank());
                    if !same_file {
                        s.push(file_char(m.from));
                    } else if !same_rank {
                        s.push(rank_char(m.from));
                    } else {
                        s.push(file_char(m.from));
                        s.push(rank_char(m.from));
                    }
                }
            }
            if m.capture {
                s.push('x');
            }
            s.push_str(&m.to.to_string());
            if let Some(p) = m.promotion {
                s.push('=');
                s.push(piece_letter(p));
            }
            s
        }
    };
    let mut after = board.clone();
    after.play(m.raw);
    if !after.checkers().is_empty() {
        s.push(if raw_moves(&after).is_empty() { '#' } else { '+' });
    }
    s
}

pub fn file_char(sq: Square) -> char {
    (b'a' + sq.file() as u8) as char
}

pub fn rank_char(sq: Square) -> char {
    (b'1' + sq.rank() as u8) as char
}

/// SAN tanpa tanda skak dan anotasi, untuk dicocokkan.
fn bare(s: &str) -> String {
    s.trim_end_matches(['+', '#', '!', '?']).to_string()
}

/// Mencari langkah sah dari masukan SAN atau koordinat.
pub fn find(moves: &[Legal], input: &str) -> Option<usize> {
    let input = input.trim();
    if input.is_empty() {
        return None;
    }
    let text = match bare(input).as_str() {
        "0-0" => "O-O".to_string(),
        "0-0-0" => "O-O-O".to_string(),
        other => other.to_string(),
    };
    // Koordinat: e2e4, e7e8q, e1g1.
    let b = text.as_bytes();
    if (b.len() == 4 || b.len() == 5)
        && (b'a'..=b'h').contains(&b[0])
        && (b'1'..=b'8').contains(&b[1])
        && (b'a'..=b'h').contains(&b[2])
        && (b'1'..=b'8').contains(&b[3])
    {
        let from: Square = text[0..2].parse().ok()?;
        let to: Square = text[2..4].parse().ok()?;
        let promo = text.get(4..5);
        let hits: Vec<usize> = moves
            .iter()
            .enumerate()
            .filter(|(_, m)| {
                m.from == from
                    && m.to == to
                    && match (promo, m.promotion) {
                        (None, None) => true,
                        (Some(p), Some(q)) => p.chars().next() == Some(promotion_char(q)),
                        _ => false,
                    }
            })
            .map(|(i, _)| i)
            .collect();
        return (hits.len() == 1).then(|| hits[0]);
    }
    // SAN; toleran terhadap promosi tanpa `=` (e8Q).
    let with_eq = {
        let mut t = text.clone();
        if let Some(last) = t.chars().last()
            && "QRBN".contains(last)
            && !t.contains('=')
            && t.len() >= 3
            && t[..t.len() - 1].chars().last().is_some_and(|c| c.is_ascii_digit())
        {
            t.insert(t.len() - 1, '=');
        }
        t
    };
    moves.iter().position(|m| bare(&m.san) == with_eq)
}

pub fn color_name(c: Color) -> &'static str {
    match c {
        Color::White => "white",
        Color::Black => "black",
    }
}
