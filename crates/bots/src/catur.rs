//! Mesin catur sendiri (SPEC §6.1): negamax alpha-beta dengan iterative
//! deepening, pencarian quiescence untuk tangkapan, urutan langkah MVV-LVA,
//! dan evaluasi bahan + tabel posisi. Generator langkah dari cozy-chess.
//!
//! Level dibatasi kedalaman dan jumlah node (bukan waktu), jadi hasilnya
//! deterministik dari seed dan sama di semua mesin. Perkiraan rating tiap
//! level berasal dari kalibrasi terhadap Stockfish (data/calibration).
//!
//! Level 5–6 (M2b) memakai mesin yang lebih kuat di [`crate::catur_search`];
//! level 1–4 tidak diubah supaya data kalibrasinya tetap berlaku.

use cozy_chess::{Board, Color, Move, Piece, Square};
use kyusin_core::{GameRng, Player, PlayerId, SeatKind, Seed, Session};
use kyusin_games::catur::{View, san};

pub const LEVELS: u8 = 6;

const MATE: i32 = 1_000_000;

struct Params {
    depth: u32,
    quiesce: bool,
    /// Gangguan acak ± per langkah akar (sentipion).
    noise: i32,
    node_limit: u64,
}

fn params(level: u8) -> Params {
    match level {
        1 => Params {
            depth: 1,
            quiesce: false,
            noise: 150,
            node_limit: 20_000,
        },
        2 => Params {
            depth: 2,
            quiesce: true,
            noise: 40,
            node_limit: 60_000,
        },
        3 => Params {
            depth: 3,
            quiesce: true,
            noise: 10,
            node_limit: 250_000,
        },
        _ => Params {
            depth: 4,
            quiesce: true,
            noise: 0,
            node_limit: 600_000,
        },
    }
}

/// Batas node mesin kuat untuk level 5–6 (D-054), bila level itu memakainya.
pub fn strong_nodes(level: u8) -> Option<u64> {
    strong_limits(level).map(|l| l.nodes)
}

/// Batas mesin kuat untuk level 5–6 (D-054).
fn strong_limits(level: u8) -> Option<crate::catur_search::Limits> {
    match level {
        5 => Some(crate::catur_search::Limits {
            nodes: 10_000,
            noise: 5,
        }),
        6 => Some(crate::catur_search::Limits {
            nodes: 40_000,
            noise: 0,
        }),
        _ => None,
    }
}

const VALUE: [i32; 6] = [100, 320, 330, 500, 900, 0];

fn value(p: Piece) -> i32 {
    VALUE[p as usize]
}

// Tabel posisi (sudut pandang putih, a8..h8 di baris pertama).
#[rustfmt::skip]
const PST: [[i32; 64]; 6] = [
    [0,0,0,0,0,0,0,0, 50,50,50,50,50,50,50,50, 10,10,20,30,30,20,10,10, 5,5,10,25,25,10,5,5,
     0,0,0,20,20,0,0,0, 5,-5,-10,0,0,-10,-5,5, 5,10,10,-20,-20,10,10,5, 0,0,0,0,0,0,0,0],
    [-50,-40,-30,-30,-30,-30,-40,-50, -40,-20,0,0,0,0,-20,-40, -30,0,10,15,15,10,0,-30, -30,5,15,20,20,15,5,-30,
     -30,0,15,20,20,15,0,-30, -30,5,10,15,15,10,5,-30, -40,-20,0,5,5,0,-20,-40, -50,-40,-30,-30,-30,-30,-40,-50],
    [-20,-10,-10,-10,-10,-10,-10,-20, -10,0,0,0,0,0,0,-10, -10,0,5,10,10,5,0,-10, -10,5,5,10,10,5,5,-10,
     -10,0,10,10,10,10,0,-10, -10,10,10,10,10,10,10,-10, -10,5,0,0,0,0,5,-10, -20,-10,-10,-10,-10,-10,-10,-20],
    [0,0,0,0,0,0,0,0, 5,10,10,10,10,10,10,5, -5,0,0,0,0,0,0,-5, -5,0,0,0,0,0,0,-5,
     -5,0,0,0,0,0,0,-5, -5,0,0,0,0,0,0,-5, -5,0,0,0,0,0,0,-5, 0,0,0,5,5,0,0,0],
    [-20,-10,-10,-5,-5,-10,-10,-20, -10,0,0,0,0,0,0,-10, -10,0,5,5,5,5,0,-10, -5,0,5,5,5,5,0,-5,
     0,0,5,5,5,5,0,-5, -10,5,5,5,5,5,0,-10, -10,0,5,0,0,0,0,-10, -20,-10,-10,-5,-5,-10,-10,-20],
    [-30,-40,-40,-50,-50,-40,-40,-30, -30,-40,-40,-50,-50,-40,-40,-30, -30,-40,-40,-50,-50,-40,-40,-30, -30,-40,-40,-50,-50,-40,-40,-30,
     -20,-30,-30,-40,-40,-30,-30,-20, -10,-20,-20,-20,-20,-20,-20,-10, 20,20,0,0,0,0,20,20, 20,30,10,0,0,10,30,20],
];

