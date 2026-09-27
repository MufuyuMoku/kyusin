//! Reversi (SPEC §6.1): game katalog pertama, bukti kontrak M1.
//!
//! Hitam (X, kursi 0) jalan duluan dari posisi awal standar. Langkah sah
//! mengapit minimal satu garis lawan; semua garis terapit dibalik. Yang
//! tidak punya langkah wajib `pass` (dan hanya saat itu). Selesai bila kedua
//! pemain tidak bisa melangkah; bidak terbanyak menang. Tidak memakai acak.
//! Perintah teks: petak `a1`..`h8`, atau `pass`.

mod board;

use std::sync::OnceLock;

use kyusin_core::game::{GameResult, create_session};
use kyusin_core::i18n::{Catalog, Lang, Localized};
use kyusin_core::{ActionSpec, Cartridge, GameError, PlayerId, RegistryError, Seed, TurnGame};
use serde::{Deserialize, Serialize};

pub use board::{Board, Color, Square};

pub const ID: &str = "reversi";

pub fn catalog() -> &'static Catalog {
    static CATALOG: OnceLock<Catalog> = OnceLock::new();
    CATALOG.get_or_init(|| {
        Catalog::from_toml(include_str!("i18n.toml")).expect("reversi/i18n.toml tidak sah")
    })
}

pub fn cartridge() -> Result<Cartridge, RegistryError> {
    Cartridge::new(
        include_str!("manifest.toml"),
        include_str!("../../../../tutorials/reversi.toml"),
        create_session::<Reversi>,
    )
}

