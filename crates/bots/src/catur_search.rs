//! Mesin catur untuk level 5–6 (SPEC §9 M2b, D-054): pencarian yang lebih
//! kuat daripada level 1–4, yang tidak diubah supaya data kalibrasinya
//! tetap berlaku.
//!
//! - Iterative deepening dengan PVS, tabel transposisi (hash Zobrist dari
//!   cozy-chess), null-move pruning, late move reduction, reverse futility
//!   pruning, perpanjangan skak, urutan langkah: langkah TT, tangkapan
//!   MVV-LVA, killer, history.
//! - Quiescence untuk tangkapan dan promosi dengan delta pruning.
//! - Evaluasi bertahap (tengah → akhir permainan): bahan, tabel posisi,
//!   mobilitas, pasangan gajah, struktur pion (ganda, terisolasi, bebas),
//!   benteng di lajur terbuka/baris ketujuh, perisai pion raja.
//!
//! Dibatasi jumlah node (bukan waktu) dan tabel transposisi dikosongkan
//! setiap langkah, jadi hasilnya deterministik dari seed.

use cozy_chess::{
    Board, Color, File, Move, Piece, Rank, Square, get_bishop_moves, get_king_moves,
    get_knight_moves, get_pawn_attacks, get_rook_moves,
};

const MATE: i32 = 1_000_000;
const MATE_BOUND: i32 = MATE - 1_000;
const INF: i32 = MATE + 1;
const MAX_PLY: usize = 128;

pub(crate) struct Limits {
    pub nodes: u64,
    /// Gangguan acak ± per langkah akar (sentipion).
    pub noise: i32,
}

// --- Evaluasi ------------------------------------------------------------

/// Bahan (tengah, akhir) per jenis bidak: pion, kuda, gajah, benteng,
/// menteri, raja.
const MATERIAL: [(i32, i32); 6] = [
    (100, 120),
    (320, 300),
    (335, 315),
    (500, 540),
    (960, 960),
    (0, 0),
];

/// Bobot fase: kuda/gajah 1, benteng 2, menteri 4; total 24 = tengah.
const PHASE: [i32; 6] = [0, 1, 1, 2, 4, 0];