#[rustfmt::skip]
const KING_END: [i32; 64] = [
    -50,-40,-30,-20,-20,-30,-40,-50, -30,-20,-10,0,0,-10,-20,-30, -30,-10,20,30,30,20,-10,-30, -30,-10,30,40,40,30,-10,-30,
    -30,-10,30,40,40,30,-10,-30, -30,-10,20,30,30,20,-10,-30, -30,-30,0,0,0,0,-30,-30, -50,-30,-30,-30,-30,-30,-30,-50,
];

/// Indeks tabel untuk petak dari sudut pandang warna.
fn pst_index(sq: Square, color: Color) -> usize {
    let file = sq.file() as usize;
    let rank = sq.rank() as usize;
    let row = match color {
        Color::White => 7 - rank,
        Color::Black => rank,
    };
    row * 8 + file
}

/// Nilai posisi dari sudut pandang pihak yang melangkah.
fn evaluate(b: &Board) -> i32 {
    let queens = b.pieces(Piece::Queen).len();
    let minors = (b.pieces(Piece::Knight) | b.pieces(Piece::Bishop) | b.pieces(Piece::Rook)).len();
    let endgame = queens == 0 || (queens <= 2 && minors <= 2);
    let mut score = 0;
    for color in [Color::White, Color::Black] {
        let sign = if color == b.side_to_move() { 1 } else { -1 };
        for piece in Piece::ALL {
            for sq in b.colored_pieces(color, piece) {
                let idx = pst_index(sq, color);
                let pos = if piece == Piece::King && endgame {
                    KING_END[idx]
                } else {
                    PST[piece as usize][idx]
                };
                score += sign * (value(piece) + pos);
            }
        }
    }
    score
}

fn moves_of(b: &Board) -> Vec<Move> {
    let mut out = Vec::new();
    b.generate_moves(|pm| {
        out.extend(pm);
        false
    });
    out
}

fn is_capture(b: &Board, mv: Move) -> bool {
    b.color_on(mv.to) == Some(!b.side_to_move())
        || (b.piece_on(mv.from) == Some(Piece::Pawn) && mv.from.file() != mv.to.file())
}

/// Urutan: promosi dan tangkapan (korban termahal, penyerang termurah) dulu.
fn order(b: &Board, moves: &mut [Move]) {
    moves.sort_by_key(|&mv| {
        let victim = b
            .piece_on(mv.to)
            .filter(|_| is_capture(b, mv))
            .map(value)
            .unwrap_or(0);
        let attacker = b.piece_on(mv.from).map(value).unwrap_or(0);
        let promo = mv.promotion.map(value).unwrap_or(0);
        -(victim * 10 - attacker / 10 + promo * 10)
    });
}

struct Search {
    nodes: u64,
    limit: u64,
    quiesce: bool,
    aborted: bool,
}

impl Search {
    fn negamax(&mut self, b: &Board, depth: u32, mut alpha: i32, beta: i32, ply: i32) -> i32 {
        self.nodes += 1;
        if self.nodes > self.limit {
            self.aborted = true;
            return 0;
        }
        let mut moves = moves_of(b);
        if moves.is_empty() {
            return if b.checkers().is_empty() {
                0
            } else {
                -MATE + ply
            };
        }
        if b.halfmove_clock() >= 100 {
            return 0;
        }
        if depth == 0 {
            return if self.quiesce {
                self.qsearch(b, alpha, beta, 0)
            } else {
                evaluate(b)
            };
        }
        order(b, &mut moves);
        let mut best = -MATE;
        for mv in moves {
            let mut child = b.clone();
            child.play_unchecked(mv);
            let score = -self.negamax(&child, depth - 1, -beta, -alpha, ply + 1);
            if self.aborted {
                return 0;
            }
            best = best.max(score);
            alpha = alpha.max(score);
            if alpha >= beta {
                break;
            }
        }
        best
    }

