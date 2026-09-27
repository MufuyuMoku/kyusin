//! Papan Reversi sebagai dua bitboard. Bit `baris * 8 + kolom`; baris 0 =
//! baris "1" di atas, kolom 0 = "a". Dipakai mesin aturan dan bot.

use serde::{Deserialize, Serialize};

const NOT_A: u64 = 0xfefe_fefe_fefe_fefe;
const NOT_H: u64 = 0x7f7f_7f7f_7f7f_7f7f;

/// Geser satu langkah ke salah satu dari delapan arah; bit yang keluar papan
/// (termasuk yang "membungkus" ke tepi seberang) dibuang.
#[inline]
fn shift(x: u64, dir: usize) -> u64 {
    match dir {
        0 => (x << 1) & NOT_A, // timur
        1 => (x >> 1) & NOT_H, // barat
        2 => x << 8,           // selatan
        3 => x >> 8,           // utara
        4 => (x << 9) & NOT_A, // tenggara
        5 => (x << 7) & NOT_H, // barat daya
        6 => (x >> 7) & NOT_A, // timur laut
        _ => (x >> 9) & NOT_H, // barat laut
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Square(u8);

impl Square {
    /// `index` 0..64.
    pub fn new(index: u8) -> Self {
        assert!(index < 64, "petak di luar papan");
        Square(index)
    }

    pub fn index(self) -> u8 {
        self.0
    }

    pub fn bit(self) -> u64 {
        1u64 << self.0
    }

    pub fn row(self) -> u8 {
        self.0 / 8
    }

    pub fn col(self) -> u8 {
        self.0 % 8
    }

    /// `a1`..`h8`, huruf kecil.
    pub fn parse(text: &str) -> Option<Square> {
        let b = text.as_bytes();
        if b.len() != 2 || !(b'a'..=b'h').contains(&b[0]) || !(b'1'..=b'8').contains(&b[1]) {
            return None;
        }
        Some(Square((b[1] - b'1') * 8 + (b[0] - b'a')))
    }

    pub fn iter(bits: u64) -> impl Iterator<Item = Square> {
        (0..64u8).filter(move |i| bits >> i & 1 == 1).map(Square)
    }
}

impl std::fmt::Display for Square {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}{}", (b'a' + self.col()) as char, self.row() + 1)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Color {
    Black,
    White,
}

impl Color {
    pub fn other(self) -> Color {
        match self {
            Color::Black => Color::White,
            Color::White => Color::Black,
        }
    }

    /// Hitam = kursi 0, putih = kursi 1.
    pub fn seat(self) -> u8 {
        match self {
            Color::Black => 0,
            Color::White => 1,
        }
    }

    pub fn from_seat(seat: u8) -> Color {
        if seat == 0 {
            Color::Black
        } else {
            Color::White
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Board {
    pub black: u64,
    pub white: u64,
}

impl Board {
    pub fn initial() -> Board {
        let sq = |s: &str| Square::parse(s).unwrap().bit();
        Board {
            black: sq("d5") | sq("e4"),
            white: sq("d4") | sq("e5"),
        }
    }

    pub fn own(&self, c: Color) -> u64 {
        match c {
            Color::Black => self.black,
            Color::White => self.white,
        }
    }

    pub fn empty(&self) -> u64 {
        !(self.black | self.white)
    }

    pub fn count(&self, c: Color) -> u32 {
        self.own(c).count_ones()
    }

    /// Semua petak tempat `c` boleh meletakkan bidak.
    pub fn moves(&self, c: Color) -> u64 {
        let own = self.own(c);
        let opp = self.own(c.other());
        let empty = self.empty();
        let mut moves = 0;
        for dir in 0..8 {
            let mut x = shift(own, dir) & opp;
            for _ in 0..5 {
                x |= shift(x, dir) & opp;
            }
            moves |= shift(x, dir) & empty;
        }
        moves
    }

    /// Bidak lawan yang terbalik bila `c` meletakkan di `sq` (0 = tidak sah).
    pub fn flips(&self, c: Color, sq: Square) -> u64 {
        if self.empty() & sq.bit() == 0 {
            return 0;
        }
        let own = self.own(c);
        let opp = self.own(c.other());
        let mut all = 0;
        for dir in 0..8 {
            let mut line = 0;
            let mut p = shift(sq.bit(), dir);
            while p & opp != 0 {
                line |= p;
                p = shift(p, dir);
            }
            if p & own != 0 {
                all |= line;
            }
        }
        all
    }

    /// Papan setelah `c` meletakkan di `sq` dengan `flips` yang sudah dihitung.
    pub fn play(&self, c: Color, sq: Square, flips: u64) -> Board {
        let placed = sq.bit() | flips;
        match c {
            Color::Black => Board {
                black: self.black | placed,
                white: self.white & !flips,
            },
            Color::White => Board {
                white: self.white | placed,
                black: self.black & !flips,
            },
        }
    }

    /// Delapan baris teks: `.` kosong, `X` hitam, `O` putih.
    pub fn rows(&self) -> Vec<String> {
        (0..8)
            .map(|r| {
                (0..8)
                    .map(|c| {
                        let bit = 1u64 << (r * 8 + c);
                        if self.black & bit != 0 {
                            'X'
                        } else if self.white & bit != 0 {
                            'O'
                        } else {
                            '.'
                        }
                    })
                    .collect()
            })
            .collect()
    }

    pub fn from_rows(rows: &[String]) -> Result<Board, String> {
        if rows.len() != 8 || rows.iter().any(|r| r.chars().count() != 8) {
            return Err("posisi harus 8 baris × 8 karakter".into());
        }
        let mut b = Board { black: 0, white: 0 };
        for (r, row) in rows.iter().enumerate() {
            for (c, ch) in row.chars().enumerate() {
                let bit = 1u64 << (r * 8 + c);
                match ch {
                    'X' => b.black |= bit,
                    'O' => b.white |= bit,
                    '.' => {}
                    other => return Err(format!("karakter posisi tidak dikenal: `{other}`")),
                }
            }
        }
        Ok(b)
    }
}