// Tabel posisi tengah permainan (sudut pandang putih, baris pertama = a8..h8).
#[rustfmt::skip]
const PST_MG: [[i32; 64]; 6] = [
    [0,0,0,0,0,0,0,0, 40,40,40,40,40,40,40,40, 10,10,20,30,30,20,10,10, 5,5,10,25,25,10,5,5,
     0,0,5,20,20,5,0,0, 5,-5,-5,5,5,-5,-5,5, 5,10,10,-20,-20,10,10,5, 0,0,0,0,0,0,0,0],
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

/// Raja di akhir permainan: ke tengah.
#[rustfmt::skip]
const KING_EG: [i32; 64] = [
    -50,-40,-30,-20,-20,-30,-40,-50, -30,-20,-10,0,0,-10,-20,-30, -30,-10,20,30,30,20,-10,-30, -30,-10,30,40,40,30,-10,-30,
    -30,-10,30,40,40,30,-10,-30, -30,-10,20,30,30,20,-10,-30, -30,-30,0,0,0,0,-30,-30, -50,-30,-30,-30,-30,-30,-30,-50,
];

/// Bidak ringan dan berat di akhir permainan: sedikit ke tengah.
#[rustfmt::skip]
const CENTER_EG: [i32; 64] = [
    -20,-10,-10,-10,-10,-10,-10,-20, -10,0,0,0,0,0,0,-10, -10,0,5,5,5,5,0,-10, -10,0,5,10,10,5,0,-10,
    -10,0,5,10,10,5,0,-10, -10,0,5,5,5,5,0,-10, -10,0,0,0,0,0,0,-10, -20,-10,-10,-10,-10,-10,-10,-20,
];

/// Bonus pion menurut baris relatif (0 = baris pertama).
const PAWN_ADVANCE_EG: [i32; 8] = [0, 0, 5, 10, 20, 35, 60, 0];
const PASSED_MG: [i32; 8] = [0, 5, 10, 15, 25, 45, 70, 0];
const PASSED_EG: [i32; 8] = [0, 10, 20, 35, 60, 95, 140, 0];

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

fn relative_rank(sq: Square, color: Color) -> usize {
    sq.rank().relative_to(color) as usize
}

/// Petak di depan `sq` (arah maju warna itu) pada lajur yang sama.
fn front_span(sq: Square, color: Color) -> u64 {
    let file = sq.file().bitboard().0;
    let rank = sq.rank() as u32;
    let ahead = match color {
        Color::White => {
            if rank >= 7 {
                0
            } else {
                !0u64 << ((rank + 1) * 8)
            }
        }
        Color::Black => {
            if rank == 0 {
                0
            } else {
                !0u64 >> ((8 - rank) * 8)
            }
        }
    };
    file & ahead
}

fn adjacent_files(sq: Square) -> u64 {
    sq.file().adjacent().0
}

/// Nilai posisi dari sudut pandang pihak yang melangkah.
pub(crate) fn evaluate(b: &Board) -> i32 {
    let mut mg = [0i32; 2];
    let mut eg = [0i32; 2];
    let mut phase = 0;
    let occ = b.occupied();
    for (ci, color) in [Color::White, Color::Black].into_iter().enumerate() {
        let us = b.colors(color);
        let them = !color;
        let my_pawns = b.colored_pieces(color, Piece::Pawn);
        let their_pawns = b.colored_pieces(them, Piece::Pawn);
        // Petak yang diserang pion lawan: tidak dihitung sebagai mobilitas.
        let mut pawn_attacks = 0u64;
        for sq in their_pawns {
            pawn_attacks |= get_pawn_attacks(sq, them).0;
        }
        let safe = !(us.0 | pawn_attacks);
        for piece in Piece::ALL {
            for sq in b.colored_pieces(color, piece) {
                let idx = pst_index(sq, color);
                let (m, e) = MATERIAL[piece as usize];
                mg[ci] += m;
                eg[ci] += e;
                phase += PHASE[piece as usize];
                match piece {
                    Piece::Pawn => {
                        mg[ci] += PST_MG[0][idx];
                        let rr = relative_rank(sq, color);
                        eg[ci] += PAWN_ADVANCE_EG[rr];
                        let file_bb = sq.file().bitboard().0;
                        if (my_pawns.0 & file_bb).count_ones() > 1 {
                            mg[ci] -= 5;
                            eg[ci] -= 10;
                        }
                        if my_pawns.0 & adjacent_files(sq) == 0 {
                            mg[ci] -= 12;
                            eg[ci] -= 15;
                        }
                        let span = front_span(sq, color);
                        let blockers = span | (span_adjacent(span));
                        if their_pawns.0 & blockers == 0 {
                            mg[ci] += PASSED_MG[rr];
                            eg[ci] += PASSED_EG[rr];
                        }
                    }
                    Piece::Knight => {
                        mg[ci] += PST_MG[1][idx];
                        eg[ci] += CENTER_EG[idx];
                        let mob = (get_knight_moves(sq).0 & safe).count_ones() as i32;
                        mg[ci] += (mob - 4) * 4;
                        eg[ci] += (mob - 4) * 4;
                    }
                    Piece::Bishop => {
                        mg[ci] += PST_MG[2][idx];
                        eg[ci] += CENTER_EG[idx];
                        let mob = (get_bishop_moves(sq, occ).0 & safe).count_ones() as i32;
                        mg[ci] += (mob - 7) * 5;
                        eg[ci] += (mob - 7) * 5;
                    }
                    Piece::Rook => {
                        mg[ci] += PST_MG[3][idx];
                        eg[ci] += CENTER_EG[idx] / 2;
                        let mob = (get_rook_moves(sq, occ).0 & safe).count_ones() as i32;
                        mg[ci] += (mob - 7) * 2;
                        eg[ci] += (mob - 7) * 4;
                        let file_bb = sq.file().bitboard().0;
                        if my_pawns.0 & file_bb == 0 {
                            if their_pawns.0 & file_bb == 0 {
                                mg[ci] += 25;
                                eg[ci] += 10;
                            } else {
                                mg[ci] += 12;
                                eg[ci] += 5;
                            }
                        }
                        if relative_rank(sq, color) == 6 {
                            mg[ci] += 10;
                            eg[ci] += 20;
                        }
                    }
                    Piece::Queen => {
                        mg[ci] += PST_MG[4][idx];
                        eg[ci] += CENTER_EG[idx];
                        let mob = ((get_rook_moves(sq, occ).0 | get_bishop_moves(sq, occ).0) & safe)
                            .count_ones() as i32;
                        mg[ci] += mob - 14;
                        eg[ci] += (mob - 14) * 2;
                    }
                    Piece::King => {
                        mg[ci] += PST_MG[5][idx];
                        eg[ci] += KING_EG[idx];
                        // Perisai pion di depan raja yang sudah berlindung.
                        let file = sq.file() as usize;
                        if file <= 2 || file >= 5 {
                            let shield_files = sq.file().bitboard().0 | adjacent_files(sq);
                            let near = get_king_moves(sq).0 | front_two(sq, color);
                            let shield = (my_pawns.0 & shield_files & near).count_ones() as i32;
                            mg[ci] += shield.min(3) * 12;
                        }
                    }
                }
            }
        }
        if b.colored_pieces(color, Piece::Bishop).len() >= 2 {
            mg[ci] += 30;
            eg[ci] += 50;
        }
    }
    let phase = phase.min(24);
    let white = ((mg[0] - mg[1]) * phase + (eg[0] - eg[1]) * (24 - phase)) / 24;
    let tempo = 10;
    match b.side_to_move() {
        Color::White => white + tempo,
        Color::Black => -white + tempo,
    }
}

fn span_adjacent(span: u64) -> u64 {
    // Lajur di kiri dan kanan setiap petak di `span`.
    let not_a = !File::A.bitboard().0;
    let not_h = !File::H.bitboard().0;
    ((span & not_a) >> 1) | ((span & not_h) << 1)
}

/// Dua petak di depan raja pada lajurnya dan lajur sebelahnya.
fn front_two(sq: Square, color: Color) -> u64 {
    let rank = sq.rank().relative_to(color) as usize;
    let mut out = 0u64;
    for step in 1..=2 {
        if rank + step > 7 {
            break;
        }
        let r = Rank::index(rank + step).relative_to(color);
        out |= r.bitboard().0;
    }
    out & (sq.file().bitboard().0 | adjacent_files(sq))
}

// --- Pencarian -------------------------------------------------------------

const VALUE: [i32; 6] = [100, 320, 330, 500, 900, 20_000];

#[derive(Clone, Copy)]
struct Entry {
    key: u64,
    mv: Option<Move>,
    score: i32,
    depth: i8,
    flag: u8,
}

const EXACT: u8 = 0;
const LOWER: u8 = 1;
const UPPER: u8 = 2;
const TT_BITS: u32 = 19;

struct Engine {
    tt: Vec<Option<Entry>>,
    nodes: u64,
    limit: u64,
    aborted: bool,
    killers: [[Option<Move>; 2]; MAX_PLY],
    history: Vec<i32>,
    path: Vec<u64>,
}

fn moves_of(b: &Board) -> Vec<Move> {
    let mut out = Vec::with_capacity(48);
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

fn victim(b: &Board, mv: Move) -> i32 {
    if !is_capture(b, mv) {
        return 0;
    }
    b.piece_on(mv.to)
        .map(|p| VALUE[p as usize])
        .unwrap_or(VALUE[0])
}

fn to_tt(score: i32, ply: usize) -> i32 {
    if score > MATE_BOUND {
        score + ply as i32
    } else if score < -MATE_BOUND {
        score - ply as i32
    } else {
        score
    }
}

fn from_tt(score: i32, ply: usize) -> i32 {
    if score > MATE_BOUND {
        score - ply as i32
    } else if score < -MATE_BOUND {
        score + ply as i32
    } else {
        score
    }
}

impl Engine {
    fn new(limit: u64) -> Self {
        Engine {
            tt: vec![None; 1 << TT_BITS],
            nodes: 0,
            limit,
            aborted: false,
            killers: [[None; 2]; MAX_PLY],
            history: vec![0; 64 * 64],
            path: Vec::with_capacity(MAX_PLY),
        }
    }

    fn slot(&self, key: u64) -> usize {
        (key as usize) & ((1 << TT_BITS) - 1)
    }

    fn probe(&self, key: u64) -> Option<Entry> {
        self.tt[self.slot(key)].filter(|e| e.key == key)
    }

    fn store(&mut self, key: u64, depth: i32, score: i32, flag: u8, mv: Option<Move>, ply: usize) {
        let slot = self.slot(key);
        let replace = match self.tt[slot] {
            Some(e) => e.key != key || depth as i8 >= e.depth,
            None => true,
        };
        if replace {
            self.tt[slot] = Some(Entry {
                key,
                mv,
                score: to_tt(score, ply),
                depth: depth.clamp(0, 127) as i8,
                flag,
            });
        }
    }

    fn order(&self, b: &Board, moves: &mut [Move], tt_move: Option<Move>, ply: usize) {
        let killers = self.killers[ply.min(MAX_PLY - 1)];
        moves.sort_by_cached_key(|&mv| {
            let score = if Some(mv) == tt_move {
                10_000_000
            } else if is_capture(b, mv) || mv.promotion.is_some() {
                let attacker = b.piece_on(mv.from).map(|p| VALUE[p as usize]).unwrap_or(0);
                let promo = mv.promotion.map(|p| VALUE[p as usize]).unwrap_or(0);
                1_000_000 + victim(b, mv) * 10 - attacker / 10 + promo
            } else if Some(mv) == killers[0] {
                900_000
            } else if Some(mv) == killers[1] {
                800_000
            } else {
                self.history[mv.from as usize * 64 + mv.to as usize]
            };
            -score
        });
    }

    fn tick(&mut self) -> bool {
        self.nodes += 1;
        if self.nodes > self.limit {
            self.aborted = true;
        }
        self.aborted
    }

    fn negamax(
        &mut self,
        b: &Board,
        mut depth: i32,
        mut alpha: i32,
        beta: i32,
        ply: usize,
        allow_null: bool,
    ) -> i32 {
        if self.tick() {
            return 0;
        }
        let key = b.hash();
        if ply > 0 && (self.path.contains(&key) || b.halfmove_clock() >= 100) {
            return 0;
        }
        if ply >= MAX_PLY - 1 {
            return evaluate(b);
        }
        let in_check = !b.checkers().is_empty();
        if in_check {
            depth += 1;
        }
        if depth <= 0 {
            return self.qsearch(b, alpha, beta, ply);
        }
        let pv = beta - alpha > 1;
        let entry = self.probe(key);
        if let Some(e) = entry
            && !pv
            && i32::from(e.depth) >= depth
        {
            let s = from_tt(e.score, ply);
            match e.flag {
                EXACT => return s,
                LOWER if s >= beta => return s,
                UPPER if s <= alpha => return s,
                _ => {}
            }
        }
        let tt_move = entry.and_then(|e| e.mv);

        if !pv && !in_check {
            let eval = evaluate(b);
            // Reverse futility: posisi jauh di atas beta di kedalaman kecil.
            if depth <= 3 && eval - 120 * depth >= beta && beta.abs() < MATE_BOUND {
                return eval;
            }
            // Null move: bila melewatkan giliran pun masih ≥ beta.
            let heavy = (b.colors(b.side_to_move())
                & !(b.pieces(Piece::Pawn) | b.pieces(Piece::King)))
            .len();
            if allow_null
                && depth >= 3
                && heavy > 0
                && eval >= beta
                && let Some(nb) = b.null_move()
            {
                let r = 2 + depth / 6;
                self.path.push(key);
                let score = -self.negamax(&nb, depth - 1 - r, -beta, -beta + 1, ply + 1, false);
                self.path.pop();
                if self.aborted {
                    return 0;
                }
                if score >= beta && score.abs() < MATE_BOUND {
                    return beta;
                }
            }
        }

        let mut moves = moves_of(b);
        if moves.is_empty() {
            return if in_check { -MATE + ply as i32 } else { 0 };
        }
        self.order(b, &mut moves, tt_move, ply);

        let original_alpha = alpha;
        let mut best = -INF;
        let mut best_move = None;
        self.path.push(key);
        for (i, &mv) in moves.iter().enumerate() {
            let quiet = !is_capture(b, mv) && mv.promotion.is_none();
            let mut child = b.clone();
            child.play_unchecked(mv);
            let gives_check = !child.checkers().is_empty();
            let score = if i == 0 {
                -self.negamax(&child, depth - 1, -beta, -alpha, ply + 1, true)
            } else {
                // Late move reduction untuk langkah tenang yang diurutkan belakang.
                let r = if depth >= 3 && i >= 3 && quiet && !in_check && !gives_check {
                    if i >= 8 { 2 } else { 1 }
                } else {
                    0
                };
                let mut s = -self.negamax(&child, depth - 1 - r, -alpha - 1, -alpha, ply + 1, true);
                if !self.aborted && s > alpha && r > 0 {
                    s = -self.negamax(&child, depth - 1, -alpha - 1, -alpha, ply + 1, true);
                }
                if !self.aborted && s > alpha && s < beta {
                    s = -self.negamax(&child, depth - 1, -beta, -alpha, ply + 1, true);
                }
                s
            };
            if self.aborted {
                self.path.pop();
                return 0;
            }
            if score > best {
                best = score;
                best_move = Some(mv);
            }
            if score > alpha {
                alpha = score;
            }
            if alpha >= beta {
                if quiet {
                    let k = &mut self.killers[ply.min(MAX_PLY - 1)];
                    if k[0] != Some(mv) {
                        k[1] = k[0];
                        k[0] = Some(mv);
                    }
                    let h = &mut self.history[mv.from as usize * 64 + mv.to as usize];
                    *h = (*h + depth * depth).min(700_000);
                }
                break;
            }
        }
        self.path.pop();
        let flag = if best >= beta {
            LOWER
        } else if best > original_alpha {
            EXACT
        } else {
            UPPER
        };
        self.store(key, depth, best, flag, best_move, ply);
        best
    }

    fn qsearch(&mut self, b: &Board, mut alpha: i32, beta: i32, ply: usize) -> i32 {
        if self.tick() {
            return 0;
        }
        let stand = evaluate(b);
        if stand >= beta || ply >= MAX_PLY - 1 {
            return stand;
        }
        alpha = alpha.max(stand);
        let mut caps: Vec<Move> = moves_of(b)
            .into_iter()
            .filter(|&m| is_capture(b, m) || m.promotion.is_some())
            .collect();
        self.order(b, &mut caps, None, ply);
        for mv in caps {
            // Delta pruning: tangkapan ini pun tidak cukup untuk menyusul alpha.
            if mv.promotion.is_none() && stand + victim(b, mv) + 200 < alpha {
                continue;
            }
            let mut child = b.clone();
            child.play_unchecked(mv);
            let score = -self.qsearch(&child, -beta, -alpha, ply + 1);
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

/// Langkah terbaik dalam batas node. `noise` diambil dari RNG pemanggil,
/// satu nilai per langkah akar (urutan `moves`).
pub(crate) fn best_move(board: &Board, moves: &[Move], noise: &[i32], limits: &Limits) -> Move {
    let mut engine = Engine::new(limits.nodes);
    let mut order: Vec<usize> = (0..moves.len()).collect();
    let mut best = moves[0];
    for depth in 1..MAX_PLY as i32 {
        let mut alpha = -INF;
        let mut depth_best: Option<(usize, i32)> = None;
        for &i in &order {
            let mut child = board.clone();
            child.play_unchecked(moves[i]);
            engine.path.push(board.hash());
            // Nilai akar = nilai pencarian + gangguan tetap langkah itu.
            let searched = if depth_best.is_none() {
                -engine.negamax(&child, depth - 1, -INF, INF, 1, true)
            } else {
                let target = alpha - noise[i];
                let s = -engine.negamax(&child, depth - 1, -target - 1, -target, 1, true);
                if !engine.aborted && s > target {
                    -engine.negamax(&child, depth - 1, -INF, -target, 1, true)
                } else {
                    s
                }
            };
            let score = searched + noise[i];
            engine.path.pop();
            if engine.aborted {
                break;
            }
            if score > alpha {
                alpha = score;
                depth_best = Some((i, score));
            }
        }
        if engine.aborted {
            // Kedalaman yang belum selesai: pakai langkah terbaik yang
            // sudah terbukti lebih baik di kedalaman ini (bila ada).
            if let Some((i, _)) = depth_best
                && depth > 1
            {
                best = moves[i];
            }
            break;
        }
        if let Some((i, score)) = depth_best {
            best = moves[i];
            if let Some(pos) = order.iter().position(|&x| x == i) {
                order.remove(pos);
                order.insert(0, i);
            }
            // Mat ditemukan: tidak perlu lebih dalam.
            if score.abs() > MATE_BOUND {
                break;
            }
        }
    }
    best
}

#[cfg(test)]
mod tests {
    use super::*;

    fn pick(fen: &str, nodes: u64) -> String {
        let b = Board::from_fen(fen, false).unwrap();
        let moves = moves_of(&b);
        let noise = vec![0; moves.len()];
        let mv = best_move(&b, &moves, &noise, &Limits { nodes, noise: 0 });
        format!("{mv}")
    }

    #[test]
    fn evaluation_is_symmetric() {
        for fen in [
            "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1",
            "r1bqkb1r/pppp1ppp/2n2n2/4p3/2B1P3/5N2/PPPP1PPP/RNBQK2R w KQkq - 4 4",
            "8/5k2/3p4/1p1Pp2p/pP2Pp1P/P4P1K/8/8 b - - 99 50",
        ] {
            let b = Board::from_fen(fen, false).unwrap();
            // Posisi dicerminkan warnanya harus bernilai sama bagi yang melangkah.
            let mirrored = mirror(fen);
            let m = Board::from_fen(&mirrored, false).unwrap();
            assert_eq!(evaluate(&b), evaluate(&m), "{fen} vs {mirrored}");
        }
    }

    fn mirror(fen: &str) -> String {
        let parts: Vec<&str> = fen.split(' ').collect();
        let rows: Vec<String> = parts[0]
            .split('/')
            .rev()
            .map(|r| {
                r.chars()
                    .map(|c| {
                        if c.is_ascii_uppercase() {
                            c.to_ascii_lowercase()
                        } else {
                            c.to_ascii_uppercase()
                        }
                    })
                    .collect()
            })
            .collect();
        let side = if parts[1] == "w" { "b" } else { "w" };
        let castle: String = if parts[2] == "-" {
            "-".into()
        } else {
            let mut c: Vec<char> = parts[2]
                .chars()
                .map(|c| {
                    if c.is_ascii_uppercase() {
                        c.to_ascii_lowercase()
                    } else {
                        c.to_ascii_uppercase()
                    }
                })
                .collect();
            c.sort_by_key(|c| (c.is_ascii_lowercase(), *c));
            c.into_iter().collect()
        };
        format!(
            "{} {side} {castle} - {} {}",
            rows.join("/"),
            parts[4],
            parts[5]
        )
    }

    #[test]
    fn finds_back_rank_mate() {
        // Mat baris belakang dalam satu langkah.
        let mv = pick("6k1/5ppp/8/8/8/8/5PPP/3R2K1 w - - 0 1", 200_000);
        assert_eq!(mv, "d1d8", "mat baris belakang");
    }

    #[test]
    fn takes_a_free_queen_and_avoids_hanging_its_own() {
        assert_eq!(pick("4k3/8/8/3q4/4P3/8/8/4K3 w - - 0 1", 100_000), "e4d5");
    }

    #[test]
    fn deterministic_for_the_same_input() {
        let fen = "r1bqkb1r/pppp1ppp/2n2n2/4p3/2B1P3/5N2/PPPP1PPP/RNBQK2R w KQkq - 4 4";
        assert_eq!(pick(fen, 150_000), pick(fen, 150_000));
    }
}