    fn qsearch(&mut self, b: &Board, mut alpha: i32, beta: i32, qply: u32) -> i32 {
        self.nodes += 1;
        if self.nodes > self.limit {
            self.aborted = true;
            return 0;
        }
        let stand = evaluate(b);
        if stand >= beta || qply >= 8 {
            return stand;
        }
        alpha = alpha.max(stand);
        let mut caps: Vec<Move> = moves_of(b)
            .into_iter()
            .filter(|&m| is_capture(b, m) || m.promotion.is_some())
            .collect();
        order(b, &mut caps);
        for mv in caps {
            let mut child = b.clone();
            child.play_unchecked(mv);
            let score = -self.qsearch(&child, -beta, -alpha, qply + 1);
            if self.aborted {
                return 0;
            }
            if score >= beta {
                return score;
            }
            alpha = alpha.max(score);
        }
        alpha
    }
}

pub struct ChessBot {
    level: u8,
    rng: GameRng,
    /// Batas node pengganti untuk level 5–6; hanya untuk alat kalibrasi.
    nodes: Option<u64>,
}

impl ChessBot {
    pub fn new(level: u8, seed: Seed) -> Self {
        ChessBot {
            level,
            rng: GameRng::from_seed(seed),
            nodes: None,
        }
    }

    /// Seperti [`ChessBot::new`], dengan batas node pengganti untuk level
    /// 5–6. Dipakai alat kalibrasi untuk mengukur beberapa kandidat
    /// sebelum konstantanya ditetapkan.
    pub fn with_nodes(level: u8, seed: Seed, nodes: Option<u64>) -> Self {
        ChessBot {
            nodes,
            ..ChessBot::new(level, seed)
        }
    }

    /// Langkah terbaik menurut level ini (sandi cozy-chess); `None` bila
    /// tidak ada langkah sah.
    pub fn choose(&mut self, board: &Board) -> Option<Move> {
        let mut moves = moves_of(board);
        if moves.is_empty() {
            return None;
        }
        if let Some(mut limits) = strong_limits(self.level) {
            if let Some(n) = self.nodes {
                limits.nodes = n;
            }
            self.rng.shuffle(&mut moves);
            let noise: Vec<i32> = moves
                .iter()
                .map(|_| {
                    if limits.noise == 0 {
                        0
                    } else {
                        self.rng.below(2 * limits.noise as u32 + 1) as i32 - limits.noise
                    }
                })
                .collect();
            return Some(crate::catur_search::best_move(
                board, &moves, &noise, &limits,
            ));
        }
        let p = params(self.level);
        self.rng.shuffle(&mut moves);
        order(board, &mut moves);
        // Gangguan tetap per langkah akar selama pencarian ini.
        let noise: Vec<i32> = moves
            .iter()
            .map(|_| {
                if p.noise == 0 {
                    0
                } else {
                    self.rng.below(2 * p.noise as u32 + 1) as i32 - p.noise
                }
            })
            .collect();
        let mut search = Search {
            nodes: 0,
            limit: p.node_limit,
            quiesce: p.quiesce,
            aborted: false,
        };
        let mut best = moves[0];
        for depth in 1..=p.depth {
            let mut alpha = -MATE - 1;
            let mut depth_best = None;
            for (i, &mv) in moves.iter().enumerate() {
                let mut child = board.clone();
                child.play_unchecked(mv);
                let score =
                    -search.negamax(&child, depth - 1, -MATE - 1, -alpha + noise[i], 1) + noise[i];
                if search.aborted {
                    break;
                }
                if score > alpha {
                    alpha = score;
                    depth_best = Some(mv);
                }
            }
            if search.aborted {
                break;
            }
            if let Some(mv) = depth_best {
                best = mv;
                // Langkah terbaik dicoba lebih dulu di kedalaman berikutnya.
                if let Some(pos) = moves.iter().position(|m| *m == mv) {
                    moves.swap(0, pos);
                }
            }
        }
        Some(best)
    }
}

impl Player for ChessBot {
    fn kind(&self) -> SeatKind {
        SeatKind::Bot { level: self.level }
    }

    fn decide(&mut self, session: &dyn Session, seat: PlayerId) -> Option<String> {
        let view: View = serde_json::from_value(session.view_data(seat)).ok()?;
        if view.giliran != Some(seat) {
            return None;
        }
        let board = Board::from_fen(&view.fen, false).ok()?;
        let mv = self.choose(&board)?;
        san::legal_moves(&board)
            .into_iter()
            .find(|m| m.raw == mv)
            .map(|m| m.san)
    }
}
