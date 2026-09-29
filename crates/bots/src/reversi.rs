//! Bot Reversi, empat level (SPEC §7.2: minimal 3; §8 Rev. 10: selisih
//! rating level berurutan paling besar 400, D-054).
//!
//! 1. Pemula: langkah sah acak.
//! 2. Menengah: satu langkah ke depan dengan tabel bobot petak (sudut
//!    bagus, petak di sebelah sudut buruk).
//! 3. Mahir: alpha-beta 2 langkah dengan bobot petak dan mobilitas.
//! 4. Kuat: alpha-beta 3 langkah; bila petak kosong tinggal 8 atau
//!    kurang, dihitung sampai akhir. (Level 3 lama, 4 langkah + hitung
//!    akhir, membuat jarak level terlalu lebar; D-054.)

use kyusin_core::{GameRng, Player, PlayerId, SeatKind, Seed, Session};
use kyusin_games::reversi::{Board, Color, Square, View};

pub const LEVELS: u8 = 4;

/// Parameter pencarian level 3 ke atas.
struct Search {
    /// Kedalaman pencarian di tengah permainan.
    depth: u32,
    /// Bila petak kosong sebanyak ini atau kurang, dihitung sampai akhir.
    endgame: u32,
}

fn search_params(level: u8) -> Search {
    match level {
        3 => Search {
            depth: 2,
            endgame: 0,
        },
        _ => Search {
            depth: 3,
            endgame: 8,
        },
    }
}

#[rustfmt::skip]
const WEIGHTS: [i32; 64] = [
    100, -20, 10,  5,  5, 10, -20, 100,
    -20, -50, -2, -2, -2, -2, -50, -20,
     10,  -2, -1, -1, -1, -1,  -2,  10,
      5,  -2, -1, -1, -1, -1,  -2,   5,
      5,  -2, -1, -1, -1, -1,  -2,   5,
     10,  -2, -1, -1, -1, -1,  -2,  10,
    -20, -50, -2, -2, -2, -2, -50, -20,
    100, -20, 10,  5,  5, 10, -20, 100,
];

pub struct ReversiBot {
    level: u8,
    rng: GameRng,
}

impl ReversiBot {
    pub fn new(level: u8, seed: Seed) -> Self {
        ReversiBot {
            level,
            rng: GameRng::from_seed(seed),
        }
    }

    /// Memilih langkah untuk `color` di `board`; `None` = harus pass.
    pub fn choose(&mut self, board: &Board, color: Color) -> Option<Square> {
        let mut moves: Vec<Square> = Square::iter(board.moves(color)).collect();
        if moves.is_empty() {
            return None;
        }
        // Acak dulu supaya pilihan yang sama bagus tidak selalu sama.
        self.rng.shuffle(&mut moves);
        match self.level {
            1 => Some(moves[0]),
            2 => moves.into_iter().max_by_key(|&sq| {
                let next = board.play(color, sq, board.flips(color, sq));
                positional(&next, color)
            }),
            level => {
                let p = search_params(level);
                let empties = board.empty().count_ones();
                let exact = empties <= p.endgame;
                let depth = if exact { empties } else { p.depth };
                order(&mut moves);
                let mut best = moves[0];
                let mut alpha = i32::MIN + 1;
                for sq in moves {
                    let next = board.play(color, sq, board.flips(color, sq));
                    let score = -negamax(&next, color.other(), depth - 1, -i32::MAX, -alpha, exact);
                    if score > alpha {
                        alpha = score;
                        best = sq;
                    }
                }
                Some(best)
            }
        }
    }
}

fn positional(board: &Board, me: Color) -> i32 {
    let score = |bits: u64| {
        Square::iter(bits)
            .map(|s| WEIGHTS[s.index() as usize])
            .sum::<i32>()
    };
    score(board.own(me)) - score(board.own(me.other()))
}

fn disc_diff(board: &Board, me: Color) -> i32 {
    board.count(me) as i32 - board.count(me.other()) as i32
}

fn evaluate(board: &Board, me: Color) -> i32 {
    let mobility =
        board.moves(me).count_ones() as i32 - board.moves(me.other()).count_ones() as i32;
    positional(board, me) + 5 * mobility
}

fn order(moves: &mut [Square]) {
    moves.sort_by_key(|s| -WEIGHTS[s.index() as usize]);
}

/// Nilai posisi dari sudut pandang `me` yang akan melangkah.
fn negamax(board: &Board, me: Color, depth: u32, mut alpha: i32, beta: i32, exact: bool) -> i32 {
    let my_moves = board.moves(me);
    if my_moves == 0 {
        if board.moves(me.other()) == 0 {
            // Selesai: menang/kalah selalu lebih penting dari bobot mana pun.
            return disc_diff(board, me) * 10_000;
        }
        return -negamax(board, me.other(), depth, -beta, -alpha, exact);
    }
    if depth == 0 {
        return if exact {
            disc_diff(board, me) * 10_000
        } else {
            evaluate(board, me)
        };
    }
    let mut moves: Vec<Square> = Square::iter(my_moves).collect();
    order(&mut moves);
    let mut best = i32::MIN + 1;
    for sq in moves {
        let next = board.play(me, sq, board.flips(me, sq));
        let score = -negamax(&next, me.other(), depth - 1, -beta, -alpha, exact);
        best = best.max(score);
        alpha = alpha.max(score);
        if alpha >= beta {
            break;
        }
    }
    best
}

impl Player for ReversiBot {
    fn kind(&self) -> SeatKind {
        SeatKind::Bot { level: self.level }
    }

    fn decide(&mut self, session: &dyn Session, seat: PlayerId) -> Option<String> {
        let view: View = serde_json::from_value(session.view_data(seat)).ok()?;
        if view.giliran != Some(seat) {
            return None;
        }
        let board = Board::from_rows(&view.papan).ok()?;
        let color = Color::from_seat(seat);
        Some(match self.choose(&board, color) {
            Some(sq) => sq.to_string(),
            None => "pass".into(),
        })
    }
}