/// Konfigurasi; posisi kustom dipakai tutorial dan tes (keadaan awal).
#[derive(Debug, Clone, Default, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Config {
    /// 8 baris × 8 karakter: `.` kosong, `X` hitam, `O` putih.
    #[serde(default)]
    pub posisi: Option<Vec<String>>,
    /// `hitam` atau `putih`; bawaan hitam.
    #[serde(default)]
    pub giliran: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Move {
    Place(Square),
    Pass,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct View {
    /// 8 baris teks: `.` kosong, `X` hitam, `O` putih.
    pub papan: Vec<String>,
    /// Kursi yang sedang melangkah; `null` bila selesai.
    pub giliran: Option<PlayerId>,
    pub kamu: PlayerId,
    /// Langkah sah pemain yang sedang melangkah.
    pub legal: Vec<String>,
    pub hitam: u32,
    pub putih: u32,
    pub terakhir: Option<String>,
    pub dibalik: Vec<String>,
    pub selesai: bool,
    /// Kursi pemenang bila selesai; kosong = seri.
    pub pemenang: Option<Vec<PlayerId>>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Reversi {
    board: Board,
    turn: Color,
    last: Option<Square>,
    flipped: u64,
    over: bool,
}

impl Reversi {
    pub fn board(&self) -> Board {
        self.board
    }

    pub fn turn(&self) -> Color {
        self.turn
    }

    fn check_over(&mut self) {
        self.over = self.board.moves(Color::Black) == 0 && self.board.moves(Color::White) == 0;
    }

    fn winners(&self) -> Vec<PlayerId> {
        let (b, w) = (
            self.board.count(Color::Black),
            self.board.count(Color::White),
        );
        match b.cmp(&w) {
            std::cmp::Ordering::Greater => vec![0],
            std::cmp::Ordering::Less => vec![1],
            std::cmp::Ordering::Equal => Vec::new(),
        }
    }
}

fn color_name(lang: Lang, c: Color) -> String {
    catalog().text(
        lang,
        match c {
            Color::Black => "black",
            Color::White => "white",
        },
        &[],
    )
}

fn outcome(lang: Lang, winners: &[PlayerId], b: u32, w: u32) -> String {
    let c = catalog();
    let (b, w) = (b.to_string(), w.to_string());
    match winners.first() {
        Some(&seat) => c.text(
            lang,
            "wins",
            &[
                ("color", &color_name(lang, Color::from_seat(seat))),
                ("b", &b),
                ("w", &w),
            ],
        ),
        None => c.text(lang, "draw", &[("b", &b), ("w", &w)]),
    }
}

impl TurnGame for Reversi {
    type Config = Config;
    type Action = Move;
    type View = View;

    fn new(config: Config, _seed: Seed) -> Result<Self, GameError> {
        let board = match &config.posisi {
            Some(rows) => Board::from_rows(rows).map_err(GameError::Config)?,
            None => Board::initial(),
        };
        let turn = match config.giliran.as_deref() {
            None | Some("hitam") => Color::Black,
            Some("putih") => Color::White,
            Some(other) => return Err(GameError::Config(format!("giliran `{other}`"))),
        };
        let mut g = Reversi {
            board,
            turn,
            last: None,
            flipped: 0,
            over: false,
        };
        g.check_over();
        Ok(g)
    }

    fn seats(&self) -> u8 {
        2
    }

    fn pending_players(&self) -> Vec<PlayerId> {
        if self.over {
            Vec::new()
        } else {
            vec![self.turn.seat()]
        }
    }

    fn legal_actions(&self, player: PlayerId) -> Vec<ActionSpec> {
        if self.over || player != self.turn.seat() {
            return Vec::new();
        }
        let moves = self.board.moves(self.turn);
        if moves == 0 {
            return vec![ActionSpec::fixed("pass")];
        }
        Square::iter(moves)
            .map(|s| ActionSpec::fixed(s.to_string()))
            .collect()
    }

    fn apply(&mut self, player: PlayerId, action: Move) -> Result<(), GameError> {
        if self.over {
            return Err(GameError::Over);
        }
        if player != self.turn.seat() {
            return Err(GameError::NotPending(player));
        }
        let moves = self.board.moves(self.turn);
        match action {
            Move::Pass => {
                if moves != 0 {
                    return Err(GameError::Illegal("pass".into()));
                }
                self.last = None;
                self.flipped = 0;
            }
            Move::Place(sq) => {
                if moves & sq.bit() == 0 {
                    return Err(GameError::Illegal(sq.to_string()));
                }
                let flips = self.board.flips(self.turn, sq);
                self.board = self.board.play(self.turn, sq, flips);
                self.last = Some(sq);
                self.flipped = flips;
            }
        }
        self.turn = self.turn.other();
        self.check_over();
        Ok(())
    }

    fn view_for(&self, player: PlayerId) -> View {
        let legal = if self.over {
            Vec::new()
        } else {
            let moves = self.board.moves(self.turn);
            if moves == 0 {
                vec!["pass".into()]
            } else {
                Square::iter(moves).map(|s| s.to_string()).collect()
            }
        };
        View {
            papan: self.board.rows(),
            giliran: (!self.over).then(|| self.turn.seat()),
            kamu: player,
            legal,
            hitam: self.board.count(Color::Black),
            putih: self.board.count(Color::White),
            terakhir: self.last.map(|s| s.to_string()),
            dibalik: Square::iter(self.flipped).map(|s| s.to_string()).collect(),
            selesai: self.over,
            pemenang: self.over.then(|| self.winners()),
        }
    }

    fn to_text(view: &View, lang: Lang) -> String {
        let c = catalog();
        let mine = view.giliran == Some(view.kamu);
        let mut out = String::from("    a   b   c   d   e   f   g   h\n");
        out.push_str("  ┌───┬───┬───┬───┬───┬───┬───┬───┐\n");
        for (r, row) in view.papan.iter().enumerate() {
            out.push_str(&format!("{} │", r + 1));
            for (col, ch) in row.chars().enumerate() {
                let sq = format!("{}{}", (b'a' + col as u8) as char, r + 1);
                let cell = match ch {
                    'X' => '●',
                    'O' => '○',
                    _ if mine && view.legal.contains(&sq) => '·',
                    _ => ' ',
                };
                out.push_str(&format!(" {cell} │"));
            }
            out.push('\n');
            out.push_str(if r == 7 {
                "  └───┴───┴───┴───┴───┴───┴───┴───┘\n"
            } else {
                "  ├───┼───┼───┼───┼───┼───┼───┼───┤\n"
            });
        }
        out.push('\n');
        out.push_str(&c.text(
            lang,
            "count",
            &[
                ("b", &view.hitam.to_string()),
                ("w", &view.putih.to_string()),
            ],
        ));
        out.push('\n');
        let you = color_name(lang, Color::from_seat(view.kamu));
        let status = match (&view.pemenang, view.giliran) {
            (Some(w), _) => outcome(lang, w, view.hitam, view.putih),
            (None, Some(_)) if mine && view.legal == ["pass"] => {
                c.text(lang, "must_pass", &[("color", &you)])
            }
            (None, Some(_)) if mine => c.text(lang, "your_turn", &[("color", &you)]),
            (None, Some(seat)) => c.text(
                lang,
                "their_turn",
                &[("color", &color_name(lang, Color::from_seat(seat)))],
            ),
            (None, None) => String::new(),
        };
        out.push_str(&status);
        out
    }

    fn is_over(&self) -> bool {
        self.over
    }

    fn result(&self) -> Option<GameResult> {
        if !self.over {
            return None;
        }
        let winners = self.winners();
        let (b, w) = (
            self.board.count(Color::Black),
            self.board.count(Color::White),
        );
        Some(GameResult {
            summary: Localized::build(|lang| outcome(lang, &winners, b, w)),
            winners,
            scores: vec![b as i64, w as i64],
        })
    }

    fn parse_command(&self, command: &str) -> Result<Move, GameError> {
        let c = command.trim();
        if c == "pass" {
            return Ok(Move::Pass);
        }
        Square::parse(c)
            .map(Move::Place)
            .ok_or_else(|| GameError::Parse(c.into()))
    }

    fn format_action(&self, action: &Move) -> String {
        match action {
            Move::Place(s) => s.to_string(),
            Move::Pass => "pass".into(),
        }
    }
}
